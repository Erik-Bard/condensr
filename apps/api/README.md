# condensr API

Axum REST API over Postgres (sqlx). Shortens URLs, lists recent links, and
serves the short-code redirects. Short codes are the base-62 encoding of the
row id, implemented in [`crates/condensr-core`](../../crates/condensr-core).

## Endpoints

| Method | Path           | Description                                        |
| ------ | -------------- | -------------------------------------------------- |
| `GET`  | `/health`      | Liveness check, returns `{"status":"ok"}`          |
| `POST` | `/api/shorten` | Shorten a URL. Body: `{"url":"https://..."}`       |
| `GET`  | `/api/links`   | List the 100 oldest links                          |
| `GET`  | `/{code}`      | `307` redirect to the original URL                 |

`POST /api/shorten` is idempotent: URLs are normalized (WHATWG-canonical) and
stored behind a `UNIQUE(long_url)` index, so the same URL always yields the
same code. The first request returns `201`, repeats return `200`.

```bash
curl -X POST http://localhost:8080/api/shorten \
  -H 'content-type: application/json' \
  -d '{"url":"https://example.com"}'
```

```json
{
  "code": "1",
  "short_url": "http://localhost:8080/1",
  "long_url": "https://example.com/"
}
```

## Prerequisites

- Rust 1.94+
- Docker (for Postgres) — or a local Postgres 17
- Optional: [`sqlx-cli`](https://crates.io/crates/sqlx-cli) for offline query
  metadata and migration tooling (`cargo install sqlx-cli --no-default-features --features rustls,postgres`)

## Setup

All commands run from the repository root.

```bash
docker compose up -d db

cp .env.example .env

cargo run -p condensr-api
```

Migrations in [`migrations/`](migrations) apply automatically on startup. When
the API is up you should see `condensr API listening on http://0.0.0.0:8080`
and `curl http://localhost:8080/health` returns `{"status":"ok"}`.

## Configuration

Read from the environment (a `.env` file is loaded in debug builds):

| Variable       | Default                 | Description                                  |
| -------------- | ----------------------- | -------------------------------------------- |
| `DATABASE_URL` | — (required)            | Postgres connection string                   |
| `BASE_URL`     | `http://localhost:8080` | Public origin used to build short links      |
| `PORT`         | `8080`                  | Port the API listens on                      |
| `RUST_LOG`     | —                       | Log filter, e.g. `condensr_api=debug,info`   |

## Tests

```bash
docker compose up -d db
cargo test -p condensr-api -p condensr-core
```

[`tests/api`](tests/api)
holds HTTP contract tests covering every route's inputs, outputs, status
codes, and error shapes (`tests/api/health.rs`, `shorten.rs`, `redirect.rs`,
`links.rs`, `errors.rs`). They run in-process against the real `Router`
(`tower::ServiceExt::oneshot`, no TCP) and each test gets its own throwaway
Postgres database on the compose `db` server, created and migrated via the
same [`pg_database::connect`](src/database/pg_database.rs) the app uses at
startup, then dropped on teardown. Override the target server with
`TEST_DATABASE_URL` (falls back to `DATABASE_URL`, then
`postgres://condensr:condensr@localhost:5432/postgres`).


## sqlx offline query data

The `sqlx::query!` macros normally check queries against a live database at
compile time. This repo pins `SQLX_OFFLINE=true` in
[`.cargo/config.toml`](../../.cargo/config.toml), so builds and rust-analyzer
validate against the checked-in [`.sqlx/`](.sqlx) metadata instead — no
running database (or `DATABASE_URL`) is needed to compile, and the Docker
build relies on the same mechanism.

After changing any SQL query or migration, regenerate the metadata with the
database running (the command bypasses the offline setting itself):

```bash
cd apps/api
cargo sqlx prepare
```

To have the macros check against the live database directly, override the
pin for one command: `SQLX_OFFLINE=false cargo check -p condensr-api`.

## Docker

The image builds from the repository root (it needs the workspace and
`condensr-core`):

```bash
docker build -f apps/api/Dockerfile .
```

Or just use compose, which wires up Postgres too:

```bash
docker compose up -d --build api
```
