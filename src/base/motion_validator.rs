use std::{rc::Rc, sync::Arc};

use super::{state_allocator::StateId, statespace::StateSpace};

#[derive(Clone, Debug, Default)]
pub struct MotionCheckStats {
    pub valid: u32,
    pub invalid: u32,
}

impl MotionCheckStats {
    pub fn valid_motion_count(&self) -> u32 {
        self.valid
    }

    pub fn invalid_motion_count(&self) -> u32 {
        self.invalid
    }

    pub fn checked_motion_count(&self) -> u32 {
        self.valid + self.invalid
    }

    pub fn valid_motion_fraction(&self) -> f64 {
        if self.valid == 0 {
            0.0
        } else {
            self.valid as f64 / (self.valid + self.invalid) as f64
        }
    }

    pub fn reset(&mut self) {
        self.valid = 0;
        self.invalid = 0;
    }
}

pub trait MotionValidator {
    fn from_state_space(state_space: Rc<dyn StateSpace>) -> Self;

    fn get_motion_check_stats(&self) -> &MotionCheckStats;
    fn get_motion_check_stats_mut(&mut self) -> &mut MotionCheckStats;

    /// Check if the path between two states (from s1 to s2) is valid. This function assumes s1
    /// is valid.
    ///
    /// This function updates the number of valid and invalid segments.
    fn check_motion(&self, s1: &StateId, s2: &StateId) -> bool;

    /// Check if the path between two states is valid. Also compute the last state that was
    /// valid and the time of that state. The time is used to parametrize the motion from s1 to s2, s1
    /// being at t = 0 and s2 being at t = 1. This function assumes s1 is valid.
    /// \param s1 start state of the motion to be checked (assumed to be valid)
    /// \param s2 final state of the motion to be checked
    /// \param lastValid first: storage for the last valid state; this need not be different from s1 or s2.
    /// second: the time (between 0 and 1) of the last valid state, on the motion from s1 to s2. If the
    /// function returns false, lastValid.first must be set to a valid state, even if that implies copying
    /// s1 to lastValid.first (in case lastValid.second = 0). If the function returns true, lastValid.first
    /// and lastValid.second should not be modified.
    ///
    /// This function updates the number of valid and invalid segments.
    fn check_motion_with_last_valid(
        &self,
        s1: &StateId,
        s2: &StateId,
        last_valid: &mut (Option<StateId>, f64),
    ) -> bool;

    fn reset_motion_counter(&mut self) {
        self.get_motion_check_stats_mut().reset();
    }
}
