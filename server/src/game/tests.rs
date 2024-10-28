use super::{broadcast::error::RecvError, lobby, player, LobbyManager};

#[test]
fn new_lobby_without_listener_should_have_one_player() {
    let mut manager = LobbyManager::new(1);

    let (lid, pid, mut host_rx, _) = manager.init_lobby(1, arcstr::literal!("Lobby"), arcstr::literal!("Player"));
    assert_eq!(lid, 0);
    assert_eq!(pid, 0);

    let player_event = host_rx.blocking_recv().unwrap();
    assert_eq!(
        player_event,
        player::PlayerJoined { id: pid.try_into().unwrap(), name: arcstr::literal!("Player") }.into()
    );
    assert!(host_rx.is_empty());
}

#[test]
fn new_lobby_with_listener_should_have_one_player() {
    let mut manager = LobbyManager::new(1);
    let mut lobby_rx = manager.subscribe();

    let (lid, pid, mut host_rx, _) = manager.init_lobby(1, arcstr::literal!("Lobby"), arcstr::literal!("Player"));
    assert_eq!(lid, 0);
    assert_eq!(pid, 0);

    let player_event = host_rx.blocking_recv().unwrap();
    assert_eq!(
        player_event,
        player::PlayerJoined { id: pid.try_into().unwrap(), name: arcstr::literal!("Player") }.into()
    );
    assert!(host_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(
        lobby_event,
        lobby::LobbyCreated { id: lid.try_into().unwrap(), name: arcstr::literal!("Lobby"), players: 1 }.into()
    );
    assert!(lobby_rx.is_empty());
}

#[test]
fn join_nonexistent_lobby() {
    let mut manager = LobbyManager::new(1);
    assert!(manager.join_player_into_lobby(0, arcstr::literal!("None")).is_none());
}

#[test]
fn one_player_joins_then_leaves_lobby() {
    let mut manager = LobbyManager::new(2);
    let mut lobby_rx = manager.subscribe();

    let (lid, host_pid, mut host_rx, _) = manager.init_lobby(2, arcstr::literal!("Lobby"), arcstr::literal!("Host"));

    let host_event = host_rx.blocking_recv().unwrap();
    assert_eq!(
        host_event,
        player::PlayerJoined { id: host_pid.try_into().unwrap(), name: arcstr::literal!("Host") }.into()
    );
    assert!(host_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(
        lobby_event,
        lobby::LobbyCreated { id: lid.try_into().unwrap(), name: arcstr::literal!("Lobby"), players: 1 }.into()
    );
    assert!(lobby_rx.is_empty());

    assert!(lobby_rx.is_empty());
    let (player_pid, mut player_rx) = manager.join_player_into_lobby(lid, arcstr::literal!("Other")).unwrap();
    assert_ne!(host_pid, player_pid);

    let player_event =
        player::PlayerJoined { id: player_pid.try_into().unwrap(), name: arcstr::literal!("Other") }.into();

    assert_eq!(host_rx.blocking_recv().unwrap(), player_event);
    assert!(host_rx.is_empty());

    assert_eq!(player_rx.blocking_recv().unwrap(), player_event);
    assert!(player_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(lobby_event, lobby::LobbyUpdated { id: lid.try_into().unwrap(), players: 2 }.into());
    assert!(lobby_rx.is_empty());

    drop(player_rx); // Must be dropped before removing the player.
    assert!(manager.remove_player_from_lobby(lid, player_pid));

    assert_eq!(host_rx.blocking_recv().unwrap(), player::PlayerLeft { id: player_pid.try_into().unwrap() }.into());
    assert!(host_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(lobby_event, lobby::LobbyUpdated { id: lid.try_into().unwrap(), players: 1 }.into());
    assert!(lobby_rx.is_empty());
}

#[test]
fn dissolve_nonexistent_lobby() {
    let mut manager = LobbyManager::<()>::new(1);
    assert!(manager.dissolve_lobby(0).is_none());
}

#[test]
fn host_leaves_lobby() {
    let mut manager = LobbyManager::new(2);
    let mut lobby_rx = manager.subscribe();

    let (lid, pid, mut host_rx, _) = manager.init_lobby(2, arcstr::literal!("Lobby"), arcstr::literal!("Host"));

    let host_event = host_rx.blocking_recv().unwrap();
    assert_eq!(host_event, player::PlayerJoined { id: pid.try_into().unwrap(), name: arcstr::literal!("Host") }.into());
    assert!(host_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(
        lobby_event,
        lobby::LobbyCreated { id: lid.try_into().unwrap(), name: arcstr::literal!("Lobby"), players: 1 }.into()
    );
    assert!(lobby_rx.is_empty());

    drop(host_rx); // Must be dropped before removing the player.
    assert!(manager.remove_player_from_lobby(lid, pid));

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    let lid = lid.try_into().unwrap();
    assert_eq!(lobby_event, lobby::LobbyUpdated { id: lid, players: 0 }.into());
    assert!(!lobby_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(lobby_event, lobby::LobbyDissolved { id: lid }.into());
    assert!(lobby_rx.is_empty());
}

#[test]
fn premature_lobby_dissolution() {
    let mut manager = LobbyManager::new(2);
    let mut lobby_rx = manager.subscribe();

    let (lid, host_pid, mut host_rx, _) = manager.init_lobby(2, arcstr::literal!("Lobby"), arcstr::literal!("Host"));

    let host_event = host_rx.blocking_recv().unwrap();
    assert_eq!(
        host_event,
        player::PlayerJoined { id: host_pid.try_into().unwrap(), name: arcstr::literal!("Host") }.into()
    );
    assert!(host_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(
        lobby_event,
        lobby::LobbyCreated { id: lid.try_into().unwrap(), name: arcstr::literal!("Lobby"), players: 1 }.into()
    );
    assert!(lobby_rx.is_empty());

    assert!(lobby_rx.is_empty());
    let (player_pid, mut player_rx) = manager.join_player_into_lobby(lid, arcstr::literal!("Other")).unwrap();
    assert_ne!(host_pid, player_pid);

    let player_event =
        player::PlayerJoined { id: player_pid.try_into().unwrap(), name: arcstr::literal!("Other") }.into();

    assert_eq!(host_rx.blocking_recv().unwrap(), player_event);
    assert!(host_rx.is_empty());

    assert_eq!(player_rx.blocking_recv().unwrap(), player_event);
    assert!(player_rx.is_empty());

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(lobby_event, lobby::LobbyUpdated { id: lid.try_into().unwrap(), players: 2 }.into());
    assert!(lobby_rx.is_empty());

    assert!(manager.dissolve_lobby(lid).is_some());

    assert_eq!(host_rx.blocking_recv().unwrap_err(), RecvError::Closed);
    assert_eq!(player_rx.blocking_recv().unwrap_err(), RecvError::Closed);

    let lobby_event = lobby_rx.blocking_recv().unwrap();
    assert_eq!(lobby_event, lobby::LobbyDissolved { id: lid.try_into().unwrap() }.into());
    assert!(lobby_rx.is_empty());
}
