# condensr

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