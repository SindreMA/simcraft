# SimHammer

SimulationCraft web app + desktop app.

## Monorepo Structure
- **web/** — Web app (FastAPI backend + Next.js frontend + Docker)
- **desktop/** — Desktop app (Tauri + Rust backend, bundles simc)

## Architecture

### Web
- **Backend**: Python 3.11 + FastAPI (port 8000)
- **Worker**: ARQ (async Redis queue) processes simc jobs
- **Frontend**: Next.js 14 App Router + TypeScript + Tailwind (port 3000)
- **Database**: SQLite via async SQLAlchemy
- **Queue**: Redis
- **Game Data**: Raidbots static JSON files loaded at startup

### Desktop
- **Backend**: Rust + Actix-web (port 17384)
- **Frontend**: Same Next.js app (static export for Tauri)
- **Sim**: Runs simc directly as subprocess, all CPU cores
- **Game Data**: Bundled JSON files in resources/

## Commands

### Web
```bash
cd web
docker compose up          # Docker
# or manually:
cd backend && uvicorn main:app --reload --port 8000
cd backend && python -m arq worker.tasks.WorkerSettings
cd frontend && npm run dev
```

### Desktop
```bash
cd desktop
npx tauri dev              # Development
npx tauri build            # Build installer
```

## Key Patterns
- Frontend shared between web and desktop via `lib/api.ts` (auto-detects API URL)
- All item/enchant/gem/bonus data from local JSON files, no Wowhead API calls
- Wowhead tooltips loaded client-side (hover popups only)
- Backend serves identical API shape in both Python (web) and Rust (desktop)
- Deploy only triggers on `web/**` changes
- Desktop build uses `output: "export"` with `generateStaticParams` placeholder for `/sim/[id]`
- Gold accent color: `#C8992A`

## Pages
- `/` — Landing page with sim type cards
- `/quick-sim` — Quick Sim (DPS + stat weights)
- `/top-gear` — Top Gear (best gear combination)
- `/sim/[id]` — Sim results with real-time progress
