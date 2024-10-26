#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LobbyCreated {
    /// Server-specific unique identifier for the lobby.
    pub id: u32,
    /// Number of players currently in the lobby (including the host).
    pub players: u32,
    /// Name of the lobby as a string.
    pub name: arcstr::ArcStr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LobbyDissolved {
    /// Server-specific unique identifier for the dissolved lobby.
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LobbyUpdated {
    /// Server-specific unique identifier for the updated lobby.
    pub id: u32,
    /// (Optional) new number of players.
    pub players: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LobbyEvent {
    Created(LobbyCreated),
    Dissolved(LobbyDissolved),
    Updated(LobbyUpdated),
}

impl From<LobbyCreated> for LobbyEvent {
    fn from(value: LobbyCreated) -> Self {
        Self::Created(value)
    }
}

impl From<LobbyDissolved> for LobbyEvent {
    fn from(value: LobbyDissolved) -> Self {
        Self::Dissolved(value)
    }
}

impl From<LobbyUpdated> for LobbyEvent {
    fn from(value: LobbyUpdated) -> Self {
        Self::Updated(value)
    }
}
