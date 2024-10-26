mod create;
mod join;
mod lobbies;

use crate::game::LobbyManager;
use arcstr::ArcStr;
use fastwebsockets::{upgrade, WebSocketError};
use http_body_util::Empty;
use hyper::{
    body::{Bytes, Incoming},
    Method, Request, Response, StatusCode,
};
use std::sync::Mutex;
use tracing::error;
use triomphe::Arc;

pub fn route(
    manager: Arc<Mutex<LobbyManager<ArcStr>>>,
    req: Request<Incoming>,
    res: &mut Response<Empty<Bytes>>,
) -> Result<(), WebSocketError> {
    if *req.method() != Method::GET {
        *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(());
    }

    if !upgrade::is_upgrade_request(&req) {
        *res.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(());
    }

    *res = match req.uri().path() {
        "/lobbies" => {
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(lobbies::run(manager, upgrade));
            response
        }
        "/create" => {
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(create::run(manager, upgrade));
            response
        }
        "/join" => {
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(join::run(manager, upgrade));
            response
        }
        _ => {
            *res.status_mut() = StatusCode::NOT_FOUND;
            return Ok(());
        }
    };

    Ok(())
}
