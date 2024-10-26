mod create;
mod join;
mod lobbies;

use fastwebsockets::{upgrade, WebSocketError};
use http_body_util::Empty;
use hyper::{
    body::{Bytes, Incoming},
    Method, Request, Response, StatusCode,
};
use tracing::error;

pub fn route(req: Request<Incoming>, res: &mut Response<Empty<Bytes>>) -> Result<(), WebSocketError> {
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
            tokio::spawn(async move {
                if let Err(err) = lobbies::run(upgrade).await {
                    error!(%err);
                }
            });
            response
        }
        "/create" => {
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(async move {
                if let Err(err) = create::run(upgrade).await {
                    error!(%err);
                }
            });
            response
        }
        "/join" => {
            let (response, upgrade) = upgrade::upgrade(req)?;
            tokio::spawn(async move {
                if let Err(err) = join::run(upgrade).await {
                    error!(%err);
                }
            });
            response
        }
        _ => {
            *res.status_mut() = StatusCode::NOT_FOUND;
            return Ok(());
        }
    };

    Ok(())
}
