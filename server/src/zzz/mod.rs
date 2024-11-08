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

impl<Player: Debug> ZipZapZop<Player> {
    /// Ticks the game state forward.
    ///
    /// * If the `player` is not the expected sender, they will be eliminated.
    /// * If the `player` is equal to `next`, `player` will be gracefully eliminated.
    #[instrument]
    pub fn tick(&mut self, player: usize, next: usize) -> Option<Player> {
        assert!(self.players.contains(player), "player does not exist in this game");

        let reassign = 'eliminate: {
            if self.curr != player {
                warn!(curr = self.curr, "player eliminated because it is not their turn");
                break 'eliminate false;
            }

            'reassign: {
                if player == next {
                    warn!("player eliminated due to graceful elimination");
                    break 'reassign;
                }

                if !self.players.contains(next) {
                    warn!("player eliminated due to invalid next player");
                    break 'reassign;
                }

                self.curr = next;
                info!("successful transition to next turn");
                return None;
            }

            // Search for the next player in the lobby
            true
        };

        let result = self.players.remove(player);

        if reassign {
            let (next, _) = self.players.iter().next().expect("at least one player must be present");
            self.curr = next;
        }

        Some(result)
    }
}
