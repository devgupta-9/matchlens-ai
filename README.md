# ReactCoach 11 — AI-Powered Adaptive Football Coaching

**Know your strengths. Read your opponent. Make the better move.**

Rust-powered, synthetic football matchup and tactical correction engine for Microsoft's **Inside the Game** hackathon.

> **Brand finalized:** ReactCoach 11. **GitHub repository still named** `matchlens-ai` until the owner renames it to `reactcoach-11` in GitHub settings. The Microsoft Innovation Studio listing must also be updated separately.

## Core value

**Assess → Compare → Simulate → Counter-respond → Recommend → Explain.**

The user starts with an assessed player, a specific opponent and a football situation. ReactCoach 11 compares illustrative player attributes and the opponent's relative vulnerabilities, evaluates alternative actions from the same starting model and recommends a change. It reports a transparent **decision fit index**, **not** a predicted success percentage.

## Implemented initial scope

- Rust engine: synthetic player profiles, bounded 0–100 traits, explicit action/opponent trait weights, deterministic original-vs-recommended comparison.
- Demo modes: open play, penalty, free kick.
- Rule-aware offside **position** check with a timing correction; not a complete offside offence or VAR decision.
- Axum REST API and synthetic match replay over SSE.
- Next.js interface for the decision lab and legacy synthetic match replay.
- Automated format/lint/test/build CI.

**Not implemented:** real-player scouting data, dynamic multi-agent simulation, Microsoft Foundry model calls, physical ball dynamics, calibrated outcome predictions or Azure deployment.

## Requirements

Rust stable (>=1.80), Node.js 22+, npm.

Run API:
```sh
cargo run -p reactcoach-api
```

Run frontend (new terminal):
```sh
cd apps/web
npm install
npm run dev
```

Open http://localhost:3000.

## Synthetic endpoints

```sh
curl http://localhost:8080/api/v1/health
curl 'http://localhost:8080/api/v1/decision-lab/demo?scenario=open_play'
curl 'http://localhost:8080/api/v1/decision-lab/demo?scenario=penalty'
curl 'http://localhost:8080/api/v1/decision-lab/demo?scenario=free_kick'
curl http://localhost:8080/api/v1/offside/demo
curl -N http://localhost:8080/api/v1/matches/demo/stream
```

## CI

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/web && npm run lint && npm run typecheck && npm run build
```

Dependency lockfiles must be generated, committed, and verified before final merge.

## Architecture / scope

- `crates/decision-engine`: matchup model, synthetic ratings, scenario comparisons and offside position.
- `crates/match-engine`: synthetic match replay.
- `crates/analytics`: deterministic match statistics.
- `crates/shared`: existing match event wire types.
- `crates/agents`: future roles (no deployed AI workflows).
- `apps/api`: Rust Axum / SSE.
- `apps/web`: Next.js decision lab and replay.
- [Product specification](docs/REACTCOACH_11_PRODUCT_SPEC.md)
- [Roadmap](docs/ROADMAP.md)

All demo player identities, ratings, movements and outcomes are **fictional**. No claim of a real professional athlete's strengths, weaknesses or reflexes, nor association with clubs, leagues or Microsoft, is implied. [MIT License](LICENSE).
