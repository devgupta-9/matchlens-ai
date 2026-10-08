//! Synthetic, auditable football matchup decision scores.
//! Scores are *heuristic fit indices*, NOT winning percentages, biological reflex
//! measurements or claims about real athletes.
pub mod offside;
pub mod responses;

pub use responses::OpponentResponseAssessment;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    OpenPlay,
    Penalty,
    FreeKick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Traits {
    pub acceleration: u8,
    pub technique: u8,
    pub passing: u8,
    pub finishing: u8,
    pub anticipation: u8,
    pub positioning: u8,
    pub reactions: u8,
    pub goalkeeping: u8,
    pub composure: u8,
}

impl Traits {
    fn values(&self) -> [u8; 9] {
        [
            self.acceleration,
            self.technique,
            self.passing,
            self.finishing,
            self.anticipation,
            self.positioning,
            self.reactions,
            self.goalkeeping,
            self.composure,
        ]
    }

    pub fn is_valid(&self) -> bool {
        self.values().iter().all(|value| *value <= 100)
    }
}

const TRAIT_NAMES: [&str; 9] = [
    "acceleration",
    "technique",
    "passing",
    "finishing",
    "anticipation",
    "positioning",
    "reactions",
    "goalkeeping",
    "composure",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: String,
    pub label: String,
    pub description: String,
    pub traits: Traits,
    pub synthetic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionAssessment {
    pub id: String,
    pub label: String,
    pub decision_fit_index: u8,
    pub player_fit_index: u8,
    pub opponent_resistance_index: u8,
    pub uncountered_fit_index: u8,
    pub counter_suppression_points: u8,
    pub chosen_opponent_response_id: String,
    pub opponent_responses: Vec<OpponentResponseAssessment>,
    pub modeled_opponent_response: String,
    pub coaching_instruction: String,
    pub relevant_player_strength: String,
    pub opponent_vulnerability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchupReport {
    pub scenario: ScenarioKind,
    pub assessed_player: PlayerProfile,
    pub opponent: PlayerProfile,
    pub original_action_id: String,
    pub recommended_action_id: String,
    pub improvement_index_points: i16,
    pub actions: Vec<ActionAssessment>,
    pub explanation: String,
    pub limitations: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvaluationError {
    #[error("attribute ratings must be between 0 and 100")]
    InvalidRatings,
    #[error("scenario must have a baseline and at least one alternative")]
    InvalidScenario,
}

struct ActionSpec {
    id: &'static str,
    label: &'static str,
    assessed_weights: [u8; 9],
    opponent_weights: [u8; 9],
    coaching: &'static str,
}

const OPEN_PLAY: [ActionSpec; 3] = [
    ActionSpec {
        id: "direct_dribble", label: "Direct dribble",
        assessed_weights: [15, 50, 0, 0, 0, 0, 0, 0, 35],
        opponent_weights: [0, 0, 0, 0, 45, 40, 15, 0, 0],

        coaching: "Avoid driving directly into a well-positioned opponent if your close control is the weaker fit.",
    },
    ActionSpec {
        id: "accelerate_wide", label: "Accelerate into the outside channel",
        assessed_weights: [70, 20, 0, 0, 0, 0, 0, 0, 10],
        opponent_weights: [45, 0, 0, 0, 0, 30, 25, 0, 0],

        coaching: "Change angle early, use the first acceleration burst, and attack space rather than the defender.",
    },
    ActionSpec {
        id: "quick_combination", label: "One-two combination",
        assessed_weights: [0, 0, 60, 0, 20, 0, 0, 0, 20],
        opponent_weights: [0, 0, 0, 0, 35, 40, 25, 0, 0],

        coaching: "Invite pressure, release an early pass, then accelerate behind the first line of pressure.",
    },
];

const PENALTY: [ActionSpec; 3] = [
    ActionSpec {
        id: "power_shot",
        label: "Early power shot",
        assessed_weights: [0, 20, 0, 60, 0, 0, 0, 0, 20],
        opponent_weights: [0, 0, 0, 0, 0, 0, 50, 50, 0],

        coaching: "Avoid relying on power alone when it is not your strongest kicking attribute.",
    },
    ActionSpec {
        id: "placed_shot",
        label: "Controlled placed shot",
        assessed_weights: [0, 30, 0, 40, 0, 0, 0, 0, 30],
        opponent_weights: [0, 0, 0, 0, 30, 0, 20, 50, 0],

        coaching:
            "Prioritize repeatable placement and composure over an uncertain maximum-power strike.",
    },
    ActionSpec {
        id: "delayed_placement",
        label: "Late controlled placement",
        assessed_weights: [0, 30, 0, 20, 0, 0, 0, 0, 50],
        opponent_weights: [0, 0, 0, 0, 40, 0, 20, 40, 0],

        coaching:
            "Hold your rhythm and use a late placement decision only when composure is sufficient.",
    },
];

const FREE_KICK: [ActionSpec; 3] = [
    ActionSpec {
        id: "direct_power",
        label: "Direct power attempt",
        assessed_weights: [0, 25, 0, 60, 0, 0, 0, 0, 15],
        opponent_weights: [0, 0, 0, 0, 0, 20, 30, 50, 0],

        coaching:
            "Use direct power only when your shooting and technical consistency justify the risk.",
    },
    ActionSpec {
        id: "placed_curl",
        label: "Placed curl",
        assessed_weights: [0, 55, 0, 30, 0, 0, 0, 0, 15],
        opponent_weights: [0, 0, 0, 0, 0, 30, 20, 50, 0],

        coaching:
            "Prioritize technique and placement, adjusting the trajectory around the modeled wall.",
    },
    ActionSpec {
        id: "short_routine",
        label: "Short combination routine",
        assessed_weights: [0, 0, 50, 0, 30, 0, 0, 0, 20],
        opponent_weights: [20, 0, 0, 0, 30, 50, 0, 0, 0],

        coaching:
            "Use passing and anticipation to move the defensive block before the final delivery.",
    },
];

fn specs(scenario: ScenarioKind) -> &'static [ActionSpec] {
    match scenario {
        ScenarioKind::OpenPlay => &OPEN_PLAY,
        ScenarioKind::Penalty => &PENALTY,
        ScenarioKind::FreeKick => &FREE_KICK,
    }
}

fn weighted(traits: &Traits, weights: &[u8; 9]) -> u8 {
    debug_assert_eq!(
        weights.iter().map(|weight| *weight as u32).sum::<u32>(),
        100
    );
    let total: u32 = traits
        .values()
        .iter()
        .zip(weights)
        .map(|(rating, weight)| u32::from(*rating) * u32::from(*weight))
        .sum();
    ((total + 50) / 100) as u8
}

fn leading_strength(traits: &Traits, weights: &[u8; 9]) -> &'static str {
    let values = traits.values();
    let position = (0..9)
        .max_by_key(|&i| u32::from(values[i]) * u32::from(weights[i]))
        .unwrap_or(0);
    TRAIT_NAMES[position]
}

fn relevant_weakness(traits: &Traits, weights: &[u8; 9]) -> &'static str {
    let values = traits.values();
    let position = (0..9)
        .max_by_key(|&i| u32::from(100u8.saturating_sub(values[i])) * u32::from(weights[i]))
        .unwrap_or(0);
    TRAIT_NAMES[position]
}

/// All actions are evaluated from the same starting profiles and unchanged weights.
/// This index is an engineering demo heuristic, not a calibrated success probability.
pub fn evaluate(
    scenario: ScenarioKind,
    assessed: PlayerProfile,
    opponent: PlayerProfile,
) -> Result<MatchupReport, EvaluationError> {
    if !assessed.traits.is_valid() || !opponent.traits.is_valid() {
        return Err(EvaluationError::InvalidRatings);
    }
    let options = specs(scenario);
    if options.len() < 2 {
        return Err(EvaluationError::InvalidScenario);
    }
    let actions: Vec<ActionAssessment> = options
        .iter()
        .map(|option| {
            let player_fit = weighted(&assessed.traits, &option.assessed_weights);
            let resistance = weighted(&opponent.traits, &option.opponent_weights);
            let vulnerability = 100u8.saturating_sub(resistance);
            let uncountered = ((u16::from(player_fit) * 2 + u16::from(vulnerability) + 1) / 3) as u8;
            let responses = responses::evaluate_responses(
                scenario,
                option.id,
                &assessed.traits,
                &opponent.traits,
            );
            // Deterministic opponent: choose the strongest available counter.
            // Stable ties favor the earlier response definition.
            let chosen = responses.iter().enumerate().max_by_key(|(index, response)| {
                (response.effectiveness_index, std::cmp::Reverse(*index))
            }).map(|(_, response)| response).expect("every supported action has counter-responses");
            // A higher modelled counter effectiveness reduces expected tactical fit.
            // This is an illustrative penalty, NOT a calibrated expected-goals model.
            let suppression = ((u16::from(chosen.effectiveness_index) + 3) / 7) as u8;
            let fit = uncountered.saturating_sub(suppression);
            ActionAssessment {
                id: option.id.to_string(),
                label: option.label.to_string(),
                decision_fit_index: fit,
                player_fit_index: player_fit,
                opponent_resistance_index: resistance,
                uncountered_fit_index: uncountered,
                counter_suppression_points: suppression,
                chosen_opponent_response_id: chosen.id.clone(),
                opponent_responses: responses,
                modeled_opponent_response: format!("{}: {}", chosen.label, chosen.explanation),
                coaching_instruction: option.coaching.to_string(),
                relevant_player_strength: leading_strength(
                    &assessed.traits,
                    &option.assessed_weights,
                ).to_string(),
                opponent_vulnerability: relevant_weakness(
                    &opponent.traits,
                    &option.opponent_weights,
                ).to_string(),
            }
        })
        .collect();

    // Stable tie-breaking: prefer the earlier defined action, never claim improvement on ties.
    let best = actions
        .iter()
        .enumerate()
        .max_by_key(|(index, action)| (action.decision_fit_index, std::cmp::Reverse(*index)))
        .map(|(index, _)| index)
        .unwrap_or(0);
    let delta =
        i16::from(actions[best].decision_fit_index) - i16::from(actions[0].decision_fit_index);
    let explanation = if delta > 0 {
        format!(
            "Replace '{}' with '{}'. The model favors the assessed player's {} against the opponent's relative {} vulnerability. Compare all alternatives; this is not a predicted success percentage.",
            actions[0].label, actions[best].label,
            actions[best].relevant_player_strength, actions[best].opponent_vulnerability
        )
    } else {
        format!(
            "The baseline '{}' is already the highest-fit modeled option. Maintain the decision and review alternative opponent responses; no improvement is supported by this synthetic score.",
            actions[0].label
        )
    };

    Ok(MatchupReport {
        scenario, assessed_player: assessed, opponent,
        original_action_id: actions[0].id.clone(),
        recommended_action_id: actions[best].id.clone(),
        improvement_index_points: delta,
        actions,
        explanation,
        limitations: "Fictional profiles, discrete opponent best-response choices and illustrative weighting; there is no continuous movement, physical trajectory or trained reaction policy. Not real-player scouting, medical reflex testing, performance forecasting or calibrated success probabilities.".into(),
    })
}

pub fn demo_profiles(scenario: ScenarioKind) -> (PlayerProfile, PlayerProfile) {
    let forward = PlayerProfile {
        id: "synthetic-forward-9".into(),
        label: "Forward 9".into(),
        description: "Fast, combination-focused attacker with less reliable close control".into(),
        traits: Traits {
            acceleration: 92,
            technique: 63,
            passing: 85,
            finishing: 68,
            anticipation: 75,
            positioning: 70,
            reactions: 78,
            goalkeeping: 9,
            composure: 72,
        },
        synthetic: true,
    };
    let defender = PlayerProfile {
        id: "synthetic-centreback-4".into(),
        label: "Centre-back 4".into(),
        description: "Strong anticipation and positioning, less acceleration".into(),
        traits: Traits {
            acceleration: 69,
            technique: 68,
            passing: 72,
            finishing: 38,
            anticipation: 94,
            positioning: 94,
            reactions: 83,
            goalkeeping: 20,
            composure: 90,
        },
        synthetic: true,
    };
    let keeper = PlayerProfile {
        id: "synthetic-keeper-1".into(),
        label: "Goalkeeper 1".into(),
        description: "Strong reactions and goalkeeping with disciplined positioning".into(),
        traits: Traits {
            acceleration: 49,
            technique: 55,
            passing: 63,
            finishing: 20,
            anticipation: 79,
            positioning: 88,
            reactions: 93,
            goalkeeping: 94,
            composure: 82,
        },
        synthetic: true,
    };
    let midfielder = PlayerProfile {
        id: "synthetic-midfielder-8".into(),
        label: "Midfielder 8".into(),
        description: "High technique, passing and composure, moderate speed".into(),
        traits: Traits {
            acceleration: 72,
            technique: 90,
            passing: 91,
            finishing: 82,
            anticipation: 80,
            positioning: 72,
            reactions: 74,
            goalkeeping: 8,
            composure: 89,
        },
        synthetic: true,
    };
    match scenario {
        ScenarioKind::OpenPlay => (forward, defender),
        ScenarioKind::Penalty => (forward, keeper),
        ScenarioKind::FreeKick => (midfielder, keeper),
    }
}

pub fn evaluate_demo(scenario: ScenarioKind) -> MatchupReport {
    let (player, opponent) = demo_profiles(scenario);
    evaluate(scenario, player, opponent).expect("checked synthetic profiles")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_demo_scenarios_are_valid_and_have_three_choices() {
        for scenario in [
            ScenarioKind::OpenPlay,
            ScenarioKind::Penalty,
            ScenarioKind::FreeKick,
        ] {
            let report = evaluate_demo(scenario);
            assert_eq!(report.actions.len(), 3);
            assert!(report.actions.iter().all(|a| a.decision_fit_index <= 100));
            assert!(report.improvement_index_points >= 0);
            assert!(report.assessed_player.synthetic && report.opponent.synthetic);
        }
    }

    #[test]
    fn first_scenario_recommends_a_change_from_direct_dribble() {
        let report = evaluate_demo(ScenarioKind::OpenPlay);
        assert_eq!(report.original_action_id, "direct_dribble");
        assert_eq!(report.recommended_action_id, "accelerate_wide");
        assert!(report.improvement_index_points > 0);
    }

    #[test]
    fn changing_player_strengths_changes_the_recommendation() {
        let (mut assessed, opponent) = demo_profiles(ScenarioKind::OpenPlay);
        assessed.traits.acceleration = 5;
        assessed.traits.technique = 100;
        assessed.traits.composure = 100;
        assessed.traits.passing = 5;
        let changed = evaluate(ScenarioKind::OpenPlay, assessed, opponent).unwrap();
        assert_eq!(changed.recommended_action_id, "direct_dribble");
    }

    #[test]
    fn invalid_player_scores_are_rejected() {
        let (mut assessed, opponent) = demo_profiles(ScenarioKind::Penalty);
        assessed.traits.composure = 101;
        assert_eq!(
            evaluate(ScenarioKind::Penalty, assessed, opponent).unwrap_err(),
            EvaluationError::InvalidRatings
        );
    }

    #[test]
    fn repeated_evaluation_is_deterministic() {
        let first = evaluate_demo(ScenarioKind::FreeKick);
        let second = evaluate_demo(ScenarioKind::FreeKick);
        assert_eq!(first.recommended_action_id, second.recommended_action_id);
        assert_eq!(
            first.improvement_index_points,
            second.improvement_index_points
        );
    }
}
