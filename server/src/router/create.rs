use crate::game::{player::PlayerJoined, LobbyManager};
use arcstr::ArcStr;
use fastwebsockets::{FragmentCollectorRead, Frame, OpCode};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, error};
use triomphe::Arc;

#[derive(Deserialize)]
struct CreateLobby {
    player: ArcStr,
    name: ArcStr,
}

#[derive(Serialize)]
struct LobbyCreated {
    id: u32,
}

#[derive(Deserialize)]
struct StartGame {
    count: u32,
}

#[derive(Serialize)]
struct GameStarted {
    uuid: ArcStr,
}

#[tracing::instrument(skip(manager, upgrade))]
pub async fn run(manager: Arc<Mutex<LobbyManager<ArcStr>>>, upgrade: fastwebsockets::upgrade::UpgradeFut) {
    let (ws_reader, mut ws_writer) = upgrade.await.unwrap().split(tokio::io::split);
    let mut ws_reader = FragmentCollectorRead::new(ws_reader);

    let mut send_fn = |_| async { unreachable!("unexpected obligated write") };
    let Frame { fin: true, opcode: OpCode::Binary, payload, .. } =
        ws_reader.read_frame::<_, core::convert::Infallible>(&mut send_fn).await.unwrap()
    else {
        unreachable!("unexpected frame format");
    };

    let CreateLobby { player, name } = rmp_serde::from_slice(&payload).unwrap();
    let (lid, pid, mut rx) = manager.lock().unwrap().init_lobby(16, name, player);
    // TODO: Gracefully remove the player from the lobby on panic.

    'host: {
        match ws_writer
            .write_frame(Frame::binary(
                rmp_serde::to_vec(&LobbyCreated { id: lid.try_into().unwrap() }).unwrap().into(),
            ))
            .await
        {
            Ok(_) => debug!(lid, pid, "notified player of newly created lobby"),
            Err(err) => {
                error!(lid, pid, ?err, "player disconnected");
                break 'host;
            }
        }

        let payload = {
            let mut signal = core::pin::pin!(ws_reader.read_frame(&mut send_fn));
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Ok(event) => match ws_writer.write_frame(Frame::binary(rmp_serde::to_vec(&event).unwrap().into())).await {
                            Ok(_) => {
                                debug!(lid, pid, "notified player of player event");
                                continue;
                            }
                            Err(err) => {
                                error!(lid, pid, ?err, "player disconnected");
                                break 'host;
                            }
                        },
                        Err(RecvError::Closed) => {
                            error!(lid, pid, "player event channel closed");
                            break 'host;
                        }
                        Err(RecvError::Lagged(count)) => {
                            error!(lid, pid, count, "player event channel lagged");
                            break 'host;
                        }
                    },
                    frame = &mut signal => match frame {
                        Ok(Frame { fin: true, opcode: OpCode::Binary, payload, .. }) => break payload,
                        Ok(_) => {
                            error!(lid, pid, "unexpected frame format");
                            break 'host;
                        }
                        Err(err) => {
                            error!(lid, pid, ?err, "player disconnected");
                            break 'host;
                        }
                    },
                }
            }
        };

        let StartGame { count } = rmp_serde::from_slice(&payload).unwrap();
        let count = usize::try_from(count).unwrap();
        let expected = manager.lock().unwrap().player_count_of_lobby(lid).unwrap();
        assert_eq!(count, expected); // TODO: Graceful Lobby Dissolve

        // TODO: Gather all players and expect an echo.

        // TODO: Generate UUID
        match ws_writer
            .write_frame(Frame::binary(rmp_serde::to_vec(&GameStarted { uuid: ArcStr::default() }).unwrap().into()))
            .await
        {
            Ok(_) => debug!(lid, pid, "acknowledged game start"),
            Err(err) => {
                error!(lid, pid, ?err, "player disconnected");
                break 'host;
            }
        }
    }

    drop(rx);
    drop(ws_reader);
    drop(ws_writer);
    assert!(manager.lock().unwrap().remove_player_from_lobby(lid, pid));
}
