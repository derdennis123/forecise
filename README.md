# Forecise - Precise Forecasting Intelligence

The single source of truth for prediction market intelligence. Aggregated odds, accuracy tracking, and AI-powered consensus forecasts.

## Quick Start

### Prerequisites
- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- [Rust](https://rustup.rs/) (1.85+)
- [Node.js](https://nodejs.org/) (22+)

### 1. Clone & Setup

```bash
git clone https://github.com/derdennis123/forecise.git
cd forecise
cp .env.example .env
```

### 2. Start Infrastructure

```bash
docker compose up -d
```

This starts:
- **PostgreSQL + TimescaleDB** on port 5432 (with full schema auto-migration)
- **Redis** on port 6379

### 3. Start Backend API

```bash
cargo run --bin forecise-api
```

API runs on `http://localhost:3001`

### 4. Start Data Workers (separate terminal)

```bash
cargo run --bin forecise-workers
```

Workers pull data from Polymarket, Metaculus, and Manifold Markets every 5-10 minutes.

### 5. Start Frontend (separate terminal)

```bash
cd frontend
npm install
npm run dev
```

Frontend runs on `http://localhost:3000`

## Architecture

```
┌─────────────────────────────────────────┐
│  Data Workers (Rust)                    │
│  Polymarket │ Metaculus │ Manifold      │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  PostgreSQL + TimescaleDB               │
│  Markets │ Odds History │ Accuracy      │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  API Server (Rust/Axum)     :3001       │
│  /api/markets │ /api/accuracy │ ...     │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  Frontend (Next.js)         :3000       │
│  Dashboard │ Markets │ Leaderboard      │
└─────────────────────────────────────────┘
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/markets` | List markets (paginated, filterable) |
| GET | `/api/markets/:id` | Market detail with sources + consensus |
| GET | `/api/markets/:id/odds` | Historical odds (time-series) |
| GET | `/api/markets/:id/sources` | Source market breakdown |
| GET | `/api/accuracy/leaderboard` | Accuracy rankings by Brier Score |
| GET | `/api/consensus/:market_id` | Latest consensus forecast |
| GET | `/api/consensus/:market_id/history` | Consensus history |

## Tech Stack

| Component | Technology |
|-----------|-----------|
| API Server | Rust (Axum) |
| Data Workers | Rust (tokio, reqwest) |
| Consensus Engine | Rust |
| Time-series DB | TimescaleDB (PostgreSQL) |
| Cache | Redis |
| Frontend | Next.js 15, React 19, Tailwind CSS v4 |
| Charts | TradingView Lightweight Charts |

## Project Structure

```
forecise/
├── crates/
│   ├── api/            # Axum REST API server
│   ├── consensus/      # Consensus engine + Brier Score
│   ├── shared/         # Shared types, models, config
│   └── workers/        # Data ingestion (Polymarket, Metaculus, Manifold)
├── frontend/           # Next.js dashboard
├── migrations/         # SQL schema (auto-loaded by Docker)
├── docker-compose.yml  # PostgreSQL + Redis
└── Cargo.toml          # Rust workspace
```
