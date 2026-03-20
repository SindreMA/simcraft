# SimHammer

SimulationCraft made simple. Run sims from your browser or download the desktop app.

**[simhammer.com](https://simhammer.com)** · **[Download Desktop App](https://github.com/sortbek/simcraft/releases/latest)**

## Features

- **Quick Sim** — Paste your SimC addon string, get DPS and ability breakdown
- **Top Gear** — Find the best gear combination from your bags, bank, and vault
- **Stat Weights** — See which stats matter most for your character
- **Desktop App** — Run everything locally with all your CPU cores, no server needed

## Project Structure

```
simhammer/
├── web/           Web app (Python backend + Next.js frontend + Docker)
├── desktop/       Desktop app (Tauri + Rust backend)
```

## Web App

### Quick Start (Docker)

```bash
git clone <repo-url> simhammer
cd simhammer/web
cp .env.example .env
docker compose up --build
```

- Frontend: http://localhost:3000
- API: http://localhost:8000

### Deploy to a VPS

1. Clone the repo on your server
2. Create `web/.env` with `SERVER_IP=your-domain.com`
3. Run `docker compose up -d --build`
4. Set up nginx as reverse proxy (port 80 → 3000 for frontend, /api/ → 8000 for backend)

## Desktop App

### Download

Grab the latest installer from [GitHub Releases](https://github.com/sortbek/simcraft/releases/latest).

### Build from Source

Prerequisites: Rust, Node.js 20+, simc binary

```bash
# Build simc from source
git clone --depth 1 https://github.com/simulationcraft/simc.git
cd simc && mkdir build && cd build
cmake .. -G "Visual Studio 17 2022" -A x64
cmake --build . --config Release --target simc
# Copy simc.exe to desktop/src-tauri/resources/simc/

# Download game data
# Place Raidbots JSON files in desktop/src-tauri/resources/data/

# Build the app
cd simhammer/desktop
npm install
npx tauri build
```

The installer will be in `desktop/src-tauri/target/release/bundle/nsis/`.

## Getting a SimC Addon String

1. Install the [SimulationCraft addon](https://www.curseforge.com/wow/addons/simulationcraft) in WoW
2. In-game, type `/simc`
3. Copy the full text from the popup window
4. Paste it into SimHammer

## Architecture

### Web
```
Browser → Next.js (3000) → FastAPI (8000) → Redis → ARQ Worker → simc
```

### Desktop
```
Tauri Window → Next.js → Rust HTTP Server (17384) → simc subprocess
```

Both use the same frontend. The desktop app replaces Python + Redis with a single Rust binary that runs simc directly using all available CPU cores.
