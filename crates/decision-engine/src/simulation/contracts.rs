use crate::PlayerProfile;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SCENARIO_VERSION: &str = "open_play_spatial_v1";
pub const TICK_MS: u32 = 100;
pub const MAX_DURATION_MS: u32 = 6000;
pub const CONTROL_RADIUS_MM: i32 = 1000;

/// Integer millimetres for positions; millimetres/second for velocities.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vector2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PitchGeometry {
    pub length_mm: i32,
    pub width_mm: i32,
}

impl Default for PitchGeometry {
    fn default() -> Self {
        Self {
            length_mm: 105_000,
            width_mm: 68_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Participant {
    Assessed,
    Opponent,
    Teammate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerState {
    pub position_mm: Vector2,
    pub velocity_mm_s: Vector2,
}

/// Possession is a single tagged value, avoiding multiple contradictory owner flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Possession {
    Owned {
        owner: Participant,
    },
    InFlight {
        from: Participant,
        to: Participant,
        target_mm: Vector2,
    },
    Loose,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BallState {
    pub position_mm: Vector2,
    pub velocity_mm_s: Vector2,
    pub possession: Possession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticalAction {
    DirectDribble,
    WideLeft,
    WideRight,
    ReleasePass,
    OneTwo,
    Retain,
}

impl TacticalAction {
    pub fn id(self) -> &'static str {
        match self {
            Self::DirectDribble => "direct_dribble",
            Self::WideLeft => "wide_left",
            Self::WideRight => "wide_right",
            Self::ReleasePass => "release_pass",
            Self::OneTwo => "one_two",
            Self::Retain => "retain",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationState {
    pub elapsed_ms: u32,
    pub assessed: PlayerState,
    pub opponent: PlayerState,
    pub teammate: Option<PlayerState>,
    pub ball: BallState,
    pub current_action: TacticalAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationConfig {
    pub pitch: PitchGeometry,
    pub tick_ms: u32,
    pub duration_ms: u32,
    pub adaptation_at_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationRequest {
    pub scenario_version: String,
    #[serde(deserialize_with = "read_profile")]
    pub assessed_player: PlayerProfile,
    #[serde(deserialize_with = "read_profile")]
    pub opponent: PlayerProfile,
    #[serde(default, deserialize_with = "read_optional_profile")]
    pub supporting_teammate: Option<PlayerProfile>,
    pub initial_state: SimulationState,
    pub config: SimulationConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementParameters {
    pub max_speed_mm_s: i32,
    pub max_acceleration_mm_s2: i32,
    pub decision_delay_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseOption {
    pub id: String,
    pub legacy_score_source_id: Option<String>,
    pub effectiveness_index: u8,
    pub target_mm: Vector2,
    pub feasible: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    ActionSelected {
        action: TacticalAction,
    },
    Adaptation {
        action: TacticalAction,
        feasible: bool,
        reason: String,
    },
    OpponentDecision {
        observed_action: TacticalAction,
        options: Vec<ResponseOption>,
        chosen_response_id: String,
        target_mm: Vector2,
    },
    PassReleased {
        from: Participant,
        to: Participant,
        target_mm: Vector2,
    },
    PossessionChanged {
        owner: Option<Participant>,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationEvent {
    pub id: String,
    pub elapsed_ms: u32,
    pub evidence: EventKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub id: String,
    pub state: SimulationState,
}

/// Lexicographic objective, with one-metre progress/clearance buckets for meaningful changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TacticalEvaluation {
    pub friendly_possession: bool,
    pub bypassed_defender: bool,
    pub forward_ball_progress_mm: i32,
    pub defender_clearance_mm: i32,
    pub objective_key: (bool, bool, i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTrace {
    pub id: String,
    pub corrective_action: Option<TacticalAction>,
    pub correction_applied: bool,
    pub events: Vec<SimulationEvent>,
    pub steps: Vec<TraceStep>,
    pub evaluation: TacticalEvaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeAssessment {
    pub action: TacticalAction,
    pub feasible_at_adaptation: bool,
    pub reason: String,
    pub trace: Option<SimulationTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    pub scenario_version: String,
    pub input: SimulationRequest,
    pub assessed_parameters: MovementParameters,
    pub opponent_parameters: MovementParameters,
    pub teammate_parameters: Option<MovementParameters>,
    pub assumptions: Vec<String>,
    pub baseline: SimulationTrace,
    pub alternatives: Vec<AlternativeAssessment>,
    pub recommended_correction: Option<TacticalAction>,
    pub recommended_trace_id: Option<String>,
    pub comparison_reason: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct SimulationError {
    pub field: &'static str,
    pub message: &'static str,
}

// The new boundary rejects unknown nested profile fields without changing the legacy API.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileInput {
    id: String,
    label: String,
    description: String,
    traits: TraitInput,
    synthetic: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TraitInput {
    acceleration: u8,
    technique: u8,
    passing: u8,
    finishing: u8,
    anticipation: u8,
    positioning: u8,
    reactions: u8,
    goalkeeping: u8,
    composure: u8,
}
impl From<ProfileInput> for PlayerProfile {
    fn from(p: ProfileInput) -> Self {
        let t = p.traits;
        Self {
            id: p.id,
            label: p.label,
            description: p.description,
            synthetic: p.synthetic,
            traits: crate::Traits {
                acceleration: t.acceleration,
                technique: t.technique,
                passing: t.passing,
                finishing: t.finishing,
                anticipation: t.anticipation,
                positioning: t.positioning,
                reactions: t.reactions,
                goalkeeping: t.goalkeeping,
                composure: t.composure,
            },
        }
    }
}
fn read_profile<'de, D: serde::Deserializer<'de>>(d: D) -> Result<PlayerProfile, D::Error> {
    ProfileInput::deserialize(d).map(Into::into)
}
fn read_optional_profile<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<PlayerProfile>, D::Error> {
    Option::<ProfileInput>::deserialize(d).map(|p| p.map(Into::into))
}
