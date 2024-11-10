use crate::{
    actor::{game::handle_game, send_fn},
    event::{
        lobby::{CreateLobby, LobbyCreated, StartGame},
        player::PlayerResponds,
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
    ready_tx: mpsc::Sender<()>,
    event_tx: mpsc::Sender<PlayerResponds>,
    broadcast_rx: broadcast::Receiver<Arc<[u8]>>,
}

impl Clone for GameStart {
    fn clone(&self) -> Self {
        let ready_tx = self.ready_tx.clone();
        let event_tx = self.event_tx.clone();
        let broadcast_rx = self.broadcast_rx.resubscribe();
        Self { broadcast_rx, ready_tx, event_tx }
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
    let (ready_tx, mut ready_rx) = mpsc::channel(count);

    match start_tx.send(GameStart { ready_tx, event_tx, broadcast_rx }) {
        Ok(count) => info!(count, "dispatched game start to listeners"),
        Err(_) => {
            error!("no receivers for game start");
            return;
        }
    }

    drop(start_tx);

    assert_eq!(ready_rx.recv().await, None, "no messages expected from game ready channel");
    info!("all players are ready to play");

    let mut zzz = ZipZapZop::new(players, pid);
    handle_game(&mut event_rx, &broadcast_tx, &mut zzz).await;
}

#[instrument(skip(lobbies, upgrade))]
pub async fn guest_actor(lobbies: &Mutex<Slab<Lobby>>, upgrade: UpgradeFut) {
    let (ws_reader, ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let ws_reader = FragmentCollectorRead::new(ws_reader);
    todo!()
}
