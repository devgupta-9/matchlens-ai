# ReactCoach 11 — Hackathon Roadmap

**Status: APPROVED — 2026-10-09.**  
**Execution baseline:** [Approved Hackathon Execution Plan](HACKATHON_EXECUTION_PLAN.md) — authoritative scope, acceptance criteria, constraints, owners and dates.  
**Target submission:** **2026-10-26** (buffer ahead of the competition deadline).

## Final product direction

**ReactCoach 11 — AI-Powered Live Tactical Intelligence**

Synthetic live football replay → meaningful event detection → spatial counterfactual and opponent-response evidence → verified Microsoft Foundry narratives → synchronized match overlay → Fan / Analyst personalization and recap.

The coaching simulator is a differentiator *inside* the match-intelligence pipeline, not a disconnected application.

## Milestones

| Proper Task | Planned dates | Status | Merge / acceptance gate |
| --- | --- | --- | --- |
| 1. Foundation | Completed | **DONE** | Phase 1 PR #1 merged; Rust/API/Next.js/replay/CI baseline |
| 2A. Spatial simulation | Completed | **DONE** | Phase 2A PR #2 merged; 61 Rust tests and 14 browser checks reported; post-merge main CI green |
| **2B. Live synthetic match intelligence** | **Oct 10–13** | **PENDING — NEXT** | Two meaningful events with replay-stable typed insight evidence and authored spatial fixtures |
| 3. Microsoft Foundry intelligence | Oct 13–17 | PENDING | Real inference; versioned grounded narratives; evidence validator; outage fallback |
| 4. Broadcast & audience experience | Oct 17–21 | PENDING | Synchronized overlays, real trace playback, two audience views, recap, mobile |
| 5. Azure deployment & QA | Oct 21–24 | PENDING | Public HTTPS demo; approved cost/credentials; CI/E2E/security/observability |
| 6. Demo & submission | Oct 24–26 | PENDING | Public repo/demo, sub-two-minute video, valid platform submission |

The dates overlap deliberately to allow early Cloud/Foundry access checks and interface integration. They are internal targets, not delivery guarantees.

## Proper Task 2B — Immediate backlog

- [ ] **2B.1** Expand the existing fictional event replay into coherent match moments while preserving established contracts.
- [ ] **2B.2** Detect at least two significant moments from factual synthetic event records with deterministic criteria.
- [ ] **2B.3** Attach openly authored and versioned spatial states to selected match-event IDs; never claim full player tracking was inferred from sparse events.
- [ ] **2B.4** Emit typed evidence: match ID, event ID/time, stats, authored scenario ID/version, comparison trace IDs, outcome, provenance and limitations.
- [ ] **2B.5** Connect evidence to the match stream and replay/seek lifecycle without stale results or duplicate insights.
- [ ] **2B.6** Add deterministic fixture, API and replay regression coverage.
- [ ] **2B gate:** Two match events independently produce stable evidence-backed spatial comparisons. Existing Rust, web and browser tests pass.

**Suggested branch:** `feat/phase-2b-live-match-intelligence`. Start from verified `main`, keep changes on a feature branch and review by PR before merge.

## Cross-cutting setup to start alongside 2B

- [ ] Verify Microsoft Foundry account, deployment regions, available models, costs and quotas.
- [ ] Obtain explicit owner approval *before* provisioning paid Azure resources.
- [ ] Prepare mock and real-model evidence-validation test plans without exposing browser credentials.
- [ ] Choose two end-to-end demo scenarios and evidence fixtures early.
- [ ] Have Agy prepare interface review after backend event/insight contracts are stable, without overlapping file edits.

## Submission readiness

- [ ] Match-triggered synthetic tactical intelligence, at least two moments
- [ ] Evidence-backed original versus corrected spatial replay
- [ ] Genuine verified Foundry explanation and deterministic fallback
- [ ] Fan and Analyst modes grounded in identical facts
- [ ] Broadcast-style overlay and match recap
- [ ] Public tested Azure demo (desktop/mobile)
- [ ] Public repo, demo video under two minutes, project description and platform submission

## Scope freeze and caveats

**Defer:** Full-team tracking/physics, real athlete predictions, unlicensed Premier League data, extensive penalty/free-kick physics, advanced VAR, unnecessary databases, and noncritical effects. Current football ratings and spatial parameters are synthetic and uncalibrated. Do not present decision indices as probabilities or sparse offside geometry as a complete Law 11 decision.

**Status discipline:** Use Proper Task / Sub-task labels and Done, In Progress, Pending, Blocked or Deferred. Tests and delivered artifacts—not plans—determine completion.

### Historical verification

- [Phase 1 verification](PHASE_1_VERIFICATION.md)
- [Phase 2A verification](PHASE_2A_VERIFICATION.md)
- [Spatial simulation contract](SPATIAL_SIMULATION_CONTRACT.md)

This roadmap supersedes the preapproval coaching-only sequence. The detailed approved execution plan is the final scope authority for the hackathon.
