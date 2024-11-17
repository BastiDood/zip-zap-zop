use crate::event::player::PlayerAction;
use jiff::Timestamp;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameStarted {
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameExpected {
    /// The game expects the player with this ID to respond.
    pub next: usize,
    pub action: PlayerAction,
    pub deadline: Timestamp,
}

#[derive(Clone, Copy, Serialize)]
pub struct GameEliminated {
    /// The ID of the eliminated player.
    pub pid: usize,
}

#[derive(Clone, Copy, Serialize)]
pub struct GameConcluded {
    /// The player ID of the winner.
    pub pid: usize,
}
