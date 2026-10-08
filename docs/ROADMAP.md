# ReactCoach 11 — Hackathon implementation roadmap

**Goal:** submit by 26 October 2026 with time for final review before the 27 October Pacific deadline.

The product is an **adaptive football decision coach**: assessed-player strengths and weaknesses, opponent vulnerabilities, scenario rules, counterfactual options, opponent responses and concrete correction advice.

## Proper Task 1 — Foundation (In progress)
- [x] Rust + Next.js workspace, synthetic replay and Axum/SSE.
- [x] GitHub CI format, lint, test and build passed on initial foundation.
- [ ] Generate and commit Cargo.lock and apps/web/package-lock.json.
- [ ] End-to-end browser replay verification and responsive QA.
- [ ] Update Innovation Studio with the finalized brand, description and repository.

## Proper Task 2 — Adaptive Decision Intelligence (In progress)
- [x] Formal core specification: docs/REACTCOACH_11_PRODUCT_SPEC.md.
- [x] Synthetic player strength/weakness profile contracts.
- [x] Deterministic action fitness comparison for open play, penalties, free kicks.
- [x] Offside *position-only* Law 11 demonstration and explicit limitations.
- [x] Read-only endpoints for synthetic decision demos and offside position.
- [x] Verify all CI checks after the pivot (GitHub Actions run 37739740707).
- [ ] Add user-editable attribute profiles with safe input validation and reranking.
- [ ] Add dynamic opponent best-response simulation (not fixed templates).
- [ ] Add precise spatial movement and play-state constraints.
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
