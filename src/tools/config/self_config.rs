// use crate::ompl::base::{Goal, Planner, ProjectionEvaluator, SpaceInformation};
// use crate::ompl::control::planners::{KPIECE1 as ControlKPIECE1, RRT as ControlRRT};
// use crate::ompl::datastructures::{
//     NearestNeighborsGNAT, NearestNeighborsGNATNoThreadSafety, NearestNeighborsSqrtApprox,
// };
// use crate::ompl::geometric::planners::{RRTConnect, KPIECE1, LBKPIECE1, RRT};
// use std::collections::HashMap;
// use std::fmt;
// use std::sync::Weak;
// use std::sync::{Arc, Mutex};

// /// This struct contains methods that automatically
// /// configure various parameters for motion planning. If expensive
// /// computation is performed, the results are cached.
// pub struct SelfConfig {
//     si: Arc<SpaceInformation>,
//     context: String,
//     impl_: Arc<Mutex<SelfConfigImpl>>,
// }

// struct SelfConfigImpl {
//     wsi: Weak<SpaceInformation>,
//     probability_of_valid_state: f64,
//     average_valid_motion_length: f64,
// }

// impl SelfConfigImpl {
//     fn new(si: Arc<SpaceInformation>) -> Self {
//         Self {
//             wsi: Arc::downgrade(&si),
//             probability_of_valid_state: -1.0,
//             average_valid_motion_length: -1.0,
//         }
//     }

//     fn get_probability_of_valid_state(&mut self) -> f64 {
//         if let Some(si) = self.wsi.upgrade() {
//             self.check_setup(&si);
//             if self.probability_of_valid_state < 0.0 {
//                 self.probability_of_valid_state = si.probability_of_valid_state(1000);
//                 // magic::TEST_STATE_COUNT
//             }
//             self.probability_of_valid_state
//         } else {
//             -1.0
//         }
//     }

//     fn get_average_valid_motion_length(&mut self) -> f64 {
//         if let Some(si) = self.wsi.upgrade() {
//             self.check_setup(&si);
//             if self.average_valid_motion_length < 0.0 {
//                 self.average_valid_motion_length = si.average_valid_motion_length(1000);
//                 // magic::TEST_STATE_COUNT
//             }
//             self.average_valid_motion_length
//         } else {
//             -1.0
//         }
//     }

//     fn configure_valid_state_sampling_attempts(&self, attempts: &mut u32) {
//         if *attempts == 0 {
//             *attempts = 100; // magic::MAX_VALID_SAMPLE_ATTEMPTS
//         }
//     }

//     fn configure_planner_range(&self, range: &mut f64, context: &str) {
//         if *range < f64::EPSILON {
//             if let Some(si) = self.wsi.upgrade() {
//                 *range = si.get_maximum_extent() * 0.2; // magic::MAX_MOTION_LENGTH_AS_SPACE_EXTENT_FRACTION
//                 println!("{}Planner range detected to be {}", context, range);
//             } else {
//                 eprintln!(
//                     "{}Unable to detect planner range. SpaceInformation instance has expired.",
//                     context
//                 );
//             }
//         }
//     }

//     fn configure_projection_evaluator(
//         &self,
//         proj: &mut Option<Arc<ProjectionEvaluator>>,
//         context: &str,
//     ) {
//         if let Some(si) = self.wsi.upgrade() {
//             self.check_setup(&si);
//             if proj.is_none() {
//                 println!("{}Attempting to use default projection.", context);
//                 *proj = si.get_state_space().get_default_projection();
//             }
//             if proj.is_none() {
//                 panic!("No projection evaluator specified");
//             }
//             proj.as_ref().unwrap().setup();
//         }
//     }

//     fn print(&self, out: &mut dyn fmt::Write) {
//         if let Some(si) = self.wsi.upgrade() {
//             writeln!(
//                 out,
//                 "Configuration parameters for space '{}'",
//                 si.get_state_space().get_name()
//             )
//             .unwrap();
//             writeln!(
//                 out,
//                 "   - probability of a valid state is {}",
//                 self.probability_of_valid_state
//             )
//             .unwrap();
//             writeln!(
//                 out,
//                 "   - average length of a valid motion is {}",
//                 self.average_valid_motion_length
//             )
//             .unwrap();
//         } else {
//             writeln!(out, "EXPIRED").unwrap();
//         }
//     }

//     fn check_setup(&self, si: &Arc<SpaceInformation>) {
//         if !si.is_setup() {
//             si.setup();
//             self.probability_of_valid_state = -1.0;
//             self.average_valid_motion_length = -1.0;
//         }
//     }
// }

// impl SelfConfig {
//     /// Construct an instance that can configure the space
//     /// encapsulated by `si`. Any information printed to the
//     /// console is prefixed by `context`.
//     pub fn new(si: Arc<SpaceInformation>, context: String) -> Self {
//         let impl_ = Arc::new(Mutex::new(SelfConfigImpl::new(si.clone())));
//         Self { si, context, impl_ }
//     }

//     /// Get the probability of a sampled state being valid (calls
//     /// `SpaceInformation::probability_of_valid_state()`)
//     pub fn get_probability_of_valid_state(&self) -> f64 {
//         let mut impl_ = self.impl_.lock().unwrap();
//         impl_.get_probability_of_valid_state()
//     }

//     /// Get the probability of a sampled state being valid (calls
//     /// `SpaceInformation::average_valid_motion_length()`)
//     pub fn get_average_valid_motion_length(&self) -> f64 {
//         let mut impl_ = self.impl_.lock().unwrap();
//         impl_.get_average_valid_motion_length()
//     }

//     /// Instances of `ValidStateSampler` need a number of attempts to be specified -- the maximum
//     /// number of times a new sample is selected and checked to be valid. This function computes a number of `attempts` such
//     /// that the probability of obtaining a valid sample is 90%.
//     pub fn configure_valid_state_sampling_attempts(&self, attempts: &mut u32) {
//         let impl_ = self.impl_.lock().unwrap();
//         impl_.configure_valid_state_sampling_attempts(attempts);
//     }

//     /// Compute what a good length for motion segments is.
//     pub fn configure_planner_range(&self, range: &mut f64) {
//         let impl_ = self.impl_.lock().unwrap();
//         impl_.configure_planner_range(range, &self.context);
//     }

//     /// If `proj` is undefined, it is set to the default
//     /// projection reported by `StateSpace::get_default_projection()`.
//     /// If no default projection is available either, an exception is thrown.
//     pub fn configure_projection_evaluator(&self, proj: &mut Option<Arc<ProjectionEvaluator>>) {
//         let impl_ = self.impl_.lock().unwrap();
//         impl_.configure_projection_evaluator(proj, &self.context);
//     }

//     /// Print the computed configuration parameters.
//     pub fn print(&self, out: &mut dyn fmt::Write) {
//         let impl_ = self.impl_.lock().unwrap();
//         impl_.print(out);
//     }

//     /// Select a default nearest neighbor datastructure for the given space.
//     ///
//     /// The default depends on the planning algorithm and the space the planner operates in:
//     /// - If the space is a metric space and the planner is single-threaded,
//     ///   then the default is `NearestNeighborsGNATNoThreadSafety`.
//     /// - If the space is a metric space and the planner is multi-threaded,
//     ///   then the default is `NearestNeighborsGNAT`.
//     /// - If the space is a not a metric space,
//     ///   then the default is `NearestNeighborsSqrtApprox`.
//     pub fn get_default_nearest_neighbors<T>(planner: &Planner) -> Box<dyn NearestNeighbors<T>> {
//         let space = planner.get_space_information().get_state_space();
//         let specs = planner.get_specs();
//         if space.is_metric_space() {
//             if specs.multithreaded {
//                 Box::new(NearestNeighborsGNAT::new())
//             } else {
//                 Box::new(NearestNeighborsGNATNoThreadSafety::new())
//             }
//         } else {
//             Box::new(NearestNeighborsSqrtApprox::new())
//         }
//     }

//     /// Given a goal specification, decide on a planner for that goal.
//     pub fn get_default_planner(goal: Arc<Goal>) -> Arc<Planner> {
//         let si = goal.get_space_information();
//         let space = si.get_state_space();
//         let si_c = si.as_control_space_information();

//         if let Some(si_c) = si_c {
//             if space.has_default_projection() {
//                 Arc::new(ControlKPIECE1::new(si_c))
//             } else {
//                 Arc::new(ControlRRT::new(si_c))
//             }
//         } else if goal.is_none() {
//             println!("No goal specified; choosing RRT as the default planner");
//             Arc::new(RRT::new(goal.get_space_information()))
//         } else if goal.has_type_goal_sampleable_region() && space.has_symmetric_interpolate() {
//             if space.has_default_projection() {
//                 Arc::new(LBKPIECE1::new(goal.get_space_information()))
//             } else {
//                 Arc::new(RRTConnect::new(goal.get_space_information()))
//             }
//         } else {
//             if space.has_default_projection() {
//                 Arc::new(KPIECE1::new(goal.get_space_information()))
//             } else {
//                 Arc::new(RRT::new(goal.get_space_information()))
//             }
//         }
//     }
// }
