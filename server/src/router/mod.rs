pub mod lobby;

use crate::actor::lobby::{guest::guest_actor, host::host_actor};
use fastwebsockets::{upgrade, WebSocketError};
use http_body_util::Empty;
use hyper::{
    body::{Bytes, Incoming},
    Method, Request, Response, StatusCode,
};
use lobby::LobbyManager;
use std::sync::Mutex;
use triomphe::Arc;

pub fn route(
    manager: Arc<Mutex<LobbyManager>>,
    req: Request<Incoming>,
    res: &mut Response<Empty<Bytes>>,
) -> Result<(), WebSocketError> {
    if *req.method() != Method::GET {
        *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(());
    }

    *res = match req.uri().path() {
        "/lobbies" => todo!("event source"),
        "/host" => {
            if !upgrade::is_upgrade_request(&req) {
                *res.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(());
            }
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(async move { host_actor(&manager, upgrade, 32).await });
            response
        }
        "/guest" => {
            if !upgrade::is_upgrade_request(&req) {
                *res.status_mut() = StatusCode::BAD_REQUEST;
                return Ok(());
            }
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(async move { guest_actor(&manager, upgrade).await });
            response
        }
        _ => {
            *res.status_mut() = StatusCode::NOT_FOUND;
            return Ok(());
        }
    };

    Ok(())
}
