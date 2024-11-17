use crate::{
    actor::{
        io::{event_to_websocket_msgpack_actor, websocket_msgpack_to_event_actor},
        lobby::{wait_for_lobby_start, LobbyStart},
        send_fn,
    },
    event::{
        lobby::{JoinLobby, LobbyJoined, LobbyPlayerJoined, LobbyPlayerLeft, StartGame},
        player::{PlayerAction, PlayerResponds},
    },
    router::lobby::{Lobby, LobbyManager},
};
use arcstr::ArcStr;
use fastwebsockets::{
    upgrade::UpgradeFut, FragmentCollectorRead, Frame, OpCode, Payload, WebSocketError, WebSocketWrite,
};
use slab::Slab;
use std::sync::Mutex;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{error, info, instrument, trace};

#[instrument(skip(ws_writer))]
async fn send_known_players<Writer>(
    ws_writer: &mut WebSocketWrite<Writer>,
    pid: usize,
    lobby: ArcStr,
    snapshot: Slab<ArcStr>,
) -> Result<(), WebSocketError>
where
    Writer: AsyncWrite + Unpin,
{
    let bytes = rmp_serde::to_vec(&LobbyJoined { pid, lobby }).unwrap();
    ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;

    for (pid, player) in snapshot {
        let bytes = rmp_serde::to_vec(&LobbyPlayerJoined { pid, player }).unwrap();
        ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;
    }

    Ok(())
}

#[instrument(skip(ws_reader, ws_writer))]
async fn wait_for_round_trip_ping<Reader, Writer>(
    ws_reader: &mut FragmentCollectorRead<Reader>,
    ws_writer: &mut WebSocketWrite<Writer>,
    count: usize,
) -> Result<bool, WebSocketError>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    let bytes = rmp_serde::to_vec(&StartGame { count }).unwrap();
    ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;
    Ok(match ws_reader.read_frame(&mut send_fn).await? {
        Frame { fin: true, opcode: OpCode::Binary, payload, .. } => payload.is_empty(),
        Frame { fin, opcode, payload, .. } => {
            error!(fin, ?opcode, ?payload, "unexpected frame format");
            false
        }
    })
}

// TODO: Refactor so that `lid` and `pid` are kept in instrumentation spans.
#[instrument(skip(lobbies, upgrade))]
pub async fn guest_actor(lobbies: &Mutex<LobbyManager>, upgrade: UpgradeFut) {
    let (ws_reader, mut ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let mut ws_reader = FragmentCollectorRead::new(ws_reader);

    let payload = match ws_reader.read_frame(&mut send_fn).await.unwrap() {
        Frame { fin: true, opcode: OpCode::Binary, payload, .. } => payload,
        Frame { fin, opcode, payload, .. } => {
            error!(fin, ?opcode, ?payload, "unexpected frame format");
            return;
        }
    };

    let JoinLobby { lid, player } = rmp_serde::from_slice(&payload).unwrap();
    info!(lid, %player, "player requested to join lobby");

    let (mut broadcast_rx, pid, lobby, snapshot) = {
        let mut guard = lobbies.lock().unwrap();
        let Some(Lobby { broadcast_tx, players, lobby }) = guard.lobbies.get_mut(lid) else {
            error!(lid, "lobby does not exist");
            return;
        };

        trace!(%lobby, "lobby found for guest");

        let clone = players.clone();
        let pid = players.insert(player.clone());

        let count = broadcast_tx.send(LobbyPlayerJoined { pid, player }.into()).expect("lobby must still be alive");
        trace!(count, "broadcasted player joined event to receivers");

        (broadcast_tx.subscribe(), pid, lobby.clone(), clone)
    };

    'lobby: {
        if let Err(err) = send_known_players(&mut ws_writer, pid, lobby, snapshot).await {
            error!(?err, "websocket writer error when sending known players");
            break 'lobby;
        }

        let LobbyStart { ready_tx, event_tx, mut broadcast_rx, count } =
            match wait_for_lobby_start(&mut ws_writer, &mut broadcast_rx).await {
                Ok(Some(event)) => event,
                Ok(None) => {
                    error!("broadcast receiver could not process new messages");
                    break 'lobby;
                }
                Err(err) => {
                    error!(?err, "websocket writer error while waiting for game start");
                    break 'lobby;
                }
            };

        'game: {
            match wait_for_round_trip_ping(&mut ws_reader, &mut ws_writer, count).await {
                Ok(true) => (),
                Ok(false) => {
                    error!("unexpected websocket frame format");
                    break 'game;
                }
                Err(err) => {
                    error!(?err, "websocket writer error while waiting for round trip ping");
                    break 'game;
                }
            }

            // Signal to the lobby that this player is ready
            info!("player is ready");
            drop(ready_tx);

            // Play the game
            tokio::spawn(async move { websocket_msgpack_to_event_actor(&mut ws_reader, &event_tx, pid).await });
            tokio::spawn(async move { event_to_websocket_msgpack_actor(&mut ws_writer, &mut broadcast_rx).await });
            return;
        }

        // Announce graceful self-elimination of player.
        event_tx
            .send(PlayerResponds { pid, next: pid, action: PlayerAction::Zip })
            .await
            .expect("game must still exist");
        return;
    }

    // Gracefully disconnect player from the lobby with notification.
    {
        let mut guard = lobbies.lock().unwrap();
        let Lobby { broadcast_tx, players, lobby } = guard.lobbies.get_mut(lid).expect("lobby must still exist");

        let player = players.remove(pid);
        info!(%lobby, %player, "player has prematurely left the lobby");

        let count = broadcast_tx
            .send(LobbyPlayerLeft { pid }.into())
            .expect("lobby event broadcast channel must still be alive");
        trace!(count, "broadcasted player leave event to receivers");
    }
}
