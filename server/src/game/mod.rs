mod lobby;
mod player;

pub use arcstr::ArcStr;
pub use lobby::LobbyEvent;
pub use player::PlayerEvent;

use slab::Slab;
use tokio::sync::broadcast;
use tracing::{debug, error, instrument, warn};

#[derive(Debug)]
pub struct Lobby<P> {
    sender: broadcast::Sender<PlayerEvent>,
    name: ArcStr,
    players: Slab<P>,
}

impl<P> Lobby<P> {
    fn player_count(&self) -> usize {
        self.players.len()
    }
}

impl Lobby<ArcStr> {
    #[instrument]
    fn add_player(&mut self, name: ArcStr) -> usize {
        let id = self.players.insert(name.clone());
        let count = self
            .sender
            .send(player::PlayerJoined { id: id.try_into().unwrap(), name }.into())
            .expect("at least one receiver had just been created");
        debug!(count, "notified receivers about new player in the lobby");
        id
    }

    #[instrument]
    fn add_player_with_subscription(&mut self, name: ArcStr) -> (usize, broadcast::Receiver<PlayerEvent>) {
        let receiver = self.sender.subscribe();
        let id = self.add_player(name);
        (id, receiver)
    }

    #[instrument]
    fn remove_player(&mut self, id: usize) -> Option<bool> {
        drop(self.players.try_remove(id)?);
        Some(match self.sender.send(player::PlayerLeft { id: id.try_into().unwrap() }.into()) {
            Ok(count) => {
                debug!(count, "notified lobby listeners of disconnected player");
                true
            }
            Err(_) => {
                warn!("last player to leave the lobby");
                false
            }
        })
    }
}

#[derive(Debug)]
pub struct LobbyManager<P> {
    sender: broadcast::Sender<LobbyEvent>,
    lobbies: Slab<Lobby<P>>,
}

impl<P> LobbyManager<P> {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender, lobbies: Slab::new() }
    }
}

impl<P: core::fmt::Debug> LobbyManager<P> {
    #[instrument]
    pub fn dissolve_lobby(&mut self, id: usize) -> bool {
        let Some(lobby) = self.lobbies.try_remove(id) else {
            return false;
        };

        drop(lobby);
        match self.sender.send(lobby::LobbyDissolved { id: id.try_into().unwrap() }.into()) {
            Ok(count) => debug!(count, "notified game listeners of dissolved lobby"),
            Err(_) => warn!("no game listeners for dissolved lobby"),
        }

        true
    }
}

impl LobbyManager<ArcStr> {
    #[instrument]
    pub fn init_lobby(
        &mut self,
        capacity: usize,
        lobby_name: ArcStr,
        player_name: ArcStr,
    ) -> (usize, usize, broadcast::Receiver<PlayerEvent>) {
        let (sender, receiver) = broadcast::channel(capacity);
        let mut lobby = Lobby { sender, name: lobby_name.clone(), players: Slab::new() };
        let player_id = lobby.add_player(player_name);

        let entry = self.lobbies.vacant_entry();
        let lobby_id = entry.key();
        let player_count = entry.insert(lobby).player_count();

        match self.sender.send(
            lobby::LobbyCreated {
                id: lobby_id.try_into().unwrap(),
                name: lobby_name,
                players: player_count.try_into().unwrap(),
            }
            .into(),
        ) {
            Ok(count) => debug!(count, "notified game listeners of new lobby"),
            Err(_) => warn!("no game listeners for new lobby"),
        }

        (lobby_id, player_id, receiver)
    }

    #[instrument]
    pub fn remove_player_from_lobby(&mut self, lobby_id: usize, player_id: usize) -> bool {
        let Some(lobby) = self.lobbies.get_mut(lobby_id) else {
            error!("lobby does not exist");
            return false;
        };

        let Some(is_valid) = lobby.remove_player(player_id) else {
            error!("player does not exist");
            return false;
        };

        match self.sender.send(
            lobby::LobbyUpdated { id: lobby_id.try_into().unwrap(), players: lobby.player_count().try_into().unwrap() }
                .into(),
        ) {
            Ok(count) => debug!(count, "notified game listeners of updated lobby with removed player"),
            Err(_) => warn!("no game listeners for new lobby"),
        }

        if !is_valid {
            assert!(self.dissolve_lobby(lobby_id));
        }

        true
    }

    #[instrument]
    pub fn join_player_into_lobby(
        &mut self,
        lobby_id: usize,
        player_name: ArcStr,
    ) -> Option<(usize, broadcast::Receiver<PlayerEvent>)> {
        let Some(lobby) = self.lobbies.get_mut(lobby_id) else {
            error!("lobby does not exist");
            return None;
        };

        let result = lobby.add_player_with_subscription(player_name);

        match self.sender.send(
            lobby::LobbyUpdated { id: lobby_id.try_into().unwrap(), players: lobby.player_count().try_into().unwrap() }
                .into(),
        ) {
            Ok(count) => debug!(count, "notified game listeners of updated lobby with new player"),
            Err(_) => warn!("no game listeners for new lobby"),
        }

        Some(result)
    }
}
