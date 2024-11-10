pub mod guest;
pub mod host;

use crate::router::lobby::{LobbyEvent, LobbyStart};
use fastwebsockets::{Frame, Payload, WebSocketError, WebSocketWrite};
use tokio::{io::AsyncWrite, sync::broadcast};
use tracing::{error, info};

async fn wait_for_lobby_start<Writer>(
    ws_writer: &mut WebSocketWrite<Writer>,
    broadcast_rx: &mut broadcast::Receiver<LobbyEvent>,
) -> Result<Option<LobbyStart>, WebSocketError>
where
    Writer: AsyncWrite + Unpin,
{
    use broadcast::error::RecvError;
    Ok(loop {
        let bytes = match broadcast_rx.recv().await {
            Ok(LobbyEvent::PlayerJoined(event)) => rmp_serde::to_vec(&event).unwrap(),
            Ok(LobbyEvent::PlayerLeft(event)) => rmp_serde::to_vec(&event).unwrap(),
            Ok(LobbyEvent::Start(event)) => {
                info!("game start notification received");
                break Some(event);
            }
            Err(RecvError::Lagged(count)) => {
                error!(count, "broadcast receiver lagged while waiting for lobby start");
                break None;
            }
            Err(RecvError::Closed) => {
                error!("broadcast receiver closed while waiting for lobby start");
                break None;
            }
        };
        ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;
    })
}
