mod actor;
/// One-to-many actor model.
mod event;
mod zzz;

mod game;
mod router;

use arcstr::ArcStr;
use std::{net::Ipv4Addr, sync::Mutex};
use tokio::net::TcpListener;
use tracing::{error, info_span, warn, Instrument};
use triomphe::Arc;

fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT")?.parse()?;

    let runtime = tokio::runtime::Builder::new_multi_thread().enable_io().enable_time().build()?;
    let signal = runtime.block_on(async {
        let mut signal = core::pin::pin!(tokio::signal::ctrl_c());
        let tcp = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;

        let http = hyper::server::conn::http1::Builder::new();
        let manager = Arc::new(Mutex::new(game::LobbyManager::<ArcStr>::new(8)));
        loop {
            let conn = tokio::select! {
                biased;
                signal = &mut signal => {
                    break signal;
                }
                conn = tcp.accept() => conn,
            };

            let (stream, addr) = match conn {
                Ok(pair) => pair,
                Err(err) => {
                    error!(?err);
                    continue;
                }
            };

            let manager = manager.clone();
            let service = hyper::service::service_fn(move |req| {
                let manager = manager.clone();
                async move {
                    let mut res = hyper::Response::default();
                    match router::route(manager, req, &mut res) {
                        Ok(()) => Ok(res),
                        Err(err) => {
                            error!(%err);
                            Err(err)
                        }
                    }
                }
            });

            let io = hyper_util::rt::TokioIo::new(stream);
            runtime.spawn(http.serve_connection(io, service).with_upgrades().instrument(info_span!("tcp", %addr)));
        }
    });

    warn!("shutting down runtime in 60 seconds at most");
    runtime.shutdown_timeout(core::time::Duration::from_secs(60));

    signal?;
    Ok(())
}
