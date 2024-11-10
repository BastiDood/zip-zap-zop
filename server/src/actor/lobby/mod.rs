pub mod guest;
pub mod host;

use crate::event::{
    lobby::{LobbyPlayerJoined, LobbyPlayerLeft},
    player::PlayerResponds,
};
use arcstr::ArcStr;
use core::convert::Infallible;
use fastwebsockets::{Frame, Payload, WebSocketError, WebSocketWrite};
use slab::Slab;
use tokio::{
    io::AsyncWrite,
    sync::{broadcast, mpsc},
};
use tracing::{error, info};
use triomphe::Arc;

#[derive(Debug)]
struct LobbyStart {
    ready_tx: mpsc::Sender<Infallible>,
    event_tx: mpsc::Sender<PlayerResponds>,
    broadcast_rx: broadcast::Receiver<Arc<[u8]>>,
    /// Number of known players in the game.
    count: usize,
}

impl Clone for LobbyStart {
    fn clone(&self) -> Self {
        let ready_tx = self.ready_tx.clone();
        let event_tx = self.event_tx.clone();
        let broadcast_rx = self.broadcast_rx.resubscribe();
        let count = self.count;
        Self { broadcast_rx, ready_tx, event_tx, count }
    }
}

#[derive(Clone, Debug)]
enum LobbyEvent {
    Start(LobbyStart),
    PlayerJoined(LobbyPlayerJoined),
    PlayerLeft(LobbyPlayerLeft),
}

impl From<LobbyStart> for LobbyEvent {
    fn from(value: LobbyStart) -> Self {
        Self::Start(value)
    }
}

impl From<LobbyPlayerJoined> for LobbyEvent {
    fn from(value: LobbyPlayerJoined) -> Self {
        Self::PlayerJoined(value)
    }
}

impl From<LobbyPlayerLeft> for LobbyEvent {
    fn from(value: LobbyPlayerLeft) -> Self {
        Self::PlayerLeft(value)
    }
}

struct Lobby {
    broadcast_tx: broadcast::Sender<LobbyEvent>,
    players: Slab<ArcStr>,
}

async fn wait_for_lobby_start<Writer>(
    ws_writer: &mut WebSocketWrite<Writer>,
    broadcast_rx: &mut broadcast::Receiver<LobbyEvent>,
) -> Result<Option<LobbyStart>, WebSocketError>
where
    Writer: AsyncWrite + Unpin,
{
    use broadcast::error::RecvError;
    Ok(loop {
        let bytes = match broadcast_rx.recv().await {
            Ok(LobbyEvent::PlayerJoined(event)) => rmp_serde::to_vec(&event).unwrap(),
            Ok(LobbyEvent::PlayerLeft(event)) => rmp_serde::to_vec(&event).unwrap(),
            Ok(LobbyEvent::Start(event)) => {
                info!("game start notification received");
                break Some(event);
            }
            Err(RecvError::Lagged(count)) => {
                error!(count, "broadcast receiver lagged while waiting for lobby start");
                break None;
            }
            Err(RecvError::Closed) => {
                error!("broadcast receiver closed while waiting for lobby start");
                break None;
            }
        };
        ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;
    })
}
