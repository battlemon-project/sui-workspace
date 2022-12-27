FROM lukemathwalker/cargo-chef:latest-rust-1.66.0-bullseye AS chef
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y  \
      cmake \
      pkg-config \
      libssl-dev \
      git \
      clang

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin battlemon_backend

FROM debian:bullseye-20221219-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
      openssl \
      jq \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/battlemon_backend /app/scripts/* ./
COPY config ./config

RUN chmod +x entry_point.sh

ENTRYPOINT ["./entry_point.sh"]
