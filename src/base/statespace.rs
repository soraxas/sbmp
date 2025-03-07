use core::fmt;
use std::any::Any;
use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::EPSILON;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, Once};

use anyhow::{anyhow, Result};

use downcast_rs::{impl_downcast, DowncastSync};

use super::param::ParamSet;
use super::state_sampler::{CompoundStateSampler, StateSampler};

pub const DEFAULT_PROJECTION_NAME: &str = "";

pub enum StateSpaceType {}

pub trait State: DowncastSync {}
impl_downcast!(sync State);

pub struct CompoundState {
    pub components: Vec<Box<dyn State>>,
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
    let mut queue: VecDeque<&dyn StateSpace> = VecDeque::new();
    queue.push_back(state_space);

    if let Some(space) = state_space.as_compound_ref() {
        // call the closure for the state
        f(state_space);

        for component in &space.components {
            queue.push_back(component.as_ref());
            visit_all_space(component.as_ref(), f);
        }
    }
}

pub trait StateSpace: DowncastSync + Debug {
    fn state_space_data(&self) -> &StateSpaceCommonData;
    fn state_space_data_mut(&mut self) -> &mut StateSpaceCommonData;
    fn is_compound(&self) -> bool;
    fn is_discrete(&self) -> bool;

    fn as_compound_ref(&self) -> Option<&CompoundStateSpace> {
        self.as_any().downcast_ref::<CompoundStateSpace>()
    }

    fn is_hybrid(&self) -> bool {
        if let Some(space) = self.as_compound_ref() {
            let mut has_continuous = false;
            let mut has_discrete = false;
            for component in &space.components {
                if component.is_hybrid() {
                    return true;
                }
                if component.is_discrete() {
                    has_discrete = true;
                } else {
                    has_continuous = true;
                }
            }
            return has_continuous && has_discrete;
        }
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

    fn valid_segment_count(&self, state1: &dyn State, state2: &dyn State) -> u32 {
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
    fn compute_signature(&self, signature: &mut Vec<i32>);
    fn get_dimension(&self) -> u32;
    fn get_maximum_extent(&self) -> f64;
    fn get_measure(&self) -> f64;
    fn enforce_bounds(&self, state: &mut dyn State);
    fn satisfies_bounds(&self, state: &dyn State) -> bool;
    fn copy_state(&self, destination: &mut dyn State, source: &dyn State);
    fn clone_state(&self, source: &dyn State) -> Box<dyn State>;
    fn distance(&self, state1: &dyn State, state2: &dyn State) -> f64;
    fn get_serialization_length(&self) -> u32;
    fn serialize(&self, serialization: &mut [u8], state: &dyn State);
    fn deserialize(&self, state: &mut dyn State, serialization: &[u8]);
    fn equal_states(&self, state1: &dyn State, state2: &dyn State) -> bool;
    fn interpolate(&self, from: &dyn State, to: &dyn State, t: f64, state: &mut dyn State);
    // fn alloc_state_sampler(&self) -> Arc<dyn StateSampler>;
    // fn set_state_sampler_allocator(&mut self, ssa: Box<dyn Fn(&Self) -> Arc<dyn StateSampler>>);
    // fn clear_state_sampler_allocator(&mut self);
    fn alloc_state(&self) -> Box<dyn State>;
    fn free_state(&self, state: Box<dyn State>);

    fn get_value_address_at_index_const(&self, state: &dyn State, index: u32) -> Option<&f64> {
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
        state: &mut dyn State,
        loc: &ValueLocation,
    ) -> Option<&mut f64> {
        // Implement logic to get the value address at location
        todo!();

        None
    }

    fn get_value_address_at_location_const(
        &self,
        state: &dyn State,
        loc: &ValueLocation,
    ) -> Option<&f64> {
        // Implement logic to get the value address at location (const)
        todo!();

        None
    }

    fn get_value_address_at_name(&self, state: &mut dyn State, name: &str) -> Option<&mut f64> {
        // Implement logic to get the value address at name
        todo!();

        None
    }

    fn get_value_address_at_name_const(&self, state: &dyn State, name: &str) -> Option<&f64> {
        // Implement logic to get the value address at name (const)
        todo!();

        None
    }

    fn copy_to_reals(&self, reals: &mut Vec<f64>, source: &dyn State) {
        // Implement logic to copy to reals
        todo!();
    }

    fn copy_from_reals(&self, destination: &mut dyn State, reals: &Vec<f64>) {
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

    fn get_value_address_at_index(&self, state: &mut dyn State, index: u32) -> Option<&mut f64> {
        // Implement logic to get the value address at index
        todo!();
        None
    }

    fn print_state(&self, state: &dyn State) {
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
        state: &mut dyn State,
        loc: &SubstateLocation,
    ) -> Option<&mut dyn State> {
        // Implement logic to get a substate at location
        None
    }

    fn get_substate_at_location_const(
        &self,
        state: &dyn State,
        loc: &SubstateLocation,
    ) -> Option<&dyn State> {
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
impl_downcast!(sync StateSpace);

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

#[derive(Debug)]
pub struct CompoundStateSpace {
    state_space_data: StateSpaceCommonData,
    components: Vec<Arc<dyn StateSpace>>,
    weights: Vec<f64>,
    weight_sum: f64,
    locked: bool,
    // ...other fields...
}

lazy_static::lazy_static! {
    static ref ALLOCATED_SPACES: AtomicU16 = 0.into();
}

impl CompoundStateSpace {
    pub fn new(name: String) -> Self {
        Self {
            state_space_data: StateSpaceCommonData::new(name),
            components: Vec::new(),
            weights: Vec::new(),
            weight_sum: 0.0,
            locked: false,
            // ...initialize other fields...
        }
    }

    pub fn from_components(
        mut components: Vec<Arc<dyn StateSpace>>,
        mut weights: Vec<f64>,
    ) -> Result<Self> {
        if components.len() != weights.len() {
            return Err(anyhow!(
                "The number of components and weights must be equal"
            ));
        }

        let mut space = Self::new(format!(
            "Space{}",
            ALLOCATED_SPACES.fetch_add(1, Ordering::Relaxed)
        ));
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
    fn state_space_data(&self) -> &StateSpaceCommonData {
        &self.state_space_data
    }

    fn state_space_data_mut(&mut self) -> &mut StateSpaceCommonData {
        &mut self.state_space_data
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

    fn enforce_bounds(&self, state: &mut dyn State) {
        let cstate = state.downcast_mut::<CompoundState>().unwrap();
        for (component, substate) in self.components.iter().zip(&mut cstate.components) {
            component.enforce_bounds(substate.as_mut());
        }
    }

    fn satisfies_bounds(&self, state: &dyn State) -> bool {
        let cstate = state.downcast_ref::<CompoundState>().unwrap();
        self.components
            .iter()
            .zip(&cstate.components)
            .all(|(component, substate)| component.satisfies_bounds(substate.as_ref()))
    }

    fn copy_state(&self, destination: &mut dyn State, source: &dyn State) {
        let cdest = destination.downcast_mut::<CompoundState>().unwrap();
        let csrc = source.downcast_ref::<CompoundState>().unwrap();
        for (component, (d, s)) in self
            .components
            .iter()
            .zip(cdest.components.iter_mut().zip(&csrc.components))
        {
            component.copy_state(d.as_mut(), s.as_ref());
        }
    }

    fn clone_state(&self, source: &dyn State) -> Box<dyn State> {
        let csrc = source.downcast_ref::<CompoundState>().unwrap();
        let mut clone = CompoundState::new();
        for component in &self.components {
            clone
                .components
                .push(component.clone_state(csrc.components[0].as_ref()));
        }
        Box::new(clone)
    }

    fn distance(&self, state1: &dyn State, state2: &dyn State) -> f64 {
        let cstate1 = state1.downcast_ref::<CompoundState>().unwrap();
        let cstate2 = state2.downcast_ref::<CompoundState>().unwrap();
        self.components
            .iter()
            .zip(&self.weights)
            .map(|(component, &weight)| {
                weight
                    * component.distance(
                        cstate1.components[0].as_ref(),
                        cstate2.components[0].as_ref(),
                    )
            })
            .sum()
    }

    fn get_serialization_length(&self) -> u32 {
        self.components
            .iter()
            .map(|c| c.get_serialization_length())
            .sum()
    }

    fn serialize(&self, serialization: &mut [u8], state: &dyn State) {
        let cstate = state.downcast_ref::<CompoundState>().unwrap();
        let mut offset = 0;
        for (component, substate) in self.components.iter().zip(&cstate.components) {
            let length = component.get_serialization_length() as usize;
            component.serialize(
                &mut serialization[offset..offset + length],
                substate.as_ref(),
            );
            offset += length;
        }
    }

    fn deserialize(&self, state: &mut dyn State, serialization: &[u8]) {
        let cstate = state.downcast_mut::<CompoundState>().unwrap();
        let mut offset = 0;
        for (component, substate) in self.components.iter().zip(&mut cstate.components) {
            let length = component.get_serialization_length() as usize;
            component.deserialize(substate.as_mut(), &serialization[offset..offset + length]);
            offset += length;
        }
    }

    fn equal_states(&self, state1: &dyn State, state2: &dyn State) -> bool {
        let cstate1 = state1.downcast_ref::<CompoundState>().unwrap();
        let cstate2 = state2.downcast_ref::<CompoundState>().unwrap();
        self.components
            .iter()
            .zip(&cstate1.components)
            .zip(&cstate2.components)
            .all(|((component, s1), s2)| component.equal_states(s1.as_ref(), s2.as_ref()))
    }

    fn interpolate(&self, from: &dyn State, to: &dyn State, t: f64, state: &mut dyn State) {
        match (
            from.downcast_ref::<CompoundState>(),
            to.downcast_ref::<CompoundState>(),
            state.downcast_mut::<CompoundState>(),
        ) {
            (Some(cfrom), Some(cto), Some(cstate)) => {
                for (i, component) in self.components.iter().enumerate() {
                    component.interpolate(
                        cfrom.components[i].as_ref(),
                        cto.components[i].as_ref(),
                        t,
                        cstate.components[i].as_mut(),
                    );
                }
            }
            _ => panic!("Invalid state type"),
        }
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

    fn alloc_state(&self) -> Box<dyn State> {
        let mut state = CompoundState::new();
        for component in &self.components {
            state.components.push(component.alloc_state());
        }
        Box::new(state)
    }

    fn free_state(&self, state: Box<dyn State>) {
        // let cstate = state.downcast::<CompoundState>().unwrap();
        // for (component, substate) in self.components.iter().zip(cstate.components) {
        //     component.free_state(substate);
        // }
    }

    fn setup(&mut self) {
        // Implement logic to setup the state space
        todo!();
    }
}
