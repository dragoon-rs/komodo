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
    #[error("Expected at least {1} shards, got {0}")]
    TooFewShards(usize, usize),
    #[error("Blocks are incompatible: {0}")]
    IncompatibleBlocks(String),
    #[error("Another error: {0}")]
    Other(String),
}
