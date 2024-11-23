# Development

## Environment Variables

Before running the game server, the following environment variables may be set.

| **Name**   | **Description**                                                | **Example** |
| ---------- | -------------------------------------------------------------- | ----------- |
| `RUST_LOG` | A [specially formatted][rust-log] filter string for game logs. | `trace`     |
| `PORT`     | The TCP port to which the game server will bind.               | `3000`      |

[rust-log]: https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html

## Running the Web Server

```bash
# Download dependencies and build the game server.
cargo build --release

# Run the game server at the specified port.
cargo run --release
```

## Linting the Codebase

```bash
# Format the code.
cargo fmt

# Lint the code
cargo clippy
```
