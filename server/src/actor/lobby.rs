use crate::{
    actor::{game::handle_game, send_fn},
    event::{
        lobby::{CreateLobby, JoinLobby, LobbyCreated, LobbyJoined, LobbyPlayerJoined, LobbyPlayerLeft, StartGame},
        player::{PlayerAction, PlayerResponds},
    },
    zzz::ZipZapZop,
};
use arcstr::ArcStr;
use fastwebsockets::{
    upgrade::UpgradeFut, FragmentCollectorRead, Frame, OpCode, Payload, WebSocketError, WebSocketWrite,
};
use slab::Slab;
use std::sync::Mutex;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{broadcast, mpsc},
};
use tracing::{error, info, instrument, trace};
use triomphe::Arc;

#[derive(Debug)]
struct LobbyStart {
    ready_tx: mpsc::Sender<()>,
    event_tx: mpsc::Sender<PlayerResponds>,
    broadcast_rx: broadcast::Receiver<Arc<[u8]>>,
    /// Number of known players in the game.
    count: usize,
}

impl Clone for LobbyStart {
    fn clone(&self) -> Self {
        let ready_tx = self.ready_tx.clone();
        let event_tx = self.event_tx.clone();
        let broadcast_rx = self.broadcast_rx.resubscribe();
        let count = self.count;
        Self { broadcast_rx, ready_tx, event_tx, count }
    }
}

#[derive(Clone, Debug)]
enum LobbyEvent {
    Start(LobbyStart),
    PlayerJoined(LobbyPlayerJoined),
    PlayerLeft(LobbyPlayerLeft),
}

impl From<LobbyStart> for LobbyEvent {
    fn from(value: LobbyStart) -> Self {
        Self::Start(value)
    }
}

impl From<LobbyPlayerJoined> for LobbyEvent {
    fn from(value: LobbyPlayerJoined) -> Self {
        Self::PlayerJoined(value)
    }
}

impl From<LobbyPlayerLeft> for LobbyEvent {
    fn from(value: LobbyPlayerLeft) -> Self {
        Self::PlayerLeft(value)
    }
}

struct Lobby {
    broadcast_tx: broadcast::Sender<LobbyEvent>,
    players: Slab<ArcStr>,
}

#[instrument(skip(ws_reader, ws_writer))]
async fn create_lobby<Reader, Writer>(
    ws_reader: &mut FragmentCollectorRead<Reader>,
    ws_writer: &mut WebSocketWrite<Writer>,
    lid: usize,
    pid: usize,
) -> anyhow::Result<usize>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    let bytes = rmp_serde::to_vec(&LobbyCreated { lid, pid })?;
    ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;

    // TODO: Relay lobby events to the Host.

    let payload = match ws_reader.read_frame(&mut send_fn).await.unwrap() {
        Frame { fin: true, opcode: OpCode::Binary, payload, .. } => payload,
        Frame { fin, opcode, payload, .. } => {
            error!(fin, ?opcode, ?payload, "unexpected frame format");
            anyhow::bail!("unexpected frame format");
        }
    };

    let StartGame { count } = rmp_serde::from_slice(&payload)?;
    Ok(count)
}

#[instrument(skip(lobbies, upgrade))]
pub async fn host_actor(lobbies: &Mutex<Slab<Lobby>>, upgrade: UpgradeFut, broadcast_capacity: usize) {
    let (ws_reader, mut ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let mut ws_reader = FragmentCollectorRead::new(ws_reader);

    let payload = match ws_reader.read_frame(&mut send_fn).await.unwrap() {
        Frame { fin: true, opcode: OpCode::Binary, payload, .. } => payload,
        Frame { fin, opcode, payload, .. } => {
            error!(fin, ?opcode, ?payload, "unexpected frame format");
            return;
        }
    };

    // TODO: Register the lobby name.
    let CreateLobby { player, lobby } = rmp_serde::from_slice(&payload).unwrap();
    info!(%lobby, %player, "player requested the lobby creation");

    let broadcast_tx = broadcast::Sender::new(broadcast_capacity);
    let mut players = Slab::with_capacity(1);

    let pid = players.insert(player);
    let lid = lobbies.lock().unwrap().insert(Lobby { broadcast_tx, players });

    let result = create_lobby(&mut ws_reader, &mut ws_writer, lid, pid).await;
    let Lobby { broadcast_tx: start_tx, players } = lobbies.lock().unwrap().remove(lid);
    let count = match result {
        Ok(count) => count,
        Err(err) => {
            error!(?err, "lobby creation failed");
            return;
        }
    };

    if count != players.len() {
        error!(count, "game was started with an incorrect number of players");
        return;
    }

    // TODO: Detach Host from the Lobby as a Player.

    let (broadcast_tx, broadcast_rx) = broadcast::channel(count);
    let (event_tx, mut event_rx) = mpsc::channel(count);
    let (ready_tx, mut ready_rx) = mpsc::channel(count);

    match start_tx.send(LobbyStart { ready_tx, event_tx, broadcast_rx, count }.into()) {
        Ok(count) => info!(count, "dispatched game start to listeners"),
        Err(_) => {
            error!("no receivers for game start");
            return;
        }
    }

    // Mark the end of events, but note that some events may have slipped through before dropping.
    drop(start_tx);

    // TODO: Timeout
    assert_eq!(ready_rx.recv().await, None, "no messages expected from game ready channel");
    drop(ready_rx);
    info!("all players are ready to play");

    let mut zzz = ZipZapZop::new(players, pid);
    handle_game(&mut event_rx, &broadcast_tx, &mut zzz).await;
}

#[instrument(skip(ws_writer))]
async fn send_known_players<Writer>(
    ws_writer: &mut WebSocketWrite<Writer>,
    pid: usize,
    snapshot: Slab<ArcStr>,
) -> Result<(), WebSocketError>
where
    Writer: AsyncWrite + Unpin,
{
    let bytes = rmp_serde::to_vec(&LobbyJoined { pid }).unwrap();
    ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;

    for (pid, player) in snapshot {
        let bytes = rmp_serde::to_vec(&LobbyPlayerJoined { pid, player }).unwrap();
        ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;
    }

    Ok(())
}

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

#[instrument(skip(lobbies, upgrade))]
pub async fn guest_actor(lobbies: &Mutex<Slab<Lobby>>, upgrade: UpgradeFut) {
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

    let (mut broadcast_rx, pid, snapshot) = {
        let mut guard = lobbies.lock().unwrap();
        let Some(Lobby { broadcast_tx, players }) = guard.get_mut(lid) else {
            error!(lid, "lobby does not exist");
            return;
        };

        let clone = players.clone();
        let pid = players.insert(player.clone());

        let count = broadcast_tx.send(LobbyPlayerJoined { pid, player }.into()).expect("lobby must still be alive");
        trace!(count, "broadcasted player joined event to receivers");

        (broadcast_tx.subscribe(), pid, clone)
    };

    'lobby: {
        if let Err(err) = send_known_players(&mut ws_writer, pid, snapshot).await {
            error!(?err, "websocket writer error when sending known players");
            break 'lobby;
        }

        let LobbyStart { ready_tx, event_tx, broadcast_rx, count } =
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
                Ok(false) => break 'game,
                Err(err) => {
                    error!(?err, "websocket writer error while waiting for round trip ping");
                    break 'game;
                }
            }

            // Signal to the lobby that this player is ready
            drop(ready_tx);

            // TODO: Play the game.

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
        let Lobby { broadcast_tx, players } = guard.get_mut(lid).expect("lobby must still exist");

        let player = players.remove(pid);
        info!(%player, "player has prematurely left the lobby");

        let count = broadcast_tx
            .send(LobbyPlayerLeft { pid }.into())
            .expect("lobby event broadcast channel must still be alive");
        trace!(count, "broadcasted player leave event to receivers");
    }
}
