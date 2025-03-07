use itertools::izip;
use rand::Rng;
use std::cmp;
use std::collections::HashMap;
use std::f64::consts::E;
use std::f64::EPSILON;

use crate::randomness::RNG;

use super::real_vector_bounds::RealVectorBounds;

pub struct RealVectorStateSpace {
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

impl RealVectorStateSpace {
    pub fn new() -> Self {
        Self {
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

    pub fn get_maximum_extent(&self) -> f64 {
        self.bounds
            .low
            .iter()
            .zip(&self.bounds.high)
            .map(|(low, high)| (high - low).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    pub fn get_measure(&self) -> f64 {
        self.bounds
            .low
            .iter()
            .zip(&self.bounds.high)
            .map(|(low, high)| high - low)
            .product()
    }

    pub fn enforce_bounds(&self, state: &mut [f64]) {
        for (s, low, high) in izip!(state, &self.bounds.low, &self.bounds.high) {
            *s = s.clamp(*low, *high);
        }
    }

    pub fn satisfies_bounds(&self, state: &[f64]) -> bool {
        state
            .iter()
            .zip(&self.bounds.low)
            .zip(&self.bounds.high)
            .all(|((s, low), high)| s - f64::EPSILON > *low && s + f64::EPSILON < *high)
    }

    pub fn distance(&self, state1: &[f64], state2: &[f64]) -> f64 {
        state1
            .iter()
            .zip(state2)
            .map(|(s1, s2)| (s1 - s2).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    pub fn equal_states(&self, state1: &[f64], state2: &[f64]) -> bool {
        state1
            .iter()
            .zip(state2)
            .all(|(s1, s2)| (s1 - s2).abs() <= f64::EPSILON * 2.0)
    }

    pub fn interpolate(&self, from: &[f64], to: &[f64], time: f64, state: &mut [f64]) {
        for (s, f, t) in izip!(state, from, to) {
            *s = f + (t - f) * time;
        }
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

    pub fn sample_uniform(&mut self, state: &mut [f64]) {
        for (state, low, high) in izip!(state, &self.space.bounds.low, &self.space.bounds.high) {
            *state = self.rng.uniform_real(*low, *high);
        }
    }

    pub fn sample_uniform_near(&mut self, state: &mut [f64], near: &[f64], distance: f64) {
        for (state, near, low, high) in
            izip!(state, near, &self.space.bounds.low, &self.space.bounds.high)
        {
            *state = self.rng.uniform_real(
                f64::max(*low, near - distance),
                f64::min(*high, near + distance),
            );
        }
    }

    pub fn sample_gaussian(&mut self, state: &mut [f64], mean: &[f64], std_dev: f64) {
        for (state, mean, low, high) in
            izip!(state, mean, &self.space.bounds.low, &self.space.bounds.high)
        {
            *state = self.rng.gaussian(*mean, std_dev).clamp(*low, *high);
        }
    }
}
