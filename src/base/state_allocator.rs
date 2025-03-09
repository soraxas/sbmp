use sbmp_derive::state_id_into_inner;
use std::cell::RefCell;

use super::state::State;

use crate::datastructure::arena::{Arena, Index};

/// A unique identifier for a state in a state space.
/// This is an index into the state space's arena.
/// The index is unique within the state space.
/// If the index is used in a different state space, it is meaningless.
#[derive(Debug)]
pub struct StateId(Index);

impl From<Index> for StateId {
    fn from(index: Index) -> Self {
        Self(index)
    }
}

impl PartialEq for StateId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for StateId {}

/// A state allocator that allocates states of type `T`.
/// This allocator uses an `Arena` to store the states.
#[derive(Debug)]
pub struct StateAllocator<T>(RefCell<Arena<T>>)
where
    T: State;

impl<T> StateAllocator<T>
where
    T: State,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self(RefCell::new(Arena::with_capacity(capacity)))
    }

    #[inline(always)]
    pub fn borrow(&self) -> std::cell::Ref<Arena<T>> {
        self.0.borrow()
    }

    #[inline(always)]
    pub fn borrow_mut(&self) -> std::cell::RefMut<Arena<T>> {
        self.0.borrow_mut()
    }
}

/// A trait for state spaces that can allocate states.
/// This trait provides functions to allocate, free, and clone states.
/// The state space must implement the `State` trait.
///
/// This traits provides functions to allocate, free, and clone states.
pub trait CanStateAllocateTrait {
    type State: State;

    fn new_state_allocator() -> StateAllocator<Self::State>;

    fn get_state_allocator(&self) -> &StateAllocator<Self::State>;

    fn alloc_default_state(&self) -> StateId
    where
        Self::State: Default,
    {
        self.get_state_allocator().borrow_mut().alloc().into()
    }

    fn alloc_arena_state_with_value(&self, state: Self::State) -> StateId {
        self.get_state_allocator().borrow_mut().insert(state).into()
    }

    fn free_arena_state(&self, state: &StateId) {
        self.get_state_allocator().borrow_mut().remove(state.0);
    }

    /// Helper function to clone the inner value of a state.
    #[state_id_into_inner]
    fn clone_state_inner_value(&self, source: &StateId) -> Box<dyn State>
    where
        Self::State: Clone,
    {
        Box::new((*source).clone())
    }

    /// Given a state id, this function runs a closure with the state.
    #[inline(always)]
    fn with_state<T>(&self, state: &StateId, mut closure: impl FnMut(&Self::State) -> T) -> T {
        let arena = self.get_state_allocator().borrow();
        let state = arena
            .get(state.0)
            .expect("State not found. Already freed? Or this Id is not for this state space?");

        closure(state)
    }

    #[inline(always)]
    fn with_state_mut<T>(
        &self,
        state: &StateId,
        mut closure: impl FnMut(&mut Self::State) -> T,
    ) -> T {
        let mut arena = self.get_state_allocator().borrow_mut();
        let state = arena
            .get_mut(state.0)
            .expect("State not found. Already freed? Or this Id is not for this state space?");

        closure(state)
    }

    #[inline(always)]
    fn with_2states<T>(
        &self,
        state1: &StateId,
        state2: &StateId,
        mut closure: impl FnMut(&Self::State, &Self::State) -> T,
    ) -> T {
        let arena = self.get_state_allocator().borrow();
        let states = arena
            .get2_uncheck(state1.0, state2.0)
            .expect("State not found. Already freed? Or this Id is not for this state space?");

        closure(states.0, states.1)
    }

    #[inline(always)]
    fn with_2states_mut<T>(
        &self,
        state1: &StateId,
        state2: &StateId,
        mut closure: impl FnMut(&mut Self::State, &mut Self::State) -> T,
    ) -> T {
        let mut arena = self.get_state_allocator().borrow_mut();
        let states = arena
            .get2_mut_uncheck(state1.0, state2.0)
            .expect("State not found. Already freed? Or this Id is not for this state space?");

        closure(states.0, states.1)
    }

    #[inline(always)]
    fn with_3states_mut<T>(
        &self,
        state1: &StateId,
        state2: &StateId,
        state3: &StateId,
        mut closure: impl FnMut(&mut Self::State, &mut Self::State, &mut Self::State) -> T,
    ) -> T {
        let mut arena = self.get_state_allocator().borrow_mut();
        let states = arena
            .get3_mut_uncheck(state1.0, state2.0, state3.0)
            .expect("State not found. Already freed? Or this Id is not for this state space?");

        closure(states.0, states.1, states.2)
    }
}
