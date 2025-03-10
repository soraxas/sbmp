use std::rc::Rc;

use downcast_rs::{impl_downcast, Downcast};

use crate::prelude::CanStateAllocateTrait;

use super::{
    state_allocator::StateId,
    statespace::{CompoundStateSpace, StateSpace},
};

pub trait StateSampler: Downcast {
    fn from_state_space(space: Rc<dyn StateSpace>) -> Self
    where
        Self: Sized;

    fn sample_uniform(&mut self, state: &mut StateId);

    fn sample_uniform_near(&mut self, state: &mut StateId, near: &StateId, distance: f64);

    fn sample_gaussian(&mut self, state: &mut StateId, mean: &StateId, std_dev: f64);
}
impl_downcast!(StateSampler);

pub struct CompoundStateSampler {
    space: Rc<dyn StateSpace>,
    pub samplers: Vec<(Box<dyn StateSampler>, f64)>,
}

impl CompoundStateSampler {
    pub fn new(space: Rc<dyn StateSpace>) -> CompoundStateSampler {
        CompoundStateSampler {
            samplers: Vec::new(),
            space,
        }
    }

    pub fn add_sampler(&mut self, sampler: Box<dyn StateSampler>, weight: f64) {
        self.samplers.push((sampler, weight));
    }
}

impl StateSampler for CompoundStateSampler {
    fn from_state_space(space: Rc<dyn StateSpace>) -> Self {
        CompoundStateSampler::new(space)
    }

    fn sample_uniform(&mut self, state: &mut StateId) {
        let space = self.space.downcast_ref::<CompoundStateSpace>().unwrap();

        space.with_state_mut(state, |state| {
            for (i, (sampler, _)) in self.samplers.iter_mut().enumerate() {
                sampler.sample_uniform(&mut state.components[i]);
            }
        });
    }

    fn sample_uniform_near(&mut self, state: &mut StateId, near: &StateId, distance: f64) {
        let space = self.space.downcast_ref::<CompoundStateSpace>().unwrap();

        space.with_2states_mut(state, near, |state, near| {
            for (i, (sampler, weight)) in self.samplers.iter_mut().enumerate() {
                if *weight > f64::EPSILON {
                    sampler.sample_uniform_near(
                        &mut state.components[i],
                        &near.components[i],
                        distance * *weight,
                    );
                } else {
                    sampler.sample_uniform(&mut state.components[i]);
                }
            }
        });
    }

    fn sample_gaussian(&mut self, state: &mut StateId, mean: &StateId, std_dev: f64) {
        let space = self.space.downcast_ref::<CompoundStateSpace>().unwrap();

        space.with_2states_mut(state, mean, |state, mean| {
            for (i, (sampler, weight)) in self.samplers.iter_mut().enumerate() {
                sampler.sample_gaussian(
                    &mut state.components[i],
                    &mean.components[i],
                    std_dev * *weight,
                );
            }
        });
    }
}
