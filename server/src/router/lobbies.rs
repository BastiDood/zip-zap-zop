use crate::{game::LobbyManager, router::play::relay_events_to_websocket};
use arcstr::ArcStr;
use fastwebsockets::{upgrade::UpgradeFut, Frame};
use std::sync::Mutex;
use tracing::instrument;

#[instrument(skip(manager, upgrade))]
pub async fn run(manager: &Mutex<LobbyManager<ArcStr>>, upgrade: UpgradeFut) {
    let mut ws = upgrade.await.unwrap();
    let (lobby_rx, lobbies) = {
        let guard = manager.lock().unwrap();
        (guard.subscribe(), guard.lobbies())
    };

    let payload = rmp_serde::to_vec(&lobbies).unwrap().into();
    ws.write_frame(Frame::binary(payload)).await.unwrap();

    relay_events_to_websocket(lobby_rx, &mut ws).await;
}
