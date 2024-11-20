use crate::{
    actor::send_fn,
    event::player::{PlayerAction, PlayerResponds, PlayerRespondsWithId},
};
use fastwebsockets::{FragmentCollectorRead, Frame, OpCode, Payload, WebSocketWrite};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{
        broadcast::{error::RecvError, Receiver},
        mpsc::{error::SendError, Sender},
    },
};
use tracing::{error, info, instrument, warn};
use triomphe::Arc;

#[instrument(skip(event_tx, ws_reader))]
pub async fn websocket_msgpack_to_event_actor<Reader>(
    ws_reader: &mut FragmentCollectorRead<Reader>,
    event_tx: &Sender<PlayerRespondsWithId>,
    pid: usize,
) where
    Reader: AsyncRead + Unpin,
{
    loop {
        let payload = match ws_reader.read_frame(&mut send_fn).await {
            Ok(Frame { fin: true, opcode: OpCode::Binary, payload, .. }) => payload,
            Ok(Frame { fin, opcode, payload, .. }) => {
                error!(fin, ?opcode, ?payload, "unexpected websocket frame received");
                break;
            }
            Err(err) => {
                error!(?err, "websocket reader error encountered");
                break;
            }
        };

        let data = match rmp_serde::from_slice(&payload) {
            Ok(event) => event,
            Err(err) => {
                error!(?err, "cannot deserialize payload");
                break;
            }
        };

        if let Err(SendError(event)) = event_tx.send(PlayerRespondsWithId { pid, data }).await {
            warn!(?event, "lobby has already shut down");
            return;
        }
    }

    // Gracefully eliminate self from the lobby
    if let Err(SendError(event)) =
        event_tx.send(PlayerRespondsWithId { pid, data: PlayerResponds { next: pid, action: PlayerAction::Zip } }).await
    {
        error!(?event, "lobby has already shut down");
        return;
    }
}

#[instrument(skip(event_rx, ws_writer))]
pub async fn event_to_websocket_msgpack_actor<Writer>(
    ws_writer: &mut WebSocketWrite<Writer>,
    event_rx: &mut Receiver<Arc<[u8]>>,
) where
    Writer: AsyncWrite + Unpin,
{
    loop {
        let bytes = match event_rx.recv().await {
            Ok(bytes) => bytes,
            Err(RecvError::Closed) => {
                info!("lobby has gracefully exited");
                break;
            }
            Err(RecvError::Lagged(count)) => {
                error!(count, "broadcast receiver lagged");
                break;
            }
        };

        let payload = Payload::Borrowed(&bytes).into();
        if let Err(err) = ws_writer.write_frame(Frame::binary(payload)).await {
            error!(?err, "websocket writer error encountered");
            break;
        }

        info!("delivered incoming event to the websocket");
    }
}
