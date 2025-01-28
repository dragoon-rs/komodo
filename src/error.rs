//! Komodo-specific errors
//!
//! there are a few linear algebra errors and some related to [crate::zk].
use thiserror::Error;

/// An error that Komodo could end up producing.
///
/// There are a few families of errors in Komodo:
/// - related to _linear algebra_
/// - related to FEC
/// - related to proving the shards
#[derive(Clone, Debug, Error, PartialEq)]
pub enum KomodoError {
    /// `{0}` is a custom error message when a matrix is invalid.
    #[error("Invalid matrix elements: {0}")]
    InvalidMatrixElements(String),
    /// `{0}` and `{1}` are the shape of the rectangular matrix.
    #[error("Matrix is not a square, ({0} x {1})")]
    NonSquareMatrix(usize, usize),
    /// `{0}` is the ID of the row where the matrix inversion failed.
    #[error("Matrix is not invertible at row {0}")]
    NonInvertibleMatrix(usize),
    /// `{0}` and `{1}` are the shape of the left matrix and `{2}` and `{3}` are the shape of the
    /// right matrix.
    #[error("Matrices don't have compatible shapes: ({0} x {1}) and ({2} x {3})")]
    IncompatibleMatrixShapes(usize, usize, usize, usize),
    /// `{0}` and `{1}` are the IDs of the non-distinct _Vandermonde_ points and `{2}` is the list
    /// of all the _Vandermonde_ points.
    #[error(
        "Seed points of a Vandermonde matrix should be distinct: {0} and {1} are the same ({2})"
    )]
    InvalidVandermonde(usize, usize, String),
    /// `{0}` is the actual number of shards and `{1}` is the expected amount.
    #[error("Expected at least {1} shards, got {0}")]
    TooFewShards(usize, usize),
    /// `{0}` is a custom error message when shards are incompatible.
    #[error("Shards are incompatible: {0}")]
    IncompatibleShards(String),
    /// `{0}` is a custom error message when blocks are incompatible.
    #[error("Blocks are incompatible: {0}")]
    IncompatibleBlocks(String),
    #[error("Degree is zero")]
    DegreeIsZero,
    /// `{0}` is the supported degree of the trusted setup and `{1}` is the actual requested
    /// polynomial degree
    #[error("too many coefficients: max is {0}, found {0}")]
    TooFewPowersInTrustedSetup(usize, usize),
    /// `{0}` is a custom error message.
    #[error("Another error: {0}")]
    Other(String),
}
