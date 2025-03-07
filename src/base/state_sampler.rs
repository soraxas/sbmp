use std::sync::Arc;

use super::{
    state::{CompoundState, State},
    statespace::StateSpace,
};

pub trait StateSampler {
    fn from_state_space(space: Arc<dyn StateSpace>) -> Self
    where
        Self: Sized;
    fn sample_uniform(&self, state: &mut dyn State);
    fn sample_uniform_near(&self, state: &mut dyn State, near: &dyn State, distance: f64);
    fn sample_gaussian(&self, state: &mut dyn State, mean: &dyn State, std_dev: f64);
}

pub struct CompoundStateSampler {
    space: Arc<dyn StateSpace>,
    pub samplers: Vec<(Arc<dyn StateSampler>, f64)>,
}

impl CompoundStateSampler {
    pub fn new(space: Arc<dyn StateSpace>) -> CompoundStateSampler {
        CompoundStateSampler {
            samplers: Vec::new(),
            space,
        }
    }

    pub fn add_sampler(&mut self, sampler: Arc<dyn StateSampler>, weight: f64) {
        self.samplers.push((sampler, weight));
    }
}

impl StateSampler for CompoundStateSampler {
    fn from_state_space(space: Arc<dyn StateSpace>) -> Self
    where
        Self: Sized,
    {
        CompoundStateSampler::new(space)
    }

    fn sample_uniform(&self, state: &mut dyn State) {
        let state = state
            .downcast_mut::<CompoundState>()
            .expect("invalid state type");
        for (i, (sampler, _)) in self.samplers.iter().enumerate() {
            sampler.sample_uniform(state.components[i].as_mut());
        }
    }

    fn sample_uniform_near(&self, state: &mut dyn State, near: &dyn State, distance: f64) {
        let state = state
            .downcast_mut::<CompoundState>()
            .expect("invalid state type");
        let near = near
            .downcast_ref::<CompoundState>()
            .expect("invalid state type");
        for (i, (sampler, weight)) in self.samplers.iter().enumerate() {
            if *weight > f64::EPSILON {
                sampler.sample_uniform_near(
                    state.components[i].as_mut(),
                    near.components[i].as_ref(),
                    distance * *weight,
                );
            } else {
                sampler.sample_uniform(state.components[i].as_mut());
            }
        }
    }

    fn sample_gaussian(&self, state: &mut dyn State, mean: &dyn State, std_dev: f64) {
        let state = state
            .downcast_mut::<CompoundState>()
            .expect("invalid state type");
        let mean = mean
            .downcast_ref::<CompoundState>()
            .expect("invalid state type");
        for (i, (sampler, weight)) in self.samplers.iter().enumerate() {
            sampler.sample_gaussian(
                state.components[i].as_mut(),
                mean.components[i].as_ref(),
                std_dev * *weight,
            );
        }
    }
}
