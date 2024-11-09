use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateLobby {
    pub player: ArcStr,
    pub name: ArcStr,
}

#[derive(Serialize)]
pub struct LobbyCreated {
    pub lid: usize,
    pub pid: usize,
}

#[derive(Deserialize)]
pub struct StartGame {
    pub count: usize,
}
