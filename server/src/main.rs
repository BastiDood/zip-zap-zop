mod router;

use std::net::Ipv4Addr;
use tokio::net::TcpListener;
use tracing::{error, info_span, warn, Instrument};

fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT")?.parse()?;

    let mut new_multi_thread = tokio::runtime::Builder::new_multi_thread();
    let runtime = new_multi_thread.enable_io().enable_time().build()?;
    let signal = runtime.block_on(async {
        let mut signal = core::pin::pin!(tokio::signal::ctrl_c());
        let tcp = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;

        let http = hyper::server::conn::http1::Builder::new();
        loop {
            let conn = tokio::select! {
                biased;
                signal = &mut signal => break signal,
                conn = tcp.accept() => conn,
            };

            let (stream, addr) = match conn {
                Ok(pair) => pair,
                Err(err) => {
                    error!(?err);
                    continue;
                }
            };

            let service = hyper::service::service_fn(move |req| async move {
                let mut res = hyper::Response::default();
                match router::route(req, &mut res) {
                    Ok(()) => Ok(res),
                    Err(err) => {
                        error!(%err);
                        Err(err)
                    }
                }
            });

            let io = hyper_util::rt::TokioIo::new(stream);
            let fut = http.serve_connection(io, service).instrument(info_span!("tcp", %addr));
            runtime.spawn(fut);
        }
    });

    warn!("shutting down runtime in 60 seconds at most");
    runtime.shutdown_timeout(core::time::Duration::from_secs(60));

    signal?;
    Ok(())
}
