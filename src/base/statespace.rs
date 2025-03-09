use core::fmt;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::EPSILON;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, Once};

use anyhow::{anyhow, Result};

use downcast_rs::{impl_downcast, Downcast, DowncastSync};
use sbmp_derive::{WithArenaAlloc, WithStateSpaceData};

use super::param::ParamSet;
use super::state::{self, State};
use super::state_sampler::{CompoundStateSampler, StateSampler};

pub const DEFAULT_PROJECTION_NAME: &str = "";

pub enum StateSpaceType {}

pub struct CompoundState {
    pub components: Vec<StateId>,
}

impl fmt::Debug for CompoundState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompoundState")
            .field("components", &"<...>")
            .finish()
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

#[derive(Debug)]
struct SubstateLocation {
    pub chain: Vec<usize>,
    pub space: Arc<dyn StateSpace>,
}

#[derive(Debug)]
struct ValueLocation {
    pub index: u32,
    pub name: String,
}

pub trait AsCompoundTrait {
    fn as_compound_ref(&self) -> Option<&CompoundStateSpace>;

    /// Calls the provided closure if this is a compound state space.
    fn as_compound_ref_closure(&self, mut f: impl FnMut(&CompoundStateSpace)) {
        if let Some(compound) = self.as_compound_ref() {
            f(compound);
        }
    }
}

impl AsCompoundTrait for Arc<dyn StateSpace> {
    fn as_compound_ref(&self) -> Option<&CompoundStateSpace> {
        self.downcast_ref::<CompoundStateSpace>()
    }
}

/// Visit all state spaces in the hierarchy.
/// The provided closure is called for each state space.
/// This is a depth-first search.
fn visit_all_space(state_space: &dyn StateSpace, f: &mut impl FnMut(&dyn StateSpace)) {
    // queue.push_back(state_space);

    f(state_space);

    if let Some(space) = state_space.as_compound_ref() {
        // call the closure for the state

        for component in &space.components {
            // let component = component.get_mut();

            visit_all_space(component.as_ref(), f);
        }
    }
}

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

pub trait CanStateAllocateTrait {
    type State: State;

    fn new_arena() -> RefCell<Arena<Self::State>>;

    fn get_arena(&self) -> &RefCell<Arena<Self::State>>;

    fn alloc_arena_state(&self) -> StateId
    where
        Self::State: Default,
    {
        self.get_arena().borrow_mut().alloc().into()
    }

    fn alloc_arena_state_with_value(&self, state: Self::State) -> StateId {
        self.get_arena().borrow_mut().insert(state).into()
    }

    fn free_arena_state(&self, state: &StateId) {
        self.get_arena().borrow_mut().remove(state.0);
    }

    /// Helper function to clone the inner value of a state.
    fn clone_state_inner_value(&self, source: &StateId) -> Box<dyn State>
    where
        Self::State: Clone,
    {
        self.with_state(source, |state| Box::new((*state).clone()))
    }

    /// Given a state id, this function runs a closure with the state.
    #[inline(always)]
    fn with_state<T>(&self, state: &StateId, mut closure: impl FnMut(&Self::State) -> T) -> T {
        let arena = self.get_arena().borrow();
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
        let mut arena = self.get_arena().borrow_mut();
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
        let arena = self.get_arena().borrow();
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
        let mut arena = self.get_arena().borrow_mut();
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
        let mut arena = self.get_arena().borrow_mut();
        let states = arena
            .get3_mut_uncheck(state1.0, state2.0, state3.0)
            .expect("State not found. Already freed? Or this Id is not for this state space?");

        closure(states.0, states.1, states.2)
    }
}

pub trait HasStateSpaceData {
    fn state_space_data(&self) -> &StateSpaceCommonData;
    fn state_space_data_mut(&mut self) -> &mut StateSpaceCommonData;
}

// impl<T> CanCloneStateInnerValue for CanStateAllocateTrait
// where
//     T: Downcast + Debug,
// {
//     fn state_space_data(&self) -> &StateSpaceCommonData {
//         self.as_any().downcast_ref::<StateSpaceCommonData>().unwrap()
//     }

//     fn state_space_data_mut(&mut self) -> &mut StateSpaceCommonData {
//         self.as_any_mut().downcast_mut::<StateSpaceCommonData>().unwrap()
//     }
// }

pub trait StateSpace: HasStateSpaceData + Downcast + Debug {
    fn is_compound(&self) -> bool {
        false
    }

    fn is_discrete(&self) -> bool {
        false
    }

    fn as_compound_ref(&self) -> Option<&CompoundStateSpace> {
        self.as_any().downcast_ref::<CompoundStateSpace>()
    }

    fn as_compound_mut(&mut self) -> Option<&mut CompoundStateSpace> {
        self.as_any_mut().downcast_mut::<CompoundStateSpace>()
    }

    fn is_hybrid(&self) -> bool {
        false
    }

    fn is_metric_space(&self) -> bool {
        true
    }
    fn has_symmetric_distance(&self) -> bool {
        true
    }
    fn has_symmetric_interpolate(&self) -> bool {
        true
    }
    fn get_name(&self) -> &str {
        &self.state_space_data().name
    }

    fn set_name(&mut self, name: String) {
        self.state_space_data_mut().name = name;
    }

    fn get_type(&self) -> i32 {
        // Implement logic to return the type of the state space
        todo!();
        0
    }

    fn includes(&self, other: &dyn StateSpace) -> bool {
        if self.get_name() == other.get_name() {
            return true;
        }

        // create a queue and push the current state space
        let mut q: VecDeque<Arc<dyn StateSpace>> = VecDeque::new();

        // push all subspaces
        if let Some(space) = self.as_compound_ref() {
            space.components.iter().for_each(|s| q.push_back(s.clone()));
        }
        while let Some(m) = q.pop_front() {
            if m.get_name() == other.get_name() {
                return true;
            }
            m.as_compound_ref_closure(|c| c.components.iter().for_each(|s| q.push_back(s.clone())));
        }
        false
    }
    fn covers(&self, other: &dyn StateSpace) -> bool {
        if self.includes(other) {
            return true;
        } else if let Some(space) = other.as_compound_ref() {
            for component in &space.components {
                if !self.includes(component.as_ref()) {
                    return false;
                }
            }
            return true;
        }
        false
    }
    fn params(&self) -> &ParamSet {
        // Implement logic to return the parameters of the state space
        &self.state_space_data().params
    }

    fn params_mut(&mut self) -> &mut ParamSet {
        // Implement logic to return mutable parameters of the state space
        &mut self.state_space_data_mut().params
    }

    fn get_longest_valid_segment_fraction(&self) -> f64 {
        self.state_space_data().longest_valid_segment_fraction
    }

    fn set_longest_valid_segment_fraction(&mut self, segment_fraction: f64) {
        // Implement logic to set the longest valid segment fraction
        if !(f64::EPSILON..=1.0 - f64::EPSILON).contains(&segment_fraction) {
            panic!("The fraction of the valid segment length must be in the range (0, 1), i.e., exclusive range");
        }
        self.state_space_data_mut().longest_valid_segment_fraction = segment_fraction;
    }

    fn valid_segment_count(&self, state1: &StateId, state2: &StateId) -> u32 {
        // Implement logic to return the valid segment count
        self.state_space_data().longest_valid_segment_count_factor
            * (self.distance(state1, state2) / self.state_space_data().longest_valid_segment).ceil()
                as u32
    }

    fn set_valid_segment_count_factor(&mut self, factor: u32) {
        if factor < 1 {
            panic!("The multiplicative factor for the valid segment count between two states must be strictly positive");
        }
        self.state_space_data_mut()
            .longest_valid_segment_count_factor = factor;
    }

    fn get_valid_segment_count_factor(&self) -> u32 {
        // Implement logic to return the valid segment count factor
        self.state_space_data().longest_valid_segment_count_factor
    }

    fn get_longest_valid_segment_length(&self) -> f64 {
        // Implement logic to return the longest valid segment length
        self.state_space_data().longest_valid_segment
    }
    fn compute_signature(&self, signature: &mut Vec<i32>) {
        if let Some(compound) = self.as_compound_ref() {
            for component in &compound.components {
                component.compute_signature(signature);
            }
        }
    }
    fn get_dimension(&self) -> u32;
    fn get_maximum_extent(&self) -> f64;
    fn get_measure(&self) -> f64;
    fn enforce_bounds(&self, state: &mut StateId);
    fn satisfies_bounds(&self, state: &StateId) -> bool;
    fn copy_state(&self, destination: &mut StateId, source: &StateId);
    fn distance(&self, state1: &StateId, state2: &StateId) -> f64;
    // fn get_serialization_length(&self) -> u32;
    // fn serialize(&self, serialization: &mut [u8], state: &StateId);
    // fn deserialize(&self, state: &mut StateId, serialization: &[u8]);
    fn equal_states(&self, state1: &StateId, state2: &StateId) -> bool;
    fn interpolate(&self, from: &StateId, to: &StateId, t: f64, state: &mut StateId);
    // fn alloc_state_sampler(&self) -> Arc<dyn StateSampler>;
    // fn set_state_sampler_allocator(&mut self, ssa: Box<dyn Fn(&Self) -> Arc<dyn StateSampler>>);
    // fn clear_state_sampler_allocator(&mut self);
    fn alloc_state(&self) -> StateId;
    fn free_state(&self, state: &StateId);

    fn clone_state(&self, source: &StateId) -> StateId {
        let mut state = self.alloc_state();
        self.copy_state(&mut state, source);
        state
    }

    fn get_value_address_at_index_const(&self, state: &StateId, index: u32) -> Option<&f64> {
        // Implement logic to get the value address at index (const)
        todo!();

        None
    }

    fn get_value_locations(&self) -> &Vec<ValueLocation> {
        // Implement logic to get value locations
        todo!();

        &Vec::new()
    }

    fn get_value_locations_by_name(&self) -> &HashMap<String, ValueLocation> {
        // Implement logic to get value locations by name
        todo!();

        &HashMap::new()
    }

    fn get_value_address_at_location(
        &self,
        state: &mut StateId,
        loc: &ValueLocation,
    ) -> Option<&mut f64> {
        // Implement logic to get the value address at location
        todo!();

        None
    }

    fn get_value_address_at_location_const(
        &self,
        state: &StateId,
        loc: &ValueLocation,
    ) -> Option<&f64> {
        // Implement logic to get the value address at location (const)
        todo!();

        None
    }

    fn get_value_address_at_name(&self, state: &mut StateId, name: &str) -> Option<&mut f64> {
        // Implement logic to get the value address at name
        todo!();

        None
    }

    fn get_value_address_at_name_const(&self, state: &StateId, name: &str) -> Option<&f64> {
        // Implement logic to get the value address at name (const)
        todo!();

        None
    }

    fn copy_to_reals(&self, reals: &mut Vec<f64>, source: &StateId) {
        // Implement logic to copy to reals
        todo!();
    }

    fn copy_from_reals(&self, destination: &mut StateId, reals: &Vec<f64>) {
        // Implement logic to copy from reals
        todo!();
    }

    fn register_projection(&mut self, name: String, projection: Box<dyn ProjectionEvaluator>) {
        // Implement logic to register a projection
        todo!();
    }

    fn register_default_projection(&mut self, projection: Box<dyn ProjectionEvaluator>) {
        // Implement logic to register the default projection
        todo!();
    }

    fn register_projections(&mut self) {
        // Implement logic to register projections
        todo!();
    }

    fn get_projection(&self, name: &str) -> Option<&Box<dyn ProjectionEvaluator>> {
        // Implement logic to get a projection
        todo!();

        None
    }

    fn get_default_projection(&self) -> Option<&Box<dyn ProjectionEvaluator>> {
        // Implement logic to get the default projection
        todo!();

        None
    }

    fn has_projection(&self, name: &str) -> bool {
        // Implement logic to check if a projection exists
        false
    }

    fn has_default_projection(&self) -> bool {
        // Implement logic to check if the default projection exists
        false
    }

    fn get_registered_projections(&self) -> &HashMap<String, Box<dyn ProjectionEvaluator>> {
        // Implement logic to get registered projections
        &self.state_space_data().projections
    }

    fn get_value_address_at_index(&self, state: &mut StateId, index: u32) -> Option<&mut f64> {
        // Implement logic to get the value address at index
        todo!();
        None
    }

    fn print_state(&self, state: &StateId) {
        // Implement logic to print a state
        todo!();
    }

    fn print_settings(&self) {
        // Implement logic to print settings
        todo!();
    }

    fn print_projections(&self) {
        // Implement logic to print projections
        todo!();
    }

    fn sanity_checks(&self, zero: f64, eps: f64, flags: u32) {
        // Implement logic to perform sanity checks
        todo!();
    }

    fn sanity_checks_default(&self) {
        // Implement logic to perform default sanity checks
        todo!();
    }

    fn alloc_subspace_state_sampler(&self, subspace: &dyn StateSpace) -> Arc<dyn StateSampler> {
        // Implement logic to allocate a subspace state sampler
        todo!();

        // Box::new(DefaultStateSampler::new(self))
    }

    fn get_substate_at_location(
        &self,
        state: &mut StateId,
        loc: &SubstateLocation,
    ) -> Option<&mut StateId> {
        // Implement logic to get a substate at location
        None
    }

    fn get_substate_at_location_const(
        &self,
        state: &StateId,
        loc: &SubstateLocation,
    ) -> Option<&StateId> {
        // Implement logic to get a substate at location (const)
        None
    }

    fn get_substate_locations_by_name(&self) -> &HashMap<String, SubstateLocation> {
        // Implement logic to get substate locations by name
        // &HashMap::new()
        todo!();
    }

    fn get_common_subspaces(&self, other: &dyn StateSpace, subspaces: &mut Vec<String>) {
        // Implement logic to get common subspaces
        todo!();
    }

    fn compute_locations(&mut self) {
        // Implement logic to compute locations
        todo!();
    }

    fn setup(&mut self);
}
impl_downcast!(StateSpace);

pub fn diagram(out: &mut String, state_space: &dyn StateSpace) {
    out.push_str("digraph StateSpace {\n");
    out.push_str(&format!("\"{}\"\n", state_space.get_name()));

    let mut queue: VecDeque<&dyn StateSpace> = VecDeque::new();
    queue.push_back(state_space);

    while let Some(m) = queue.pop_front() {
        if let Some(compound) = m.as_compound_ref() {
            for (subspace, weight) in compound.iter_component_and_weight() {
                queue.push_back(subspace.as_ref());
                out.push_str(&format!(
                    "\"{}\" -> \"{}\" [label=\"{}\"];\n",
                    compound.get_name(),
                    subspace.get_name(),
                    weight,
                ));
            }
        }
    }

    out.push_str("}\n");
}

pub fn list(out: &mut String, state_space: &dyn StateSpace) {
    visit_all_space(state_space, &mut |state_space: &dyn StateSpace| {
        out.push_str(&format!(
            "{}: {}\n",
            state_space.get_name(),
            state_space.get_type()
        ));
    });
}

pub trait ProjectionEvaluator: Send + Sync {
    fn setup(&self);
    fn params(&self) -> &ParamSet;
    fn project(&self, state: &dyn State, projection: &mut [f64]);
    fn get_dimension(&self) -> usize;
    fn get_bounds(&self) -> (Vec<f64>, Vec<f64>);
    fn print_settings(&self);
}

pub struct StateSpaceCommonData {
    pub name: String,
    pub max_extents: f64,
    pub longest_valid_segment: f64,
    pub longest_valid_segment_fraction: f64,
    pub longest_valid_segment_count_factor: u32,
    pub projections: HashMap<String, Box<dyn ProjectionEvaluator>>,
    pub params: ParamSet,
    pub value_locations_in_order: Vec<ValueLocation>,
    pub value_locations_by_name: HashMap<String, ValueLocation>,
    pub substate_locations_by_name: HashMap<String, SubstateLocation>,
}

impl fmt::Debug for StateSpaceCommonData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateSpaceCommonData")
            .field("max_extents", &self.max_extents)
            .field(
                "longest_valid_segment_fraction",
                &self.longest_valid_segment_fraction,
            )
            .field("longest_valid_segment", &self.longest_valid_segment)
            .field(
                "longest_valid_segment_count_factor",
                &self.longest_valid_segment_count_factor,
            )
            .field("projections", &"<..>")
            .field("params", &self.params)
            .field("value_locations_in_order", &self.value_locations_in_order)
            .field("value_locations_by_name", &self.value_locations_by_name)
            .field(
                "substate_locations_by_name",
                &self.substate_locations_by_name,
            )
            .finish()
    }
}

impl StateSpaceCommonData {
    pub fn new(name: String) -> Self {
        Self {
            name,
            max_extents: 0.0,
            longest_valid_segment: 0.0,
            longest_valid_segment_fraction: 0.01, // 1 %
            longest_valid_segment_count_factor: 1,
            projections: HashMap::new(),
            params: ParamSet::default(),
            value_locations_in_order: Vec::new(),
            value_locations_by_name: HashMap::new(),
            substate_locations_by_name: HashMap::new(),
        }
    }
}

impl Default for StateSpaceCommonData {
    fn default() -> Self {
        // default name to Space + number
        Self::new(format!(
            "Space{}",
            ALLOCATED_SPACES.fetch_add(1, Ordering::Relaxed)
        ))
    }
}

#[derive(WithStateSpaceData, WithArenaAlloc)]
#[arena_alloc(state_type = "CompoundState")]
// #[arena_alloc(default_capacity = 150)]
pub struct CompoundStateSpace {
    state_space_data: StateSpaceCommonData,
    arena: RefCell<Arena<CompoundState>>,
    components: Vec<Arc<dyn StateSpace>>,
    weights: Vec<f64>,
    weight_sum: f64,
    locked: bool,
    // ...other fields...
}

// impl CanStateAllocateTrait for CompoundStateSpace {

//     type State = CompoundState;

//     fn new_arena() -> Arena<Self::State> {
//         Arena::with_capacity(3)
//     }

//     fn get_arena_mut(&mut self) -> &mut Arena<Self::State> {
//         &mut self.arena
//     }

//     fn get_arena(&self) -> &Arena<Self::State> {
//         &self.arena
//     }
// }

impl fmt::Debug for CompoundStateSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompoundStateSpace")
            .field("state_space_data", &self.state_space_data)
            .field("components", &"<...>")
            .field("weights", &self.weights)
            .field("weight_sum", &self.weight_sum)
            .field("locked", &self.locked)
            .finish()
    }
}

lazy_static::lazy_static! {
    static ref ALLOCATED_SPACES: AtomicU16 = 0.into();
}

impl Default for CompoundStateSpace {
    fn default() -> Self {
        Self {
            arena: Self::new_arena(),
            state_space_data: StateSpaceCommonData::default(),
            components: Vec::new(),
            weights: Vec::new(),
            weight_sum: 0.0,
            locked: false,
            // ...initialize other fields...
        }
    }
}

impl CompoundStateSpace {
    pub fn from_components(
        mut components: Vec<Arc<dyn StateSpace>>,
        mut weights: Vec<f64>,
    ) -> Result<Self> {
        if components.len() != weights.len() {
            return Err(anyhow!(
                "The number of components and weights must be equal"
            ));
        }

        let mut space = Self::default();
        for (component, weight) in components.drain(..).zip(weights.drain(..)) {
            space.add_subspace(component, weight)?;
        }
        Ok(space)
    }

    pub fn add_subspace(&mut self, component: Arc<dyn StateSpace>, weight: f64) -> Result<()> {
        if self.locked {
            return Err(anyhow!(
                "This state space is locked. No further components can be added"
            ));
        }
        if weight < 0.0 {
            return Err(anyhow!("Subspace weight cannot be negative"));
        }
        self.components.push(component);
        self.weights.push(weight);
        self.weight_sum += weight;
        Ok(())
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn get_subspace_count(&self) -> usize {
        self.components.len()
    }

    pub fn get_subspace(&self, index: usize) -> &Arc<dyn StateSpace> {
        &self.components[index]
    }

    pub fn get_subspace_weight(&self, index: usize) -> f64 {
        self.weights[index]
    }

    pub fn iter_component_and_weight(&self) -> impl Iterator<Item = (&Arc<dyn StateSpace>, &f64)> {
        self.components.iter().zip(&self.weights)
    }

    pub fn set_subspace_weight(&mut self, index: usize, weight: f64) {
        if weight < 0.0 {
            panic!("Subspace weight cannot be negative");
        }
        self.weight_sum += weight - self.weights[index];
        self.weights[index] = weight;
    }
}

impl StateSpace for CompoundStateSpace {
    fn is_hybrid(&self) -> bool {
        let mut has_continuous = false;
        let mut has_discrete = false;
        for component in &self.components {
            if component.is_hybrid() {
                return true;
            }
            if component.is_discrete() {
                has_discrete = true;
            } else {
                has_continuous = true;
            }
        }
        has_continuous && has_discrete
    }

    fn is_compound(&self) -> bool {
        true
    }

    fn is_discrete(&self) -> bool {
        self.components.iter().all(|c| c.is_discrete())
    }

    fn compute_signature(&self, signature: &mut Vec<i32>) {
        // Implement logic to compute the signature of the state space
    }

    fn get_dimension(&self) -> u32 {
        self.components.iter().map(|c| c.get_dimension()).sum()
    }

    fn get_maximum_extent(&self) -> f64 {
        self.components
            .iter()
            .zip(&self.weights)
            .map(|(c, &w)| w * c.get_maximum_extent())
            .sum()
    }

    fn get_measure(&self) -> f64 {
        self.components
            .iter()
            .zip(&self.weights)
            .map(|(c, &w)| w * c.get_measure())
            .product()
    }

    fn enforce_bounds(&self, state: &mut StateId) {
        self.with_state_mut(state, |state| {
            for (component, substate) in self.components.iter().zip(&mut state.components) {
                component.enforce_bounds(substate);
            }
        });
    }

    fn satisfies_bounds(&self, state: &StateId) -> bool {
        self.with_state(state, |state| {
            self.components
                .iter()
                .zip(&state.components)
                .all(|(component, substate)| component.satisfies_bounds(substate))
        })
    }

    fn copy_state(&self, destination: &mut StateId, source: &StateId) {
        self.with_2states_mut(source, destination, |source, destination| {
            for (component, (d, s)) in self
                .components
                .iter()
                .zip(destination.components.iter_mut().zip(&source.components))
            {
                component.copy_state(d, s);
            }
        });
    }

    fn clone_state(&self, source: &StateId) -> StateId {
        let mut clone = CompoundState::new();

        self.with_state_mut(source, |source| {
            for (component, s) in self.components.iter().zip(&source.components) {
                clone.components.push(component.clone_state(s));
            }
        });

        self.alloc_arena_state_with_value(clone)
    }

    fn distance(&self, state1: &StateId, state2: &StateId) -> f64 {
        self.with_2states(state1, state2, |state1, state2| {
            self.components
                .iter()
                .zip(&self.weights)
                .map(|(component, &weight)| {
                    weight * component.distance(&state1.components[0], &state2.components[0])
                })
                .sum()
        })
    }

    // fn get_serialization_length(&self) -> u32 {
    //     self.components
    //         .iter()
    //         .map(|c| c.get_serialization_length())
    //         .sum()
    // }

    // fn serialize(&self, serialization: &mut [u8], state: &dyn State) {
    //     let cstate = downcast_state!(state, CompoundState);
    //     let mut offset = 0;
    //     for (component, substate) in self.components.iter().zip(&cstate.components) {
    //         let length = component.get_serialization_length() as usize;
    //         component.serialize(
    //             &mut serialization[offset..offset + length],
    //             substate.as_ref(),
    //         );
    //         offset += length;
    //     }
    // }

    // fn deserialize(&self, state: &mut dyn State, serialization: &[u8]) {
    //     let cstate = downcast_state!(mut state, CompoundState););
    //     let mut offset = 0;
    //     for (component, substate) in self.components.iter().zip(&mut cstate.components) {
    //         let length = component.get_serialization_length() as usize;
    //         component.deserialize(substate.as_mut(), &serialization[offset..offset + length]);
    //         offset += length;
    //     }
    // }

    fn equal_states(&self, state1: &StateId, state2: &StateId) -> bool {
        self.with_2states(state1, state2, |state1, state2| {
            self.components
                .iter()
                .zip(&state1.components)
                .zip(&state2.components)
                .all(|((component, s1), s2)| component.equal_states(s1, s2))
        })
    }

    fn interpolate(&self, from: &StateId, to: &StateId, t: f64, state: &mut StateId) {
        self.with_3states_mut(from, to, state, |from, to, state| {
            for (i, component) in self.components.iter().enumerate() {
                component.interpolate(
                    &from.components[i],
                    &to.components[i],
                    t,
                    &mut state.components[i],
                );
            }
        });
    }

    // fn alloc_default_state_sampler(self: &Arc<Self>) -> Arc<dyn StateSampler> {
    //     // Implement logic to allocate a default state sampler
    //     Arc::new(CompoundStateSampler::new(self.clone()))
    // }

    // fn alloc_state_sampler(&self) -> Arc<dyn StateSampler> {
    //     // Implement logic to allocate a state sampler
    //     Box::new(DefaultStateSampler::new(self))
    // }

    // fn clear_state_sampler_allocator(&mut self) {
    //     // Implement logic to clear the state sampler allocator
    // }

    fn alloc_state(&self) -> StateId {
        let mut cstate = CompoundState::new();
        for component in &self.components {
            cstate.components.push(component.alloc_state());
        }
        self.alloc_arena_state_with_value(cstate)
    }

    fn free_state(&self, state: &StateId) {
        self.with_state(state, |cstate| {
            for (component, substate) in self.components.iter().zip(cstate.components.iter()) {
                component.free_state(substate);
            }
        });
        self.free_arena_state(state);
    }

    fn setup(&mut self) {
        // Implement logic to setup the state space
        todo!();
    }
}
