use crate::{
    event::{
        game::{GameConcludes, GameEliminates, GameEvent},
        player::PlayerResponds,
    },
    zzz::{GameWinnerError, TickResult, ZipZapZop},
};
use core::fmt::Debug;
use tokio::sync::{
    broadcast::{error::SendError, Sender},
    mpsc::Receiver,
};
use tracing::{error, info, info_span, instrument, trace, warn};
use triomphe::Arc;

#[instrument(skip(broadcast_tx, event_rx))]
async fn handle_game_tick<Player: Debug>(
    broadcast_tx: &Sender<Arc<[u8]>>,
    event_rx: &mut Receiver<PlayerResponds>,
    zzz: &mut ZipZapZop<Player>,
) -> Result<bool, Arc<[u8]>> {
    match zzz.winner() {
        Ok((pid, player)) => {
            info!(pid, ?player, "game concluded with winner");
            let bytes = rmp_serde::to_vec(&GameEvent::from(GameConcludes { pid })).unwrap().into();
            let count = broadcast_tx.send(bytes).map_err(|SendError(bytes)| bytes)?;
            trace!(count, "broadcasted game event");
            return Ok(false);
        }
        Err(GameWinnerError::EmptyLobby) => {
            warn!("exiting from empty lobby");
            return Ok(false);
        }
        Err(GameWinnerError::MorePlayers) => (),
    }

    let expects = *zzz.expects();
    let bytes = rmp_serde::to_vec(&GameEvent::from(expects)).unwrap().into();
    let count = broadcast_tx.send(bytes).map_err(|SendError(bytes)| bytes)?;
    trace!(count, "broadcasted game event");

    while let Some(event) = event_rx.recv().await {
        let span = info_span!("player-event", ?event);
        let _guard = span.enter();

        let pid = event.pid;
        let player = match zzz.tick(event) {
            TickResult::NoOp => {
                info!("game state no-op transition");
                continue;
            }
            TickResult::Proceed => {
                info!("game state successfully transitioned");
                break;
            }
            TickResult::Eliminated(player) => player,
        };

        let bytes = rmp_serde::to_vec(&GameEvent::from(GameEliminates { pid })).unwrap().into();
        let count = broadcast_tx.send(bytes).map_err(|SendError(bytes)| bytes)?;
        trace!(count, "broadcasted game event");

        info!(?player, "player eliminated");
        break;
    }

    Ok::<_, Arc<[u8]>>(true)
}

#[instrument(skip(broadcast_tx, event_rx, zzz))]
pub async fn handle_game<Player: Debug>(
    event_rx: &mut Receiver<PlayerResponds>,
    broadcast_tx: &Sender<Arc<[u8]>>,
    zzz: &mut ZipZapZop<Player>,
) {
    loop {
        match handle_game_tick(broadcast_tx, event_rx, zzz).await {
            Ok(true) => continue,
            Ok(false) => break,
            Err(bytes) => {
                error!(?bytes, "all receivers have been dropped");
                break;
            }
        }
    }
}
