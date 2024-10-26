use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayerJoined {
    pub id: u32,
    pub name: arcstr::ArcStr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayerLeft {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PlayerEvent {
    Joined(PlayerJoined),
    Left(PlayerLeft),
}

impl From<PlayerJoined> for PlayerEvent {
    fn from(event: PlayerJoined) -> Self {
        Self::Joined(event)
    }
}

impl From<PlayerLeft> for PlayerEvent {
    fn from(event: PlayerLeft) -> Self {
        Self::Left(event)
    }
}
