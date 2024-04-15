//! Komodo-specific errors
//!
//! there are a few linear algebra errors and some related to ZK.
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum KomodoError {
    #[error("Invalid matrix elements: {0}")]
    InvalidMatrixElements(String),
    #[error("Matrix is not a square, ({0} x {1})")]
    NonSquareMatrix(usize, usize),
    #[error("Matrix is not invertible at row {0}")]
    NonInvertibleMatrix(usize),
    #[error("Matrices don't have compatible shapes: ({0} x {1}) and ({2} x {3})")]
    IncompatibleMatrixShapes(usize, usize, usize, usize),
    #[error(
        "Seed points of a Vandermonde matrix should be distinct: {0} and {1} are the same ({2})"
    )]
    InvalidVandermonde(usize, usize, String),
    #[error("Expected at least {1} shards, got {0}")]
    TooFewShards(usize, usize),
    #[error("Shards are incompatible: {0}")]
    IncompatibleShards(String),
    #[error("Blocks are incompatible: {0}")]
    IncompatibleBlocks(String),
    #[error("Degree is zero")]
    DegreeIsZero,
    #[error("too many coefficients: max is {0}, found {0}")]
    TooFewPowersInTrustedSetup(usize, usize),
    #[error("Another error: {0}")]
    Other(String),
}
