FROM node:23.8.0-alpine3.21 AS static
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable pnpm
WORKDIR /app
COPY client/pnpm-lock.yaml .
RUN pnpm fetch
COPY client/package.json .
RUN pnpm install --offline
COPY client .
ENV PUBLIC_ZZZ_WEBSOCKET_BASE_URL=wss://zip-zap-zop.fly.dev
RUN pnpm build && pnpm prune --prod --ignore-scripts

FROM lukemathwalker/cargo-chef:0.1.71-rust-1.84.1-alpine3.21 AS chef
WORKDIR /app

FROM chef AS planner
COPY ./server .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./server .
RUN cargo build --locked --release

FROM gcr.io/distroless/static-debian12:nonroot-amd64
COPY --from=builder /app/target/release/zip-zap-zop /
COPY --from=static /app/build /game
EXPOSE 3000
ENV RUST_LOG="info"
ENV PORT="3000"
CMD ["/zip-zap-zop"]
