/// support ref or mut
///
/// Usage:
/// let state = downcast_state!(state, RealVectorState);
/// equivalent to:
/// let state = state.downcast_ref::<RealVectorState>().expect("invalid state type. Cannot cast to the provided type");
#[macro_export]
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

// /// A macro that allows performing actions on one or more states in a mutable or immutable context.
// /// It supports passing multiple states and handles both mutable and immutable states differently.
// /// This macro works recursively to process multiple states, executing the specified body of code
// /// in the context of each state.
// ///
// /// ## Parameters:
// /// - `$space:ident`: The space or context that the state belongs to. This can be any identifier that
// ///   represents the context within which the state is being modified or accessed.
// /// - `$state:ident`: The state to be accessed or modified. This can either be mutable (`mut`) or
// ///   immutable, and the macro will behave differently depending on whether the state is mutable.
// /// - `$($rest:ident)+`: One or more additional states to be processed in the same way, allowing for
// ///   recursive processing of multiple states.
// /// - `{ $($body:tt)* }`: The body of code that should be executed within the context of the state.
// ///
// /// ## Behavior:
// /// - The macro first checks if the state is mutable (`mut $state`). If it is, it calls `with_state_mut`.
// /// - If the state is not mutable, it calls `with_state`.
// /// - The macro recursively processes additional states, passing them to the closure one by one.
// /// - The code inside the body is executed within the closure for each state.
// ///
// /// ## Usage Examples:
// /// ```rust
// /// struct Space;
// /// impl Space {
// ///     fn with_state<T>(&self, _state: &T, closure: impl FnOnce(&T)) {
// ///         closure(&_state);
// ///     }
// ///
// ///     fn with_state_mut<T>(&self, _state: &mut T, closure: impl FnOnce(&mut T)) {
// ///         closure(&_state);
// ///     }
// /// }
// ///
// /// fn main() {
// ///     let space = Space;
// ///     let mut state1 = "state1";
// ///     let mut state2 = "state2";
// ///
// ///     // Example of mutable state
// ///     with_inner_states!(space @ mut state1, state2 @ {
// ///         println!("Modifying state1: {}", state1);
// ///         println!("Accessing state2: {}", state2);
// ///     });
// ///
// ///     // Example of immutable state
// ///     with_inner_states!(space @ state1, state2 @ {
// ///         println!("Accessing state1: {}", state1);
// ///         println!("Accessing state2: {}", state2);
// ///     });
// /// }
// /// ```
// ///
// /// ## Explanation of the example:
// /// - `space @ mut state1, state2`: The first state (`state1`) is mutable, while `state2` is immutable.
// ///   - `with_state_mut` will be used to mutate `state1`.
// ///   - `with_state` will be used for the immutable `state2`.
// /// - The closure prints the values of `state1` and `state2` inside the body.
// #[macro_export]
// macro_rules! with_inner_states {
//     // Handle mutable states and recursively process remaining states
//     ($space:ident @ mut $state:ident, $( $($rest:ident)+ ),* @ { $($body:tt)* }) => {
//         $space.with_state_mut($state, |$state| {
//             with_inner_states!($space @ $($($rest)+),* @ { $($body)* })
//         })
//     };

//     // Handle immutable states and recursively process remaining states
//     ($space:ident @ $state:ident, $( $($rest:ident)+ ),* @ { $($body:tt)* }) => {
//         $space.with_state($state, |$state| {
//             with_inner_states!($space @ $($($rest)+),* @ { $($body)* })
//         })
//     };

//     // Base case: Handle a mutable state without additional states to process
//     ($space:ident @ mut $state:ident @ { $($body:tt)* }) => {
//         $space.with_state_mut($state, |$state| {
//             { $($body)* }
//         })
//     };

//     // Base case: Handle an immutable state without additional states to process
//     ($space:ident @ $state:ident @ { $($body:tt)* }) => {
//         $space.with_state($state, |$state| {
//             { $($body)* }
//         })
//     };
// }
