use ark_ff::Field;
use thiserror::Error;

#[derive(Clone, PartialEq, Default, Debug)]
pub(super) struct Matrix<T: Field> {
    pub elements: Vec<T>,
    pub height: usize,
    width: usize,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum LinalgError {
    #[error("Invalid matrix elements: {0}")]
    InvalidMatrixElements(String),
    #[error("Matrix is not a square")]
    NonSquareMatrix(usize, usize),
    #[error("Matrix is not invertible at row {0}")]
    NonInvertibleMatrix(usize),
    #[error("Matrices don't have compatible shapes: ({0} x {1}) and ({2} x {3})")]
    IncompatibleMatrixShapes(usize, usize, usize, usize),
    #[error("Another error: {0}")]
    Other(String),
}

impl<T: Field> Matrix<T> {
    fn from_diagonal(diagonal: Vec<T>) -> Self {
        let size = diagonal.len();

        let mut elements = Vec::new();
        elements.resize(size * size, T::zero());
        for i in 0..size {
            elements[i * size + i] = diagonal[i];
        }

        Self {
            elements,
            height: size,
            width: size,
        }
    }

    fn identity(size: usize) -> Self {
        Self::from_diagonal(vec![T::one(); size])
    }

    pub(super) fn vandermonde(points: &[T], height: usize) -> Self {
        let width = points.len();

        let mut elements = Vec::new();
        elements.resize(height * width, T::zero());

        for (j, pj) in points.iter().enumerate() {
            for i in 0..height {
                elements[i * width + j] = pj.pow([i as u64]);
            }
        }

        Self {
            elements,
            height,
            width,
        }
    }

    pub(super) fn from_vec_vec(matrix: Vec<Vec<T>>) -> Result<Self, LinalgError> {
        let height = matrix.len();
        let width = matrix[0].len();

        for (i, row) in matrix.iter().enumerate() {
            if row.len() != width {
                return Err(LinalgError::InvalidMatrixElements(format!(
                    "expected rows to be of same length {}, found {} at row {}",
                    width,
                    row.len(),
                    i
                )));
            }
        }

        let mut elements = Vec::new();
        elements.resize(height * width, T::zero());
        for i in 0..height {
            for j in 0..width {
                elements[i * width + j] = matrix[i][j];
            }
        }

        Ok(Self {
            elements,
            height,
            width,
        })
    }

    fn get(&self, i: usize, j: usize) -> T {
        self.elements[i * self.width + j]
    }

    fn set(&mut self, i: usize, j: usize, value: T) {
        self.elements[i * self.width + j] = value;
    }

    // compute _row / value_
    fn divide_row_by(&mut self, row: usize, value: T) {
        for j in 0..self.width {
            self.set(row, j, self.get(row, j) / value);
        }
    }

    // compute _destination = destination + source * value_
    fn multiply_row_by_and_add_to_row(&mut self, source: usize, value: T, destination: usize) {
        for j in 0..self.width {
            self.set(
                destination,
                j,
                self.get(destination, j) + self.get(source, j) * value,
            );
        }
    }

    pub(super) fn invert(&self) -> Result<Self, LinalgError> {
        if self.height != self.width {
            return Err(LinalgError::NonSquareMatrix(self.height, self.width));
        }

        let mut inverse = Self::identity(self.height);
        let mut matrix = self.clone();

        for i in 0..matrix.height {
            let pivot = matrix.get(i, i);
            if pivot.is_zero() {
                return Err(LinalgError::NonInvertibleMatrix(i));
            }

            inverse.divide_row_by(i, pivot);
            matrix.divide_row_by(i, pivot);

            for k in 0..matrix.height {
                if k != i {
                    let factor = matrix.get(k, i);
                    inverse.multiply_row_by_and_add_to_row(i, -factor, k);
                    matrix.multiply_row_by_and_add_to_row(i, -factor, k);
                }
            }
        }

        Ok(inverse)
    }

    pub(super) fn mul(&self, rhs: &Self) -> Result<Self, LinalgError> {
        if self.width != rhs.height {
            return Err(LinalgError::IncompatibleMatrixShapes(
                self.height,
                self.width,
                rhs.height,
                rhs.width,
            ));
        }

        let height = self.height;
        let width = rhs.width;
        let common = self.width;

        let mut elements = Vec::new();
        elements.resize(height * width, T::zero());

        for i in 0..height {
            for j in 0..width {
                elements[i * width + j] = (0..common).map(|k| self.get(i, k) * rhs.get(k, j)).sum();
            }
        }

        Ok(Self {
            elements,
            height,
            width,
        })
    }

    pub(super) fn transpose(&self) -> Self {
        let height = self.width;
        let width = self.height;

        let mut elements = Vec::new();
        elements.resize(height * width, T::zero());

        for i in 0..height {
            for j in 0..width {
                elements[i * width + j] = self.get(j, i);
            }
        }

        Self {
            elements,
            height,
            width,
        }
    }
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use ark_std::{One, Zero};
    use rand::Rng;

    use super::{LinalgError, Matrix};

    fn random_field_element<T: Field>() -> T {
        let mut rng = rand::thread_rng();
        let element: u128 = rng.gen();
        T::from(element)
    }

    #[test]
    fn from_vec_vec() {
        let actual = Matrix::from_vec_vec(vec![
            vec![Fr::from(2), Fr::zero(), Fr::zero()],
            vec![Fr::zero(), Fr::from(3), Fr::zero()],
            vec![Fr::zero(), Fr::zero(), Fr::from(4)],
            vec![Fr::from(2), Fr::from(3), Fr::from(4)],
        ])
        .unwrap();
        let expected = Matrix {
            elements: vec![
                Fr::from(2),
                Fr::zero(),
                Fr::zero(),
                Fr::zero(),
                Fr::from(3),
                Fr::zero(),
                Fr::zero(),
                Fr::zero(),
                Fr::from(4),
                Fr::from(2),
                Fr::from(3),
                Fr::from(4),
            ],
            height: 4,
            width: 3,
        };
        assert_eq!(actual, expected);

        let matrix = Matrix::from_vec_vec(vec![vec![Fr::zero()], vec![Fr::zero(), Fr::zero()]]);
        assert!(matrix.is_err());
        assert!(matches!(
            matrix.err().unwrap(),
            LinalgError::InvalidMatrixElements(..)
        ));
    }

    #[test]
    fn diagonal() {
        let actual = Matrix::<Fr>::from_diagonal(vec![Fr::from(2), Fr::from(3), Fr::from(4)]);
        let expected = Matrix::from_vec_vec(vec![
            vec![Fr::from(2), Fr::zero(), Fr::zero()],
            vec![Fr::zero(), Fr::from(3), Fr::zero()],
            vec![Fr::zero(), Fr::zero(), Fr::from(4)],
        ])
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn identity() {
        let actual = Matrix::<Fr>::identity(3);
        let expected = Matrix::from_vec_vec(vec![
            vec![Fr::one(), Fr::zero(), Fr::zero()],
            vec![Fr::zero(), Fr::one(), Fr::zero()],
            vec![Fr::zero(), Fr::zero(), Fr::one()],
        ])
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn multiplication() {
        let a = Matrix::from_vec_vec(vec![
            vec![Fr::from(9), Fr::from(4), Fr::from(3)],
            vec![Fr::from(8), Fr::from(5), Fr::from(2)],
            vec![Fr::from(7), Fr::from(6), Fr::from(1)],
        ])
        .unwrap();
        let b = Matrix::from_vec_vec(vec![
            vec![Fr::from(1), Fr::from(2), Fr::from(3)],
            vec![Fr::from(4), Fr::from(5), Fr::from(6)],
            vec![Fr::from(7), Fr::from(8), Fr::from(9)],
        ])
        .unwrap();

        assert!(matches!(
            a.mul(&Matrix::from_vec_vec(vec![vec![Fr::from(1), Fr::from(2)]]).unwrap()),
            Err(LinalgError::IncompatibleMatrixShapes(3, 3, 1, 2))
        ));

        let product = a.mul(&b).unwrap();
        let expected = Matrix::from_vec_vec(vec![
            vec![Fr::from(46), Fr::from(62), Fr::from(78)],
            vec![Fr::from(42), Fr::from(57), Fr::from(72)],
            vec![Fr::from(38), Fr::from(52), Fr::from(66)],
        ])
        .unwrap();
        assert_eq!(product, expected);
    }

    #[test]
    fn inverse() {
        let matrix = Matrix::<Fr>::identity(3);
        let inverse = matrix.invert().unwrap();
        assert_eq!(Matrix::<Fr>::identity(3), inverse);

        let matrix = Matrix::<Fr>::from_diagonal(vec![Fr::from(2), Fr::from(3), Fr::from(4)]);
        let inverse = matrix.invert().unwrap();
        assert_eq!(matrix.mul(&inverse).unwrap(), Matrix::<Fr>::identity(3));
        assert_eq!(inverse.mul(&matrix).unwrap(), Matrix::<Fr>::identity(3));

        let n = 20;
        let matrix = Matrix::from_vec_vec(
            (0..n)
                .map(|_| (0..n).map(|_| random_field_element()).collect())
                .collect::<Vec<Vec<Fr>>>(),
        )
        .unwrap();
        let inverse = matrix.invert().unwrap();
        assert_eq!(matrix.mul(&inverse).unwrap(), Matrix::<Fr>::identity(n));
        assert_eq!(inverse.mul(&matrix).unwrap(), Matrix::<Fr>::identity(n));

        let inverse = Matrix::from_vec_vec(vec![
            vec![Fr::one(), Fr::zero(), Fr::zero()],
            vec![Fr::zero(), Fr::one(), Fr::zero()],
        ])
        .unwrap()
        .invert();
        assert!(inverse.is_err());
        assert!(matches!(
            inverse.err().unwrap(),
            LinalgError::NonSquareMatrix(..)
        ));

        let inverse =
            Matrix::<Fr>::from_diagonal(vec![Fr::zero(), Fr::from(3), Fr::from(4)]).invert();
        assert!(inverse.is_err());
        assert!(matches!(
            inverse.err().unwrap(),
            LinalgError::NonInvertibleMatrix(0)
        ));

        let inverse = Matrix::from_vec_vec(vec![
            vec![Fr::one(), Fr::one(), Fr::zero()],
            vec![Fr::zero(), Fr::zero(), Fr::zero()],
            vec![Fr::zero(), Fr::zero(), Fr::one()],
        ])
        .unwrap()
        .invert();
        assert!(inverse.is_err());
        assert!(matches!(
            inverse.err().unwrap(),
            LinalgError::NonInvertibleMatrix(1)
        ));
    }

    #[test]
    fn vandermonde() {
        let actual = Matrix::vandermonde(
            &[
                Fr::from(0),
                Fr::from(1),
                Fr::from(2),
                Fr::from(3),
                Fr::from(4),
            ],
            4,
        );
        #[rustfmt::skip]
        let expected = Matrix::from_vec_vec(vec![
            vec![Fr::from(1), Fr::from(1), Fr::from(1), Fr::from(1), Fr::from(1)],
            vec![Fr::from(0), Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(4)],
            vec![Fr::from(0), Fr::from(1), Fr::from(4), Fr::from(9), Fr::from(16)],
            vec![Fr::from(0), Fr::from(1), Fr::from(8), Fr::from(27), Fr::from(64)],
        ])
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn transpose() {
        let matrix = Matrix::from_vec_vec(vec![
            vec![Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(10)],
            vec![Fr::from(4), Fr::from(5), Fr::from(6), Fr::from(11)],
            vec![Fr::from(7), Fr::from(8), Fr::from(9), Fr::from(12)],
        ])
        .unwrap();
        let transpose = Matrix::from_vec_vec(vec![
            vec![Fr::from(1), Fr::from(4), Fr::from(7)],
            vec![Fr::from(2), Fr::from(5), Fr::from(8)],
            vec![Fr::from(3), Fr::from(6), Fr::from(9)],
            vec![Fr::from(10), Fr::from(11), Fr::from(12)],
        ])
        .unwrap();

        assert_eq!(matrix.transpose(), transpose);
    }
}
