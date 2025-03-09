use itertools::izip;
use nalgebra::DVector;
use sbmp_derive::{state_id_into_inner, WithStateAlloc, WithStateSpaceData};
use std::collections::HashMap;
use std::sync::Arc;

use crate::base::state::State;
use crate::base::state_allocator::{StateAllocator, StateId};
use crate::base::state_sampler::StateSampler;
use crate::base::statespace::{HasStateSpaceData, StateSpace, StateSpaceCommonData};
use crate::prelude::CanStateAllocateTrait;
use crate::randomness::RNG;

use super::real_vector_bounds::RealVectorBounds;

// write a derive macro that automatically add a member struct of type HashMap<String, usize> with name HAHA to the struct

#[derive(Debug, WithStateSpaceData, WithStateAlloc)]
#[state_alloc(state_type = "RealVectorState")]
pub struct RealVectorStateSpace {
    state_space_data: StateSpaceCommonData,
    state_allocator: StateAllocator<RealVectorState>,
    state_bytes: usize,
    // pub(crate) dimension: u32,
    pub(crate) bounds: RealVectorBounds,
    pub(crate) dimension_names: Vec<String>,
    pub(crate) dimension_index: HashMap<String, usize>,
}

impl Default for RealVectorStateSpace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RealVectorState {
    pub values: DVector<f64>,
}

impl State for RealVectorState {}

impl RealVectorStateSpace {
    pub fn new() -> Self {
        Self {
            state_space_data: StateSpaceCommonData::default(),
            state_allocator: Self::new_state_allocator(),
            // dimension: 0,
            state_bytes: 0,
            bounds: RealVectorBounds {
                low: vec![],
                high: vec![],
            },
            dimension_names: vec![],
            dimension_index: HashMap::new(),
        }
    }

    pub fn add_dimension(&mut self, name: Option<String>, min_bound: f64, max_bound: f64) {
        self.bounds.low.push(min_bound);
        self.bounds.high.push(max_bound);
        self.dimension_names.push(name.unwrap_or_default());
        self.state_bytes = self.dimension_names.len() * std::mem::size_of::<f64>();
    }

    pub fn set_bounds(&mut self, bounds: RealVectorBounds) {
        bounds.check();
        assert_eq!(
            bounds.low.len(),
            bounds.high.len(),
            "Bounds do not match dimension of state space"
        );
        self.bounds = bounds;
    }

    pub fn get_dimension_name(&self, index: usize) -> &str {
        &self.dimension_names[index]
    }

    pub fn set_dimension_name(&mut self, index: usize, name: String) {
        self.dimension_names[index] = name.clone();
        self.dimension_index.insert(name, index);
    }
}

impl StateSpace for RealVectorStateSpace {
    fn get_dimension(&self) -> u32 {
        self.bounds.low.len() as u32
    }
    fn get_maximum_extent(&self) -> f64 {
        self.bounds
            .low
            .iter()
            .zip(&self.bounds.high)
            .map(|(low, high)| (high - low).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    fn get_measure(&self) -> f64 {
        self.bounds
            .low
            .iter()
            .zip(&self.bounds.high)
            .map(|(low, high)| high - low)
            .product()
    }

    #[state_id_into_inner]
    fn enforce_bounds(&self, state: &mut StateId) {
        for (s, low, high) in izip!(state.values.iter_mut(), &self.bounds.low, &self.bounds.high) {
            *s = s.clamp(*low, *high);
        }
    }

    #[state_id_into_inner]
    fn satisfies_bounds(&self, state: &StateId) -> bool {
        state
            .values
            .iter()
            .zip(&self.bounds.low)
            .zip(&self.bounds.high)
            .all(|((s, low), high)| s - f64::EPSILON > *low && s + f64::EPSILON < *high)
    }

    #[state_id_into_inner]
    fn distance(&self, state1: &StateId, state2: &StateId) -> f64 {
        (&state1.values - &state2.values).norm()
    }

    #[state_id_into_inner]
    fn equal_states(&self, state1: &StateId, state2: &StateId) -> bool {
        (&state1.values - &state2.values)
            .abs()
            .iter()
            .all(|x| *x <= f64::EPSILON * 2.0)
    }

    #[state_id_into_inner]
    fn interpolate(&self, from: &StateId, to: &StateId, t: f64, state: &mut StateId) {
        state.values = &from.values + (&to.values - &from.values) * t;
    }

    fn setup(&mut self) {
        todo!()
    }

    #[state_id_into_inner]
    fn copy_state(&self, destination: &mut StateId, source: &StateId) {
        destination.values.copy_from(&source.values);
    }

    fn alloc_state(&self) -> StateId where {
        self.alloc_arena_state_with_value(RealVectorState {
            values: DVector::zeros(self.dimension_names.len()),
        })
    }

    fn free_state(&self, state: &StateId) {
        self.free_arena_state(state);
    }
}

pub struct RealVectorStateSampler {
    space: Arc<dyn StateSpace>,
    rng: RNG,
}

impl StateSampler for RealVectorStateSampler {
    fn from_state_space(space: Arc<dyn StateSpace>) -> Self {
        Self {
            space,
            rng: RNG::new(),
        }
    }

    fn sample_uniform(&mut self, state: &mut StateId) {
        let space = self.space.downcast_ref::<RealVectorStateSpace>().unwrap();

        space.with_state_mut(state, |state| {
            let state = &mut state.values;
            for (state, low, high) in izip!(state, &space.bounds.low, &space.bounds.high) {
                *state = self.rng.uniform_real(*low, *high);
            }
        });
    }

    fn sample_uniform_near(&mut self, state: &mut StateId, near: &StateId, distance: f64) {
        let space = self.space.downcast_ref::<RealVectorStateSpace>().unwrap();

        space.with_2states_mut(state, near, |state, near| {
            let state = &mut state.values;
            let near = &near.values;

            for (state, near, low, high) in
                izip!(state, near, &space.bounds.low, &space.bounds.high)
            {
                *state = self.rng.uniform_real(
                    f64::max(*low, *near - distance),
                    f64::min(*high, *near + distance),
                );
            }
        });
    }

    fn sample_gaussian(&mut self, state: &mut StateId, mean: &StateId, std_dev: f64) {
        let space = self.space.downcast_ref::<RealVectorStateSpace>().unwrap();

        space.with_2states_mut(state, mean, |state, mean| {
            let state = &mut state.values;
            let mean = &mean.values;

            for (state, mean, low, high) in
                izip!(state, mean, &space.bounds.low, &space.bounds.high)
            {
                *state = self.rng.gaussian(*mean, std_dev).clamp(*low, *high);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use statrs::assert_almost_eq;

    #[test]
    fn test_rv_distance() {
        let mut space = RealVectorStateSpace::new();

        space.add_dimension(None, 0.0, 1.0);
        space.add_dimension(None, 1.0, 1.9);

        let state1 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![0.5, 1.5]),
        });
        let state2 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![1.0, 1.5]),
        });
        assert_almost_eq!(space.distance(&state1, &state2), 0.5, f64::EPSILON);

        space.with_state_mut(&state2, |state| {
            state.values[1] = 1.;
        });
        assert_almost_eq!(
            space.distance(&state1, &state2),
            (2.0f64 * 0.5 * 0.5).sqrt(),
            f64::EPSILON
        );
    }

    #[test]
    fn test_rv_sample() {
        let mut space = RealVectorStateSpace::new();

        space.add_dimension(None, 0.0, 1.0);
        space.add_dimension(None, 1.0, 1.9);

        let space = Arc::new(space);
        let mut state1 = space.alloc_state();

        let mut sampler = RealVectorStateSampler::from_state_space(space.clone());

        for _ in 0..100 {
            sampler.sample_uniform(&mut state1);
            // dbg!(space.clone_state_inner_value(&state1)
            space.with_state(&state1, |state| {
                assert!(0. < state.values[0] && state.values[0] < 1.);
                assert!(1. < state.values[1] && state.values[1] < 1.9);
            });
        }

        let state1 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![0.5, 1.5]),
        });
        let state2 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![1.0, 1.5]),
        });
        assert_almost_eq!(space.distance(&state1, &state2), 0.5, f64::EPSILON);

        space.with_state_mut(&state2, |state| {
            state.values[1] = 1.;
        });
        assert_almost_eq!(
            space.distance(&state1, &state2),
            (2.0f64 * 0.5 * 0.5).sqrt(),
            f64::EPSILON
        );
    }

    #[test]
    fn test_rv_interpolate() {
        let mut space = RealVectorStateSpace::new();

        space.add_dimension(None, 0.0, 1.0);
        space.add_dimension(None, 1.0, 1.9);
        space.add_dimension(None, 100.0, 100.9);

        let state1 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![0.5, 1.5, 8.0]),
        });
        let state1_same = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![0.5, 1.5, 8.0]),
        });
        let state2 = space.alloc_arena_state_with_value(RealVectorState {
            values: DVector::from_vec(vec![1.0, 1.5, 100.0]),
        });

        assert!(state1 == state1);
        assert!(state1 != state2);
        assert!(space.equal_states(&state1, &state1_same));
        assert!(!space.equal_states(&state1, &state2));

        let mut result = space.alloc_state();
        space.interpolate(&state1, &state2, 0.5, &mut result);
        assert_eq!(
            space
                .clone_state_inner_value(&result)
                .downcast::<RealVectorState>()
                .unwrap()
                .values,
            DVector::from_vec(vec![0.75, 1.5, 54.0])
        );
        // assert_almost_eq!();
    }
}
