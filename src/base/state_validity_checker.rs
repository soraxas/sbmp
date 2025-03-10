use std::sync::Arc;

use super::{state_allocator::StateId, statespace::StateSpace};

#[derive(Debug, Copy, Clone, Default)]
pub enum ClearanceComputationType {
    /// No clearance computation
    #[default]
    None,
    /// Compute the clearance by sampling in the direction of the nearest obstacle
    Sampling,
    /// Compute the clearance by sampling in the direction of the nearest obstacle and
    /// then refining the result
    SamplingRefinement,
}

pub trait StateValidityChecker {
    fn is_valid(&self, state: &StateId) -> bool;

    /// Return the specifications (capabilities of this state validity checker)
    fn specs(&self) -> ClearanceComputationType {
        ClearanceComputationType::default()
    }

    /// Flag indicating that this state validity checker can return
    /// a direction that moves a state away from being invalid. */
    fn has_valid_direction_computation(&self) -> bool {
        false
    }

    /// Check if the state is valid and return the clearance.
    fn is_valid_with_distance(&self, state: &StateId) -> (bool, Option<f64>) {
        (self.is_valid(state), self.clearance(state))
    }

    /// Check if the state is valid and return the clearance.
    /// If a direction that moves the state towards a valid state is available,
    /// a valid state is also set
    fn is_valid_with_distance_and_state(
        &self,
        state: &StateId,
        valid_state: &mut StateId,
        valid_state_available: &mut bool,
    ) -> (bool, Option<f64>) {
        (
            self.is_valid(state),
            self.clearance_with_state(state, valid_state, valid_state_available),
        )
    }
    fn clearance(&self, _state: &StateId) -> Option<f64> {
        None
    }

    /// Report the distance to the nearest invalid state when starting from state, and if possible,
    /// also specify a valid state validState in the direction that moves away from the colliding
    /// state. The validStateAvailable flag is set to true if validState is updated.
    fn clearance_with_state(
        &self,
        state: &StateId,
        _valid_state: &mut StateId,
        valid_state_available: &mut bool,
    ) -> Option<f64> {
        *valid_state_available = false;
        self.clearance(state)
    }
}

pub type StateValidityCheckerFn = Box<dyn Fn(&StateId) -> bool>;

/// A state validity checker that uses a functional approach.
struct FunctionalStateValidityChecker(StateValidityCheckerFn);

impl StateValidityChecker for FunctionalStateValidityChecker {
    fn is_valid(&self, state: &StateId) -> bool {
        (self.0)(state)
    }
}

// impl that turns function into checker
impl From<StateValidityCheckerFn> for Box<dyn StateValidityChecker> {
    fn from(checker: StateValidityCheckerFn) -> Box<dyn StateValidityChecker> {
        Box::new(FunctionalStateValidityChecker(checker))
    }
}

/// The simplest state validity checker: all states are valid.
pub struct AllValidStateValidityChecker;

impl StateValidityChecker for AllValidStateValidityChecker {
    fn is_valid(&self, _state: &StateId) -> bool {
        true
    }
}
