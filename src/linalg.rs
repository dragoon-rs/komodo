use ark_ff::Field;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use rand::Rng;

use crate::error::KomodoError;

#[derive(Clone, PartialEq, Default, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct Matrix<T: Field> {
    pub elements: Vec<T>,
    pub height: usize,
    pub width: usize,
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

    pub fn vandermonde(points: &[T], height: usize) -> Self {
        let width = points.len();

        let mut elements = Vec::new();
        elements.resize(height * width, T::zero());

        for (j, pj) in points.iter().enumerate() {
            let mut el = T::one();
            for i in 0..height {
                elements[i * width + j] = el;
                el *= pj;
            }
        }

        Self {
            elements,
            height,
            width,
        }
    }

    pub fn random(n: usize, m: usize) -> Self {
        let mut rng = rand::thread_rng();

        Matrix::from_vec_vec(
            (0..n)
                .map(|_| {
                    (0..m)
                        .map(|_| {
                            let element: u128 = rng.gen();
                            T::from(element)
                        })
                        .collect()
                })
                .collect::<Vec<Vec<T>>>(),
        )
        .unwrap()
    }

    pub(super) fn from_vec_vec(matrix: Vec<Vec<T>>) -> Result<Self, KomodoError> {
        let height = matrix.len();
        let width = matrix[0].len();

        for (i, row) in matrix.iter().enumerate() {
            if row.len() != width {
                return Err(KomodoError::InvalidMatrixElements(format!(
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

    pub(super) fn get_col(&self, j: usize) -> Option<Vec<T>> {
        if j >= self.width {
            return None;
        }

        Some((0..self.height).map(|i| self.get(i, j)).collect())
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

    pub(super) fn invert(&self) -> Result<Self, KomodoError> {
        if self.height != self.width {
            return Err(KomodoError::NonSquareMatrix(self.height, self.width));
        }

        let mut inverse = Self::identity(self.height);
        let mut matrix = self.clone();

        for i in 0..matrix.height {
            let pivot = matrix.get(i, i);
            if pivot.is_zero() {
                return Err(KomodoError::NonInvertibleMatrix(i));
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

    pub(super) fn mul(&self, rhs: &Self) -> Result<Self, KomodoError> {
        if self.width != rhs.height {
            return Err(KomodoError::IncompatibleMatrixShapes(
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

    pub(super) fn truncate(&self, rows: Option<usize>, cols: Option<usize>) -> Self {
        let width = if let Some(w) = cols {
            self.width - w
        } else {
            self.width
        };

        let height = if let Some(h) = rows {
            self.height - h
        } else {
            self.height
        };

        let mut elements = Vec::new();
        elements.resize(height * width, T::zero());

        for i in 0..height {
            for j in 0..width {
                elements[i * width + j] = self.get(i, j);
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

    use super::{KomodoError, Matrix};

    fn vec_to_elements<T: Field>(elements: Vec<u128>) -> Vec<T> {
        elements.iter().map(|&x| T::from(x)).collect()
    }

    fn mat_to_elements<T: Field>(mat: Vec<Vec<u128>>) -> Vec<Vec<T>> {
        mat.iter().cloned().map(vec_to_elements).collect()
    }

    #[test]
    fn from_vec_vec() {
        let actual = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![2, 0, 0],
            vec![0, 3, 0],
            vec![0, 0, 4],
            vec![2, 3, 4],
        ]))
        .unwrap();
        let expected = Matrix {
            elements: vec_to_elements(vec![2, 0, 0, 0, 3, 0, 0, 0, 4, 2, 3, 4]),
            height: 4,
            width: 3,
        };
        assert_eq!(actual, expected);

        let matrix = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![vec![0], vec![0, 0]]));
        assert!(matrix.is_err());
        assert!(matches!(
            matrix.err().unwrap(),
            KomodoError::InvalidMatrixElements(..)
        ));
    }

    #[test]
    fn diagonal() {
        let actual = Matrix::<Fr>::from_diagonal(vec_to_elements(vec![2, 3, 4]));
        let expected = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![2, 0, 0],
            vec![0, 3, 0],
            vec![0, 0, 4],
        ]))
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn identity() {
        let actual = Matrix::<Fr>::identity(3);
        let expected = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 0, 0],
            vec![0, 1, 0],
            vec![0, 0, 1],
        ]))
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn multiplication() {
        let a = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![9, 4, 3],
            vec![8, 5, 2],
            vec![7, 6, 1],
        ]))
        .unwrap();
        let b = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
        ]))
        .unwrap();

        assert!(matches!(
            a.mul(&Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![vec![1, 2]])).unwrap()),
            Err(KomodoError::IncompatibleMatrixShapes(3, 3, 1, 2))
        ));

        let product = a.mul(&b).unwrap();
        let expected = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![46, 62, 78],
            vec![42, 57, 72],
            vec![38, 52, 66],
        ]))
        .unwrap();
        assert_eq!(product, expected);
    }

    #[test]
    fn inverse() {
        let matrix = Matrix::<Fr>::identity(3);
        let inverse = matrix.invert().unwrap();
        assert_eq!(Matrix::<Fr>::identity(3), inverse);

        let matrix = Matrix::<Fr>::from_diagonal(vec_to_elements(vec![2, 3, 4]));
        let inverse = matrix.invert().unwrap();
        assert_eq!(matrix.mul(&inverse).unwrap(), Matrix::<Fr>::identity(3));
        assert_eq!(inverse.mul(&matrix).unwrap(), Matrix::<Fr>::identity(3));

        let n = 20;
        let matrix = Matrix::random(n, n);
        let inverse = matrix.invert().unwrap();
        assert_eq!(matrix.mul(&inverse).unwrap(), Matrix::<Fr>::identity(n));
        assert_eq!(inverse.mul(&matrix).unwrap(), Matrix::<Fr>::identity(n));

        let inverse =
            Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![vec![1, 0, 0], vec![0, 1, 0]]))
                .unwrap()
                .invert();
        assert!(inverse.is_err());
        assert!(matches!(
            inverse.err().unwrap(),
            KomodoError::NonSquareMatrix(..)
        ));

        let inverse = Matrix::<Fr>::from_diagonal(vec_to_elements(vec![0, 3, 4])).invert();
        assert!(inverse.is_err());
        assert!(matches!(
            inverse.err().unwrap(),
            KomodoError::NonInvertibleMatrix(0)
        ));

        let inverse = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 1, 0],
            vec![0, 0, 0],
            vec![0, 0, 1],
        ]))
        .unwrap()
        .invert();
        assert!(inverse.is_err());
        assert!(matches!(
            inverse.err().unwrap(),
            KomodoError::NonInvertibleMatrix(1)
        ));
    }

    #[test]
    fn vandermonde() {
        let actual = Matrix::<Fr>::vandermonde(&mat_to_elements(vec![vec![0, 1, 2, 3, 4]])[0], 4);
        #[rustfmt::skip]
        let expected = Matrix::from_vec_vec(mat_to_elements(vec![
            vec![1, 1, 1, 1, 1],
            vec![0, 1, 2, 3, 4],
            vec![0, 1, 4, 9, 16],
            vec![0, 1, 8, 27, 64],
        ]))
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn transpose() {
        let matrix = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 2, 3, 10],
            vec![4, 5, 6, 11],
            vec![7, 8, 9, 12],
        ]))
        .unwrap();
        let transpose = Matrix::from_vec_vec(mat_to_elements(vec![
            vec![1, 4, 7],
            vec![2, 5, 8],
            vec![3, 6, 9],
            vec![10, 11, 12],
        ]))
        .unwrap();

        assert_eq!(matrix.transpose(), transpose);
    }

    #[test]
    fn truncate() {
        let matrix = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 2, 3, 10],
            vec![4, 5, 6, 11],
            vec![7, 8, 9, 12],
        ]))
        .unwrap();

        assert_eq!(matrix.truncate(None, None), matrix);
        assert_eq!(matrix.truncate(Some(0), None), matrix);
        assert_eq!(matrix.truncate(None, Some(0)), matrix);
        assert_eq!(matrix.truncate(Some(0), Some(0)), matrix);

        let truncated =
            Matrix::from_vec_vec(mat_to_elements(vec![vec![1, 2], vec![4, 5]])).unwrap();
        assert_eq!(matrix.truncate(Some(1), Some(2)), truncated);
    }

    #[test]
    fn get_cols() {
        let matrix = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 2, 3, 10],
            vec![4, 5, 6, 11],
            vec![7, 8, 9, 12],
        ]))
        .unwrap();

        assert!(matrix.get_col(10).is_none());

        assert_eq!(matrix.get_col(0), Some(vec_to_elements(vec![1, 4, 7])));
        assert_eq!(matrix.get_col(3), Some(vec_to_elements(vec![10, 11, 12])));
    }
}
