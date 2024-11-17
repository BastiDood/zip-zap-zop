use crate::event::player::PlayerAction;
use jiff::Timestamp;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameExpected {
    /// The game expects the player with this ID to respond.
    pub curr: usize,
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

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "type")]
pub enum GameEvent {
    Expected(GameExpected),
    Eliminated(GameEliminated),
    Concluded(GameConcluded),
}

impl From<GameExpected> for GameEvent {
    fn from(value: GameExpected) -> Self {
        Self::Expected(value)
    }
}

impl From<GameEliminated> for GameEvent {
    fn from(value: GameEliminated) -> Self {
        Self::Eliminated(value)
    }
}

impl From<GameConcluded> for GameEvent {
    fn from(value: GameConcluded) -> Self {
        Self::Concluded(value)
    }
}
