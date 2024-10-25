use fastwebsockets::WebSocketError;
use http_body_util::Empty;
use hyper::{
    body::{Bytes, Incoming},
    Method, Request, Response, StatusCode,
};

pub fn route(req: Request<Incoming>, res: &mut Response<Empty<Bytes>>) -> Result<(), WebSocketError> {
    if *req.method() != Method::GET {
        *res.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(());
    }

    if !fastwebsockets::upgrade::is_upgrade_request(&req) {
        *res.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(());
    }

    *res = match req.uri().path() {
        "/lobbies" => {
            let (response, upgrade) = fastwebsockets::upgrade::upgrade(req)?;
            response
        }
        "/create" => {
            let (response, upgrade) = fastwebsockets::upgrade::upgrade(req)?;
            response
        }
        "/join" => {
            let (response, upgrade) = fastwebsockets::upgrade::upgrade(req)?;
            response
        }
        _ => {
            *res.status_mut() = StatusCode::NOT_FOUND;
            return Ok(());
        }
    };

    Ok(())
}
