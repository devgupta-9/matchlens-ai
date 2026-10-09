use super::{motion::*, *};

fn traces(report: &SimulationReport) -> impl Iterator<Item = &SimulationTrace> {
    std::iter::once(&report.baseline)
        .chain(report.alternatives.iter().filter_map(|a| a.trace.as_ref()))
}
fn response_ids(trace: &SimulationTrace) -> Vec<&str> {
    trace
        .events
        .iter()
        .filter_map(|e| match &e.evidence {
            EventKind::OpponentDecision {
                chosen_response_id, ..
            } => Some(chosen_response_id.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn repeated_rollouts_have_identical_serialized_evidence() {
    let first = simulate(demo_request()).unwrap();
    let second = simulate(demo_request()).unwrap();
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
}

#[test]
fn counterfactuals_clone_the_same_start_and_share_the_prefix() {
    let r = simulate(demo_request()).unwrap();
    for t in traces(&r) {
        assert_eq!(t.steps[0].state, r.input.initial_state);
        for (a, b) in t.steps.iter().zip(&r.baseline.steps).take(12) {
            assert_eq!(a.state, b.state);
        }
        assert_eq!(response_ids(t)[0], response_ids(&r.baseline)[0]);
    }
}

#[test]
fn elapsed_time_and_stable_identifiers_are_monotonic() {
    let r = simulate(demo_request()).unwrap();
    for t in traces(&r) {
        assert_eq!(t.steps.len(), 41);
        for (i, s) in t.steps.iter().enumerate() {
            assert_eq!(s.state.elapsed_ms, i as u32 * 100);
            assert_eq!(s.id, format!("{}:step:{i:03}", t.id));
        }
        assert!(t
            .events
            .windows(2)
            .all(|w| w[0].elapsed_ms <= w[1].elapsed_ms));
        for (i, e) in t.events.iter().enumerate() {
            assert_eq!(e.id, format!("{}:event:{i:03}", t.id));
        }
    }
}

#[test]
fn all_rollout_player_movements_respect_speed_acceleration_and_pitch() {
    let mut request = demo_request();
    request.config.duration_ms = MAX_DURATION_MS;
    let r = simulate(request).unwrap();
    for t in traces(&r) {
        for w in t.steps.windows(2) {
            for (before, after, m) in [
                (
                    &w[0].state.assessed,
                    &w[1].state.assessed,
                    r.assessed_parameters,
                ),
                (
                    &w[0].state.opponent,
                    &w[1].state.opponent,
                    r.opponent_parameters,
                ),
            ] {
                assert!(inside(after.position_mm, r.input.config.pitch));
                assert!(l1(after.velocity_mm_s) <= m.max_speed_mm_s);
                assert!(
                    distance(before.velocity_mm_s, after.velocity_mm_s)
                        <= m.max_acceleration_mm_s2 / 10
                );
                assert_eq!(
                    difference(after.position_mm, before.position_mm),
                    Vector2 {
                        x: after.velocity_mm_s.x / 10,
                        y: after.velocity_mm_s.y / 10
                    }
                );
            }
            assert!(distance(w[0].state.ball.position_mm, w[1].state.ball.position_mm) <= 1600);
        }
    }
}

#[test]
fn boundaries_and_turns_never_snap_or_teleport() {
    let pitch = PitchGeometry::default();
    let mut profile = demo_request().assessed_player;
    for rating in [0, 1, 50, 99, 100] {
        profile.traits.acceleration = rating;
        let m = parameters(&profile.traits);
        for start in [
            Vector2 { x: 0, y: 0 },
            Vector2 {
                x: 105000,
                y: 68000,
            },
            Vector2 { x: 100, y: 67900 },
            Vector2 { x: 50000, y: 34000 },
        ] {
            let mut p = PlayerState {
                position_mm: start,
                velocity_mm_s: Vector2::default(),
            };
            for tick in 0..400 {
                let destination = match (tick / 50) % 4 {
                    0 => Vector2 { x: 0, y: 68000 },
                    1 => Vector2 { x: 105000, y: 0 },
                    2 => Vector2 {
                        x: 105000,
                        y: 68000,
                    },
                    _ => Vector2 { x: 0, y: 0 },
                };
                let before = p.clone();
                advance(&mut p, destination, m, pitch);
                assert!(inside(p.position_mm, pitch), "{rating} {tick} {p:?}");
                assert!(l1(p.velocity_mm_s) <= m.max_speed_mm_s);
                assert!(
                    distance(p.velocity_mm_s, before.velocity_mm_s)
                        <= m.max_acceleration_mm_s2 / 10,
                    "{rating} {tick}"
                );
                assert_eq!(
                    difference(p.position_mm, before.position_mm),
                    Vector2 {
                        x: p.velocity_mm_s.x / 10,
                        y: p.velocity_mm_s.y / 10
                    }
                );
            }
        }
    }
}

#[test]
fn possession_has_one_existing_owner_with_bounded_control_distance() {
    let r = simulate(demo_request()).unwrap();
    for t in traces(&r) {
        for step in &t.steps {
            let s = &step.state;
            assert!(inside(s.ball.position_mm, r.input.config.pitch));
            if let Possession::Owned { owner } = s.ball.possession {
                let pos = match owner {
                    Participant::Assessed => s.assessed.position_mm,
                    Participant::Opponent => s.opponent.position_mm,
                    Participant::Teammate => s.teammate.as_ref().unwrap().position_mm,
                };
                assert!(
                    contact(s.ball.position_mm, s.ball.position_mm, pos).is_some(),
                    "{} {}",
                    t.id,
                    s.elapsed_ms
                );
            }
        }
    }
}

#[test]
fn opponent_scores_select_only_feasible_responses() {
    let r = simulate(demo_request()).unwrap();
    for t in traces(&r) {
        for event in &t.events {
            if let EventKind::OpponentDecision {
                options,
                chosen_response_id,
                ..
            } = &event.evidence
            {
                let chosen = options
                    .iter()
                    .find(|o| o.id == *chosen_response_id)
                    .unwrap();
                assert!(chosen.feasible);
                assert!(options
                    .iter()
                    .filter(|o| o.feasible)
                    .all(|o| o.effectiveness_index <= chosen.effectiveness_index));
            }
        }
    }
    assert_eq!(response_ids(&r.baseline)[0], "close_wide_angle");
}

#[test]
fn profile_changes_can_switch_defender_counter() {
    let mut request = demo_request();
    request.opponent.traits.acceleration = 100;
    request.opponent.traits.reactions = 100;
    request.opponent.traits.anticipation = 0;
    request.opponent.traits.positioning = 0;
    let r = simulate(request).unwrap();
    assert_eq!(response_ids(&r.baseline)[0], "track_wide_run");
}

#[test]
fn unreachable_counter_uses_bounded_recovery_fallback() {
    let mut request = demo_request();
    request.initial_state.opponent.position_mm = Vector2 {
        x: 100000,
        y: 60000,
    };
    let r = simulate(request).unwrap();
    assert_eq!(response_ids(&r.baseline)[0], "recover_toward_ball");
    if let EventKind::OpponentDecision { options, .. } = &r.baseline.events[1].evidence {
        assert!(options[..2].iter().all(|o| !o.feasible));
    } else {
        panic!("expected response evidence");
    }
}

#[test]
fn tied_scores_preserve_first_defined_response() {
    let mut request = demo_request();
    request.initial_state.current_action = TacticalAction::DirectDribble;
    request.opponent.traits.acceleration = 0;
    request.opponent.traits.positioning = 0;
    request.opponent.traits.anticipation = 0;
    request.opponent.traits.reactions = 0;
    request.assessed_player.traits.acceleration = 0;
    request.assessed_player.traits.technique = 0;
    request.assessed_player.traits.composure = 0;
    let r = simulate(request).unwrap();
    assert_eq!(response_ids(&r.baseline)[0], "hold_central_lane");
}

#[test]
fn correction_is_applied_and_observed_after_synthetic_delay() {
    let r = simulate(demo_request()).unwrap();
    for t in r.alternatives.iter().filter_map(|a| a.trace.as_ref()) {
        assert!(t.correction_applied);
        assert_eq!(response_ids(t).len(), 2);
        let event = t
            .events
            .iter()
            .filter(|e| matches!(e.evidence, EventKind::OpponentDecision { .. }))
            .nth(1)
            .unwrap();
        assert_eq!(
            event.elapsed_ms,
            r.input.config.adaptation_at_ms + r.opponent_parameters.decision_delay_ms
        );
        if let EventKind::OpponentDecision {
            observed_action, ..
        } = event.evidence
        {
            assert_eq!(Some(observed_action), t.corrective_action);
        }
    }
}

#[test]
fn recommendations_require_strict_objective_improvement() {
    let r = simulate(demo_request()).unwrap();
    if let Some(action) = r.recommended_correction {
        let chosen = r
            .alternatives
            .iter()
            .find(|a| a.action == action)
            .unwrap()
            .trace
            .as_ref()
            .unwrap();
        assert!(chosen.evaluation.objective_key > r.baseline.evaluation.objective_key);
        assert_eq!(r.recommended_trace_id.as_deref(), Some(chosen.id.as_str()));
        assert!(traces(&r).all(|t| t.evaluation.objective_key <= chosen.evaluation.objective_key));
    } else {
        assert!(
            traces(&r).all(|t| t.evaluation.objective_key <= r.baseline.evaluation.objective_key)
        );
    }
}

#[test]
fn a_lost_ball_does_not_manufacture_a_correction() {
    let mut request = demo_request();
    request.initial_state.opponent.position_mm = Vector2 { x: 46200, y: 35200 };
    let r = simulate(request).unwrap();
    assert!(!r.baseline.evaluation.friendly_possession);
    assert_eq!(r.recommended_correction, None);
    assert!(r
        .alternatives
        .iter()
        .all(|a| !a.feasible_at_adaptation && a.trace.is_none()));
}

#[test]
fn missing_teammate_disables_pass_and_one_two() {
    let mut request = demo_request();
    request.supporting_teammate = None;
    request.initial_state.teammate = None;
    let r = simulate(request).unwrap();
    for action in [TacticalAction::ReleasePass, TacticalAction::OneTwo] {
        let a = r.alternatives.iter().find(|a| a.action == action).unwrap();
        assert!(!a.feasible_at_adaptation);
        assert!(a.trace.is_none());
    }
}

#[test]
fn pass_and_one_two_record_actual_ball_transitions() {
    let r = simulate(demo_request()).unwrap();
    for action in [TacticalAction::ReleasePass, TacticalAction::OneTwo] {
        let t = r
            .alternatives
            .iter()
            .find(|a| a.action == action)
            .unwrap()
            .trace
            .as_ref()
            .unwrap();
        assert!(t.events.iter().any(|e| matches!(
            e.evidence,
            EventKind::PassReleased {
                from: Participant::Assessed,
                ..
            }
        )));
        assert!(t
            .steps
            .iter()
            .any(|s| matches!(s.state.ball.possession, Possession::InFlight { .. })));
    }
}

#[test]
fn invalid_coordinates_and_extreme_velocities_are_rejected_without_panics() {
    for position in [
        Vector2 { x: -1, y: 34000 },
        Vector2 { x: 105001, y: 0 },
        Vector2 { x: 1, y: 68001 },
        Vector2 {
            x: i32::MAX,
            y: i32::MIN,
        },
    ] {
        let mut r = demo_request();
        r.initial_state.assessed.position_mm = position;
        assert!(simulate(r).is_err());
    }
    for velocity in [
        Vector2 { x: i32::MIN, y: 0 },
        Vector2 { x: 7001, y: 0 },
        Vector2 { x: 5000, y: 5000 },
    ] {
        let mut r = demo_request();
        r.initial_state.assessed.velocity_mm_s = velocity;
        assert!(simulate(r).is_err());
    }
}

#[test]
fn impossible_outward_velocity_is_rejected() {
    let mut r = demo_request();
    r.initial_state.assessed.position_mm.x = 105000;
    r.initial_state.assessed.velocity_mm_s.x = 4000;
    assert!(simulate(r).is_err());
}

#[test]
fn invalid_timing_and_unbounded_durations_are_rejected() {
    for (tick, duration, adaptation) in [
        (0, 4000, 1200),
        (50, 4000, 1200),
        (100, 0, 1200),
        (100, 6001, 1200),
        (100, 4001, 1200),
        (100, 4000, 0),
        (100, 4000, 1250),
        (100, 4000, 3500),
        (100, 4000, u32::MAX),
    ] {
        let mut r = demo_request();
        r.config.tick_ms = tick;
        r.config.duration_ms = duration;
        r.config.adaptation_at_ms = adaptation;
        assert!(simulate(r).is_err());
    }
}

#[test]
fn inconsistent_ownership_teammate_and_profiles_are_rejected() {
    let mut r = demo_request();
    r.initial_state.ball.possession = Possession::Owned {
        owner: Participant::Opponent,
    };
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.initial_state.ball.position_mm.x += 1;
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.supporting_teammate = None;
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.opponent.synthetic = false;
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.opponent.id = r.assessed_player.id.clone();
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.opponent.traits.anticipation = 101;
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.assessed_player.description = "x".repeat(1025);
    assert!(simulate(r).is_err());
}

#[test]
fn invalid_scenario_pitch_and_action_space_are_rejected() {
    let mut r = demo_request();
    r.scenario_version = "v999".into();
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.config.pitch.length_mm = i32::MAX;
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.initial_state.assessed.position_mm.y = 68000;
    r.initial_state.ball.position_mm.y = 68000;
    assert!(simulate(r).is_err());
    let mut r = demo_request();
    r.initial_state.current_action = TacticalAction::OneTwo;
    assert!(simulate(r).is_err());
}

#[test]
fn trace_serde_roundtrip_preserves_the_entire_evidence() {
    let report = simulate(demo_request()).unwrap();
    let bytes = serde_json::to_vec(&report).unwrap();
    let roundtrip: SimulationReport = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(bytes, serde_json::to_vec(&roundtrip).unwrap());
    let mut request = serde_json::to_value(demo_request()).unwrap();
    request["initial_state"]["extra"] = true.into();
    assert!(serde_json::from_value::<SimulationRequest>(request).is_err());
}

#[test]
fn ball_segment_interception_prevents_skipping_a_stationary_defender() {
    let point = contact(
        Vector2 { x: 0, y: 0 },
        Vector2 { x: 1600, y: 0 },
        Vector2 { x: 800, y: 0 },
    )
    .unwrap();
    assert_eq!(point, Vector2 { x: 800, y: 0 });
    assert!(contact(
        Vector2 { x: 0, y: 0 },
        Vector2 { x: 1600, y: 0 },
        Vector2 { x: 800, y: 1001 }
    )
    .is_none());
}

#[test]
fn losing_branches_cannot_recommend_opponent_ball_progress() {
    let mut request = demo_request();
    request.supporting_teammate = None;
    request.initial_state.teammate = None;
    let r = simulate(request).unwrap();
    assert!(traces(&r).all(|t| !t.evaluation.friendly_possession));
    assert_eq!(r.recommended_correction, None);
    assert!(traces(&r).all(|t| t.evaluation.objective_key == (false, false, 0, 0)));
}

#[test]
fn nested_profile_and_trait_unknown_fields_are_rejected() {
    for field in ["assessed_player", "opponent", "supporting_teammate"] {
        let mut request = serde_json::to_value(demo_request()).unwrap();
        request[field]["unknown"] = true.into();
        assert!(serde_json::from_value::<SimulationRequest>(request).is_err());
        let mut request = serde_json::to_value(demo_request()).unwrap();
        request[field]["traits"]["unknown"] = true.into();
        assert!(serde_json::from_value::<SimulationRequest>(request).is_err());
    }
}

#[test]
fn moving_initial_state_and_emergency_braking_remain_bounded() {
    let mut request = demo_request();
    request.initial_state.assessed.velocity_mm_s = Vector2 { x: 3000, y: 2000 };
    request.initial_state.ball.velocity_mm_s = request.initial_state.assessed.velocity_mm_s;
    let r = simulate(request).unwrap();
    for t in traces(&r) {
        for w in t.steps.windows(2) {
            assert!(
                distance(
                    w[0].state.assessed.velocity_mm_s,
                    w[1].state.assessed.velocity_mm_s
                ) <= r.assessed_parameters.max_acceleration_mm_s2 / 10
            );
            assert!(inside(
                w[1].state.assessed.position_mm,
                r.input.config.pitch
            ));
        }
    }
    let m = r.assessed_parameters;
    let mut p = PlayerState {
        position_mm: Vector2 {
            x: 104000,
            y: 34000,
        },
        velocity_mm_s: Vector2 { x: 1000, y: 0 },
    };
    assert!(valid_player(&p, m, PitchGeometry::default()));
    for _ in 0..100 {
        let before = p.clone();
        advance(
            &mut p,
            Vector2 {
                x: 105000,
                y: 34000,
            },
            m,
            PitchGeometry::default(),
        );
        assert!(inside(p.position_mm, PitchGeometry::default()));
        assert!(distance(p.velocity_mm_s, before.velocity_mm_s) <= m.max_acceleration_mm_s2 / 10);
    }
}

#[test]
fn stationary_target_is_reached_with_controlled_braking() {
    let m = parameters(&demo_request().opponent.traits);
    let mut p = PlayerState {
        position_mm: Vector2 { x: 52000, y: 36500 },
        velocity_mm_s: Vector2::default(),
    };
    let destination = Vector2 { x: 48000, y: 37000 };
    for _ in 0..40 {
        advance(&mut p, destination, m, PitchGeometry::default());
    }
    assert!(distance(p.position_mm, destination) <= 100);
    assert!(l1(p.velocity_mm_s) <= 100);
}

#[test]
fn fictional_demo_recommends_evidence_supported_pass_not_failed_one_two() {
    let report = simulate(demo_request()).unwrap();
    assert!(!report.baseline.evaluation.friendly_possession);
    assert_eq!(
        report.recommended_correction,
        Some(TacticalAction::ReleasePass)
    );
    let pass = report
        .alternatives
        .iter()
        .find(|a| a.action == TacticalAction::ReleasePass)
        .unwrap()
        .trace
        .as_ref()
        .unwrap();
    assert!(pass.evaluation.friendly_possession && pass.evaluation.bypassed_defender);
    assert_eq!(
        response_ids(pass),
        vec!["close_wide_angle", "intercept_passing_lane"]
    );
    let combination = report
        .alternatives
        .iter()
        .find(|a| a.action == TacticalAction::OneTwo)
        .unwrap()
        .trace
        .as_ref()
        .unwrap();
    assert!(combination.events.iter().any(|e| matches!(
        e.evidence,
        EventKind::PassReleased {
            from: Participant::Teammate,
            to: Participant::Assessed,
            ..
        }
    )));
    assert!(!combination.evaluation.friendly_possession);
}

#[test]
fn optional_teammate_can_be_omitted_from_json() {
    let mut input = serde_json::to_value(demo_request()).unwrap();
    input.as_object_mut().unwrap().remove("supporting_teammate");
    input["initial_state"]
        .as_object_mut()
        .unwrap()
        .remove("teammate");
    let request: SimulationRequest = serde_json::from_value(input).unwrap();
    let report = simulate(request).unwrap();
    assert_eq!(report.recommended_correction, None);
    assert!(report.input.supporting_teammate.is_none());
}

#[test]
fn adaptation_must_follow_the_initial_defender_response() {
    let mut request = demo_request();
    request.opponent.traits.reactions = 0;
    request.config.adaptation_at_ms = 500;
    assert!(simulate(request.clone()).is_err());
    request.config.adaptation_at_ms = 600;
    let report = simulate(request).unwrap();
    for trace in traces(&report) {
        let first_response = trace
            .events
            .iter()
            .find(|e| matches!(e.evidence, EventKind::OpponentDecision { .. }))
            .unwrap();
        assert!(first_response.elapsed_ms < report.input.config.adaptation_at_ms);
        assert_eq!(response_ids(trace)[0], response_ids(&report.baseline)[0]);
    }
}
