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
pub struct PlayerReady {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum PlayerEvent {
    Joined(PlayerJoined),
    Left(PlayerLeft),
    Ready(PlayerReady),
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
