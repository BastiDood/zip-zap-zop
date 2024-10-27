use crate::game::PlayerEvent;
use fastwebsockets::{FragmentCollectorRead, Frame, OpCode, Payload, WebSocketWrite};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::broadcast::Receiver,
};
use tracing::{error, info, instrument};

pub async fn send_fn<T>(_: T) -> Result<(), &'static str> {
    Err("unexpected obligated write")
}

#[instrument(skip(ws_reader, ws_writer, rx))]
pub async fn play<Reader, Writer>(
    ws_reader: &mut FragmentCollectorRead<Reader>,
    ws_writer: &mut WebSocketWrite<Writer>,
    rx: &mut Receiver<PlayerEvent>,
    pid: usize,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    ws_writer
        .write_frame(Frame::binary(Payload::Borrowed(Default::default())))
        .await
        .inspect_err(|err| error!(?err, "player disconnected when acknowledging game start"))?;
    info!("acknowledgement request sent");

    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } = ws_reader.read_frame(&mut send_fn).await? else {
        anyhow::bail!("unexpected frame format");
    };

    anyhow::ensure!(payload.is_empty(), "unexpected acknowledgement format");

    // TODO: How do we communicate to the other players in the lobby that we've sent a command?

    todo!("implement game logic")
}
