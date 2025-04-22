//! Komodo-specific errors
//!
//! There are a few linear algebra errors and some related to [crate::zk].
use thiserror::Error;

/// An error that Komodo could end up producing.
///
/// There are a few families of errors in Komodo:
/// - related to _linear algebra_
/// - related to FEC
/// - related to proving the shards
#[derive(Clone, Debug, Error, PartialEq)]
pub enum KomodoError {
    #[error("expected rows to be of same length {expected}, found {found} at row {row}")]
    InvalidMatrixElements {
        expected: usize,
        found: usize,
        row: usize,
    },
    /// `{0}` and `{1}` are the shape of the rectangular matrix.
    #[error("Matrix is not a square, ({0} x {1})")]
    NonSquareMatrix(usize, usize),
    /// `{0}` is the ID of the row where the matrix inversion failed.
    #[error("Matrix is not invertible at row {0}")]
    NonInvertibleMatrix(usize),
    #[error("Matrices don't have compatible shapes: {left:?}, {right:?}")]
    IncompatibleMatrixShapes {
        left: (usize, usize),
        right: (usize, usize),
    },
    #[error(
        "Seed points of a Vandermonde matrix should be distinct: {first_index} and {second_index} are the same ({value_repr})"
    )]
    InvalidVandermonde {
        first_index: usize,
        second_index: usize,
        value_repr: String,
    },
    /// `{0}` is the actual number of shards and `{1}` is the expected amount.
    #[error("Expected at least {1} shards, got {0}")]
    TooFewShards(usize, usize),
    #[error("Shards are incompatible ({key} is not the same at {index}: {left} vs {right})")]
    IncompatibleShards {
        key: String,
        index: usize,
        left: String,
        right: String,
    },
    #[error("Blocks are incompatible ({key} is not the same at {index}: {left} vs {right})")]
    IncompatibleBlocks {
        key: String,
        index: usize,
        left: String,
        right: String,
    },
    #[error("Degree is zero")]
    DegreeIsZero,
    #[error("too many coefficients: max is {powers}, found {coefficients}")]
    TooFewPowersInTrustedSetup { powers: usize, coefficients: usize },
    /// `{0}` is a custom error message.
    #[error("Another error: {0}")]
    Other(String),
}
