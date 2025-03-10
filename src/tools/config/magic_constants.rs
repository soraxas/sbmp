/// We strive to minimize the use of constants in the code, but at
/// places, this is necessary. These constants typically do not
/// have to be changed, but we chose to expose their functionality
/// for the more curious user. The constants in this module
/// include values typically used in the computation of default
/// settings.
pub mod magic {
    /// When the cell sizes for a projection are
    /// automatically computed, this value defines the number of
    /// parts into which each dimension is split.
    pub const PROJECTION_DIMENSION_SPLITS: f64 = 20.0;

    /// When no cell sizes are specified for a projection, they are inferred like so:
    /// 1. approximate extent of projected space by taking a number of samples (the constant below)
    /// 2. compute the cell sizes by dividing the extent by PROJECTION_DIMENSION_SPLITS
    pub const PROJECTION_EXTENTS_SAMPLES: u32 = 100;

    /// When a bounding box of projected states cannot be inferred,
    /// it will be estimated by sampling states. To get closer to the true
    /// bounding box, we grow the bounding box of the projected sampled
    /// states by 5% on each side.
    pub const PROJECTION_EXPAND_FACTOR: f64 = 0.05;

    /// For planners: if default values are to be used for
    /// the maximum length of motions, this constant defines what
    /// fraction of the space extent (computed with
    /// ompl::base::SpaceInformation::getMaximumExtent()) is to be
    /// used as the maximum length of a motion
    pub const MAX_MOTION_LENGTH_AS_SPACE_EXTENT_FRACTION: f64 = 0.2;

    /// For cost-based planners it has been observed that smaller ranges
    /// are typically suitable. The same range computation strategy is used for all
    /// planners, but for cost planners an additional factor (smaller than 1) is
    pub const COST_MAX_MOTION_LENGTH_AS_SPACE_EXTENT_FRACTION: f64 = 0.175;

    /// When standard deviation is needed for Gaussian
    /// sampling in the state space, by default the value of the
    /// standard deviation is computed as a fraction of the extent
    /// of the space
    pub const STD_DEV_AS_SPACE_EXTENT_FRACTION: f64 = 0.1;

    /// When multiple attempts are needed to generate valid
    /// samples, this value defines the default number of
    /// attempts
    pub const MAX_VALID_SAMPLE_ATTEMPTS: u32 = 100;

    /// Maximum number of sampling attempts to find a valid state,
    /// without checking whether the allowed time elapsed. This value
    /// should not really be changed.
    pub const FIND_VALID_STATE_ATTEMPTS_WITHOUT_TERMINATION_CHECK: u32 = 2;

    /// When multiple states need to be generated as part
    /// of the computation of various information (usually through
    /// stochastic processes), this parameter controls how many
    /// samples are generated.
    pub const TEST_STATE_COUNT: u32 = 1000;

    /// Default number of close solutions to choose from a path experience database
    /// (library) for further filtering used in the Lightning Framework
    pub const NEAREST_K_RECALL_SOLUTIONS: u32 = 10;
}