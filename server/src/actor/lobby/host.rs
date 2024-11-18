use crate::{
    actor::{
        game::handle_game,
        io::{event_to_websocket_msgpack_actor, websocket_msgpack_to_event_actor},
        lobby::{wait_for_lobby_start, LobbyEvent, LobbyStart},
        send_fn,
    },
    event::{
        game::GameStarted,
        lobby::{CreateLobby, LobbyCreated, StartGame},
        player::PlayerResponds,
        Event,
    },
    router::lobby::{Lobby, LobbyManager},
    zzz::ZipZapZop,
};
use core::time::Duration;
use fastwebsockets::{upgrade::UpgradeFut, FragmentCollectorRead, Frame, OpCode, Payload, WebSocketWrite};
use slab::Slab;
use std::sync::Mutex;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{broadcast, mpsc},
    task::JoinHandle,
    time::timeout,
};
use tracing::{error, info, instrument, trace};

#[instrument(skip(ws_writer, broadcast_rx))]
async fn detach_host<Writer>(
    mut ws_writer: WebSocketWrite<Writer>,
    mut broadcast_rx: broadcast::Receiver<LobbyEvent>,
    pid: usize,
) -> mpsc::Sender<PlayerResponds>
where
    Writer: AsyncWrite + Send + Unpin + 'static,
{
    let LobbyStart { ready_tx, event_tx, mut broadcast_rx, count } =
        wait_for_lobby_start(&mut ws_writer, &mut broadcast_rx)
            .await
            .expect("host websocket connection failed")
            .expect("origin lobby was prematurely closed");
    trace!(count, "game start command received with player count");

    let bytes = rmp_serde::to_vec_named(&Event::from(GameStarted { count })).unwrap();
    ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await.expect("host websocket writer failed");

    // Signal to the lobby that this player is ready
    info!("player is ready");
    drop(ready_tx);

    // Partial detachment of host handlers
    tokio::spawn(async move { event_to_websocket_msgpack_actor(&mut ws_writer, &mut broadcast_rx).await });
    event_tx // lobby must surrender ownership over the `ws_reader`
}

#[instrument(skip(ws_reader, ws_writer, broadcast_rx))]
async fn detach_host_while_waiting_for_start_command<Reader, Writer>(
    ws_reader: &mut FragmentCollectorRead<Reader>,
    mut ws_writer: WebSocketWrite<Writer>,
    broadcast_rx: broadcast::Receiver<LobbyEvent>,
    lid: usize,
    pid: usize,
) -> anyhow::Result<(usize, JoinHandle<mpsc::Sender<PlayerResponds>>)>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin + Send + 'static,
{
    let bytes = rmp_serde::to_vec_named(&Event::from(LobbyCreated { lid, pid }))?;
    ws_writer.write_frame(Frame::binary(Payload::Owned(bytes))).await?;

    // Relay lobby events to the host
    let handle = tokio::spawn(detach_host(ws_writer, broadcast_rx, pid));

    let payload = match ws_reader.read_frame(&mut send_fn).await? {
        Frame { fin: true, opcode: OpCode::Binary, payload, .. } => payload,
        Frame { fin, opcode, payload, .. } => {
            error!(fin, ?opcode, ?payload, "unexpected frame format");
            anyhow::bail!("unexpected frame format");
        }
    };

    let StartGame { count } = rmp_serde::from_slice(&payload)?;
    Ok((count, handle))
}

#[instrument(skip(lobbies, upgrade))]
pub async fn host_actor(lobbies: &Mutex<LobbyManager>, upgrade: UpgradeFut, broadcast_capacity: usize) {
    let (ws_reader, ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let mut ws_reader = FragmentCollectorRead::new(ws_reader);

    let payload = match ws_reader.read_frame(&mut send_fn).await.unwrap() {
        Frame { fin: true, opcode: OpCode::Binary, payload, .. } => payload,
        Frame { fin, opcode, payload, .. } => {
            error!(fin, ?opcode, ?payload, "unexpected frame format");
            return;
        }
    };

    let CreateLobby { player, lobby } = rmp_serde::from_slice(&payload).unwrap();
    info!(%lobby, %player, "player requested the lobby creation");

    let (broadcast_tx, broadcast_rx) = broadcast::channel(broadcast_capacity);
    let mut players = Slab::with_capacity(1);

    let pid = players.insert(player);
    let lid = lobbies.lock().unwrap().lobbies.insert(Lobby { broadcast_tx, players, lobby });

    let result = detach_host_while_waiting_for_start_command(&mut ws_reader, ws_writer, broadcast_rx, lid, pid).await;
    let Lobby { broadcast_tx: start_tx, players, lobby } = lobbies.lock().unwrap().lobbies.remove(lid);
    trace!(%lobby, "lobby removed by host");

    let (count, handle) = match result {
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

    // TODO: Let's more cleverly depend on the player count for the channel capacity.
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

    drop(start_tx);

    // Fulfill the responder half of the host's I/O actor
    match handle.await {
        Ok(event_tx) => {
            tokio::spawn(async move { websocket_msgpack_to_event_actor(&mut ws_reader, &event_tx, pid).await });
            info!("detached host successfully joined");
        }
        Err(err) => {
            drop(ws_reader);
            error!(?err, "detached host failed to join");
        }
    }

    match timeout(Duration::from_secs(4), ready_rx.recv()).await {
        Ok(Some(_)) => unreachable!("no messages expected from game ready channel"),
        Ok(None) => info!("all players are ready to play"),
        Err(err) => error!(?err, "timeout elapsed - only some players responded in time"),
    }

    drop(ready_rx);

    let mut zzz = ZipZapZop::new(players, pid);
    handle_game(&mut event_rx, &broadcast_tx, &mut zzz).await;
}
