use crate::{
    actor::{game::handle_game, send_fn},
    event::{
        lobby::{CreateLobby, LobbyCreated, StartGame},
        player::PlayerResponded,
    },
    zzz::ZipZapZop,
};
use arcstr::ArcStr;
use fastwebsockets::{upgrade::UpgradeFut, FragmentCollectorRead, Frame, OpCode, Payload, WebSocketWrite};
use slab::Slab;
use std::sync::Mutex;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{broadcast, mpsc},
};
use tracing::{error, info, instrument};
use triomphe::Arc;

pub struct GameStart {
    broadcast_rx: broadcast::Receiver<Arc<[u8]>>,
    event_tx: mpsc::Sender<PlayerResponded>,
}

impl Clone for GameStart {
    fn clone(&self) -> Self {
        let broadcast_rx = self.broadcast_rx.resubscribe();
        let event_tx = self.event_tx.clone();
        Self { broadcast_rx, event_tx }
    }
}

struct Lobby {
    broadcast_tx: broadcast::Sender<GameStart>,
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

#[instrument(skip(lobbies))]
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
    let CreateLobby { player, name } = rmp_serde::from_slice(&payload).unwrap();
    info!(lobby = %name, %player, "player requested the lobby creation");

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

    match start_tx.send(GameStart { broadcast_rx, event_tx }) {
        Ok(count) => info!(count, "dispatched game start to listeners"),
        Err(_) => {
            error!("no receivers for game start");
            return;
        }
    }

    drop(start_tx);

    // TODO: Synchronize with all players.

    let mut zzz = ZipZapZop::new(players, pid);
    handle_game(&broadcast_tx, &mut event_rx, &mut zzz).await;
}

#[instrument(skip(lobbies, upgrade))]
pub async fn guest_actor(lobbies: &Mutex<Slab<Lobby>>, upgrade: UpgradeFut) {
    let (ws_reader, ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let ws_reader = FragmentCollectorRead::new(ws_reader);
    todo!()
}
