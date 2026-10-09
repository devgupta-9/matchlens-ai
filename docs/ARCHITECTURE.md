# ReactCoach 11 — architecture

## Phase 1 vertical slice

```text
Synthetic events (fictional, checked into Rust source)
    -> reactcoach-match-engine (validation, deterministic ordering, score)
    -> reactcoach-analytics (pure, evidence-computable counters)
    -> Axum JSON snapshot + Server-Sent Events endpoints
    -> Next.js client (scoreboard, pitch, timeline)
```

Rust workspace:
- `crates/decision-engine`: validated synthetic traits, scenario action scores,
  discrete opponent best-response arbitration, and separate offside-position geometry.
- `crates/shared`: serializable event, statistics, snapshot contracts.
- `crates/analytics`: counts discrete actions; explicitly avoids a misleading time-possession estimate.
- `crates/match-engine`: validates coordinates/IDs and computes reproducible snapshots at any time.
- `crates/agents`: planned roles and handoff schema. **No real model calls implemented.**
- `apps/api`: Axum REST/SSE API with tracing and origin-restricted CORS.
- `apps/web`: Next.js presentation.

The decision-coaching path is separate from synthetic match replay:

```text
Editable synthetic assessed-player and opponent profiles + scenario
    -> Axum validation (16 KB request limit; public inputs marked synthetic)
    -> Rust evaluates three actions using the same starting profiles
    -> Rust evaluates two counters per action and selects the strongest counter
    -> Counter suppression reduces the action's heuristic fit index
    -> Original / recommended action, index-point delta, and template coaching text
    -> Next.js comparison and opponent counter review
```

The legacy decision lab compares a single decision with a discrete response. Its
scoring model remains unchanged. The additive Phase 2A simulation path below owns
position, velocity, ball transport and elapsed-time state.
Profile `acceleration` is a synthetic trait index, not a physical acceleration.
Offside is a separate scalar position check at the pass instant; it is not a
complete Law 11 offence assessment. Coaching explanations are deterministic
templates. No Foundry agents execute these calculations or generate text yet.

Scenario changes cancel pending evaluations. Profile editing is paused during
evaluation, and unapplied edits are labeled. Replay snapshot responses are applied
only if they belong to the most recently requested snapshot.

## Phase 2A spatial decision simulation

```text
Validated versioned synthetic profiles + initial world state + bounded config
    -> integer mm / 100ms decision-engine::simulation movement controller
    -> spatial feasibility screen + existing opponent strength/weakness indices
    -> defender counter, then assessed-player adaptation and another counter
    -> same-start baseline and corrective branches, full tick/event evidence
    -> possession / bypass / progress / clearance objective comparison
    -> additive Axum simulation API (four bounded blocking workers)
```

`simulation/contracts.rs` owns typed Serde contracts; `motion.rs` owns integer
movement, braking, ball segments and parameter conversions; `rollout.rs` owns
validation, opponent policy, adaptation and comparison. Traces expose participant
IDs and world snapshots for a future animated pitch. No frontend redesign or
external AI service is part of this milestone. Law 11 assessment remains separate.

See [the versioned contract and modeling assumptions](SPATIAL_SIMULATION_CONTRACT.md)
and [Phase 2A verification](PHASE_2A_VERIFICATION.md). The supporting teammate is
stationary; movement budgets use conservative L1 geometry; response feasibility
is a horizon preview and does not guarantee timely interception. These are explicit
uncalibrated modeling conventions, not realistic biomechanics or success probabilities.

## Data integrity and AI safety

1. All events are explicitly labeled synthetic, and no proprietary match feed is required.
2. Raw event records are the source of truth; never ask an LLM to invent numeric match statistics.
3. Evidence identifiers, confidence and distinct 'observed vs interpreted' fields belong in Phase 3.
4. Current action share is the proportion of logged actions and must not be described as possession.
5. The demo replays a short fictional match sequence, not a live league fixture.

## Planned later phases (NOT yet shipped)

- Microsoft Foundry REST integration and credential management.
- Multi-agent interpretation, grounded output validators and trace IDs.
- Azure PostgreSQL/SQLx persistence and a reliable ingestion pipeline.
- Deployment to Azure Container Apps, cloud observability, secret management.
- Broadcast-mode UI, localization and video submission.

## Current API surface

| HTTP | Path | Response |
| --- | --- | --- |
| GET | /api/v1/health | service status |
| GET | /api/v1/decision-lab/demo?scenario=open_play | synthetic matchup (also penalty, free_kick) |
| POST | /api/v1/decision-lab/evaluate | validated editable profiles and matchup report |
| GET | /api/v1/offside/demo | original and corrected offside-position examples |
| POST | /api/v1/simulations/open-play | bounded baseline and adaptive spatial traces |
| GET | /api/v1/matches/demo/events | all demo events |
| GET | /api/v1/matches/demo/snapshot?at=420 | deterministic snapshot at second |
| GET | /api/v1/matches/demo/stream | replay SSE: match-event, replay-complete |

The stream replays events about every 1.25 seconds. Reconnecting begins the demo stream again.
The frontend closes the connection when the completion event arrives.

## Security notes

- The API restricts CORS to `WEB_ORIGIN` (default localhost:3000). Configure it per deployment.
- Never commit model/API credentials. `.env.example` contains placeholders only.
- CORS is not an authentication boundary. All demo endpoints currently use public synthetic data.
- This is a foundation prototype, not a deployed production sports feed.
