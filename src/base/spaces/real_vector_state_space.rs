use itertools::izip;
use nalgebra::DVector;
use rand::Rng;
use sbmp_derive::WithStateSpaceData;
use std::cmp;
use std::collections::HashMap;
use std::f64::consts::E;
use std::f64::EPSILON;

use crate::base::state::State;
use crate::base::statespace::{HasStateSpaceData, StateSpace, StateSpaceCommonData};
use crate::downcast_state;
use crate::randomness::RNG;

use super::real_vector_bounds::RealVectorBounds;

// write a derive macro that automatically add a member struct of type HashMap<String, usize> with name HAHA to the struct

#[derive(Debug, WithStateSpaceData)]
pub struct RealVectorStateSpace {
    state_space_data: StateSpaceCommonData,
    state_bytes: usize,
    pub(crate) dimension: usize,
    pub(crate) bounds: RealVectorBounds,
    pub(crate) dimension_names: Vec<String>,
    pub(crate) dimension_index: HashMap<String, usize>,
}

impl Default for RealVectorStateSpace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct RealVectorState {
    pub values: DVector<f64>,
}

impl State for RealVectorState {}

impl RealVectorStateSpace {
    pub fn new() -> Self {
        Self {
            state_space_data: StateSpaceCommonData::default(),
            dimension: 0,
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
        self.dimension += 1;
        self.state_bytes = self.dimension * std::mem::size_of::<f64>();
        self.bounds.low.push(min_bound);
        self.bounds.high.push(max_bound);
        self.dimension_names.push(name.unwrap_or_default());
    }

    pub fn set_bounds(&mut self, bounds: RealVectorBounds) {
        bounds.check();
        assert_eq!(
            bounds.low.len(),
            self.dimension,
            "Bounds do not match dimension of state space"
        );
        self.bounds = bounds;
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }

    pub fn get_dimension_name(&self, index: usize) -> &str {
        &self.dimension_names[index]
    }

    pub fn set_dimension_name(&mut self, index: usize, name: String) {
        self.dimension_names[index] = name.clone();
        self.dimension_index.insert(name, index);
    }
}

// support ref or mut
macro_rules! cast_to_rv_state {
    // Catch-all pattern for all input, calls downcast_state! with 'from' and 'RealVectorState'
    ($($input:tt)*) => {
        downcast_state!($($input)*, RealVectorState)
    };
}

impl StateSpace for RealVectorStateSpace {
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

    fn enforce_bounds(&self, state: &mut dyn State) {
        let state = &mut cast_to_rv_state!(mut state).values;
        for (s, low, high) in izip!(state, &self.bounds.low, &self.bounds.high) {
            *s = s.clamp(*low, *high);
        }
    }

    fn satisfies_bounds(&self, state: &dyn State) -> bool {
        let state = &cast_to_rv_state!(state).values;

        state
            .iter()
            .zip(&self.bounds.low)
            .zip(&self.bounds.high)
            .all(|((s, low), high)| s - f64::EPSILON > *low && s + f64::EPSILON < *high)
    }

    fn distance(&self, state1: &dyn State, state2: &dyn State) -> f64 {
        let state1 = &cast_to_rv_state!(state1).values;
        let state2 = &cast_to_rv_state!(state2).values;
        (state1 - state2).norm()
    }

    fn equal_states(&self, state1: &dyn State, state2: &dyn State) -> bool {
        let state1 = &cast_to_rv_state!(state1).values;
        let state2 = &cast_to_rv_state!(state2).values;

        (state1 - state2)
            .abs()
            .iter()
            .all(|x| *x <= f64::EPSILON * 2.0)
    }

    fn interpolate(&self, from: &dyn State, to: &dyn State, time: f64, state: &mut dyn State) {
        let from = &cast_to_rv_state!(from).values;
        let to = &cast_to_rv_state!(to).values;
        let state = &mut cast_to_rv_state!(mut state).values;

        *state = from + (to - from) * time;
    }

    fn setup(&mut self) {
        todo!()
    }
}

pub struct RealVectorStateSampler<'a> {
    space: &'a RealVectorStateSpace,
    rng: RNG,
}

impl<'a> RealVectorStateSampler<'a> {
    pub fn new(space: &'a RealVectorStateSpace) -> Self {
        Self {
            space,
            rng: RNG::new(),
        }
    }

    pub fn sample_uniform(&mut self, state: &mut dyn State) {
        let state = &mut cast_to_rv_state!(mut state).values;
        for (state, low, high) in izip!(state, &self.space.bounds.low, &self.space.bounds.high) {
            *state = self.rng.uniform_real(*low, *high);
        }
    }

    pub fn sample_uniform_near(&mut self, state: &mut dyn State, near: &dyn State, distance: f64) {
        let state = &mut cast_to_rv_state!(mut state).values;
        let near = &cast_to_rv_state!(near).values;

        for (state, near, low, high) in
            izip!(state, near, &self.space.bounds.low, &self.space.bounds.high)
        {
            *state = self.rng.uniform_real(
                f64::max(*low, near - distance),
                f64::min(*high, near + distance),
            );
        }
    }

    pub fn sample_gaussian(&mut self, state: &mut dyn State, mean: &dyn State, std_dev: f64) {
        let state = &mut cast_to_rv_state!(mut state).values;
        let mean = &cast_to_rv_state!(mean).values;

        for (state, mean, low, high) in
            izip!(state, mean, &self.space.bounds.low, &self.space.bounds.high)
        {
            *state = self.rng.gaussian(*mean, std_dev).clamp(*low, *high);
        }
    }
}
