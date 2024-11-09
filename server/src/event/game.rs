use crate::event::player::PlayerAction;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GameExpects {
    /// The game expects the player with this ID to respond.
    pub curr: usize,
    pub action: PlayerAction,
}

impl GameExpects {
    pub const fn new(curr: usize) -> Self {
        Self { curr, action: PlayerAction::Zip }
    }

    pub fn tick(&mut self, next: usize) {
        self.set_curr(next);
        self.action = self.action.next();
    }

    pub fn set_curr(&mut self, next: usize) {
        self.curr = next;
    }
}
