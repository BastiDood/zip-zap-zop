#[cfg(test)]
mod tests;

use crate::event::{
    game::GameExpected,
    player::{PlayerAction, PlayerResponds, PlayerRespondsWithId},
};
use core::fmt::Debug;
use jiff::Timestamp;
use slab::Slab;
use tracing::{info, instrument, warn};

pub enum GameWinnerError {
    /// Occurs when the lobby has no players to begin with.
    EmptyLobby,
    /// Occurs when there are more players in the lobby to declare a winner.
    MorePlayers,
}

#[derive(Debug)]
pub struct ZipZapZop<Player> {
    players: Slab<Player>,
    curr: usize,
    action: PlayerAction,
}

impl<Player> ZipZapZop<Player> {
    pub const fn new(players: Slab<Player>, curr: usize) -> Self {
        Self { players, curr, action: PlayerAction::Zip }
    }

    /// The action for the next expected message.
    pub const fn expects(&self, deadline: Timestamp) -> GameExpected {
        let Self { curr, action, .. } = *self;
        GameExpected { next: curr, action, deadline }
    }

    pub fn winner(&self) -> Result<(usize, &Player), GameWinnerError> {
        let mut iter = self.players.iter();
        let first = iter.next().ok_or(GameWinnerError::EmptyLobby)?;
        if iter.next().is_none() {
            Ok(first)
        } else {
            Err(GameWinnerError::MorePlayers)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TickResult<Player> {
    /// The game received an invalid player. This should be ignored.
    NoOp,
    /// The player issued a legal command.
    Proceed,
    /// The player was eliminated as a result of this game tick.
    Eliminated(Player),
}

impl<Player: Debug> ZipZapZop<Player> {
    /// Ticks the game state forward.
    ///
    /// * If the `player` does not exist in the lobby, this is a no-op.
    /// * If the `player` is not the expected sender, they will be eliminated.
    /// * If the `player` is equal to `next`, `player` will be gracefully eliminated.
    #[instrument]
    pub fn tick(
        &mut self,
        PlayerRespondsWithId { pid, data: PlayerResponds { next, action } }: PlayerRespondsWithId,
    ) -> TickResult<Player> {
        if !self.players.contains(pid) {
            warn!("player does not exist in the game");
            return TickResult::NoOp;
        }

        let must_reassign = 'eliminate: {
            if pid != self.curr {
                warn!(curr = self.curr, "player eliminated because it is not their turn");
                break 'eliminate false;
            }

            if pid == next {
                warn!("player eliminated due to graceful elimination");
                break 'eliminate true;
            }

            if !self.players.contains(next) {
                warn!("player eliminated due to invalid next player");
                break 'eliminate true;
            }

            if action != self.action {
                warn!(action = ?self.action, "player eliminated due to unexpected action");
                break 'eliminate true;
            }

            self.curr = next;
            self.action = self.action.next();

            info!("successful transition to next turn");
            return TickResult::Proceed;
        };

        let result = self.players.remove(pid);

        if must_reassign {
            let (next, _) = self.players.iter().next().expect("at least one player must be present");
            self.curr = next;
        }

        TickResult::Eliminated(result)
    }
}
