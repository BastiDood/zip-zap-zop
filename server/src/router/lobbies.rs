use crate::game::LobbyManager;
use arcstr::ArcStr;
use fastwebsockets::{upgrade::UpgradeFut, FragmentCollectorRead};
use std::sync::Mutex;
use tracing::instrument;

#[instrument(skip(manager, upgrade))]
pub async fn run(manager: &Mutex<LobbyManager<ArcStr>>, upgrade: UpgradeFut) {
    let (ws_reader, ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let ws_reader = FragmentCollectorRead::new(ws_reader);
    todo!()
}
