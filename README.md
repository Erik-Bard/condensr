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

Every stable release publishes a multi-architecture (`linux/amd64` and
`linux/arm64`) API image to GitHub Container Registry. The image requires an
existing Postgres database; it applies this project's migrations at startup.

```bash
docker run --detach --name condensr-api --restart unless-stopped \
  --publish 8080:8080 \
  --env-file .env \
  ghcr.io/erik-bard/condensr:1.0.0
```

Set at least `DATABASE_URL` and `BASE_URL` in `.env`; see
[`.env.example`](.env.example). Replace `1.0.0` with a published version and
pin production deployments to that exact version (or to the image digest shown
on the package page). `latest` is provided for evaluation only.

The first published package is private by default. A maintainer must make it
public once in its GitHub package settings; after that, users can pull it
without signing in.

## Releasing

Releases are tag-driven. Once the desired commit is on `main` and its checks
have passed, create and push a semantic-version tag:

```bash
git tag -a v1.0.0 -m "v1.0.0"
git push origin v1.0.0
```

The release workflow verifies the tagged code, builds and publishes the API
image, generates build provenance, then creates the GitHub Release with
generated notes. It publishes the image tags `1.0.0` and `1.0`; stable releases
also update `latest`. Pre-releases such as `v1.1.0-rc.1` do not update `latest`.
There is no manual Docker build, registry login, or GitHub Release step.

## License

This project is licensed under the [MIT License](LICENSE). You may use, modify,
and self-host it, subject to the terms of that license.
