mod create;
mod join;
mod lobbies;
mod play;

use crate::game::LobbyManager;
use arcstr::ArcStr;
use fastwebsockets::{upgrade, WebSocketError};
use http_body_util::Empty;
use hyper::{
    body::{Bytes, Incoming},
    Method, Request, Response, StatusCode,
};
use std::sync::Mutex;
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
            tokio::spawn(async move { lobbies::run(&manager, upgrade).await });
            response
        }
        "/create" => {
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(async move { create::run(&manager, upgrade).await });
            response
        }
        "/join" => {
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(async move { join::run(&manager, upgrade).await });
            response
        }
        _ => {
            *res.status_mut() = StatusCode::NOT_FOUND;
            return Ok(());
        }
    };

    Ok(())
}
