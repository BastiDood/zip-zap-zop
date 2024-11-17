use crate::{
    event::{
        game::{GameConcluded, GameEliminated, GameExpected},
        player::PlayerResponds,
        Event,
    },
    zzz::{GameWinnerError, TickResult, ZipZapZop},
};
use core::{fmt::Debug, time::Duration};
use jiff::Timestamp;
use tokio::{
    sync::{
        broadcast::{error::SendError, Sender},
        mpsc::Receiver,
    },
    time::timeout,
};
use tracing::{error, info, info_span, instrument, trace, warn};
use triomphe::Arc;

#[instrument(skip(broadcast_tx, event_rx))]
async fn handle_game_tick<Player: Debug>(
    broadcast_tx: &Sender<Arc<[u8]>>,
    event_rx: &mut Receiver<PlayerResponds>,
    zzz: &mut ZipZapZop<Player>,
    round: &mut u32,
) -> Result<bool, Arc<[u8]>> {
    match zzz.winner() {
        Ok((pid, player)) => {
            info!(pid, ?player, "game concluded with winner");
            let bytes = rmp_serde::to_vec_named(&Event::from(GameConcluded { pid })).unwrap().into();
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

    let secs = 4.0 * (-f64::from(*round) / 16.0).exp();
    let duration = Duration::from_secs_f64(secs);
    let deadline = Timestamp::now().saturating_add(duration);

    let expects = zzz.expects(deadline);
    let bytes = rmp_serde::to_vec_named(&Event::from(expects)).unwrap().into();
    let count = broadcast_tx.send(bytes).map_err(|SendError(bytes)| bytes)?;
    trace!(count, "broadcasted game event");

    loop {
        let event = match timeout(duration, event_rx.recv()).await {
            Ok(Some(event)) => event,
            Ok(None) => {
                error!("all players have left the game");
                return Ok(false);
            }
            Err(err) => {
                warn!(?err, "round timeout elapsed - gracefully eliminating current player");
                let GameExpected { next, action, .. } = expects;
                PlayerResponds { pid: next, next, action }
            }
        };

        let span = info_span!("player-event", ?event);
        let _guard = span.enter();

        let pid = event.pid;
        match zzz.tick(event) {
            TickResult::NoOp => {
                info!("game state no-op transition");
                continue;
            }
            TickResult::Proceed => {
                info!("game state successfully transitioned");
                *round += 1; // increment per successful transition
                break;
            }
            TickResult::Eliminated(player) => {
                let bytes = rmp_serde::to_vec_named(&Event::from(GameEliminated { pid })).unwrap().into();
                let count = broadcast_tx.send(bytes).map_err(|SendError(bytes)| bytes)?;
                trace!(count, "broadcasted game event");
                info!(?player, "player eliminated");
                *round = 0; // reset per elimination
                break;
            }
        }
    }

    Ok::<_, Arc<[u8]>>(true)
}

#[instrument(skip(broadcast_tx, event_rx, zzz))]
pub async fn handle_game<Player: Debug>(
    event_rx: &mut Receiver<PlayerResponds>,
    broadcast_tx: &Sender<Arc<[u8]>>,
    zzz: &mut ZipZapZop<Player>,
) {
    let mut round = 0;
    loop {
        match handle_game_tick(broadcast_tx, event_rx, zzz, &mut round).await {
            Ok(true) => continue,
            Ok(false) => break,
            Err(bytes) => {
                error!(?bytes, "all receivers have been dropped");
                break;
            }
        }
    }
}
