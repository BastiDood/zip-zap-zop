pub mod game;
pub mod lobby;
pub mod player;

use game::{GameConcluded, GameEliminated, GameExpected, GameStarted};
use lobby::{LobbyCreated, LobbyJoined, LobbyPlayerJoined, LobbyPlayerLeft};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Event {
    LobbyCreated(LobbyCreated),
    LobbyJoined(LobbyJoined),
    LobbyPlayerJoined(LobbyPlayerJoined),
    LobbyPlayerLeft(LobbyPlayerLeft),
    GameStarted(GameStarted),
    GameExpected(GameExpected),
    GameEliminated(GameEliminated),
    GameConcluded(GameConcluded),
}

impl From<LobbyCreated> for Event {
    fn from(value: LobbyCreated) -> Self {
        Self::LobbyCreated(value)
    }
}

impl From<LobbyJoined> for Event {
    fn from(value: LobbyJoined) -> Self {
        Self::LobbyJoined(value)
    }
}

impl From<LobbyPlayerJoined> for Event {
    fn from(value: LobbyPlayerJoined) -> Self {
        Self::LobbyPlayerJoined(value)
    }
}

impl From<LobbyPlayerLeft> for Event {
    fn from(value: LobbyPlayerLeft) -> Self {
        Self::LobbyPlayerLeft(value)
    }
}

impl From<GameStarted> for Event {
    fn from(value: GameStarted) -> Self {
        Self::GameStarted(value)
    }
}

impl From<GameExpected> for Event {
    fn from(value: GameExpected) -> Self {
        Self::GameExpected(value)
    }
}

impl From<GameEliminated> for Event {
    fn from(value: GameEliminated) -> Self {
        Self::GameEliminated(value)
    }
}

impl From<GameConcluded> for Event {
    fn from(value: GameConcluded) -> Self {
        Self::GameConcluded(value)
    }
}
