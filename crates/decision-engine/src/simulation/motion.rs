use super::*;
use crate::Traits;

pub(super) fn parameters(t: &Traits) -> MovementParameters {
    MovementParameters {
        max_speed_mm_s: 4000 + 30 * i32::from(t.acceleration),
        max_acceleration_mm_s2: 2000 + 40 * i32::from(t.acceleration),
        decision_delay_ms: (1 + u32::from(100 - t.reactions) / 25) * TICK_MS,
    }
}

pub(super) fn l1(v: Vector2) -> i32 {
    v.x.abs() + v.y.abs()
}
pub(super) fn difference(a: Vector2, b: Vector2) -> Vector2 {
    Vector2 {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
pub(super) fn distance(a: Vector2, b: Vector2) -> i32 {
    l1(difference(a, b))
}
pub(super) fn inside(p: Vector2, pitch: PitchGeometry) -> bool {
    p.x >= 0 && p.x <= pitch.length_mm && p.y >= 0 && p.y <= pitch.width_mm
}
pub(super) fn bounded(p: Vector2, pitch: PitchGeometry) -> Vector2 {
    Vector2 {
        x: p.x.clamp(0, pitch.length_mm),
        y: p.y.clamp(0, pitch.width_mm),
    }
}

/// Conservative total distance including the current tick, then braking each subsequent tick.
fn stopping_distance(v: i32, axis_delta: i32) -> i32 {
    let n = i64::from((v + axis_delta - 1) / axis_delta);
    let sum = n * i64::from(v) - n * (n - 1) * i64::from(axis_delta) / 2;
    ((sum + 9) / 10) as i32
}

fn safe_speed(room: i32, max: i32, axis_delta: i32) -> i32 {
    let (mut low, mut high) = (0, max);
    while low < high {
        let mid = (low + high + 1) / 2;
        if stopping_distance(mid, axis_delta) <= room {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    low
}

pub(super) fn valid_player(p: &PlayerState, m: MovementParameters, pitch: PitchGeometry) -> bool {
    if !inside(p.position_mm, pitch)
        || i64::from(p.velocity_mm_s.x).abs() > i64::from(m.max_speed_mm_s)
        || i64::from(p.velocity_mm_s.y).abs() > i64::from(m.max_speed_mm_s)
        || l1(p.velocity_mm_s) > m.max_speed_mm_s
    {
        return false;
    }
    let delta = m.max_acceleration_mm_s2 / 20;
    // Initial state must admit braking beginning on the first tick.
    [
        (p.position_mm.x, p.velocity_mm_s.x, pitch.length_mm),
        (p.position_mm.y, p.velocity_mm_s.y, pitch.width_mm),
    ]
    .into_iter()
    .all(|(pos, v, limit)| {
        let room = if v < 0 { pos } else { limit - pos };
        stopping_distance((v.abs() - delta).max(0), delta) <= room
    })
}

/// L1 speed budget; half the acceleration budget on each axis. No boundary snapping.
pub(super) fn advance(
    p: &mut PlayerState,
    target: Vector2,
    m: MovementParameters,
    pitch: PitchGeometry,
) {
    let d = difference(target, p.position_mm);
    let norm = i64::from(l1(d));
    let delta = m.max_acceleration_mm_s2 / 20;
    let mut desired = if norm == 0 {
        Vector2::default()
    } else {
        let speed = i64::from(m.max_speed_mm_s).min(norm * 10);
        Vector2 {
            x: (i64::from(d.x) * speed / norm) as i32,
            y: (i64::from(d.y) * speed / norm) as i32,
        }
    };
    // Brake before a destination, rather than circling it at the full speed cap.
    desired.x = desired.x.signum()
        * desired
            .x
            .abs()
            .min(safe_speed(d.x.abs(), m.max_speed_mm_s, delta));
    desired.y = desired.y.signum()
        * desired
            .y
            .abs()
            .min(safe_speed(d.y.abs(), m.max_speed_mm_s, delta));
    let axis = |pos: i32, v: i32, goal: i32, limit: i32| {
        let wanted = v + (goal - v).clamp(-delta, delta);
        let room = if wanted < 0 { pos } else { limit - pos };
        let cap = safe_speed(room, m.max_speed_mm_s, delta);
        wanted.clamp(-cap, cap)
    };
    let mut velocity = Vector2 {
        x: axis(
            p.position_mm.x,
            p.velocity_mm_s.x,
            desired.x,
            pitch.length_mm,
        ),
        y: axis(
            p.position_mm.y,
            p.velocity_mm_s.y,
            desired.y,
            pitch.width_mm,
        ),
    };
    // Independent axis steering may briefly exceed the combined budget during turns.
    // Remove the excess from growing components, preserving each axis acceleration bound.
    let mut excess = (l1(velocity) - m.max_speed_mm_s).max(0);
    for (new, old) in [
        (&mut velocity.x, p.velocity_mm_s.x),
        (&mut velocity.y, p.velocity_mm_s.y),
    ] {
        let reduction = excess.min((new.abs() - old.abs()).max(0));
        *new -= new.signum() * reduction;
        excess -= reduction;
    }
    p.velocity_mm_s = velocity;
    p.position_mm.x += velocity.x / 10;
    p.position_mm.y += velocity.y / 10;
}

pub(super) fn move_ball(from: Vector2, to: Vector2, budget: i32) -> Vector2 {
    let d = difference(to, from);
    let norm = distance(from, to);
    if norm <= budget {
        return to;
    }
    Vector2 {
        x: from.x + (i64::from(d.x) * i64::from(budget) / i64::from(norm)) as i32,
        y: from.y + (i64::from(d.y) * i64::from(budget) / i64::from(norm)) as i32,
    }
}

/// Closest point on the ball's segment to the defender's end-of-tick position.
/// All products use i64; validated pitch bounds prevent overflow.
pub(super) fn contact(from: Vector2, to: Vector2, defender: Vector2) -> Option<Vector2> {
    let d = difference(to, from);
    let q = difference(defender, from);
    let squared = i64::from(d.x).pow(2) + i64::from(d.y).pow(2);
    let dot = i64::from(q.x) * i64::from(d.x) + i64::from(q.y) * i64::from(d.y);
    let point = if squared == 0 {
        from
    } else {
        let dot = dot.clamp(0, squared);
        Vector2 {
            x: from.x + (i64::from(d.x) * dot / squared) as i32,
            y: from.y + (i64::from(d.y) * dot / squared) as i32,
        }
    };
    let offset = difference(point, defender);
    (i64::from(offset.x).pow(2) + i64::from(offset.y).pow(2) <= i64::from(CONTROL_RADIUS_MM).pow(2))
        .then_some(point)
}
