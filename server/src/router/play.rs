use crate::game::PlayerEvent;
use fastwebsockets::{FragmentCollectorRead, Frame, OpCode, Payload, WebSocket, WebSocketWrite};
use serde::Serialize;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{
        broadcast::{self, error::RecvError},
        watch, Mutex,
    },
};
use tracing::{error, info, instrument};
use triomphe::Arc;

pub async fn send_fn<T>(_: T) -> Result<(), &'static str> {
    Err("unexpected obligated write")
}

#[instrument(skip(event_rx, ws_writer))]
pub async fn relay_events_to_websocket<Event, Io>(
    mut event_rx: broadcast::Receiver<Event>,
    ws_writer: &mut WebSocket<Io>,
) where
    Event: Clone + Serialize,
    Io: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        match event_rx.recv().await {
            Ok(event) => {
                // If sending panics, the host will eventually realize it cannot send to the writer.
                let bytes = rmp_serde::to_vec(&event).unwrap().into();
                ws_writer.write_frame(Frame::binary(bytes)).await.unwrap();
            }
            Err(err) => match err {
                RecvError::Closed => {
                    info!("events channel closed");
                    break;
                }
                RecvError::Lagged(count) => {
                    error!(count, "events channel lagged");
                    unreachable!("events channel lagged by {count}");
                }
            },
        }
    }
}

#[instrument(skip(event_rx, ws_writer))]
pub async fn relay_events_to_websocket_writer_with_lock<Event, Writer>(
    mut event_rx: broadcast::Receiver<Event>,
    ws_writer: &Mutex<WebSocketWrite<Writer>>,
) where
    Event: Clone + Serialize,
    Writer: AsyncWrite + Unpin,
{
    loop {
        match event_rx.recv().await {
            Ok(event) => {
                // If sending panics, the host will eventually realize it cannot send to the writer.
                let bytes = rmp_serde::to_vec(&event).unwrap().into();
                ws_writer.lock().await.write_frame(Frame::binary(bytes)).await.unwrap();
            }
            Err(err) => match err {
                RecvError::Closed => {
                    info!("events channel closed");
                    break;
                }
                RecvError::Lagged(count) => {
                    error!(count, "events channel lagged");
                    unreachable!("events channel lagged by {count}");
                }
            },
        }
    }
}

#[instrument(skip(ws_reader, ws_writer, event_tx, ready_rx))]
pub async fn play<Reader, Writer>(
    ws_reader: &mut FragmentCollectorRead<Reader>,
    ws_writer: Arc<Mutex<WebSocketWrite<Writer>>>,
    event_tx: &broadcast::Sender<PlayerEvent>,
    mut ready_rx: watch::Receiver<bool>,
    pid: usize,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    // Wait for the host to be ready
    drop(ready_rx.wait_for(Clone::clone).await.inspect_err(|err| error!(?err))?);

    ws_writer
        .lock()
        .await
        .write_frame(Frame::binary(Payload::Borrowed(Default::default())))
        .await
        .inspect_err(|err| error!(?err, "player disconnected when acknowledging game start"))?;
    info!("acknowledgement request sent");

    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } = ws_reader.read_frame(&mut send_fn).await? else {
        anyhow::bail!("unexpected frame format");
    };
    anyhow::ensure!(payload.is_empty(), "unexpected acknowledgement format");

    // Notify host that the player is ready
    drop(ready_rx);

    todo!("implement game logic")
}
