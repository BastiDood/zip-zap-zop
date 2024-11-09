#[cfg(test)]
mod tests;

use core::fmt::Debug;
use slab::Slab;
use tracing::{info, instrument, warn};

#[derive(Debug)]
pub struct ZipZapZop<Player> {
    players: Slab<Player>,
    curr: usize,
}

impl<Player> ZipZapZop<Player> {
    pub const fn new(players: Slab<Player>, curr: usize) -> Self {
        Self { players, curr }
    }

    /// The player ID for the next expected message.
    pub const fn curr(&self) -> usize {
        self.curr
    }

    /// The number of players currently in the game.
    pub fn len(&self) -> usize {
        self.players.len()
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
    pub fn tick(&mut self, player: usize, next: usize) -> TickResult<Player> {
        if !self.players.contains(player) {
            warn!("player does not exist in the game");
            return TickResult::NoOp;
        }

        let must_reassign = 'eliminate: {
            if self.curr != player {
                warn!(curr = self.curr, "player eliminated because it is not their turn");
                break 'eliminate false;
            }

            if player == next {
                warn!("player eliminated due to graceful elimination");
                break 'eliminate true;
            }

            if !self.players.contains(next) {
                warn!("player eliminated due to invalid next player");
                break 'eliminate true;
            }

            self.curr = next;
            info!("successful transition to next turn");
            return TickResult::Proceed;
        };

        let result = self.players.remove(player);

        if must_reassign {
            let (next, _) = self.players.iter().next().expect("at least one player must be present");
            self.curr = next;
        }

        TickResult::Eliminated(result)
    }
}
