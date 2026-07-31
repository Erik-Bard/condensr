# condensr

[![Core CI](https://github.com/Erik-Bard/condensr/actions/workflows/core-tests.yml/badge.svg?branch=main)](https://github.com/Erik-Bard/condensr/actions/workflows/core-tests.yml?query=branch%3Amain)
[![API CI](https://github.com/Erik-Bard/condensr/actions/workflows/api-tests.yml/badge.svg?branch=main)](https://github.com/Erik-Bard/condensr/actions/workflows/api-tests.yml?query=branch%3Amain)
[![Desktop CI](https://github.com/Erik-Bard/condensr/actions/workflows/desktop-ci.yml/badge.svg?branch=main)](https://github.com/Erik-Bard/condensr/actions/workflows/desktop-ci.yml?query=branch%3Amain)

A URL shortener, built in Rust.

| Part                                             | What it is                                                                 | Docs                                     |
| ------------------------------------------------ | -------------------------------------------------------------------------- | ---------------------------------------- |
| [`crates/condensr-core`](crates/condensr-core)   | Pure, dependency-light logic (base-62 encode/decode). No I/O.               | —                                        |
| [`apps/api`](apps/api)                           | Axum REST API over Postgres (sqlx). Serves shorten, list, and redirects.    | [API README](apps/api/README.md)         |
| [`apps/desktop`](apps/desktop)                   | Tauri desktop app (React + Vite frontend, Rust `src-tauri`). Also runs as a plain web app. | [Desktop README](apps/desktop/README.md) |

```
condensr/
├── Cargo.toml            cargo workspace root
├── crates/
│   └── condensr-core/    pure encode/decode logic
├── apps/
│   ├── api/              axum + sqlx REST API
│   │   ├── migrations/
│   │   └── Dockerfile
│   └── desktop/          tauri app (React + Vite UI)
│       ├── src-tauri/
│       └── Dockerfile    web build of the UI (nginx)
├── docker-compose.yml    full stack: db + api + web
└── .env.example
```

## Just want to try it? (Docker only)

The only prerequisite is Docker. From the repository root:

```bash
docker compose up -d --build
```

That starts:

- **Postgres** on `localhost:5432`
- **API** on [http://localhost:8080](http://localhost:8080/health)
- **Web UI** on [http://localhost:1420](http://localhost:1420)

Open http://localhost:1420, paste a URL, and the short link
(`http://localhost:8080/{code}`) redirects to it. Tear it all down with
`docker compose down` (add `-v` to also drop the database data).

## Developing locally

Prerequisites: Rust 1.94+, Node 22+ with pnpm, Docker (for Postgres).

```bash
docker compose up -d db

cp .env.example .env

set -a; . ./.env; set +a
cargo run -p condensr-api

cd apps/desktop && pnpm install && pnpm tauri dev
```

Full setup, configuration, and troubleshooting details:

- **Backend API** — [apps/api/README.md](apps/api/README.md)
- **Desktop app** — [apps/desktop/README.md](apps/desktop/README.md)

Run the tests with:

```bash
cargo test -p condensr-api -p condensr-core
```

## Released API images

Every stable release publishes a public multi-architecture (`linux/amd64` and
`linux/arm64`) API image to GitHub Container Registry. The image supports
PostgreSQL 14 or later and requires credentials that can apply this project's
migrations at startup. CI verifies PostgreSQL 14 and 17; the local Compose
development database uses PostgreSQL 17.

The default deployment contract is direct environment injection:

```bash
docker run --detach --name condensr-api --restart unless-stopped \
  --publish 8080:8080 \
  --env DATABASE_URL='postgres://user:password@database.example:5432/condensr?sslmode=require' \
  --env BASE_URL='https://short.example' \
  ghcr.io/erik-bard/condensr:VERSION
```

Replace `VERSION` with a published semantic version. Pin production deployments
to that exact version or to the digest shown on the package page. `latest` is
provided for evaluation only.

Docker can read variables from a host file and inject them into the container:

```bash
docker run --detach --name condensr-api --restart unless-stopped \
  --publish 8080:8080 \
  --env-file ./condensr.env \
  ghcr.io/erik-bard/condensr:VERSION
```

Every released image also includes opt-in exact-origin CORS and per-container
shortening throttling. Configure them at container startup; no source checkout
or custom Rust build is required. See the
[API configuration guide](apps/api/README.md#optional-http-policies) for the
environment variables, trusted-proxy behavior, and multi-replica limitation.

## License

This project is licensed under the [MIT License](LICENSE). You may use, modify,
and self-host it, subject to the terms of that license.
