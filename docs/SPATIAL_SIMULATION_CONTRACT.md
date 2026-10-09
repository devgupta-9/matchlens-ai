# Phase 2A spatial simulation contract

`POST /api/v1/simulations/open-play` accepts `SimulationRequest` and returns
`SimulationReport`. This is an additive API; existing matchup scores, penalty,
free-kick, offside-position and replay contracts are unchanged. No LLM is involved.

The authoritative types live in `crates/decision-engine/src/simulation/contracts.rs`.
The only supported scenario version is `open_play_spatial_v1`. New request objects,
including nested profiles and traits, reject unknown fields. All profiles must be
explicitly synthetic, with unique IDs (1..64 UTF-8 bytes), labels (1..128 bytes),
descriptions (0..1024 bytes), and nine integer ratings in 0..100.

## Coordinate and movement conventions

- Pitch: exactly 105000 by 68000 millimetres, inclusive boundary coordinates.
- Origin: (0,0); attack proceeds in positive x. Left decreases y; right increases y.
- Positions: integer mm; velocity components: integer mm/second. Movement direction
  is the signed velocity vector; a stationary player has no inferred facing angle.
- Fixed tick: 100 ms. Duration: at most 6000 ms (60 transitions, 61 snapshots).
  Adaptation is tick-aligned, at least 500 ms after the start, with more than 500 ms
  remaining. It must occur strictly after the first modeled defender response.
  Combined timing constraints make 1100 ms the shortest valid duration.
- Synthetic acceleration rating `a` maps to speed `4000 + 30*a` mm/s and
  acceleration `2000 + 40*a` mm/s². These uncalibrated parameters are modeling
  conventions, not measurements or predictions of a real athlete.
- The speed budget is the L1 norm `abs(vx)+abs(vy)`. Each axis changes velocity by
  at most half the acceleration budget per tick. These conservative diamond-shaped
  movement constraints also bound Euclidean speed and acceleration, but introduce
  direction-dependent movement. They are not realistic biomechanics.
- Integrate each coordinate by `velocity_component / 10`, truncating toward zero.
  This creates less than 1 mm rounding per axis/tick. There is no floating-point
  calculation or random seed. Outputs are reproducible integer traces.
- Before approaching a target or boundary, the controller reduces its desired
  speed using the remaining discrete braking distance. For axis speed `v` and
  per-tick decrement `d`, it reserves the rounded-up sum of
  `v, max(v-d,0), ...` divided by 10. Initial velocities must admit braking on the
  first tick. Boundary control never clamps the player's position or teleports it.
  A changed tactical target can still be overshot while the player brakes.
- Synthetic reactions `r` map to `(1 + floor((100-r)/25))*100` ms of decision delay.
  This is a policy cadence, not neurological reflex latency.

## Initial state and ball constraints

The attacker starts owning a colocated ball with matching velocity and time zero.
The initial action is `direct_dribble`, `wide_left`, or `wide_right`. Both active
players may have validated initial velocities. The defender must not initially
overlap the ball's 1 m control radius. Teammate profile and state must be supplied
together; the optional teammate starts and remains stationary in this scenario.

Possession is one tagged value: `owned { owner }`,
`in_flight { from, to, target_mm }`, or `loose`. Owner identifiers refer to
`assessed`, `opponent`, and optional `teammate`; profiles are echoed once in the
report input and are not repeated in every snapshot.

Ball transport uses the L1 budget `(14000 + 20*passing)` mm/s, based on the passer's
synthetic rating. Passes travel to a fixed destination. A 1000 mm Euclidean control
radius enables deterministic possession changes. Interception tests the ball's
whole movement segment against the defender's end-of-tick position, stopping the
ball at the actual contact point. Receiving or interception does not snap the ball
to a player's centre; the next controlled movement follows the owner within its
budget. There is no aerial trajectory, spin, random accuracy error or collision
response. Loose balls remain stationary until the defender enters their control
radius. Assessed-player and teammate loose-ball recovery are outside this scenario.

## Multi-step interaction and comparable alternatives

1. Validate the initial wide/direct action against available pitch space.
2. Advance players using bounded movement. After the synthetic observation delay,
   select an opponent counter at the actual state.
3. At `adaptation_at_ms`, evaluate all other actions in stable order:
   direct dribble, wide left, wide right, release pass, one-two, retain.
   Passing requires attacker possession and a supporting teammate 2..25 L1 metres
   away, outside defender control. Availability means the action can be initiated;
   it does not promise a completed pass, successful bypass, or retained possession.
4. Each feasible branch clones the same original state and repeats the identical
   initial action and response. Only its adaptation differs. The defender observes
   that changed action after the same policy delay and selects a new counter.
5. Simulate to the same time horizon and compare every branch with the same objective.

Defensive counters reuse the unchanged legacy strength-versus-weakness indices.
Targets implement central lane hold, challenge, wide tracking, channel closure,
passing-lane interception, and runner/receiver coverage. Each candidate is screened
by a deterministic movement preview: it must finish within 2 L1 metres of its target
within the remaining duration. This is a necessary operational horizon screen,
not proof that the defender reaches the lane before the ball. The first-defined
feasible response wins equal scores; if none is feasible, bounded recovery toward
the ball is the fallback. `legacy_score_source_id` discloses the reused score.
For a single pass, the legacy combination counters are renamed
`intercept_passing_lane` and `close_receiver` to avoid implying a return pass.
Targets are held between the two modeled observation decisions; this is not
continuous perception or an optimal adversarial search.

For a one-two, the attacker runs while the first pass travels. On receipt, the
teammate returns to the attacker's actual position at that instant. The attacker
then brakes toward that rendezvous. If the attacker is outside the control radius
when the ball arrives, it becomes loose. A failed or unfinished return is recorded.
Retain means decelerate toward the current position; it cannot stop momentum instantly.

## Tactical objective and recommendation

Compare a lexicographic tuple in this order:

1. Friendly settled possession (attacker or supporting teammate).
2. Ball at least 1000 mm ahead of the immediate defender in x, with friendly possession.
3. Forward ball progress from the common initial position, in whole-metre buckets.
4. L1 ball-to-defender clearance in whole-metre buckets, capped at 15 metres.

Progress and clearance objective components are zero without friendly possession,
so an opponent's movement cannot improve the assessed player's recommendation.
Raw progress and clearance remain in the evidence. Negative progress uses floor
division. Baseline wins ties, followed by first-defined alternatives. A correction
is returned only for a strictly greater tuple. All comparisons use the same horizon,
initial geometry, profiles, movement policy and objective. This prioritizes avoiding
turnover over territorial gain; it is not expected goals or a success probability.
Whole-metre buckets define ranking resolution, not statistical confidence or a
guaranteed one-metre improvement: a smaller displacement can cross a bucket boundary.
The report preserves the exact millimetre differences so reviewers can judge significance.

## Response and trace

The report contains the validated input, derived movement parameters, assumptions,
baseline, five alternative assessments, optional recommendation and trace ID,
comparison reason, and limitations. Infeasible alternatives have a reason and a
null trace. Each trace contains:

- A stable branch ID and tick IDs (`baseline:step:000`, etc.).
- A complete world snapshot per tick, including time, velocities, ball and action.
- Ordered events with stable branch-local IDs: action selection, adaptation,
  available/filtered counters, chosen response, passing and possession changes.
- The raw outcome components and objective tuple.

The positional trace and participant identifiers can drive a future animated pitch.
Current frontend contracts and screens are unchanged. Event reasons derive from
the actual state, and comparison text makes only an objective improvement claim.
Future coaching explanations must cite these events rather than inventing capabilities.
Law 11 position assessment remains a separate endpoint; these sparse players cannot
establish an offside offence or second-last-opponent geometry.

## Error and resource contract

Errors use `{ "error": { "code", "message", "field" } }` on this endpoint only.
Existing endpoint error behavior is preserved.

| Status | Code | Cause |
| --- | --- | --- |
| 400 / 422 | `invalid_json` | Malformed JSON or typed schema mismatch |
| 413 | `invalid_json` | Request exceeds 16 KiB |
| 415 | `invalid_json` | Missing/unsupported JSON content type |
| 422 | `invalid_scenario` | Invalid version, ratings, geometry, timing or ownership |
| 503 | `simulation_busy` | Four simulation workers already active |
| 504 | `simulation_timeout` | Worker response exceeds two seconds |
| 500 | `simulation_failed` | Unexpected worker failure |

There is no waiting queue. Work runs off Tokio's async executor. A capacity permit
remains owned by the blocking worker even after a response timeout, so a timeout
cannot admit additional work while the original worker continues. The worker is not
forcibly cancelled, but its deterministic loops are bounded: at most six branches,
60 movement transitions each, and two response decisions each with two previews of
at most 60 transitions. Metadata lengths and fixed event schemas bound response
size; API tests enforce a one-MiB ceiling on a maximum-duration fixture. This is not
authentication or comprehensive production denial-of-service protection.

## Reproduce the fictional scenario

Windows:

```powershell
powershell -NoProfile -File scripts/cargo.ps1 run -p reactcoach-decision-engine --example open_play --locked -- request
powershell -NoProfile -File scripts/cargo.ps1 run -p reactcoach-decision-engine --example open_play --locked
$body = Get-Content docs/examples/open-play-request.json -Raw
Invoke-RestMethod http://localhost:8080/api/v1/simulations/open-play -Method Post -ContentType application/json -Body $body
```

On other platforms, replace the wrapper with `cargo`. The checked-in request is
generated from `demo_request()`. The observed baseline and alternatives are recorded
in `PHASE_2A_VERIFICATION.md`; no recommendation is hardcoded to a named player.
