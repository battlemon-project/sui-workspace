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
    && cargo install --version=0.6.2 sqlx-cli --no-default-features --features native-tls,postgres \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

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
COPY --from=builder /app/backend/migrations /app/backend/scripts/init_db_compose.sh ./
RUN chmod +x init_db_compose.sh
ENTRYPOINT ["./init_db_compose.sh"]
