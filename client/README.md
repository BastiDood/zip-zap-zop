# Development

## Environment Variables

In a `.env` file, the following variables must be defined before building the web application.

| **Name**                        | **Description**                                            | **Example**            |
| ------------------------------- | ---------------------------------------------------------- | ---------------------- |
| `PUBLIC_ZZZ_WEBSOCKET_BASE_URL` | The WebSocket base URL where the game server is hosted in. | `ws://localhost:3000/` |

## Running the Web Server

```bash
# Install the dependencies.
pnpm install

# Synchronize auto-generated files from SvelteKit.
pnpm sync

# Start the development server with live reloading + hot module replacement.
pnpm dev

# Compile the production build (i.e., with optimizations).
pnpm build

# Start the production preview server.
pnpm preview
```

## Linting the Codebase

```bash
# Check Formatting
pnpm fmt # prettier

# Apply Formatting Auto-fix
pnpm fmt:fix # prettier --write

# Check Linting Rules
pnpm lint:html   # linthtml
pnpm lint:css    # stylelint
pnpm lint:js     # eslint
pnpm lint:svelte # svelte-check

# Check All Lints in Parallel
pnpm lint
```
