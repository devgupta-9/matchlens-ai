# Phase 2A verification and handoff

Verified locally on 8 October 2026, on `feat/phase-2-spatial-rollout`, forked from
the clean, fetched `feat/phase-1-foundation` commit
`bc2994d18f8428f5654f1e5894f7b58ae9d77381`. Phase 2 remains local, with no push or
merge into main, deployment, paid service or global AI configuration change.
The user separately authorized merging Phase 1 PR #1; that integration is tracked
below and does not publish Phase 2 implementation.

## Proper Tasks and sub-tasks

| Proper Task | Status | Completed sub-tasks |
| --- | --- | --- |
| 1. Environment/capabilities | Done | Branch/remote/worktree inspection; requested docs, Rust/API/Playwright contracts reviewed; Phase 1 baseline checks passed; local Phase 2 branch created; peer discovery checked |
| 2. Spatial state engine | Done | Versioned integer world state; bounded movement and braking; synthetic conversions; time/ownership/profile/geometry validation |
| 3. Multi-step tactical engine | Done | Initial wide/direct action; feasible defender counter; later attacker adaptation; second response; same-start branches; transparent objective and nullable recommendation |
| 4. Evidence | Done | Stable tick/event/branch IDs; full snapshots; considered counters and actions; derived assumptions, outcomes and limitations |
| 5. API | Done | Additive POST route; structured errors; 16 KiB input limit; four blocking workers; two-second response timeout; duration/branch bounds |
| 6. Verification | Done | Required Rust and web checks; 61 Rust tests and 14 unchanged desktop/mobile browser checks; live endpoint determinism and validation smoke checks |
| 7. Review/handoff | Done | Numerical/state/API/spec review; original scoring preserved; architecture, roadmap, contracts, examples and limitations documented |
| Bounded independent peer review | Blocked | Repository outside configured delegation roots; proceeded independently as instructed |

Phase 2A is complete within its stated scope. Phase 2B remains pending.

## Implementation and files

- `crates/decision-engine/src/simulation/`: `contracts.rs`, `motion.rs`, `rollout.rs`,
  `mod.rs`, `tests.rs`. State contracts, integer movement, opponent policy,
  counterfactual orchestration and 29 focused simulation tests.
- `crates/decision-engine/src/lib.rs`: exports the additive module only; existing
  trait weights and scoring calculations are unchanged.
- `crates/decision-engine/examples/open_play.rs`: prints the fictional request or
  complete deterministic report without an API server.
- `crates/decision-engine/Cargo.toml` and `Cargo.lock`: existing `serde_json` used
  for tests/examples; lock regenerated offline, format version 4. No new runtime dependency.
- `apps/api/src/simulations.rs`: typed handler, structured errors, bounded worker
  capacity and six new API contract tests. `main.rs` merges the new router.
- `apps/api/Cargo.toml`: enables Tokio's existing `sync` feature for the semaphore.
- `README.md`, `ARCHITECTURE.md`, `ROADMAP.md`, `REACTCOACH_11_PRODUCT_SPEC.md`:
  update shipped scope and preserve the legacy model's distinctions.
- `SPATIAL_SIMULATION_CONTRACT.md`, this handoff, and `docs/examples/`:
  contracts, assumptions, reproducible request and observed outcome/event summary.

No frontend source or existing Playwright test was changed.

## Architecture and contracts

`POST /api/v1/simulations/open-play` takes profiles, `initial_state`, `config`, and
`scenario_version: open_play_spatial_v1`. It returns the echoed validated input,
derived parameters, original trace, feasible/infeasible alternatives, optional
recommended correction/trace ID, and evidence-based comparison. Every branch has
the same origin, profiles, initial action, first response and time horizon.

The pitch convention is 105 m by 68 m, attack toward +x, with 100 ms ticks and
integer millimetres. All movement and control parameters are synthetic. Speed and
acceleration budgets use conservative L1 geometry with per-axis braking reserves.
The objective prioritizes settled friendly possession, then defender bypass,
forward ball progress and clearance. Losing or unsettled branches cannot improve
the objective through the opponent's ball progress. Baseline wins equal objectives.

Read [the contract](SPATIAL_SIMULATION_CONTRACT.md) for all conversions, thresholds,
input constraints, errors and execution limits. Participant IDs, velocities, ball
ownership and tick snapshots are sufficient for future comparison animation.
Offside position assessment and legacy penalty/free-kick models remain separate.

## Observed deterministic example

Use [the generated request](examples/open-play-request.json) or the Rust example.
The fictional attacker has acceleration 92, passing 85 and technique 63; the
defender has acceleration 69, positioning 94 and anticipation 94. Starting geometry:
attacker (45,34) m, defender (52,36.5) m, stationary teammate (57,26) m.
Both branches begin with `wide_right`, adapt at 1200 ms and finish at 4000 ms.

| Evidence | Original continuation | Corrected `release_pass` |
| --- | --- | --- |
| First defender decision | 100 ms: `close_wide_angle` | Same decision and shared starting trace |
| Attacker adaptation | Continues original plan | 1200 ms: pass released to supporting teammate |
| Defender adapts | Holds selected counter target | 1300 ms: `intercept_passing_lane` |
| Possession result | 1600 ms: defender takes control | 2500 ms: teammate controls completed pass |
| Friendly possession at horizon | False | True |
| Immediate defender bypass | False | True |
| Raw forward ball displacement | 3028 mm (opponent ownership) | 12000 mm |
| Ball-to-defender L1 clearance | 0 mm | 11471 mm |
| Objective tuple | `(false,false,0,0)` | `(true,true,12,11)` |

The final ball is at (48.028,37.028) m with the defender in the baseline, and
(57,26) m with the supporting teammate after the pass. The recommendation is
supported by preserved possession and actual territorial movement, not a fabricated
success percentage. A feasible one-two initiates both passes, but fails to finish
with settled possession and is not recommended. A no-teammate variant has no
improving corrective branch and returns no recommendation.

The actual outcome components, final world states and selected/filtered response
events are in [the live-generated summary](examples/open-play-summary.json).
The endpoint and example return full traces, not just this summary.

## Checks executed

Windows Rust commands used the existing `scripts/cargo.ps1` wrapper. Final checks:

| Check | Result |
| --- | --- |
| `fmt --all -- --check` | Passed |
| `clippy --workspace --all-targets --locked -- -D warnings` | Passed; zero warnings |
| `test --workspace --locked` | 61 passed, 0 failed, 0 ignored |
| `npm run lint` | Passed |
| `npm run typecheck` | Passed |
| `npm run build` | Passed; production Next.js build |
| `CI=true npm run test:e2e` | 14 passed, 0 failed; desktop and mobile Chromium |
| Live POST of checked-in fixture, repeated | Identical complete output; pass recommendation matches engine evidence |
| Live early adaptation with reactions 0 | HTTP 422; adaptation cannot precede/ coincide with first defender response |

Rust totals: agents 1, analytics 2, API 13 (7 existing + 6 new), decision engine 41
(12 existing + 29 new), match engine 4, shared 0. All doc-test targets also passed,
with zero doc-tests. The 26 Phase 1 tests remain, with 35 additions.

Browser tests ran against freshly started servers and a newly built API, not the
old Phase 1 process. They verified scenario changes, edited profiles and counters,
offside-position demonstrations, replay completion, stale-response regressions,
visible errors/recovery and API boundaries. Browser tests do not visualize the new
simulation, which has no new frontend screen. No manual visual redesign review is claimed.

Coverage includes repeat determinism, shared counterfactual prefixes, elapsed time
and IDs, boundary braking, moving starts, turn acceleration/speed limits, ball
ownership and segment interception, feasibility and fallback counters, profile-based
counter changes, stable ties, adaptation timing, honest no-improvement outcomes,
missing teammates, one-two failure, extreme integer and malformed/non-finite JSON
inputs, unknown nested fields, inconsistent state, resource limits and serialization.

Verification caught and resolved destination overshoot in the movement controller.
Review additionally prevented opponent progress from recommending a losing branch,
and rejected an adaptation coinciding with the first defender response. Regression
tests preserve these behaviors. The final live check identified an older running
binary; it was rebuilt and restarted before confirming the new timing validation.

## Peer capability result

Exposed `peer_capabilities` reported bridge **v0.3.0**, Antigravity CLI **1.3.1**,
fresh (not cached) model discovery including `gemini-3.1-pro-high`, and efforts
`low`, `medium`, `high`. `delegate_peer` exposes explicit model, effort,
selectionReason and execution mode, with delegation depth limited to one.

Workspace preflight returned:
`Git repository is outside configured delegation roots: D:\reactcoach-11\reactcoach-11`.
Account execution/authentication therefore remains unverified. No peer was invoked,
no review findings are attributed to a peer, and no global environment was repaired
or bypassed. Codex remained the sole implementation and review owner.

## Limitations and Phase 2B

- Uncalibrated profiles, diamond-shaped movement budgets, 1 m control radius and
  policy delays are explicit modeling conventions, not real player measurements.
- Two active players and one stationary receiver; no body collision, aerial ball,
  spin, stochastic passing error, fouls, goalkeeper, goals or full-team physics.
- Defender targets are held between two observation decisions. Reachability is a
  horizon screen; endpoint defender positions approximate segment interception.
- The objective credits supporting-player possession; this is local attacking
  continuity and territory, not the assessed player's eventual scoring probability.
- Loose-ball recovery and unfinished passes are unresolved tactical states. One-two
  returns use a fixed rendezvous, and the moving attacker can miss it honestly.
- Existing offside analysis remains position-only. No Law 11 offence is inferred
  from the sparse open-play simulation.
- Input, worker and trace bounds are tested. Full production load/security testing,
  authentication, Phase 2 remote CI, LLM explanation and cloud deployment remain outside scope.

Recommended Phase 2B: first consume these traces in an interactive pitch with shared
playback controls and visible assumptions; then add editable geometry, passing-angle
controls, moving-receiver/rendezvous cases and more precise interception timing, with
focused numerical/property tests. Keep Foundry explanations dependent on verified
event evidence and defer their integration until this visual scenario is reviewable.
