//! One-dimensional synthetic offside *position* demonstrator.
//! It deliberately cannot decide an offside *offence*: active involvement,
//! deliberate play, restarts, and other Law 11 conditions are not modeled.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OffsideFrame {
    /// Coordinate along pitch length toward opponents' goal, 0..=100.
    /// For this MVP the attacking direction is always towards x=100.
    pub attacker_forward_edge: f32,
    pub ball_forward_edge: f32,
    pub second_last_opponent_forward_edge: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OffsidePosition {
    OnsidePosition,
    OffsidePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffsideAssessment {
    pub position: OffsidePosition,
    pub attacker_forward_edge: f32,
    pub offside_line: f32,
    pub explanation: String,
    pub rule_scope: String,
}

pub fn assess_position(frame: OffsideFrame) -> Option<OffsideAssessment> {
    let numbers = [frame.attacker_forward_edge, frame.ball_forward_edge, frame.second_last_opponent_forward_edge];
    if numbers.iter().any(|n| !n.is_finite() || !(0.0..=100.0).contains(n)) {
        return None;
    }
    let line = frame.ball_forward_edge.max(frame.second_last_opponent_forward_edge);
    let offside = frame.attacker_forward_edge > 50.0 && frame.attacker_forward_edge > line;
    Some(OffsideAssessment {
        position: if offside { OffsidePosition::OffsidePosition } else { OffsidePosition::OnsidePosition },
        attacker_forward_edge: frame.attacker_forward_edge,
        offside_line: line,
        explanation: if offside {
            "The modeled attacker's eligible body edge is beyond the ball and second-last opponent in the opponents' half at the pass instant.".into()
        } else {
            "The modeled attacker's eligible body edge is not beyond both the ball and second-last opponent in the opponents' half at the pass instant.".into()
        },
        rule_scope: "Position check only (IFAB Law 11). Not an offside offence decision; involvement, restarts and special circumstances require separate evaluation. All coordinates are synthetic proxies.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ahead_of_ball_and_second_last_opponent_is_offside_position() {
        let outcome = assess_position(OffsideFrame { attacker_forward_edge: 79.0, ball_forward_edge: 65.0, second_last_opponent_forward_edge: 74.0 }).unwrap();
        assert_eq!(outcome.position, OffsidePosition::OffsidePosition);
    }
    #[test]
    fn level_with_second_last_opponent_is_onside_position() {
        let outcome = assess_position(OffsideFrame { attacker_forward_edge: 74.0, ball_forward_edge: 65.0, second_last_opponent_forward_edge: 74.0 }).unwrap();
        assert_eq!(outcome.position, OffsidePosition::OnsidePosition);
    }
    #[test]
    fn cannot_be_offside_position_in_own_half() {
        let outcome = assess_position(OffsideFrame { attacker_forward_edge: 49.0, ball_forward_edge: 20.0, second_last_opponent_forward_edge: 40.0 }).unwrap();
        assert_eq!(outcome.position, OffsidePosition::OnsidePosition);
    }
    #[test]
    fn ball_ahead_of_attacker_is_onside_position() {
        let outcome = assess_position(OffsideFrame { attacker_forward_edge: 73.0, ball_forward_edge: 80.0, second_last_opponent_forward_edge: 60.0 }).unwrap();
        assert_eq!(outcome.position, OffsidePosition::OnsidePosition);
    }
    #[test]
    fn invalid_coordinates_rejected() {
        assert!(assess_position(OffsideFrame { attacker_forward_edge: f32::NAN, ball_forward_edge: 64.0, second_last_opponent_forward_edge: 70.0 }).is_none());
    }
}
