//! Versioned, integer spatial rollout. All physical parameters are synthetic conventions.
mod contracts;
mod motion;
mod rollout;
pub use contracts::*;
pub use rollout::{demo_request, simulate};

#[cfg(test)]
mod tests;
