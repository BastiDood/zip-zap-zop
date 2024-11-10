use crate::{
    event::{
        game::GameExpects,
        player::{PlayerAction, PlayerResponds},
    },
    zzz::{TickResult, ZipZapZop},
};
use slab::Slab;

#[test]
fn non_existent_player_should_noop() {
    let mut players = Slab::new();
    let pid = players.insert(());
    let key = players.vacant_key();

    let mut zzz = ZipZapZop::new(players, pid);
    assert_eq!(zzz.tick(PlayerResponds { pid: key, next: key, action: PlayerAction::Zip }), TickResult::NoOp);
    assert_eq!(*zzz.expects(), GameExpects { curr: pid, action: PlayerAction::Zip });
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(pid).copied(), Some(()));
}

#[test]
fn non_curr_player_should_be_eliminated() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(
        zzz.tick(PlayerResponds { pid: next, next: curr, action: PlayerAction::Zip }),
        TickResult::Eliminated("next")
    );
    assert_eq!(*zzz.expects(), GameExpects { curr, action: PlayerAction::Zip });
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(next), None);
}

#[test]
fn curr_player_graceful_elimination() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(
        zzz.tick(PlayerResponds { pid: curr, next: curr, action: PlayerAction::Zip }),
        TickResult::Eliminated("curr")
    );
    assert_eq!(*zzz.expects(), GameExpects { curr: next, action: PlayerAction::Zip });
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(curr), None);
}

#[test]
fn curr_player_should_be_eliminated_from_valid_lobby_for_non_existent_next() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");
    let key = players.vacant_key();

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(
        zzz.tick(PlayerResponds { pid: curr, next: key, action: PlayerAction::Zip }),
        TickResult::Eliminated("curr")
    );
    assert_eq!(*zzz.expects(), GameExpects { curr: next, action: PlayerAction::Zip });
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(curr), None);
}

#[test]
fn curr_player_should_be_eliminated_from_valid_lobby_for_unexpected_action() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(zzz.tick(PlayerResponds { pid: curr, next, action: PlayerAction::Zap }), TickResult::Eliminated("curr"));
    assert_eq!(*zzz.expects(), GameExpects { curr: next, action: PlayerAction::Zip });
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(curr), None);
}

#[test]
fn successful_transition() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(zzz.tick(PlayerResponds { pid: curr, next, action: PlayerAction::Zip }), TickResult::Proceed);
    assert_eq!(*zzz.expects(), GameExpects { curr: next, action: PlayerAction::Zap });
    assert_eq!(zzz.len(), 2);
    assert_eq!(zzz.players.get(curr).copied(), Some("curr"));
    assert_eq!(zzz.players.get(next).copied(), Some("next"));
}

#[test]
fn multiple_successful_transitions() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");
    let mut zzz = ZipZapZop::new(players, curr);

    assert_eq!(zzz.tick(PlayerResponds { pid: curr, next, action: PlayerAction::Zip }), TickResult::Proceed);
    assert_eq!(*zzz.expects(), GameExpects { curr: next, action: PlayerAction::Zap });
    assert_eq!(zzz.len(), 2);
    assert_eq!(zzz.players.get(curr).copied(), Some("curr"));
    assert_eq!(zzz.players.get(next).copied(), Some("next"));

    assert_eq!(zzz.tick(PlayerResponds { pid: next, next: curr, action: PlayerAction::Zap }), TickResult::Proceed);
    assert_eq!(*zzz.expects(), GameExpects { curr, action: PlayerAction::Zop });
    assert_eq!(zzz.len(), 2);
    assert_eq!(zzz.players.get(curr).copied(), Some("curr"));
    assert_eq!(zzz.players.get(next).copied(), Some("next"));

    assert_eq!(zzz.tick(PlayerResponds { pid: curr, next, action: PlayerAction::Zop }), TickResult::Proceed);
    assert_eq!(*zzz.expects(), GameExpects { curr: next, action: PlayerAction::Zip });
    assert_eq!(zzz.len(), 2);
    assert_eq!(zzz.players.get(curr).copied(), Some("curr"));
    assert_eq!(zzz.players.get(next).copied(), Some("next"));
}
