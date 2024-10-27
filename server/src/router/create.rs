use crate::{
    game::{
        player::{PlayerLeft, PlayerReady},
        Lobby, LobbyManager, PlayerEvent,
    },
    router::play::{play, send_fn},
};
use arcstr::ArcStr;
use core::{mem::replace, time::Duration};
use fastwebsockets::{upgrade::UpgradeFut, FragmentCollectorRead, Frame, OpCode, WebSocketWrite};
use serde::{Deserialize, Serialize};
use slab::Slab;
use std::sync::Mutex;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::broadcast::{error::RecvError, Receiver},
    time::timeout,
};
use tracing::{debug, error, info, instrument};
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

#[instrument(skip(manager, ws_reader, ws_writer, rx))]
async fn wait_for_start_command<Reader, Writer>(
    manager: &Mutex<LobbyManager<ArcStr>>,
    ws_reader: &mut FragmentCollectorRead<Reader>,
    ws_writer: &mut WebSocketWrite<Writer>,
    rx: &mut Receiver<PlayerEvent>,
    lid: usize,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    ws_writer
        .write_frame(Frame::binary(rmp_serde::to_vec(&LobbyCreated { id: lid.try_into()? })?.into()))
        .await
        .inspect_err(|err| error!(?err, "player disconnected when sending lobby creation notification"))?;
    debug!("notified player of newly created lobby");

    let payload = {
        let func = &mut send_fn;
        let mut signal = core::pin::pin!(ws_reader.read_frame(func));
        loop {
            tokio::select! {
                biased;
                event = rx.recv() => {
                    let event = event.inspect_err(|err| match err {
                        RecvError::Closed => error!("player event channel closed"),
                        RecvError::Lagged(count) => error!(count, "player event channel lagged"),
                    })?;
                    ws_writer
                        .write_frame(Frame::binary(rmp_serde::to_vec(&event)?.into()))
                        .await
                        .inspect_err(|err| error!(?err, "failed to relay player event to host"))?;
                    debug!(?event, "notified player of player event");
                }
                frame = &mut signal => match frame.inspect_err(|err| error!(?err, "player disconnected from websocket"))? {
                    Frame { fin: true, opcode: OpCode::Binary, payload, .. } => break payload,
                    _ => anyhow::bail!("unexpected frame format"),
                }
            }
        }
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

#[instrument(skip(rx))]
async fn wait_for_all_players(players: &mut Slab<bool>, rx: &mut Receiver<PlayerEvent>) -> anyhow::Result<()> {
    let mut count = 0;
    while count < players.len() {
        let event = rx.recv().await.inspect_err(|err| match err {
            RecvError::Closed => unreachable!("detached host always holds one sender"),
            RecvError::Lagged(count) => error!(count, "lobby has too many messages"),
        })?;
        match event {
            PlayerEvent::Joined(_) => unreachable!("lobby must have already been dissolved"),
            PlayerEvent::Left(PlayerLeft { id }) => {
                let id = id.try_into()?;
                let Some(flag) = players.try_remove(id) else {
                    error!(id, "non-existent player tried to leave the lobby");
                    anyhow::bail!("non-existent player {id} tried to leave the lobby");
                };
                if flag {
                    info!(id, "ready player has left the lobby");
                    count -= 1; // undo the tracker
                } else {
                    info!(id, "pending player has left the lobby");
                }
            }
            PlayerEvent::Ready(PlayerReady { id }) => {
                let id = id.try_into()?;
                let Some(flag) = players.get_mut(id) else {
                    error!(id, "unknown player is ready");
                    anyhow::bail!("unknown player {id}");
                };
                if replace(flag, true) {
                    error!(id, "duplicate acknowledgement from player");
                    anyhow::bail!("duplicate acknowledgement from player {id}");
                }
                count += 1;
            }
        };
    }
    anyhow::Ok(())
}

#[instrument(skip(manager, upgrade))]
pub async fn run(manager: Arc<Mutex<LobbyManager<ArcStr>>>, upgrade: UpgradeFut) {
    let (ws_reader, mut ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let mut ws_reader = FragmentCollectorRead::new(ws_reader);

    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } = ws_reader.read_frame(&mut send_fn).await.unwrap()
    else {
        unreachable!("unexpected frame format");
    };

    let CreateLobby { player, name } = rmp_serde::from_slice(&payload).unwrap();
    let (lid, pid, mut receiver) = manager.lock().unwrap().init_lobby(16, name, player);

    let result = wait_for_start_command(&manager, &mut ws_reader, &mut ws_writer, &mut receiver, lid).await;
    let Some(Lobby { sender, name, players }) = manager.lock().unwrap().dissolve_lobby(lid) else {
        error!(lid, "lobby has already been dissolved unexpectedly");
        return;
    };

    info!(id = lid, name = name.as_str(), ?players, "lobby committed");
    drop(name);

    if let Err(err) = result {
        error!(%err);
        return;
    }

    let mut rx = sender.subscribe();
    let tx = sender.clone(); // TODO: Notify players of round results.
    tokio::spawn(async move {
        if let Err(err) = play(&mut ws_reader, &mut ws_writer, &sender, &mut receiver, pid).await {
            error!(%err);
            return;
        }
    });

    // TODO: Notify everyone in the lobby that the game is about to start.

    // Wait for all players to announce ready state
    let mut players: Slab<_> = players.into_iter().map(|(pid, _)| (pid, false)).collect();
    match timeout(Duration::from_secs(10), wait_for_all_players(&mut players, &mut rx)).await {
        Ok(result) => result.expect("game protocol violated"),
        Err(err) => {
            error!(%err);
            return;
        }
    }

    todo!("zip zap zop cycle")
}
