# condensr desktop

Tauri 2 desktop app for condensr. The UI is React + TypeScript built with
Vite; the Rust side ([`src-tauri/`](src-tauri)) exposes `shorten` and
`list_links` commands that call the [condensr API](../api) over HTTP.

The UI also runs as a plain web app: [`src/api.ts`](src/api.ts) detects
whether it is inside Tauri and falls back to `fetch()` against the API
directly. That is what the `web` service in the root
[`docker-compose.yml`](../../docker-compose.yml) serves.

## Prerequisites

- Node 22+ and pnpm 10 (`corepack enable` gives you pnpm)
- Rust 1.94+
- Tauri system dependencies — on Windows, WebView2 is preinstalled; for other
  platforms see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)
- A running condensr API (see the [API README](../api/README.md))

## Setup

```bash
cd apps/desktop
pnpm install

pnpm tauri dev
```

`pnpm tauri dev` starts the Vite dev server and opens the desktop window with
hot reload. The first run compiles the Rust side, which takes a few minutes.

The app expects the API at `http://localhost:8080`. Point it elsewhere with:

| Variable           | Used by                 | Default                 |
| ------------------ | ----------------------- | ----------------------- |
| `CONDENSR_API_URL` | Tauri (Rust commands)   | `http://localhost:8080` |
| `VITE_API_URL`     | Browser mode (`fetch`)  | same origin             |

## Browser mode (no Tauri)

```bash
pnpm dev
```

This serves the UI at `http://localhost:1420` without the desktop shell.
Requests go straight from the browser to the API, so either set
`VITE_API_URL=http://localhost:8080` or serve it behind a proxy that forwards
`/api` (the Docker image does the latter with nginx, see
[`nginx.conf`](nginx.conf)).

## Production build

```bash
pnpm tauri build
```

Installers and the standalone binary land in
`src-tauri/target/release/bundle/`.

## Docker (web build only)

The [`Dockerfile`](Dockerfile) builds the Vite frontend and serves it with
nginx, proxying `/api` to the `api` compose service. It is intended to be run
via the root compose file:

```bash
docker compose up -d --build web
```
