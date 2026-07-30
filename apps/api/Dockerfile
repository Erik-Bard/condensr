FROM rust:1.94.1-slim-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps/api ./apps/api
COPY apps/desktop/src-tauri/Cargo.toml ./apps/desktop/src-tauri/Cargo.toml
RUN mkdir -p apps/desktop/src-tauri/src \
    && touch apps/desktop/src-tauri/src/lib.rs \
    && echo 'fn main() {}' > apps/desktop/src-tauri/src/main.rs \
    && echo 'fn main() {}' > apps/desktop/src-tauri/build.rs
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked -p condensr-api

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 condensr \
    && useradd --uid 10001 --gid 10001 --no-create-home \
        --home-dir /nonexistent --shell /usr/sbin/nologin condensr
WORKDIR /app
COPY --from=builder --chown=10001:10001 /build/target/release/condensr-api /usr/local/bin/condensr-api
USER 10001:10001
EXPOSE 8080
CMD ["condensr-api"]
