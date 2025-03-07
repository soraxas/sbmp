#[macro_export]
/// support ref or mut
///
/// Usage:
/// let state = downcast_state!(state, RealVectorState);
/// equivalent to:
/// let state = state.downcast_ref::<RealVectorState>().expect("invalid state type. Cannot cast to the provided type");
macro_rules! downcast_state {
    // Case when mut is not provided, with type as input
    ($state:expr, $type:ty) => {
        $state
            .downcast_ref::<$type>()
            .expect("invalid state type. Cannot cast to the provided type")
    };

    // Case when mut is provided, with type as input
    (mut $state:expr, $type:ty) => {
        $state
            .downcast_mut::<$type>()
            .expect("invalid state type. Cannot cast to the provided type")
    };
}
