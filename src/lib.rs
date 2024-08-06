//! Komodo: Cryptographically-proven Erasure Coding
#[cfg(any(feature = "kzg", feature = "aplonk"))]
mod algebra;
#[cfg(feature = "aplonk")]
pub mod aplonk;
#[cfg(any(feature = "kzg", feature = "aplonk"))]
mod conversions;
pub mod error;
pub mod fec;
pub mod field;
pub mod fs;
#[cfg(feature = "kzg")]
pub mod kzg;
pub mod linalg;
pub mod semi_avid;
pub mod zk;
