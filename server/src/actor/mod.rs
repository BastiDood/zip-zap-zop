pub mod game;
pub mod io;

use tokio::sync::{broadcast, mpsc};

/// One-to-many actor.
pub struct Actor<In, Out> {
    broadcast_rx: broadcast::Receiver<In>,
    mpsc_tx: mpsc::Sender<Out>,
}

/// Handle to a one-to-many actor.
pub struct Handle<In, Out> {
    broadcast_tx: broadcast::Sender<Out>,
    mpsc_rx: mpsc::Receiver<In>,
}

/// Create a one-to-many message-passing model.
pub fn create<ActorToHandle, HandleToActor: Clone>(
    handle_to_actor_capacity: usize,
    actor_to_handle_capacity: usize,
) -> (Actor<HandleToActor, ActorToHandle>, Handle<ActorToHandle, HandleToActor>) {
    let (broadcast_tx, broadcast_rx) = broadcast::channel(handle_to_actor_capacity);
    let (mpsc_tx, mpsc_rx) = mpsc::channel(actor_to_handle_capacity);
    (Actor { broadcast_rx, mpsc_tx }, Handle { broadcast_tx, mpsc_rx })
}
