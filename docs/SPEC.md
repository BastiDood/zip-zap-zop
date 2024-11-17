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

The client requests for a listing of all open lobbies by connecting to the event stream at `/lobbies`. The endpoint streams the following messages over time. The client must update the user interface accordingly.


```rust
struct LobbyCreated {
    /// Server-specific unique identifier for the lobby.
    lid: usize,
    /// Number of players currently in the lobby (including the host).
    players: usize,
    /// Name of the lobby as a string.
    lobby: Box<str>,
}
```

```rust
struct LobbyDissolved {
    /// Server-specific unique identifier for the dissolved lobby.
    lid: usize,
}
```

> [!CAUTION]
> Note that lobby IDs may be reused when the old one has been dissolved.

#### Create a New Lobby

A host can create a new lobby by connecting to the `/create` WebSocket endpoint. The client must then immediately send the following message to the server.

```rust
struct CreateLobby {
    /// The proposed name of the lobby.
    lobby: Box<str>,
    /// The username of the player.
    player: Box<str>,
}
```

The server immediately responds with the newly created lobby ID.

```rust
struct LobbyCreated {
    /// Unique identifier for the lobby.
    lid: usize,
    /// Unique identifier for the player.
    pid: usize,
}
```

The server then follows up with a stream of lobby events.

```rust
struct LobbyPlayerJoined {
    /// Unique identifier for the new player.
    pid: usize,
    /// The name of the new player.
    player: Box<str>,
}
```

```rust
struct LobbyPlayerLeft {
    /// Unique identifier for the leaving player.
    pid: usize,
}
```

> [!CAUTION]
> Note that player IDs may be reused when the old one has been dissolved.

#### Join an Existing Lobby

A player can join an existing lobby by connecting to the `/join` WebSocket endpoint. The client must then immediately identify themselves to the lobby.

```rust
struct JoinLobby {
    /// Unique identifier for the lobby.
    lid: usize,
    /// The username of the player.
    player: Box<str>,
}
```

The server responds with a player ID.

```rust
struct LobbyJoined {
    /// Name of the lobby.
    lobby: Box<str>,
    /// Unique identifier for the new player.
    pid: usize,
}
```

The server then follows up with a stream of lobby events.

```rust
struct LobbyPlayerJoined {
    /// Unique identifier for the new player.
    pid: usize,
    /// The name of the new player.
    player: Box<str>,
}
```

```rust
struct LobbyPlayerLeft {
    /// Unique identifier for the leaving player.
    pid: usize,
}
```

When the host has begun the game, the server will send each player (including the host) a random UUID for synchronization.

```rust
struct GameStarted {
    /// The number of players currently known by the game server.
    count: usize,
}
```

If the client does not know the same number of peers, then it must disconnect itself from the lobby as a sanity check.

To acknowledge the game start, an empty message must be pinged back to the game server. Once all players have responded, the game starts as in the ["Start the Game"](#start-the-game) section.

If any of the players fail to respond within a timeout, the server treats the player as a graceful self-elimination.

#### Leave the Lobby

To leave the current lobby, the client simply closes the WebSocket connection. There is no need to announce the departure. The server is expected to relay this to the other players in the lobby.

If the host leaves the game, the lobby is dissolved. Everyone in the live feed of open lobbies must be notified.

> [!IMPORTANT]
> If there are no more players left in the lobby, the server must relay this to everyone listening on the live feed of open lobbies.

#### Start the Game

At any point in time, the host may start the game by sending the current number of players in the lobby. This must match the server's internal count. This is done as a sanity check.

```rust
struct StartGame {
    count: usize,
}
```

The server then sends each player a `GameStarted` event.

```rust
struct GameStarted {
    /// The number of players currently known by the game server.
    count: usize,
}
```

If any of the players fail to respond within a timeout, the server treats the player as a graceful self-elimination.

### Game Management

At the start of every turn, the server notifies everyone whose turn it is. This player is expected to respond within

```rust
struct GameExpected {
    /// The player expected to respond.
    pid: usize,
    /// 0 => Zip
    /// 1 => Zop
    /// 2 => Zap
    action: u8,
    /// [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) timestamp of the deadline.
    deadline: Box<str>,
}
```


#### Point to the Next Player

The expected player must respond in the following format.

```rust
struct PlayerResponds {
    /// The next expected player to respond.
    next: usize,
    /// 0 => Zip
    /// 1 => Zop
    /// 2 => Zap
    action: u8,
}
```

#### Eliminate a Player

If (1) an unexpected player responds, (2) an expected player responds incorrectly, or (3) a player disconnects, they must be eliminated from the game. Everyone is notified about this event.

```rust
struct GameEliminated {
    /// The player expected to respond.
    pid: usize,
}
```

The next `GameExpected` message will be of the player who caused the elimination, but must now start at Zip. If no such "previous" player exists, the next Zip is chosen randomly.

> [!NOTE]
> The eliminated player may continue to spectate the game. Ideally, the eliminated players must be put at a lower priority than the actual players in the game.

#### End the Game

The game ends when there is only one player left. At this point, the server closes the connection after the sending the final `GameEliminated` message. The client is expected to render this state properly. The server concludes the game by sending a `GameConcluded` message.

```rust
struct GameConcluded {
    pid: usize,
}
```

## Technical Details

### Host

1. Create the Lobby with (1) a `broadcast` channel for lobby events and (2) an `mpsc` channel for the game ready event.
1. Dissolve the Lobby if any of these steps fail:
   1. Host signals the `GameStart`.
   1. Host detaches from the Lobby in Player mode.
1. Lobby is removed from advertisement.
1. `SYNC-1`: Lobby broadcasts to the Players (1) a new `mpsc` sender to which player actions will be sent and (2) a new `broadcast` receiver to which game events will be sent.
1. `SYNC-2`: Wait for the game ready `mpsc` channel to close.
1. Drop the lone broadcast sender of game events if the following game loop fails.
   1. `SYNC-3`: Broadcast to all players the next expected message.
   1. `SYNC-4`: Wait for a player to respond.
      1. If a player incorrectly responds, notify everyone that this player has been eliminated.
      1. If a player correctly responds, proceed.
1. `SYNC-5`: Lobby reports that the game has concluded with a winner if there is only one player left.
1. Lobby drops the lone broadcast sender.
1. `SYNC-6`: Lobby drops the lone `mpsc` receiver.

### Player

1. Join the Lobby by subscribing to the `broadcast` channel for lobby events.
1. Disconnect from the Lobby if any of these steps fail.
   1. `SYNC-1`: Receive the new `mpsc` sender for player events and `broadcast` receiver for game events.
   1. Ping the WebSocket for client readiness.
   1. Wait for the client to respond back as ready.
   1. Drop own handle of the old `broadcast` channel for lobby events.
1. `SYNC-2`: Drop the game ready `mpsc` channel.
1. Gracefully eliminate self from the game if any of these steps fail.
    1. `SYNC-3`: Receive the next expected message
    1. `SYNC-4`: Relay the broadcasted game expectation.
1. `SYNC-5`: Gracefully exit the game upon announcement of winner.
1. Drop the `broadcast` receiver.
1. `SYNC-6`: Wait for the `mpsc` sender to close.
