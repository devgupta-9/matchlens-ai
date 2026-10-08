# MatchLens AI

**Beyond the scoreline. Understand the game.**

A Rust-powered synthetic football intelligence prototype for Microsoft's **Inside the Game** developer hackathon.

## Current status

**Phase 1 foundation — in progress.** An Axum service streams synthetic match events and computes
deterministic match snapshots; a Next.js dashboard renders the replay. No Microsoft Foundry
model calls, Azure deployments, real football data, or fully fledged tactical agents are claimed yet.

## Stack

- Rust (Axum, Tokio, Serde) for API, deterministic replay and statistics.
- Next.js 16 / React 19 / TypeScript for the interactive dashboard.
- Server-Sent Events for event delivery.
- Microsoft Foundry + Azure PostgreSQL + Azure Container Apps **planned** for later phases.

## Local development

Prerequisites: Rust stable toolchain (1.80+), Node.js 22+ and npm.

Terminal 1:

```sh
cargo run -p matchlens-api
```

Terminal 2:

```sh
cd apps/web
npm install
npm run dev
```

Visit [http://localhost:3000](http://localhost:3000).
The Rust API listens on port 8080, with CORS allowing `http://localhost:3000` by default.
The UI starts a repeatable 9-event synthetic replay when the page loads.
Refresh the page to replay again.

```sh
curl http://localhost:8080/api/v1/health
curl 'http://localhost:8080/api/v1/matches/demo/snapshot?at=420'
curl -N http://localhost:8080/api/v1/matches/demo/stream
```

## Quality checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/web && npm run lint && npm run typecheck && npm run build
```

The CI job runs these checks. Initial dependency lockfiles must still be generated and committed.

## Repository map

- `apps/api` — Rust API / SSE
- `apps/web` — Next.js interface
- `crates/shared` — event contracts
- `crates/analytics` — deterministic metrics
- `crates/match-engine` — validated replay engine
- `crates/agents` — future agent-role contract (no live orchestration yet)
- `docs/ARCHITECTURE.md` — architecture and limitations
- `docs/ROADMAP.md` — hackathon milestones

## Data provenance

All bundled match records, team names, player names and statistics are **fictional and synthetic**.
No club, league or Microsoft endorsement is implied. See [LICENSE](LICENSE).
