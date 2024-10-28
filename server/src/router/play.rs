use crate::game::PlayerEvent;
use fastwebsockets::{FragmentCollectorRead, Frame, OpCode, Payload, WebSocketWrite};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{broadcast, watch},
};
use tracing::{error, info, instrument};

pub async fn send_fn<T>(_: T) -> Result<(), &'static str> {
    Err("unexpected obligated write")
}

#[instrument(skip(ws_reader, ws_writer, event_tx, event_rx, ready_rx))]
pub async fn play<Reader, Writer>(
    ws_reader: &mut FragmentCollectorRead<Reader>,
    ws_writer: &mut WebSocketWrite<Writer>,
    event_tx: &broadcast::Sender<PlayerEvent>,
    event_rx: &mut broadcast::Receiver<PlayerEvent>,
    mut ready_rx: watch::Receiver<bool>,
    pid: usize,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    // TODO: Concurrently listen for player events.

    // Wait for the host to be ready
    drop(ready_rx.wait_for(Clone::clone).await.inspect_err(|err| error!(?err))?);

    ws_writer
        .write_frame(Frame::binary(Payload::Borrowed(Default::default())))
        .await
        .inspect_err(|err| error!(?err, "player disconnected when acknowledging game start"))?;
    info!("acknowledgement request sent");

    // TODO: Concurrently listen for player events.
    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } = ws_reader.read_frame(&mut send_fn).await? else {
        anyhow::bail!("unexpected frame format");
    };
    anyhow::ensure!(payload.is_empty(), "unexpected acknowledgement format");

    // Notify host that we are ready
    drop(ready_rx);

    todo!("implement game logic")
}
