# ReactCoach 11 — Adaptive Football Decision Intelligence

**Status:** Approved product direction, early implementation. **Project name:** ReactCoach 11 (final name). **Repository:** `devgupta-9/reactcoach-11`.

## Mission

Given an **assessed player**, an **opponent**, and a **football situation**, identify strengths and weaknesses on both sides; compare the assessed player's original decision with possible corrective decisions; model realistic opponent responses; and explain what should change and why.

**Core loop:** Assess → Compare → Simulate → Counter-respond → Recommend → Explain.

**Success metric:** We can show why a proposed correction beats the original *in the same synthetic model and initial situation*, with explicit assumptions, confidence limits, and traceable input ratings. Never call a heuristic fitness index a calibrated match success probability.

## Scenarios

| Module | Inputs | Corrective result | Status |
| --- | --- | --- | --- |
| Open-play matchup | Assessed + opponent ratings, action choices | Change dribble direction or use combination play | Initial deterministic scorer |
| Penalty matchup | Taker + goalkeeper strengths | Compare power, placement and delay | Initial deterministic scorer |
| Free kick | Kicker + goalkeeper / wall model | Compare power, placed curl and short routine | Initial deterministic scorer (wall not dynamically simulated) |
| Offside analysis | Attacker/body edge, ball, second-last opponent at pass instant | Compare early versus correctly timed run | Position-only MVP; NOT an offence decision |
| Full team interactions | Coordinated movement, passing, formations | Team strategies and several opponents | Deferred |

Offside authority: IFAB Law 11, https://theifab.com/laws/latest/offside/. Offside **position** is not the same as an offside **offence**. A lawful referee decision depends on involvement in active play and contextual exceptions not present in the MVP. Our simple scalar geometry is a demonstration and cannot replace player-tracking or officiating.

## Modeling architecture

- **Profiles:** stable ID, role description and nine bounded 0–100 traits: acceleration, technique, passing, finishing, anticipation, positioning, reactions, goalkeeping, composure. Demo values are *invented*, not licensed scouting metrics or a claim about real named footballers.
- **Decision options:** predefined, scenario-specific possible actions with explicit actor and opponent weights (each totaling 100).
- **Reproducible score:** actor fit = weighted assessed trait score; opponent resistance = weighted opponent traits; opponent vulnerability = 100 − resistance; **decision fit index = round((2 × actor fit + vulnerability) / 3)**. 0–100 dimensionless heuristic index, **not probability**.
- **Comparison:** Evaluate all options using exactly the same two player profiles; preserve original action as a fixed baseline, select highest fit with stable ties, show delta in **index points**.
- **Opponent reactions:** for each action, evaluate two discrete opponent counters. The opponent chooses the largest effectiveness index from the same player profiles (stable tie-break). Response fit = round((3 × opponent weighted fit + (100 − assessed player's weighted escape fit)) / 4); suppression = round(response fit / 7). The attacking decision fit is reduced by the chosen suppression. This is synthetic best-response arbitration, not a physically simulated or learned response policy. Multi-step spatial interaction remains pending. Do not claim neural reflex measurement or calibrated real-world predictions.
- **Evidence:** include contributing traits and opponent vulnerabilities in output. After a real data source is introduced, require provenance, sample size, data licence, confidence and uncertainty.
- **Rules layer:** Offside checks separate from subjective recommendations. Never have an LLM decide whether offside law applies.

## Agent roles (planned)

1. **Scenario interpreter:** Turns a user intent into supported structured inputs; never invent ratings.
2. **Counterfactual planner:** Requests deterministic simulations and compares feasible options.
3. **Opponent response challenger:** Searches for counters and failure modes.
4. **Evidence/rules verifier:** Checks conclusions against modeled inputs, calculations and applicable laws.
5. **Coaching explainer:** Writes actionable instructions in fan or analyst language.

Microsoft Foundry will be the language/agent layer, not the authority for mathematical scores or the Laws of the Game. These agents are NOT implemented in the current prototype.

## Differentiators and product UX

A dual-player comparison with strengths and weaknesses, a selection of match situations, a visible original vs recommended decision, index-point delta, opponent's possible response, and side-by-side explanations. We should let users edit synthetic profiles and scene geometry later and see recommendations re-evaluated. In the current foundation, bundled demo profiles supply the first repeatable fixtures.

## Acceptance criteria (first interactive vertical slice)

- [x] Three modeled decision scenarios available from Rust.
- [x] Validated synthetic profiles, reproducible recommendation and baseline comparison.
- [x] Offside **position-only** geometry module with rule-limit messaging.
- [ ] Full browser verification: user selects a scenario and sees recommendation, opponent response and limitations.
- [ ] Input validation for user-edited traits and scenario state.
- [x] Deterministic discrete opponent best-response arbitration with two counter-options per attacking action.
- [ ] Multi-step opponent action/response simulation, spatial physics and trajectories.
- [ ] Microsoft Foundry agent workflows and evidence checker.
- [ ] Data-backed real-player profiles, if legal access/licence can be established.
- [ ] Full end-to-end tests and Azure deploy.

## Hackathon scope discipline

Ship one *deep* adaptive matchup demo, with penalty/free-kick baseline variants and a carefully labeled offside-position analyzer. Defer full 22-player physics, tracking-data training, advanced VAR officiating and any claim of exact real-athlete reflex prediction. The demo should let judges modify a modeled weakness and see the suggested decision change.

## Data and ethical controls

This project uses fictitious player profiles and match events; named real athletes may be contextual examples but do not receive unsourced performance scores. Never claim endorsement by Premier League, Microsoft, clubs, athletes or governing bodies. Do not expose external APIs or credentials in frontend code or commits.
