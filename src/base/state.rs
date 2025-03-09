use downcast_rs::{impl_downcast, DowncastSync};

use super::state_allocator::StateId;

pub trait State: DowncastSync + std::fmt::Debug {
    // fn as_any<T>(&self) -> T;

    // fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
impl_downcast!(sync State);

#[derive(Debug)]
pub struct CompoundState {
    pub components: Vec<StateId>,
}

impl Default for CompoundState {
    fn default() -> Self {
        Self::new()
    }
}

impl CompoundState {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
}

impl State for CompoundState {}
// pub trait Sampler {
//     fn sample(&self, state: &mut dyn State);
// }

// pub trait CompoundSampler: Sampler {
//     fn sample(&self, state: &mut CompoundState);
// }

// pub struct CompoundStateSampler {
// }

// impl CompoundSampler for CompoundStateSampler {
//     fn sample(&self, state: &mut CompoundState) {
//         for component in state.components.iter_mut() {
//             // component.sample();
//         }
//     }
// }

// fn test () {

// }
