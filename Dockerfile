ARG RUST_VERSION=1.96

FROM rust:${RUST_VERSION} AS build

WORKDIR /app

ENV SQLX_OFFLINE=true

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp /app/target/release/bounty-bot /bin/bounty-bot

FROM debian:trixie-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /bin/bounty-bot ./bounty-bot

# If .env doesn't exist that's fine, probably
COPY .env* ./.env
# Replace with .env.example if not exists
RUN cp -u -p ./.env.example ./.env

CMD ["./bounty-bot"]