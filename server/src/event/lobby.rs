use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateLobby {
    pub player: ArcStr,
    pub lobby: ArcStr,
}

#[derive(Serialize)]
pub struct LobbyCreated {
    pub lid: usize,
    pub pid: usize,
}

#[derive(Deserialize)]
pub struct JoinLobby {
    pub lid: usize,
    pub player: ArcStr,
}

#[derive(Serialize)]
pub struct LobbyJoined {
    pub lobby: ArcStr,
    pub pid: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct LobbyPlayerJoined {
    pub pid: usize,
    pub player: ArcStr,
}

#[derive(Clone, Debug, Serialize)]
pub struct LobbyPlayerLeft {
    pub pid: usize,
}

#[derive(Serialize, Deserialize)]
pub struct StartGame {
    pub count: usize,
}
