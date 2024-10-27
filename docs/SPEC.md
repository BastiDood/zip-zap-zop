# Zip Zap Zop

_Zip Zap Zop_ is a very simple game. We start with a lobby of `n > 1` players. During each turn, a player has three ways to respond: **Zip**, **Zap**, or **Zop** (in that order specifically!).

* Player A says Zip and points to Player C.
* Player C says Zap and points to Player B.
* Player B says Zop and points to Player C.
* Player C says Zip and points to Player A.
* And the cycle goes so on...

A player is eliminated if (1) they took too long to respond or (2) they responded in violation of the strict Zip-Zap-Zop order. After each turn, the deadline decays exponentially (e.g., from `10` seconds to `5` seconds to `2.5` seconds and so on). The last player standing wins the game.

> [!NOTE]
> By convention, the first player is chosen randomly (and must therefore be the first Zip). Play continues from there.

## Game Protocol

The game server sends messages in the [MessagePack] format to minimize the size of the payload—an important consideration in any network protocol for a game. Messages are delivered in binary format via [WebSockets].

[MessagePack]: https://msgpack.org/
[WebSockets]: https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API

### Lobby Management

#### List All Open Lobbies

The client requests for a listing of all open lobbies by connecting to the WebSocket endpoint at `/lobbies`. The WebSocket streams the following messages over time. The client must update the user interface accordingly.


```rust
struct LobbyCreated {
    /// Server-specific unique identifier for the lobby.
    id: u32,
    /// Number of players currently in the lobby (including the host).
    players: u32,
    /// Name of the lobby as a string.
    name: Box<str>,
}
```

```rust
struct LobbyDissolved {
    /// Server-specific unique identifier for the dissolved lobby.
    id: u32,
}
```

```rust
struct LobbyUpdated {
    /// Server-specific unique identifier for the updated lobby.
    id: u32,
    /// (Optional) new number of players.
    players: u32,
}
```

> [!CAUTION]
> Note that lobby IDs may be reused when the old one has been dissolved.

#### Create a New Lobby

A host can create a new lobby by connecting to the `/create` WebSocket endpoint. The client must then immediately send the following message to the server.

```rust
struct CreateLobby {
    /// The username of the player.
    player: Box<str>,
    /// The proposed name of the lobby.
    name: Box<str>,
}
```

The server immediately responds with the newly created lobby ID.

```rust
struct LobbyCreated {
    /// Unique identifier for the lobby.
    id: u32,
}
```

The server then follows up with a stream of lobby events.

```rust
struct PlayerJoined {
    /// Unique identifier for the new player.
    id: u32,
    /// The name of the new player.
    name: Box<str>,
}
```

```rust
struct PlayerLeft {
    /// Unique identifier for the leaving player.
    id: u32,
}
```

> [!CAUTION]
> Note that player IDs may be reused when the old one has been dissolved.

#### Join an Existing Lobby

A player can join an existing lobby by connecting to the `/join` WebSocket endpoint. The client must then immediately identify themselves to the lobby.

```rust
struct CreateLobby {
    /// The username of the player.
    player: Box<str>,
}
```

The server then follows up with a stream of lobby events.

```rust
struct PlayerJoined {
    /// Unique identifier for the new player.
    id: u32,
    /// The name of the new player.
    name: Box<str>,
}
```

```rust
struct PlayerLeft {
    /// Unique identifier for the leaving player.
    id: u32,
}
```

When the host has begun the game, the server will send each player (including the host) a random UUID for synchronization.

```rust
struct GameStarted {
    uuid: Box<str>,
}
```

Each client must then acknowledge the game start by echoing this UUID back to the server. Once all players have responded, the game starts as in the ["Start the Game"](#start-the-game) section.

If one of the players do not respond within a timeout, the game must be aborted. The server will simply close the connection.

#### Leave the Lobby

To leave the current lobby, the client simply closes the WebSocket connection. There is no need to announce the departure. The server is expected to relay this to the other players in the lobby.

If the host leaves the game, the lobby is dissolved. Everyone in the live feed of open lobbies must be notified.

> [!IMPORTANT]
> If there are no more players left in the lobby, the server must relay this to everyone listening on the live feed of open lobbies.

#### Start the Game

At any point in time, the host may start the game by sending the current number of players in the lobby. This must match the server's internal count. This is done as a sanity check.

```rust
struct StartGame {
    count: u32,
}
```

The server then sends each player an empty WebSocket frame. If any of the players fail to respond with another empty WebSocket frame in time, the game is aborted. The server immediately closes the connection. Otherwise, the game proceeds as in the ["Game Management"](#game-management) section.

### Game Management

At the start of every turn, the server notifies everyone whose turn it is. This player is expected to respond within

```rust
struct Expect {
    /// The player expected to respond.
    id: u32,
}
```


#### Point to the Next Player

The expected player must respond in the following format.

```rust
struct Proceed {
    /// The next expected player to respond.
    id: u32,
    /// 0 => Zip
    /// 1 => Zop
    /// 2 => Zap
    response: u8,
}
```

#### Eliminate a Player

If (1) an unexpected player responds, (2) an expected player responds incorrectly, or (3) a player disconnects, they must be eliminated from the game. Everyone is notified about this event.

```rust
struct PlayerEliminated {
    /// The player expected to respond.
    id: u32,
}
```

The next `Expect` message will be of the player who caused the elimination, but must now start at Zip. If no such "previous" player exists, the next Zip is chosen randomly.

> [!NOTE]
> The eliminated player may continue to spectate the game. Ideally, the eliminated players must be put at a lower priority than the actual players in the game.

#### End the Game

The game ends when there is only one player left. At this point, the server closes the connection after the sending the final `PlayerEliminated` message. The client is expected to render this state properly.
