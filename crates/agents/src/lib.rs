//! Agent workflow contracts only. Microsoft Foundry calls are NOT implemented yet.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    TacticalAnalyst,
    NarrativeWriter,
    AudiencePersonalizer,
    EvidenceValidator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub role: AgentRole,
    pub description: String,
}

/// Describes planned handoffs, not an active AI or model invocation.
pub fn planned_workflow() -> Vec<AgentStep> {
    vec![
        AgentStep {
            role: AgentRole::TacticalAnalyst,
            description: "Interpret supplied deterministic analytics".into(),
        },
        AgentStep {
            role: AgentRole::NarrativeWriter,
            description: "Draft evidence-linked match narrative".into(),
        },
        AgentStep {
            role: AgentRole::AudiencePersonalizer,
            description: "Adapt tone and language for audience".into(),
        },
        AgentStep {
            role: AgentRole::EvidenceValidator,
            description: "Check claims against source event identifiers".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validator_is_final_step() {
        assert_eq!(
            planned_workflow().last().unwrap().role,
            AgentRole::EvidenceValidator
        );
    }
}
