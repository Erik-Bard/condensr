---
name: verify
description: Build, launch, and visually verify the condensr desktop app (web mode) end-to-end
---

# Verify condensr-desktop

## Build

```powershell
pnpm --filter condensr-desktop build
```

## Launch (web mode)

Backend: the `condensr-api` Docker container usually already listens on 8080 (`docker compose up -d db api`). A native fallback exists at `target\debug\condensr-api.exe` (needs `DATABASE_URL=postgres://condensr:condensr@localhost:5432/condensr`), but it exits with code 255 if the container already holds the port.

```powershell
pnpm --filter condensr-desktop dev
```

Vite serves http://localhost:1420 and proxies `/api` + `/health` to 8080.

## Drive and screenshot

Static pages via headless Edge (`--user-data-dir` is required or no screenshot is written):

```powershell
& "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe" --headless=new --disable-gpu --no-first-run --user-data-dir="$env:TEMP\edgeprof" --window-size=1280,900 --virtual-time-budget=8000 --screenshot="out.png" http://localhost:1420/
```

Interactive flows via `playwright-core` with the system Edge (no browser download):

```js
import { chromium } from "playwright-core";
const browser = await chromium.launch({ channel: "msedge", headless: true });
```

## Flows worth driving

- Main page: header count, form, table rows with relative timestamps
- Shorten: fill input, submit, expect "Short link ready" banner + highlighted new row (writes a row to the dev DB)
- Redirect page: http://localhost:1420/<code> — use `--virtual-time-budget=600` to capture before the 900ms redirect fires
- Error/empty state: `docker stop condensr-api`, reload, expect error banner + empty card; `docker start condensr-api` after
