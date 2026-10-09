# ReactCoach 11 — Hackathon implementation roadmap

**Goal:** submit by 26 October 2026 with time for final review before the 27 October Pacific deadline.

The product is an **adaptive football decision coach**: assessed-player strengths and weaknesses, opponent vulnerabilities, scenario rules, counterfactual options, opponent responses and concrete correction advice.

## Proper Task 1 — Foundation (Done; merged into main)
- [x] Rust + Next.js workspace, synthetic replay and Axum/SSE.
- [x] GitHub CI format, lint, test and build passed on initial foundation.
- [x] Generate and commit Cargo.lock and apps/web/package-lock.json.
- [x] End-to-end browser replay verification and desktop/mobile overflow checks.
- [x] Local Rust formatting, Clippy, workspace tests, Next.js lint/types/build, and browser regressions (8 October 2026).
- [x] Verify Phase 1 head bc2994d remotely: four successful Rust/Next.js check runs on 8 October 2026.
- [x] Rename GitHub repository to reactcoach-11 and confirm branch/PR history.
- [x] Innovation Studio naming and project details updated (owner confirmed).

## Proper Task 2 — Adaptive Decision Intelligence (In progress)
- [x] Formal core specification: docs/REACTCOACH_11_PRODUCT_SPEC.md.
- [x] Synthetic player strength/weakness profile contracts.
- [x] Deterministic action fitness comparison for open play, penalties, free kicks.
- [x] Offside *position-only* Law 11 demonstration and explicit limitations.
- [x] Read-only endpoints for synthetic decision demos and offside position.
- [x] Verify all CI checks after the pivot (GitHub Actions run 37739740707).
- [x] Add user-editable attribute profiles with safe input validation and reranking.
- [x] Add deterministic opponent best-response comparison across two possible counters for each scenario action (discrete traits-based model).
- [x] Phase 2A: same-start spatial rollouts, defender counters, assessed-player adaptation, evidence and additive API.
- [x] Phase 2A: integer movement budgets, boundary braking, ownership validation and bounded execution.
- [x] Phase 2A final audit (9 October 2026): 61 Rust tests, 14 desktop/mobile browser checks, lint/types/build, reproducible fixtures and complete diff review.
- [x] Phase 2A pushed to `feat/phase-2-spatial-rollout`; exact implementation head `9e271da` passed remote push CI (run 37883819787).
- [x] Phase 2A PR #2 targets main; push and PR CI passed on reviewed publication head `4bba7ba` (runs 37888004552 and 37888062360).
- [ ] Phase 2A publication: apply the requested draft status, title and description to PR #2 (GitHub edit form awaits Submit), then confirm final-head CI; merge requires separate approval.
- [ ] Phase 2B: connect spatial evidence to synthetic match events and an animated tactical pitch, with editable geometry and comparison playback.
- [ ] Phase 2B: richer observation cadence and interception timing, with additional geometry/property tests.
- [ ] Add scenario-specific constraints: walls, goalkeeper positions, pass timing.
- [ ] Run product UX tests with realistic synthetic situations.

## Proper Task 3 — Microsoft Foundry Intelligence (Pending)
- [ ] Set up securely scoped Foundry credentials and budget limits.
- [ ] Rust orchestration: interpreter, planner, opponent challenger, rules/evidence verifier, coaching explainer.
- [ ] Structured outputs, citations to synthetic events and model assumptions.
- [ ] Negative tests for fabricated player data, unsupported claims and rules violations.

## Proper Task 4 — Presentation (Pending)
- [ ] Editable on-pitch tactics, comparison timelines and action animation.
- [ ] Fan/analyst coaching modes, accessibility, responsive layouts.
- [ ] Data-provenance explanation and transparent metric definitions.

## Proper Task 5 — Submission (Pending)
- [ ] Azure deployment and observability.
- [ ] Synthetic adversarial scenarios, security and performance audit.
- [ ] Public repository and architecture guide.
- [ ] Under-2-minute demonstration video and Innovation Studio final submission.

**Scope constraint:** This MVP does not infer real-world player psychology, exact human reflexes or success probabilities without valid data.
