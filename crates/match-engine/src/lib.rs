//! Deterministic, validated event replay. No external sports feeds are used.
use reactcoach_analytics::summarize;
use reactcoach_shared::{EventKind, MatchEvent, MatchSnapshot, PitchPosition, Score, Team};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ReplayError {
    #[error("duplicate event id {0}")]
    DuplicateId(u64),
    #[error("invalid pitch coordinates in event {0}")]
    InvalidPosition(u64),
    #[error("player is missing for event {0}")]
    MissingPlayer(u64),
}

pub fn snapshot_at(
    match_id: &str,
    events: &[MatchEvent],
    at_seconds: u32,
) -> Result<MatchSnapshot, ReplayError> {
    let mut ids = HashSet::new();
    for event in events {
        if !ids.insert(event.id) {
            return Err(ReplayError::DuplicateId(event.id));
        }
        if !event.position.x.is_finite()
            || !event.position.y.is_finite()
            || !(0.0..=100.0).contains(&event.position.x)
            || !(0.0..=100.0).contains(&event.position.y)
        {
            return Err(ReplayError::InvalidPosition(event.id));
        }
        if event.player.trim().is_empty() {
            return Err(ReplayError::MissingPlayer(event.id));
        }
    }

    let mut selected: Vec<MatchEvent> = events
        .iter()
        .filter(|event| event.timestamp_seconds <= at_seconds)
        .cloned()
        .collect();
    selected.sort_by_key(|event| (event.timestamp_seconds, event.id));

    let mut score = Score::default();
    for event in &selected {
        if event.kind == EventKind::Goal {
            match event.team {
                Team::Home => score.home += 1,
                Team::Away => score.away += 1,
            }
        }
    }
    Ok(MatchSnapshot {
        match_id: match_id.to_string(),
        elapsed_seconds: at_seconds,
        score,
        statistics: summarize(&selected),
        events_processed: selected.len(),
        latest_event: selected.last().cloned(),
    })
}

/// Explicitly fictional event records used for demos and tests only.
pub fn demo_events() -> Vec<MatchEvent> {
    fn event(
        id: u64,
        time: u32,
        team: Team,
        kind: EventKind,
        player: &str,
        x: f32,
        y: f32,
    ) -> MatchEvent {
        MatchEvent {
            id,
            timestamp_seconds: time,
            team,
            kind,
            player: player.to_string(),
            position: PitchPosition { x, y },
        }
    }
    vec![
        event(1, 0, Team::Home, EventKind::KickOff, "Home 8", 50.0, 50.0),
        event(2, 40, Team::Home, EventKind::Pass, "Home 6", 42.0, 30.0),
        event(3, 85, Team::Away, EventKind::Tackle, "Away 4", 54.0, 37.0),
        event(4, 120, Team::Away, EventKind::Pass, "Away 10", 65.0, 67.0),
        event(5, 180, Team::Away, EventKind::Shot, "Away 9", 86.0, 47.0),
        event(
            6,
            245,
            Team::Home,
            EventKind::Turnover,
            "Home 5",
            35.0,
            60.0,
        ),
        event(7, 300, Team::Home, EventKind::Pass, "Home 7", 64.0, 42.0),
        event(8, 365, Team::Home, EventKind::Shot, "Home 11", 86.0, 55.0),
        event(9, 420, Team::Home, EventKind::Goal, "Home 9", 92.0, 48.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_before_and_after_goal() {
        let events = demo_events();
        let before = snapshot_at("demo", &events, 419).unwrap();
        let after = snapshot_at("demo", &events, 420).unwrap();
        assert_eq!(before.score.home, 0);
        assert_eq!(after.score.home, 1);
        assert_eq!(after.statistics.home_shots, 2);
        assert_eq!(after.events_processed, 9);
    }
    #[test]
    fn replay_is_order_independent() {
        let events = demo_events();
        let reversed: Vec<_> = events.iter().rev().cloned().collect();
        assert_eq!(
            snapshot_at("demo", &events, 420).unwrap(),
            snapshot_at("demo", &reversed, 420).unwrap()
        );
    }
    #[test]
    fn invalid_coordinates_rejected() {
        let mut events = demo_events();
        events[0].position.x = 101.0;
        assert_eq!(
            snapshot_at("demo", &events, 420).unwrap_err(),
            ReplayError::InvalidPosition(1)
        );
    }
    #[test]
    fn duplicate_ids_rejected() {
        let mut events = demo_events();
        events[1].id = 1;
        assert_eq!(
            snapshot_at("demo", &events, 420).unwrap_err(),
            ReplayError::DuplicateId(1)
        );
    }
}
