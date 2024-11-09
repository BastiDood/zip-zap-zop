use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayerAction {
    Zip,
    Zap,
    Zop,
}

impl PlayerAction {
    pub const fn next(self) -> Self {
        match self {
            Self::Zip => Self::Zap,
            Self::Zap => Self::Zop,
            Self::Zop => Self::Zip,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlayerResponded {
    /// ID of the player who responded.
    pub pid: usize,
    /// The targeted next player in the game.
    pub next: usize,
    pub action: PlayerAction,
}
