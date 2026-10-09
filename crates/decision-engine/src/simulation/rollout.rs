use super::{motion::*, *};
use crate::{demo_profiles, responses::evaluate_responses, ScenarioKind};

const ACTIONS: [TacticalAction; 6] = [
    TacticalAction::DirectDribble,
    TacticalAction::WideLeft,
    TacticalAction::WideRight,
    TacticalAction::ReleasePass,
    TacticalAction::OneTwo,
    TacticalAction::Retain,
];

fn invalid(field: &'static str, message: &'static str) -> SimulationError {
    SimulationError { field, message }
}

fn validate(r: &SimulationRequest) -> Result<(), SimulationError> {
    if r.scenario_version != SCENARIO_VERSION {
        return Err(invalid("scenario_version", "unsupported scenario version"));
    }
    let c = &r.config;
    if c.pitch.length_mm != 105_000 || c.pitch.width_mm != 68_000 {
        return Err(invalid(
            "config.pitch",
            "v1 requires the documented 105000 by 68000 mm pitch",
        ));
    }
    if c.tick_ms != TICK_MS
        || !(1000..=MAX_DURATION_MS).contains(&c.duration_ms)
        || c.duration_ms % TICK_MS != 0
        || c.adaptation_at_ms < 500
        || c.adaptation_at_ms % TICK_MS != 0
        || c.adaptation_at_ms.saturating_add(500) >= c.duration_ms
    {
        return Err(invalid("config", "require 100 ms ticks, 1000..6000 ms duration and an aligned adaptation with more than 500 ms remaining"));
    }
    let mut ids: Vec<&str> = Vec::new();
    for p in [&r.assessed_player, &r.opponent]
        .into_iter()
        .chain(r.supporting_teammate.iter())
    {
        if !p.synthetic
            || !p.traits.is_valid()
            || p.id.is_empty()
            || p.id.len() > 64
            || p.label.is_empty()
            || p.label.len() > 128
            || p.description.len() > 1024
            || ids.contains(&p.id.as_str())
        {
            return Err(invalid("profiles", "require unique bounded IDs, labels, descriptions, synthetic profiles and ratings 0..100"));
        }
        ids.push(p.id.as_str());
    }
    if c.adaptation_at_ms <= parameters(&r.opponent.traits).decision_delay_ms {
        return Err(invalid(
            "config.adaptation_at_ms",
            "adaptation must occur after the initial opponent response has begun",
        ));
    }
    let s = &r.initial_state;
    if s.elapsed_ms != 0
        || !matches!(
            s.current_action,
            TacticalAction::DirectDribble | TacticalAction::WideLeft | TacticalAction::WideRight
        )
    {
        return Err(invalid(
            "initial_state",
            "scenario starts at time zero with a dribble or wide action",
        ));
    }
    if s.teammate.is_some() != r.supporting_teammate.is_some() {
        return Err(invalid(
            "supporting_teammate",
            "teammate profile and state must be supplied together",
        ));
    }
    for (p, profile) in [
        (&s.assessed, &r.assessed_player),
        (&s.opponent, &r.opponent),
    ]
    .into_iter()
    .chain(s.teammate.iter().zip(r.supporting_teammate.iter()))
    {
        if !valid_player(p, parameters(&profile.traits), c.pitch) {
            return Err(invalid(
                "initial_state.players",
                "invalid position, speed or insufficient boundary braking room",
            ));
        }
    }
    if s.teammate
        .as_ref()
        .is_some_and(|p| p.velocity_mm_s != Vector2::default())
    {
        return Err(invalid(
            "initial_state.teammate",
            "v1 supporting teammate must start stationary",
        ));
    }
    if s.ball.possession
        != (Possession::Owned {
            owner: Participant::Assessed,
        })
        || s.ball.position_mm != s.assessed.position_mm
        || s.ball.velocity_mm_s != s.assessed.velocity_mm_s
    {
        return Err(invalid(
            "initial_state.ball",
            "attacker must initially own a colocated ball with matching velocity",
        ));
    }
    if contact(
        s.ball.position_mm,
        s.ball.position_mm,
        s.opponent.position_mm,
    )
    .is_some()
    {
        return Err(invalid(
            "initial_state.opponent",
            "opponent initially overlaps the ball control radius",
        ));
    }
    let (_, reason) = action_feasibility(s, s.current_action, c.pitch);
    if let Some(reason) = reason {
        return Err(invalid("initial_state.current_action", reason));
    }
    Ok(())
}

fn action_feasibility(
    s: &SimulationState,
    action: TacticalAction,
    pitch: PitchGeometry,
) -> (bool, Option<&'static str>) {
    if s.ball.possession
        != (Possession::Owned {
            owner: Participant::Assessed,
        })
    {
        return (
            false,
            Some("assessed player does not currently own the ball"),
        );
    }
    let pos = s.assessed.position_mm;
    let reason = match action {
        TacticalAction::WideLeft if pos.y < 3000 => Some("insufficient space in the left channel"),
        TacticalAction::WideRight if pos.y > pitch.width_mm - 3000 => {
            Some("insufficient space in the right channel")
        }
        TacticalAction::DirectDribble if pos.x > pitch.length_mm - 2000 => {
            Some("insufficient forward pitch space")
        }
        TacticalAction::ReleasePass | TacticalAction::OneTwo => match &s.teammate {
            None => Some("a supporting teammate is required"),
            Some(t) if !(2000..=25000).contains(&distance(pos, t.position_mm)) => {
                Some("teammate must be 2..25 L1 metres away")
            }
            Some(t) if contact(t.position_mm, t.position_mm, s.opponent.position_mm).is_some() => {
                Some("teammate is inside the defender control radius")
            }
            _ => None,
        },
        _ => None,
    };
    (reason.is_none(), reason)
}

fn target(s: &SimulationState, a: TacticalAction, pitch: PitchGeometry) -> Vector2 {
    let p = s.assessed.position_mm;
    bounded(
        match a {
            TacticalAction::DirectDribble | TacticalAction::OneTwo => Vector2 {
                x: p.x + 15_000,
                y: p.y,
            },
            TacticalAction::WideLeft => Vector2 {
                x: p.x + 12_000,
                y: p.y - 12_000,
            },
            TacticalAction::WideRight => Vector2 {
                x: p.x + 12_000,
                y: p.y + 12_000,
            },
            TacticalAction::ReleasePass | TacticalAction::Retain => p,
        },
        pitch,
    )
}

fn event(trace: &mut SimulationTrace, time: u32, evidence: EventKind) {
    trace.events.push(SimulationEvent {
        id: format!("{}:event:{:03}", trace.id, trace.events.len()),
        elapsed_ms: time,
        evidence,
    });
}

fn choose_response(
    r: &SimulationRequest,
    s: &SimulationState,
    attacker_target: Vector2,
    trace: &mut SimulationTrace,
) -> Vector2 {
    let legacy_action = match s.current_action {
        TacticalAction::WideLeft | TacticalAction::WideRight => "accelerate_wide",
        TacticalAction::ReleasePass | TacticalAction::OneTwo => "quick_combination",
        _ => "direct_dribble",
    };
    let remaining = r.config.duration_ms - s.elapsed_ms;
    let m = parameters(&r.opponent.traits);
    let mut options = Vec::new();
    for response in evaluate_responses(
        ScenarioKind::OpenPlay,
        legacy_action,
        &r.assessed_player.traits,
        &r.opponent.traits,
    ) {
        let p = s.assessed.position_mm;
        let receiver = s.teammate.as_ref().map_or(p, |t| t.position_mm);
        let response_target = bounded(
            match response.id.as_str() {
                "hold_central_lane" => Vector2 {
                    x: p.x + 3000,
                    y: p.y,
                },
                "step_into_tackle" => s.ball.position_mm,
                "track_wide_run" => move_ball(p, attacker_target, 3000),
                "close_wide_angle" => Vector2 {
                    x: p.x + 3000,
                    y: p.y + (attacker_target.y - p.y).signum() * 3000,
                },
                "intercept_return_pass" => Vector2 {
                    x: (p.x + receiver.x) / 2,
                    y: (p.y + receiver.y) / 2,
                },
                "follow_receiving_runner" if s.current_action == TacticalAction::ReleasePass => {
                    receiver
                }
                "follow_receiving_runner" => Vector2 {
                    x: p.x + 3000,
                    y: p.y,
                },
                _ => s.ball.position_mm,
            },
            r.config.pitch,
        );
        // A bounded deterministic movement preview screens out physically unreachable targets.
        let mut preview = s.opponent.clone();
        for _ in 0..remaining / TICK_MS {
            advance(&mut preview, response_target, m, r.config.pitch);
        }
        let feasible = distance(preview.position_mm, response_target) <= 2000;
        options.push(ResponseOption {
            id: match (s.current_action, response.id.as_str()) {
                (TacticalAction::ReleasePass, "intercept_return_pass") => {
                    "intercept_passing_lane".into()
                }
                (TacticalAction::ReleasePass, "follow_receiving_runner") => "close_receiver".into(),
                _ => response.id.clone(),
            },
            legacy_score_source_id: Some(response.id),
            effectiveness_index: response.effectiveness_index,
            target_mm: response_target,
            feasible,
            reason: if feasible {
                "movement preview reaches within 2 L1 metres of target by horizon"
            } else {
                "target unreachable within remaining bounded movement horizon"
            }
            .into(),
        });
    }
    // Strict greater-than keeps first-defined ties stable, unlike Iterator::max_by_key.
    let mut best: Option<&ResponseOption> = None;
    for option in &options {
        if option.feasible
            && best.is_none_or(|b| option.effectiveness_index > b.effectiveness_index)
        {
            best = Some(option);
        }
    }
    let (chosen_response_id, target_mm) = best.map_or_else(
        || ("recover_toward_ball".to_owned(), s.ball.position_mm),
        |b| (b.id.clone(), b.target_mm),
    );
    if best.is_none() {
        options.push(ResponseOption {
            id: chosen_response_id.clone(),
            legacy_score_source_id: None,
            effectiveness_index: 0,
            target_mm,
            feasible: true,
            reason: "bounded recovery movement; no claim of reaching or stopping the attacker"
                .into(),
        });
    }
    event(
        trace,
        s.elapsed_ms,
        EventKind::OpponentDecision {
            observed_action: s.current_action,
            options,
            chosen_response_id,
            target_mm,
        },
    );
    target_mm
}

fn release(
    s: &mut SimulationState,
    from: Participant,
    to: Participant,
    trace: &mut SimulationTrace,
) {
    let target_mm = match to {
        Participant::Assessed => s.assessed.position_mm,
        Participant::Teammate => s.teammate.as_ref().expect("validated teammate").position_mm,
        Participant::Opponent => s.opponent.position_mm,
    };
    s.ball.possession = Possession::InFlight {
        from,
        to,
        target_mm,
    };
    event(
        trace,
        s.elapsed_ms,
        EventKind::PassReleased {
            from,
            to,
            target_mm,
        },
    );
}

fn update_ball(r: &SimulationRequest, s: &mut SimulationState, trace: &mut SimulationTrace) {
    let old = s.ball.position_mm;
    let possession = s.ball.possession.clone();
    let target = match possession {
        Possession::Owned {
            owner: Participant::Assessed,
        } => s.assessed.position_mm,
        Possession::Owned {
            owner: Participant::Opponent,
        } => s.opponent.position_mm,
        Possession::Owned {
            owner: Participant::Teammate,
        } => s.teammate.as_ref().expect("validated teammate").position_mm,
        Possession::InFlight { target_mm, .. } => target_mm,
        Possession::Loose => old,
    };
    let passing = match possession {
        Possession::InFlight {
            from: Participant::Teammate,
            ..
        } => {
            r.supporting_teammate
                .as_ref()
                .expect("validated teammate")
                .traits
                .passing
        }
        _ => r.assessed_player.traits.passing,
    };
    let next = move_ball(old, target, (14000 + 20 * i32::from(passing)) / 10);
    let already_defender = matches!(
        possession,
        Possession::Owned {
            owner: Participant::Opponent
        }
    );
    if !already_defender {
        if let Some(point) = contact(old, next, s.opponent.position_mm) {
            s.ball.position_mm = point;
            s.ball.possession = Possession::Owned {
                owner: Participant::Opponent,
            };
            event(
                trace,
                s.elapsed_ms,
                EventKind::PossessionChanged {
                    owner: Some(Participant::Opponent),
                    reason: "defender control radius intersects ball movement segment".into(),
                },
            );
        } else {
            s.ball.position_mm = next;
        }
    } else {
        s.ball.position_mm = next;
    }
    if let Possession::InFlight { to, target_mm, .. } = s.ball.possession.clone() {
        if s.ball.position_mm == target_mm {
            let receiver = match to {
                Participant::Assessed => s.assessed.position_mm,
                Participant::Teammate => {
                    s.teammate.as_ref().expect("validated teammate").position_mm
                }
                Participant::Opponent => s.opponent.position_mm,
            };
            let received = contact(target_mm, target_mm, receiver).is_some();
            s.ball.possession = if received {
                Possession::Owned { owner: to }
            } else {
                Possession::Loose
            };
            event(
                trace,
                s.elapsed_ms,
                EventKind::PossessionChanged {
                    owner: received.then_some(to),
                    reason: if received {
                        "receiver controls ball at the actual pass destination"
                    } else {
                        "receiver not within control radius at pass destination"
                    }
                    .into(),
                },
            );
        }
    }
    s.ball.velocity_mm_s = Vector2 {
        x: (s.ball.position_mm.x - old.x) * 10,
        y: (s.ball.position_mm.y - old.y) * 10,
    };
}

fn evaluation(initial: &SimulationState, s: &SimulationState) -> TacticalEvaluation {
    let friendly = matches!(
        s.ball.possession,
        Possession::Owned {
            owner: Participant::Assessed | Participant::Teammate
        }
    );
    let bypassed = friendly && s.ball.position_mm.x >= s.opponent.position_mm.x + 1000;
    let progress = s.ball.position_mm.x - initial.ball.position_mm.x;
    let clearance = distance(s.ball.position_mm, s.opponent.position_mm);
    TacticalEvaluation {
        friendly_possession: friendly,
        bypassed_defender: bypassed,
        forward_ball_progress_mm: progress,
        defender_clearance_mm: clearance,
        objective_key: (
            friendly,
            bypassed,
            if friendly {
                progress.div_euclid(1000)
            } else {
                0
            },
            if friendly {
                clearance.min(15000) / 1000
            } else {
                0
            },
        ),
    }
}

fn run(r: &SimulationRequest, correction: Option<TacticalAction>, id: String) -> SimulationTrace {
    let mut s = r.initial_state.clone();
    let mut trace = SimulationTrace {
        id,
        corrective_action: correction,
        correction_applied: false,
        steps: Vec::new(),
        events: Vec::new(),
        evaluation: evaluation(&s, &s),
    };
    let mut attacker_target = target(&s, s.current_action, r.config.pitch);
    let mut defender_target = s.opponent.position_mm;
    let defender_params = parameters(&r.opponent.traits);
    let mut response_at = defender_params.decision_delay_ms;
    let mut pass_started = false;
    let mut returned = false;
    event(
        &mut trace,
        0,
        EventKind::ActionSelected {
            action: s.current_action,
        },
    );
    loop {
        if s.elapsed_ms == r.config.adaptation_at_ms {
            if let Some(action) = correction {
                let (feasible, reason) = action_feasibility(&s, action, r.config.pitch);
                event(
                    &mut trace,
                    s.elapsed_ms,
                    EventKind::Adaptation {
                        action,
                        feasible,
                        reason: reason
                            .unwrap_or("available in the actual adaptation state")
                            .into(),
                    },
                );
                if feasible {
                    s.current_action = action;
                    trace.correction_applied = true;
                    attacker_target = target(&s, action, r.config.pitch);
                    response_at = s.elapsed_ms + defender_params.decision_delay_ms;
                }
            }
        }
        if s.elapsed_ms == response_at {
            defender_target = choose_response(r, &s, attacker_target, &mut trace);
        }
        if matches!(
            s.current_action,
            TacticalAction::ReleasePass | TacticalAction::OneTwo
        ) && !pass_started
            && trace.correction_applied
        {
            release(
                &mut s,
                Participant::Assessed,
                Participant::Teammate,
                &mut trace,
            );
            pass_started = true;
        }
        if s.current_action == TacticalAction::OneTwo
            && !returned
            && matches!(
                s.ball.possession,
                Possession::Owned {
                    owner: Participant::Teammate
                }
            )
        {
            release(
                &mut s,
                Participant::Teammate,
                Participant::Assessed,
                &mut trace,
            );
            // Rendezvous is the actual attacker position; braking still obeys acceleration limits.
            attacker_target = s.assessed.position_mm;
            returned = true;
        }
        trace.steps.push(TraceStep {
            id: format!("{}:step:{:03}", trace.id, s.elapsed_ms / TICK_MS),
            state: s.clone(),
        });
        if s.elapsed_ms == r.config.duration_ms {
            break;
        }
        advance(
            &mut s.assessed,
            attacker_target,
            parameters(&r.assessed_player.traits),
            r.config.pitch,
        );
        advance(
            &mut s.opponent,
            defender_target,
            defender_params,
            r.config.pitch,
        );
        // Supporting player is deliberately stationary in this bounded scenario.
        s.elapsed_ms += TICK_MS;
        update_ball(r, &mut s, &mut trace);
    }
    trace.evaluation = evaluation(&r.initial_state, &s);
    trace
}

pub fn simulate(r: SimulationRequest) -> Result<SimulationReport, SimulationError> {
    validate(&r)?;
    let baseline = run(&r, None, "baseline".into());
    let adaptation_state = &baseline.steps[(r.config.adaptation_at_ms / TICK_MS) as usize].state;
    let mut alternatives = Vec::new();
    for action in ACTIONS {
        if action == r.initial_state.current_action {
            continue;
        }
        let (feasible, reason) = action_feasibility(adaptation_state, action, r.config.pitch);
        alternatives.push(AlternativeAssessment {
            action,
            feasible_at_adaptation: feasible,
            reason: reason
                .unwrap_or("available in the shared adaptation state")
                .into(),
            trace: feasible.then(|| run(&r, Some(action), format!("alternative:{}", action.id()))),
        });
    }
    let mut best = &baseline;
    for trace in alternatives.iter().filter_map(|a| a.trace.as_ref()) {
        if trace.correction_applied
            && trace.evaluation.objective_key > best.evaluation.objective_key
        {
            best = trace;
        }
    }
    let recommended_correction = best.corrective_action;
    let recommended_trace_id = recommended_correction.map(|_| best.id.clone());
    let comparison_reason = if let Some(action) = recommended_correction {
        format!("{} improves the lexicographic objective from {:?} to {:?}; inspect {} opponent-decision and movement events for the spatial evidence.", action.id(), baseline.evaluation.objective_key, best.evaluation.objective_key, best.id)
    } else {
        "No feasible corrective branch improves the baseline objective at the documented one-metre resolution; no correction is recommended.".into()
    };
    Ok(SimulationReport {
        scenario_version: SCENARIO_VERSION.into(),
        assessed_parameters: parameters(&r.assessed_player.traits), opponent_parameters: parameters(&r.opponent.traits),
        teammate_parameters: r.supporting_teammate.as_ref().map(|p| parameters(&p.traits)),
        assumptions: vec!["Coordinates are integer mm; origin is (0,0), attack toward +x; left means decreasing y. Fixed 105m x 68m pitch and 100ms ticks.".into(),
            "Synthetic acceleration rating a maps to speed 4000+30a mm/s and acceleration 2000+40a mm/s2; L1 speed and acceleration budgets, half acceleration per axis. These are uncalibrated conventions.".into(),
            "Synthetic reactions r map to decision delay (1+floor((100-r)/25))*100ms, not measured reflex latency.".into(),
            "Ball speed is 14000+20*passing mm/s using the passer rating and L1 movement. Control radius is 1000mm; no random pass errors. Teammate stays stationary.".into(),
            "Counter targets are screened by bounded movement preview within 2 L1 metres at the horizon, then ranked by unchanged legacy indices; first-defined ties win.".into(),
            "Objective maximizes friendly settled possession, then 1m defender bypass, then forward ball progress in 1m buckets, then L1 clearance in 1m buckets capped at 15m. Without friendly possession, progress and clearance cannot improve the objective. Baseline wins ties.".into()],
        limitations: vec!["Fictional attributes and uncalibrated parameters; no probabilities, expected goals or claims about real athletes.".into(),
            "Two active players and a stationary optional receiver; no collisions, spin, aerial ball, stamina, fouls, goalkeeper, goals or full team tactics.".into(),
            "Interception checks the ball segment against the defender's end-of-tick position. Counter feasibility is a horizon screen, not proof of interception before ball arrival.".into(),
            "Passes aim at a fixed destination; one-two returns may become loose when the moving receiver cannot stop in time. In-flight or loose balls do not count as settled possession.".into(),
            "This spatial scenario performs no Law 11 offence assessment; the existing offside-position endpoint remains separate.".into()],
        input: r, baseline, alternatives, recommended_correction, recommended_trace_id, comparison_reason,
    })
}

pub fn demo_request() -> SimulationRequest {
    let (assessed_player, opponent) = demo_profiles(ScenarioKind::OpenPlay);
    let mut teammate = assessed_player.clone();
    teammate.id = "synthetic-support-001".into();
    teammate.label = "Fictional supporting teammate".into();
    let player = |x, y| PlayerState {
        position_mm: Vector2 { x, y },
        velocity_mm_s: Vector2::default(),
    };
    SimulationRequest {
        scenario_version: SCENARIO_VERSION.into(),
        assessed_player,
        opponent,
        supporting_teammate: Some(teammate),
        initial_state: SimulationState {
            elapsed_ms: 0,
            assessed: player(45000, 34000),
            opponent: player(52000, 36500),
            teammate: Some(player(57000, 26000)),
            ball: BallState {
                position_mm: Vector2 { x: 45000, y: 34000 },
                velocity_mm_s: Vector2::default(),
                possession: Possession::Owned {
                    owner: Participant::Assessed,
                },
            },
            current_action: TacticalAction::WideRight,
        },
        config: SimulationConfig {
            pitch: PitchGeometry::default(),
            tick_ms: TICK_MS,
            duration_ms: 4000,
            adaptation_at_ms: 1200,
        },
    }
}
