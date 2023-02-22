FROM lukemathwalker/cargo-chef:latest-rust-1.66.0-bullseye AS chef
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y  \
      cmake \
      pkg-config \
      libssl-dev \
      git \
      clang \
      openssl \
      postgresql-client \
    && rustup toolchain install nightly-aarch64-unknown-linux-gnu \
    && cargo install --version=0.6.2 sqlx-cli --no-default-features --features native-tls,postgres \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS  builder
WORKDIR /app
ENV SQLX_OFFLINE=true \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse \
    CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo +nightly chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo +nightly build --release

FROM debian:bullseye-20221219-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      openssl \
      jq \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*


# backend app
FROM runtime AS backend
WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend/scripts/entrypoint.sh ./
COPY --from=builder /app/backend/config ./config
RUN chmod +x entrypoint.sh
ENTRYPOINT ["./entrypoint.sh"]

# indexer app
FROM runtime AS indexer
WORKDIR /app
COPY --from=builder /app/target/release/indexer /app/indexer/scripts/entrypoint.sh ./
COPY --from=builder /app/indexer/config ./config
RUN chmod +x entrypoint.sh
ENTRYPOINT ["./entrypoint.sh"]

# migrator
FROM chef AS migrator
WORKDIR /app
COPY ./backend/migrations ./backend/scripts/init_db_compose.sh ./
RUN chmod +x init_db_compose.sh
ENTRYPOINT ["./init_db_compose.sh"]
