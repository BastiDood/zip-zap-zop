use super::{TickResult, ZipZapZop};
use slab::Slab;

#[test]
fn non_existent_player_should_noop() {
    let mut players = Slab::new();
    let pid = players.insert(());
    let key = players.vacant_key();

    let mut zzz = ZipZapZop::new(players, pid);
    assert_eq!(zzz.tick(key, key), TickResult::NoOp);
    assert_eq!(zzz.curr(), pid);
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(pid).copied(), Some(()));
}

#[test]
fn non_curr_player_should_be_eliminated() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(zzz.tick(next, curr), TickResult::Eliminated("next"));
    assert_eq!(zzz.curr(), curr);
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(next), None);
}

#[test]
fn curr_player_graceful_elimination() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(zzz.tick(curr, curr), TickResult::Eliminated("curr"));
    assert_eq!(zzz.curr(), next);
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
    assert_eq!(zzz.tick(curr, key), TickResult::Eliminated("curr"));
    assert_eq!(zzz.curr(), next);
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(curr), None);
}

#[test]
fn successful_transition() {
    let mut players = Slab::new();
    let curr = players.insert("curr");
    let next = players.insert("next");

    let mut zzz = ZipZapZop::new(players, curr);
    assert_eq!(zzz.tick(curr, next), TickResult::Proceed);
    assert_eq!(zzz.curr(), next);
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

    assert_eq!(zzz.tick(curr, next), TickResult::Proceed);
    assert_eq!(zzz.curr(), next);
    assert_eq!(zzz.len(), 2);
    assert_eq!(zzz.players.get(curr).copied(), Some("curr"));
    assert_eq!(zzz.players.get(next).copied(), Some("next"));

    assert_eq!(zzz.tick(next, curr), TickResult::Proceed);
    assert_eq!(zzz.curr(), curr);
    assert_eq!(zzz.len(), 2);
    assert_eq!(zzz.players.get(curr).copied(), Some("curr"));
    assert_eq!(zzz.players.get(next).copied(), Some("next"));

    assert_eq!(zzz.tick(curr, curr), TickResult::Eliminated("curr"));
    assert_eq!(zzz.curr(), next);
    assert_eq!(zzz.len(), 1);
    assert_eq!(zzz.players.get(curr).copied(), None);
    assert_eq!(zzz.players.get(next).copied(), Some("next"));
}
