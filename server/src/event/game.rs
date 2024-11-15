use crate::event::player::PlayerAction;
use jiff::Timestamp;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameExpects {
    /// The game expects the player with this ID to respond.
    pub curr: usize,
    pub action: PlayerAction,
    pub deadline: Timestamp,
}

#[derive(Clone, Copy, Serialize)]
pub struct GameEliminates {
    /// The ID of the eliminated player.
    pub pid: usize,
}

#[derive(Clone, Copy, Serialize)]
pub struct GameConcludes {
    /// The player ID of the winner.
    pub pid: usize,
}

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "type")]
pub enum GameEvent {
    Expects(GameExpects),
    Eliminates(GameEliminates),
    Concludes(GameConcludes),
}

impl From<GameExpects> for GameEvent {
    fn from(value: GameExpects) -> Self {
        Self::Expects(value)
    }
}

impl From<GameEliminates> for GameEvent {
    fn from(value: GameEliminates) -> Self {
        Self::Eliminates(value)
    }
}

impl From<GameConcludes> for GameEvent {
    fn from(value: GameConcludes) -> Self {
        Self::Concludes(value)
    }
}
