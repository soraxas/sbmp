use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProlateHyperspheroidError {
    #[error("The transverse diameter must be greater than zero.")]
    InvalidTransverseDiameter,
    #[error("The transformation is not up to date. Has the transverse diameter been set?")]
    TransformationNotUpToDate,
}
