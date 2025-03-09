use downcast_rs::{impl_downcast, DowncastSync};

pub trait State: DowncastSync + std::fmt::Debug {
    // fn as_any<T>(&self) -> T;

    // fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
impl_downcast!(sync State);

#[derive(Debug)]
pub struct CompoundState {
    pub components: Vec<Box<dyn State>>,
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

    // pub fn as_component<T: 'static>(&self, index: usize) -> Option<&T> {
    //     self.components.get(index)?.as_any().downcast_ref::<T>()
    // }

    // pub fn as_component_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
    //     self.components.get_mut(index)?.as_any_mut().downcast_mut::<T>()
    // }
}

impl State for CompoundState {
    // fn as_any<T>(&self) -> T {
    //     self
    // }

    // fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    //     self
    // }
}

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
