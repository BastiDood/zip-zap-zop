use crate::{
    game::{Lobby, LobbyManager},
    router::play::{play, relay_events_to_websocket_writer_with_lock, send_fn},
};
use arcstr::ArcStr;
use fastwebsockets::{upgrade::UpgradeFut, FragmentCollectorRead, Frame, OpCode};
use serde::{Deserialize, Serialize};
use std::sync::Mutex as StdMutex;
use tokio::{io::AsyncRead, sync::Mutex as TokioMutex};
use tracing::{error, error_span, info, instrument};
use triomphe::Arc;

#[derive(Deserialize)]
struct CreateLobby {
    player: ArcStr,
    name: ArcStr,
}

#[derive(Serialize)]
struct LobbyCreated {
    id: u32,
}

#[derive(Deserialize)]
struct StartGame {
    count: u32,
}

#[instrument(skip(manager, ws_reader))]
async fn wait_for_start_command<Reader>(
    manager: &StdMutex<LobbyManager<ArcStr>>,
    ws_reader: &mut FragmentCollectorRead<Reader>,
    lid: usize,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
{
    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } = ws_reader.read_frame(&mut send_fn).await? else {
        anyhow::bail!("unexpected frame format");
    };

    let StartGame { count } = rmp_serde::from_slice(&payload)?;
    let count = usize::try_from(count)?;

    let Some(expected) = manager.lock().unwrap().player_count_of_lobby(lid) else {
        anyhow::bail!("lobby {lid} no longer exists");
    };

    if count != expected {
        error!(count, expected, "host count mismatched with expected");
        anyhow::bail!("host responded with mismatched player count ({count} != {expected})");
    }

    Ok(())
}

#[instrument(skip(manager, upgrade))]
pub async fn run(manager: &StdMutex<LobbyManager<ArcStr>>, upgrade: UpgradeFut) {
    let (ws_reader, mut ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let mut ws_reader = FragmentCollectorRead::new(ws_reader);

    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } = ws_reader.read_frame(&mut send_fn).await.unwrap()
    else {
        unreachable!("unexpected frame format");
    };

    let CreateLobby { player, name } = rmp_serde::from_slice(&payload).unwrap();
    let (lid, pid, event_rx, ready_rx) = manager.lock().unwrap().init_lobby(16, name, player);

    if let Err(err) = ws_writer
        .write_frame(Frame::binary(rmp_serde::to_vec(&LobbyCreated { id: lid.try_into().unwrap() }).unwrap().into()))
        .await
    {
        error!(%err);
        let Lobby { name, players, .. } =
            manager.lock().unwrap().dissolve_lobby(lid).expect("lobby had just been created");
        info!(lid, %name, ?players, "lobby has been prematurely dissolved");
        return;
    }

    let ws_writer = Arc::new(TokioMutex::new(ws_writer));
    let bg_relay = tokio::spawn({
        let ws_writer = ws_writer.clone();
        async move { relay_events_to_websocket_writer_with_lock(event_rx, &ws_writer).await }
    });

    let result = wait_for_start_command(manager, &mut ws_reader, lid).await;
    let Lobby { sender, watcher, name, players } =
        manager.lock().unwrap().dissolve_lobby(lid).expect("lobby must exist within host task");
    info!(lid, %name, ?players, "lobby has been dissolved");

    'relay: {
        if let Err(err) = result {
            error!(%err);
            break 'relay;
        }

        let bg_play = tokio::spawn(async move { play(&mut ws_reader, ws_writer, &sender, ready_rx, pid).await });

        // Wait for all workers to be ready
        assert!(!watcher.send_replace(true));
        watcher.closed().await;
        drop(watcher);

        bg_play.abort();
        if let Err(err) = bg_play.await {
            error_span!("bg_play", %err).in_scope(|| {
                if err.is_panic() {
                    info!("relay panicked");
                } else if err.is_cancelled() {
                    info!("relay cancelled");
                }
            });
        }
    }

    bg_relay.abort();
    if let Err(err) = bg_relay.await {
        error_span!("bg_relay", %err).in_scope(|| {
            if err.is_panic() {
                info!("relay panicked");
            } else if err.is_cancelled() {
                info!("relay cancelled");
            }
        });
    }
}
