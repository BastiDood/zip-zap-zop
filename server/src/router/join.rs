use crate::{
    game::LobbyManager,
    router::play::{play, relay_events_to_websocket, send_fn},
};
use arcstr::ArcStr;
use fastwebsockets::{upgrade::UpgradeFut, FragmentCollectorRead, Frame, OpCode};
use serde::Deserialize;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;
use tracing::{error, error_span, info, instrument, warn};
use triomphe::Arc;

#[derive(Deserialize)]
struct JoinLobby {
    lobby: u32,
    player: ArcStr,
}

#[instrument(skip(manager, upgrade))]
pub async fn run(manager: &StdMutex<LobbyManager<ArcStr>>, upgrade: UpgradeFut) {
    let (ws_reader, ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let mut ws_reader = FragmentCollectorRead::new(ws_reader);

    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } = ws_reader.read_frame(&mut send_fn).await.unwrap()
    else {
        unreachable!("unexpected frame format");
    };

    let JoinLobby { lobby, player } = rmp_serde::from_slice(&payload).unwrap();
    let lid = lobby.try_into().unwrap();

    // TODO: Return list of known players in the lobby.

    let Some((pid, event_tx, event_rx, ready_rx)) = manager.lock().unwrap().join_player_into_lobby(lid, player) else {
        error!(lid, "cannot join non-existent lobby");
        return;
    };

    let ws_writer = Arc::new(TokioMutex::new(ws_writer));
    let bg_relay = tokio::spawn(relay_events_to_websocket(event_rx, ws_writer.clone()));

    // TODO: Abort the current task if the background worker dies.

    if let Err(err) = play(&mut ws_reader, ws_writer, &event_tx, ready_rx, pid).await {
        error!(%err);
        if manager.lock().unwrap().remove_player_from_lobby(lid, pid) {
            info!(lid, pid, "player disconnected from the lobby");
        } else {
            // FIXME: There is a possible race condition where the `lid` was reused.
            warn!(lid, pid, "lobby was already dissolved when the player disconnected");
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
