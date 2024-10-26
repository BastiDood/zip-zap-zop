use crate::game::LobbyManager;
use arcstr::ArcStr;
use std::sync::Mutex;
use triomphe::Arc;

#[tracing::instrument(skip(manager, upgrade))]
pub async fn run(manager: Arc<Mutex<LobbyManager<ArcStr>>>, upgrade: fastwebsockets::upgrade::UpgradeFut) {
    let ws = upgrade.await;
    todo!()
}
