use crate::event::{
    lobby::{LobbyPlayerJoined, LobbyPlayerLeft},
    player::PlayerRespondsWithId,
};
use arcstr::ArcStr;
use core::convert::Infallible;
use slab::Slab;
use tokio::sync::{broadcast, mpsc};
use triomphe::Arc;

#[derive(Debug)]
pub struct LobbyStart {
    pub ready_tx: mpsc::Sender<Infallible>,
    pub event_tx: mpsc::Sender<PlayerRespondsWithId>,
    pub broadcast_rx: broadcast::Receiver<Arc<[u8]>>,
    /// Number of known players in the game.
    pub count: usize,
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
pub enum LobbyEvent {
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

pub struct Lobby {
    pub broadcast_tx: broadcast::Sender<LobbyEvent>,
    pub lobby: ArcStr,
    pub players: Slab<ArcStr>,
}

#[derive(Default)]
pub struct LobbyManager {
    pub lobbies: Slab<Lobby>,
}
