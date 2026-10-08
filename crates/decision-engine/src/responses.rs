//! Deterministic synthetic opponent *best response* modeling.
//! Every attacking action has two distinct counter-options evaluated from
//! the same synthetic player profiles; opponent chooses strongest modeled
//! response, which reduces the action's fit index. Not a physical simulation.

use crate::{ScenarioKind, Traits};
use serde::{Deserialize, Serialize};

/// A single feasible opponent response to the assessed player's action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpponentResponseAssessment {
    pub id: String,
    pub label: String,
    pub explanation: String,
    pub effectiveness_index: u8,
    pub opponent_dominant_trait: String,
    pub assessed_escape_trait: String,
}

struct ResponseSpec {
    id: &'static str,
    label: &'static str,
    explanation: &'static str,
    opponent_weights: [u8; 9],
    escape_weights: [u8; 9],
}

const RESPONSES: [[ResponseSpec; 2]; 9] = [
    [
        ResponseSpec {
            id: "hold_central_lane",
            label: "Hold the central lane",
            explanation: "Uses anticipation and positioning to obstruct the direct route.",
            opponent_weights: [0, 0, 0, 0, 60, 40, 0, 0, 0],
            escape_weights: [0, 75, 0, 0, 0, 0, 0, 0, 25],
        },
        ResponseSpec {
            id: "step_into_tackle",
            label: "Step into the challenge",
            explanation: "Uses speed and reactions to challenge the ball carrier early.",
            opponent_weights: [45, 0, 0, 0, 0, 15, 40, 0, 0],
            escape_weights: [40, 40, 0, 0, 0, 0, 0, 0, 20],
        },
    ],
    [
        ResponseSpec {
            id: "track_wide_run",
            label: "Track the outside run",
            explanation: "Uses acceleration and reactions to recover alongside the wide movement.",
            opponent_weights: [75, 0, 0, 0, 0, 0, 25, 0, 0],
            escape_weights: [80, 20, 0, 0, 0, 0, 0, 0, 0],
        },
        ResponseSpec {
            id: "close_wide_angle",
            label: "Close the attacking angle",
            explanation: "Anticipates the wide route and defends the channel instead of racing.",
            opponent_weights: [0, 0, 0, 0, 30, 70, 0, 0, 0],
            escape_weights: [0, 60, 0, 0, 0, 0, 0, 0, 40],
        },
    ],
    [
        ResponseSpec {
            id: "intercept_return_pass",
            label: "Intercept the return pass",
            explanation: "Reads the combination and occupies the return passing lane.",
            opponent_weights: [0, 0, 0, 0, 50, 50, 0, 0, 0],
            escape_weights: [0, 0, 80, 0, 20, 0, 0, 0, 0],
        },
        ResponseSpec {
            id: "follow_receiving_runner",
            label: "Follow the receiving runner",
            explanation:
                "Follows the player making the return run instead of stepping toward the ball.",
            opponent_weights: [55, 0, 0, 0, 0, 45, 0, 0, 0],
            escape_weights: [65, 0, 0, 0, 35, 0, 0, 0, 0],
        },
    ],
    [
        ResponseSpec {
            id: "commit_early_dive",
            label: "Commit to an early dive",
            explanation: "Chooses a side early, relying on reflexes and shot-stopping.",
            opponent_weights: [0, 0, 0, 0, 0, 0, 55, 45, 0],
            escape_weights: [0, 0, 0, 70, 0, 0, 0, 0, 30],
        },
        ResponseSpec {
            id: "protect_central_goal",
            label: "Protect the central goal",
            explanation: "Maintains a central position to respond to a less precise strike.",
            opponent_weights: [0, 0, 0, 0, 0, 55, 0, 45, 0],
            escape_weights: [0, 35, 0, 45, 0, 0, 0, 0, 20],
        },
    ],
    [
        ResponseSpec {
            id: "anticipate_placement",
            label: "Anticipate the corner",
            explanation: "Uses anticipation to pre-position for a controlled placed shot.",
            opponent_weights: [0, 0, 0, 0, 55, 0, 0, 45, 0],
            escape_weights: [0, 65, 0, 15, 0, 0, 0, 0, 20],
        },
        ResponseSpec {
            id: "reaction_save",
            label: "Wait and react",
            explanation:
                "Avoids committing early and instead reacts to the ball's actual trajectory.",
            opponent_weights: [0, 0, 0, 0, 0, 0, 65, 35, 0],
            escape_weights: [0, 20, 0, 60, 0, 0, 0, 0, 20],
        },
    ],
    [
        ResponseSpec {
            id: "delay_goalkeeper_dive",
            label: "Delay the goalkeeper dive",
            explanation: "Holds position and uses composure before committing.",
            opponent_weights: [0, 0, 0, 0, 0, 0, 35, 35, 30],
            escape_weights: [0, 0, 0, 20, 0, 0, 0, 0, 80],
        },
        ResponseSpec {
            id: "read_runup",
            label: "Read the run-up",
            explanation: "Uses anticipation and reactions to predict the shooter's intention.",
            opponent_weights: [0, 0, 0, 0, 65, 0, 35, 0, 0],
            escape_weights: [0, 45, 0, 0, 0, 0, 0, 0, 55],
        },
    ],
    [
        ResponseSpec {
            id: "protect_near_post",
            label: "Protect the near post",
            explanation: "Uses positioning and goalkeeping to cover the direct shooting path.",
            opponent_weights: [0, 0, 0, 0, 0, 45, 0, 55, 0],
            escape_weights: [0, 20, 0, 60, 0, 0, 0, 0, 20],
        },
        ResponseSpec {
            id: "track_power_shot",
            label: "Track the power shot",
            explanation: "Relies on reactions and shot-stopping to meet a powerful delivery.",
            opponent_weights: [0, 0, 0, 0, 0, 0, 60, 40, 0],
            escape_weights: [0, 30, 0, 50, 0, 0, 0, 0, 20],
        },
    ],
    [
        ResponseSpec {
            id: "reposition_for_curl",
            label: "Reposition against curl",
            explanation: "Shifts the initial goalkeeping line to anticipate the bent trajectory.",
            opponent_weights: [0, 0, 0, 0, 0, 45, 0, 55, 0],
            escape_weights: [0, 70, 0, 15, 0, 0, 0, 0, 15],
        },
        ResponseSpec {
            id: "late_curl_save",
            label: "React to the curl",
            explanation: "Waits for the flight of the ball and attempts a reactive save.",
            opponent_weights: [0, 0, 0, 0, 0, 0, 65, 35, 0],
            escape_weights: [0, 40, 0, 45, 0, 0, 0, 0, 15],
        },
    ],
    [
        ResponseSpec {
            id: "press_short_receiver",
            label: "Press the short receiver",
            explanation:
                "Steps to the receiver quickly, seeking to break the short passing routine.",
            opponent_weights: [50, 0, 0, 0, 0, 50, 0, 0, 0],
            escape_weights: [0, 0, 65, 0, 35, 0, 0, 0, 0],
        },
        ResponseSpec {
            id: "protect_delivery_zone",
            label: "Protect the delivery zone",
            explanation:
                "Holds the line and anticipates the second pass rather than chasing the first.",
            opponent_weights: [0, 0, 0, 0, 55, 45, 0, 0, 0],
            escape_weights: [0, 0, 60, 0, 0, 0, 0, 0, 40],
        },
    ],
];

fn action_index(scenario: ScenarioKind, action: &str) -> Option<usize> {
    match (scenario, action) {
        (ScenarioKind::OpenPlay, "direct_dribble") => Some(0),
        (ScenarioKind::OpenPlay, "accelerate_wide") => Some(1),
        (ScenarioKind::OpenPlay, "quick_combination") => Some(2),
        (ScenarioKind::Penalty, "power_shot") => Some(3),
        (ScenarioKind::Penalty, "placed_shot") => Some(4),
        (ScenarioKind::Penalty, "delayed_placement") => Some(5),
        (ScenarioKind::FreeKick, "direct_power") => Some(6),
        (ScenarioKind::FreeKick, "placed_curl") => Some(7),
        (ScenarioKind::FreeKick, "short_routine") => Some(8),
        _ => None,
    }
}

pub fn evaluate_responses(
    scenario: ScenarioKind,
    action: &str,
    assessed: &Traits,
    opponent: &Traits,
) -> Vec<OpponentResponseAssessment> {
    let Some(index) = action_index(scenario, action) else {
        return Vec::new();
    };
    RESPONSES[index]
        .iter()
        .map(|option| {
            let opponent_fit = super::weighted(opponent, &option.opponent_weights);
            let escape_fit = super::weighted(assessed, &option.escape_weights);
            // This is a deliberately simple index, NOT a match success percentage.
            let effectiveness =
                ((u16::from(opponent_fit) * 3 + u16::from(100 - escape_fit) + 2) / 4) as u8;
            OpponentResponseAssessment {
                id: option.id.into(),
                label: option.label.into(),
                explanation: option.explanation.into(),
                effectiveness_index: effectiveness,
                opponent_dominant_trait: super::leading_strength(
                    opponent,
                    &option.opponent_weights,
                )
                .into(),
                assessed_escape_trait: super::leading_strength(assessed, &option.escape_weights)
                    .into(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_counterchoices_for_every_scenario_action() {
        for (scenario, ids) in [
            (
                ScenarioKind::OpenPlay,
                ["direct_dribble", "accelerate_wide", "quick_combination"],
            ),
            (
                ScenarioKind::Penalty,
                ["power_shot", "placed_shot", "delayed_placement"],
            ),
            (
                ScenarioKind::FreeKick,
                ["direct_power", "placed_curl", "short_routine"],
            ),
        ] {
            for action in ids {
                let (player, opponent) = crate::demo_profiles(scenario);
                let responses =
                    evaluate_responses(scenario, action, &player.traits, &opponent.traits);
                assert_eq!(responses.len(), 2);
                assert!(responses
                    .iter()
                    .all(|response| response.effectiveness_index <= 100));
            }
        }
    }

    #[test]
    fn opponent_best_response_switches_with_their_strengths() {
        let (attacker, mut defender) = crate::demo_profiles(ScenarioKind::OpenPlay);
        defender.traits.acceleration = 0;
        defender.traits.reactions = 0;
        defender.traits.positioning = 100;
        defender.traits.anticipation = 100;
        let first = evaluate_responses(
            ScenarioKind::OpenPlay,
            "accelerate_wide",
            &attacker.traits,
            &defender.traits,
        );
        let first_best = first.iter().max_by_key(|r| r.effectiveness_index).unwrap();
        assert_eq!(first_best.id, "close_wide_angle");

        defender.traits.acceleration = 100;
        defender.traits.reactions = 100;
        defender.traits.positioning = 0;
        defender.traits.anticipation = 0;
        let second = evaluate_responses(
            ScenarioKind::OpenPlay,
            "accelerate_wide",
            &attacker.traits,
            &defender.traits,
        );
        let second_best = second.iter().max_by_key(|r| r.effectiveness_index).unwrap();
        assert_eq!(second_best.id, "track_wide_run");
    }
}
