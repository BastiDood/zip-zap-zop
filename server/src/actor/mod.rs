pub mod game;
pub mod io;
pub mod lobby;

/// Used for split streams in [`fastwebsockets`].
async fn send_fn<T>(_: T) -> Result<(), &'static str> {
    Err("unexpected obligated write")
}
