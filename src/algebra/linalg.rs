//! Some linear algebra fun over elements in $\mathbb{F}_p$.
//!
//! This module mainly contains an implementation of matrices over a finite
//! field $\mathbb{F}_p$.
use ark_ff::Field;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::{Rng, RngCore};

use crate::error::KomodoError;

/// A matrix defined over a finite field $\mathbb{F}_p$.
///
/// Internally, a matrix is just a vector of field elements whose length is
/// exactly the width times the height and where elements are organized row by
/// row.
#[derive(Clone, PartialEq, Default, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct Matrix<T: Field> {
    /// $h \times w$ elements in $\mathbb{F}_p$.
    pub elements: Vec<T>,
    /// the number of rows $h$.
    pub height: usize,
    /// the number of columns $w$.
    pub width: usize,
}

impl<T: Field> Matrix<T> {
    /// Builds a matrix from a diagonal of elements in $\mathbb{F}_p$.
    ///
    /// # Example
    /// Building a diagonal matrix from the diagonal $(1, 2, 3, 4)$ would give
    /// $ \begin{pmatrix}
    ///     1 & . & . & . \\\\
    ///     . & 2 & . & . \\\\
    ///     . & . & 3 & . \\\\
    ///     . & . & . & 4 \\\\
    /// \end{pmatrix} $.
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

    /// Builds the identity matrix $I_n$ of a given size $n$.
    ///
    /// # Example
    /// The identity of size $3$ is
    /// $ I_3 = \begin{pmatrix}
    ///     1 & . & . \\\\
    ///     . & 1 & . \\\\
    ///     . & . & 1 \\\\
    /// \end{pmatrix} $.
    fn identity(size: usize) -> Self {
        Self::from_diagonal(vec![T::one(); size])
    }

    /// Builds a _Vandermonde_ matrix for some _seed points_.
    ///
    /// Actually, this is the transpose of the Vandermonde matrix defined in the
    /// [Wikipedia article][article], i.e. there are as many columns as there
    /// are seed points, the $(\alpha_i)_{1 \leq i \leq m}$, and there are as
    /// many rows, $n$, as there are powers of the seed points.
    ///
    /// $ M = V_n(\alpha_1, ..., \alpha_m)^T = \begin{pmatrix}
    ///     1                & 1                & ...    & 1                \\\\
    ///     \alpha_1         & \alpha_2         & ...    & \alpha_m         \\\\
    ///     \alpha_1^2       & \alpha_2^2       & ...    & \alpha_m^2       \\\\
    ///     \vdots           & \vdots           & \ddots & \vdots           \\\\
    ///     \alpha_1^{n - 1} & \alpha_2^{n - 1} & ...    & \alpha_m^{n - 1} \\\\
    /// \end{pmatrix} $
    ///
    /// > **Note**
    /// >
    /// > If you are sure the points are distinct and don't want to perform any
    /// > runtime check to ensure that condition, have a look at
    /// > [`Self::vandermonde_unchecked`].
    ///
    /// # Example
    /// Let's compute $V_4(0, 1, 2, 3, 4)^T$:
    /// ```rust
    /// # use ark_ff::Field;
    /// # use komodo::algebra::linalg::Matrix;
    /// // helper to convert integers to field elements
    /// fn vec_to_elements<T: Field>(elements: Vec<u128>) -> Vec<T>
    /// # {
    /// #    elements.iter().map(|&x| T::from(x)).collect()
    /// # }
    /// # type T = ark_bls12_381::Fr;
    ///
    /// let seed_points = vec_to_elements(vec![0, 1, 2, 3, 4]);
    /// let height = 4;
    ///
    /// let expected = vec_to_elements(vec![
    ///     1, 1, 1,  1,  1,
    ///     0, 1, 2,  3,  4,
    ///     0, 1, 4,  9, 16,
    ///     0, 1, 8, 27, 64,
    /// ]);
    ///
    /// assert_eq!(
    ///     Matrix::<T>::vandermonde(&seed_points, height).unwrap(),
    ///     Matrix { elements: expected, height, width: seed_points.len() }
    /// );
    /// ```
    ///
    /// [article]: https://en.wikipedia.org/wiki/Vandermonde_matrix
    pub fn vandermonde(points: &[T], height: usize) -> Result<Self, KomodoError> {
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                if points[i] == points[j] {
                    return Err(KomodoError::InvalidVandermonde {
                        first_index: i,
                        second_index: j,
                        value_repr: format!("{}", points[i]),
                    });
                }
            }
        }

        Ok(Self::vandermonde_unchecked(points, height))
    }

    /// The unchecked version of [`Self::vandermonde`].
    pub fn vandermonde_unchecked(points: &[T], height: usize) -> Self {
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

    /// Builds a completely random matrix of shape $n \times m$.
    pub fn random<R: RngCore>(n: usize, m: usize, rng: &mut R) -> Self {
        Self {
            elements: (0..(n * m)).map(|_| T::from(rng.gen::<u128>())).collect(),
            height: n,
            width: m,
        }
    }

    /// Builds a matrix from a "_matrix_" of elements.
    ///
    /// > **Note**
    /// >
    /// > If you are sure each row should have the same length and don't want to
    /// > perform any runtime check to ensure that condition, have a look at
    /// > [`Self::from_vec_vec_unchecked`].
    ///
    /// # Example
    /// ```rust
    /// # use komodo::algebra::linalg::Matrix;
    /// # use ark_ff::Field;
    /// // helper to convert integers to field elements
    /// fn vec_to_elements<T: Field>(elements: Vec<u128>) -> Vec<T>
    /// # {
    /// #    elements.iter().map(|&x| T::from(x)).collect()
    /// # }
    /// // helper to convert integers to field elements, in a "matrix"
    /// fn mat_to_elements<T: Field>(mat: Vec<Vec<u128>>) -> Vec<Vec<T>>
    /// # {
    /// #     mat.iter().cloned().map(vec_to_elements).collect()
    /// # }
    /// # type T = ark_bls12_381::Fr;
    ///
    /// let elements = mat_to_elements(vec![
    ///     vec![0, 1, 2, 3],
    ///     vec![4, 5, 6, 7],
    ///     vec![8, 9, 0, 1],
    /// ]);
    ///
    /// let height = elements.len();
    /// let width = elements[0].len();
    ///
    /// let expected = vec_to_elements(vec![
    ///     0, 1, 2, 3,
    ///     4, 5, 6, 7,
    ///     8, 9, 0, 1,
    /// ]);
    ///
    /// assert_eq!(
    ///     Matrix::<T>::from_vec_vec(elements).unwrap(),
    ///     Matrix { elements: expected, height, width }
    /// );
    /// ```
    pub fn from_vec_vec(matrix: Vec<Vec<T>>) -> Result<Self, KomodoError> {
        if matrix.is_empty() {
            return Ok(Self {
                elements: vec![],
                height: 0,
                width: 0,
            });
        }

        let width = matrix[0].len();
        for (i, row) in matrix.iter().enumerate() {
            if row.len() != width {
                return Err(KomodoError::InvalidMatrixElements {
                    expected: width,
                    found: row.len(),
                    row: i,
                });
            }
        }

        Ok(Self::from_vec_vec_unchecked(matrix))
    }

    /// The unchecked version of [`Self::from_vec_vec`].
    pub fn from_vec_vec_unchecked(matrix: Vec<Vec<T>>) -> Self {
        let height = matrix.len();
        let width = matrix[0].len();

        let mut elements = Vec::new();
        elements.resize(height * width, T::zero());
        for i in 0..height {
            for j in 0..width {
                elements[i * width + j] = matrix[i][j];
            }
        }

        Self {
            elements,
            height,
            width,
        }
    }

    fn get(&self, i: usize, j: usize) -> T {
        self.elements[i * self.width + j]
    }

    fn set(&mut self, i: usize, j: usize, value: T) {
        self.elements[i * self.width + j] = value;
    }

    /// Extracts a single column from the matrix.
    ///
    /// > **Note**
    /// >
    /// > Returns `None` if the provided index is out of bounds.
    pub(crate) fn get_col(&self, j: usize) -> Option<Vec<T>> {
        if j >= self.width {
            return None;
        }

        Some((0..self.height).map(|i| self.get(i, j)).collect())
    }

    /// Computes $\text{row} = \frac{\text{row}}{\text{value}}$.
    fn divide_row_by(&mut self, row: usize, value: T) {
        for j in 0..self.width {
            self.set(row, j, self.get(row, j) / value);
        }
    }

    /// Computes $\text{destination} = \text{destination} + \text{source} \times \text{value}$.
    fn multiply_row_by_and_add_to_row(&mut self, source: usize, value: T, destination: usize) {
        for j in 0..self.width {
            self.set(
                destination,
                j,
                self.get(destination, j) + self.get(source, j) * value,
            );
        }
    }

    /// Computes the inverse of the matrix.
    ///
    /// If $M \in \mathcal{M}_{n \times n}(\mathbb{F}_p)$ is an invertible matrix,
    /// then [`Self::invert`] computes $M^{-1}$ such that
    /// $$ MM^{-1} = M^{-1}M = I_n$$
    pub fn invert(&self) -> Result<Self, KomodoError> {
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

    /// Swaps rows $i$ and $j$, inplace.
    ///
    /// > **Note**
    /// >
    /// > This function assumes both $i$ and $j$ are in bounds, unexpected
    /// > results are expected if $i$ or $j$ are out of bounds.
    fn swap_rows(&mut self, i: usize, j: usize) {
        for k in 0..self.width {
            self.elements.swap(i * self.width + k, j * self.width + k);
        }
    }

    /// Computes the rank of the matrix.
    ///
    /// Let $M \in \mathcal{M}_{n \times m}(\mathbb{F}_p)$ and $r(M)$ its rank:
    /// - the rank is always smaller than the min between the height and the
    ///   width of any matrix, $r(M) \leq \min(n, m)$
    /// - a square and invertible matrix will have _full rank_, i.e. it will
    ///   be equal to its size, if $M$ is invertible, then $r(M) = n$
    ///
    /// > **Note**
    /// >
    /// > See the [_Wikipedia article_](https://en.wikipedia.org/wiki/Rank_(linear_algebra))
    /// > for more information
    pub fn rank(&self) -> usize {
        let mut mat = self.clone();
        let mut i = 0;

        for j in 0..self.width {
            let mut found = false;
            // look for the first non-zero pivot in the j-th column
            for k in i..self.height {
                if !mat.get(k, j).is_zero() {
                    mat.swap_rows(i, k); // move the non-zero element to the diagonal
                    found = true;
                    break;
                }
            }

            if found {
                // update the bottom-right part of the matrix
                for k in (i + 1)..self.height {
                    let ratio = mat.get(k, j) / mat.get(i, j);
                    for l in j..self.width {
                        let el = mat.get(i, l);
                        mat.set(k, l, mat.get(k, l) - ratio * el);
                    }
                }
                i += 1;
            }
        }

        let nb_non_zero_rows = (0..self.height)
            .filter(|i| {
                let row = mat.elements[(i * self.width)..((i + 1) * self.width)].to_vec();
                row.iter().any(|&x| !x.is_zero())
            })
            .collect::<Vec<_>>()
            .len();

        nb_non_zero_rows
    }

    /// Computes the matrix multiplication with another matrix.
    ///
    /// Let $A \in \mathcal{M}_{a \times b}(\mathbb{F}_p) \sim \texttt{lhs}$ and
    /// $B \in \mathcal{M}\_{c \times d}(\mathbb{F}_p) \sim \texttt{rhs}$ then
    /// `lhs.mul(rhs)` will compute $A \times B$.
    ///
    /// > **Note**
    /// >
    /// > Both matrices should have compatible shapes, i.e. if `self` has shape
    /// > `(a, b)` and `rhs` has shape `(c, d)`, then `b == c`.
    pub fn mul(&self, rhs: &Self) -> Result<Self, KomodoError> {
        if self.width != rhs.height {
            return Err(KomodoError::IncompatibleMatrixShapes {
                left: (self.height, self.width),
                right: (rhs.height, rhs.width),
            });
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

    /// Computes the transpose of the matrix.
    ///
    /// > **Note**
    /// >
    /// > see the [_Wikipedia article_](https://en.wikipedia.org/wiki/Transpose)
    pub fn transpose(&self) -> Self {
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

    /// Truncates the matrix to the provided shape, from right and bottom.
    ///
    /// # Example
    /// If a matrix has shape $(10, 11)$ and is truncated to $(5, 7)$, the $5$
    /// bottom rows and $4$ right columns will be removed.
    pub(crate) fn truncate(&self, rows: Option<usize>, cols: Option<usize>) -> Self {
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

impl<T: Field> std::fmt::Display for Matrix<T> {
    /// an example matrix with the identity of order 3
    /// ```text
    /// /1 0 0\
    /// |0 1 0|
    /// \0 0 1/
    /// ```
    ///
    /// - zero elements will show as "0" instead of a blank string
    /// - elements that are bigger than the format size will be cropped, i.e.
    ///     - by default, the format size is undefined an thus elements won't be cropped
    ///     - if the format looks like `{:5}`, any element whose representation is bigger than 5
    ///     characters will be cropped
    /// - the default cropping is done with `...` but adding `#` to the format string will use `*`
    /// instead
    ///
    /// a few examples of a matrix with some random elements that are too big to be shown in 5
    /// characters
    ///
    /// - when the format is `{:5}`
    /// ```text
    /// /1     0     20... 0    \
    /// |0     1     32... 0    |
    /// |0     0     0     0    |
    /// |0     0     0     11...|
    /// \0     0     0     17.../
    /// ```
    /// - when the format is `{:#}` or `{:#1}`
    /// ```text
    /// /1 0 * 0\
    /// |0 1 * 0|
    /// |0 0 0 0|
    /// |0 0 0 *|
    /// \0 0 0 */
    /// ```
    /// - when the format is `{:#5}`
    /// ```text
    /// /1     0     *     0    \
    /// |0     1     *     0    |
    /// |0     0     0     0    |
    /// |0     0     0     *    |
    /// \0     0     0     *    /
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..self.height {
            let start = if i == 0 {
                "/"
            } else if i == self.height - 1 {
                "\\"
            } else {
                "|"
            };
            write!(f, "{}", start)?;

            for j in 0..self.width {
                let x = self.get(i, j);
                let y = if x.is_zero() {
                    "0".to_string()
                } else {
                    format!("{}", x)
                };

                if let Some(w) = f.width() {
                    if y.len() > w {
                        if f.alternate() {
                            write!(f, "{:width$}", "*", width = w)?;
                        } else {
                            let t = if w > 3 { w - 3 } else { 0 };
                            write!(
                                f,
                                "{:width$}",
                                format!("{}{}", y.chars().take(t).collect::<String>(), "..."),
                                width = w
                            )?;
                        }
                    } else {
                        write!(f, "{:width$}", format!("{}", y), width = w)?;
                    }
                } else if f.alternate() && y.len() > 1 {
                    write!(f, "*")?;
                } else {
                    write!(f, "{}", y)?;
                }

                if j < self.width - 1 {
                    write!(f, " ")?;
                }
            }

            let end = if i == 0 {
                "\\"
            } else if i == self.height - 1 {
                "/"
            } else {
                "|"
            };
            writeln!(f, "{}", end)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::Fr;
    use ark_ff::Field;

    use super::{KomodoError, Matrix};

    // two wrapped functions to make the tests more readable

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
        assert_eq!(
            matrix.err().unwrap(),
            KomodoError::InvalidMatrixElements {
                expected: 1,
                found: 2,
                row: 1,
            }
        );
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

        assert_eq!(
            a.mul(&Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![vec![1, 2]])).unwrap()),
            Err(KomodoError::IncompatibleMatrixShapes {
                left: (3, 3),
                right: (1, 2)
            })
        );

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
    fn random() {
        let mut rng = ark_std::test_rng();

        for n in 0..10 {
            for m in 0..10 {
                let mat = Matrix::<Fr>::random(n, m, &mut rng);
                assert_eq!(mat.elements.len(), n * m);
                assert_eq!(mat.width, m);
                assert_eq!(mat.height, n);
            }
        }
    }

    #[test]
    fn inverse() {
        let mut rng = ark_std::test_rng();

        let matrix = Matrix::<Fr>::identity(3);
        let inverse = matrix.invert().unwrap();
        assert_eq!(Matrix::<Fr>::identity(3), inverse);

        let matrix = Matrix::<Fr>::from_diagonal(vec_to_elements(vec![2, 3, 4]));
        let inverse = matrix.invert().unwrap();
        assert_eq!(matrix.mul(&inverse).unwrap(), Matrix::<Fr>::identity(3));
        assert_eq!(inverse.mul(&matrix).unwrap(), Matrix::<Fr>::identity(3));

        for n in 1..20 {
            let matrix = Matrix::random(n, n, &mut rng);
            let inverse = matrix.invert().unwrap();
            assert_eq!(matrix.mul(&inverse).unwrap(), Matrix::<Fr>::identity(n));
            assert_eq!(inverse.mul(&matrix).unwrap(), Matrix::<Fr>::identity(n));
        }

        let inverse =
            Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![vec![1, 0, 0], vec![0, 1, 0]]))
                .unwrap()
                .invert();
        assert!(inverse.is_err());
        assert_eq!(inverse.err().unwrap(), KomodoError::NonSquareMatrix(2, 3));

        let inverse = Matrix::<Fr>::from_diagonal(vec_to_elements(vec![0, 3, 4])).invert();
        assert!(inverse.is_err());
        assert_eq!(inverse.err().unwrap(), KomodoError::NonInvertibleMatrix(0));

        let inverse = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 1, 0],
            vec![0, 0, 0],
            vec![0, 0, 1],
        ]))
        .unwrap()
        .invert();
        assert!(inverse.is_err());
        assert_eq!(inverse.err().unwrap(), KomodoError::NonInvertibleMatrix(1));
    }

    #[test]
    fn vandermonde() {
        assert_eq!(
            Matrix::<Fr>::vandermonde(&vec_to_elements(vec![0, 4, 2, 3, 4]), 4),
            Err(KomodoError::InvalidVandermonde {
                first_index: 1,
                second_index: 4,
                value_repr: "4".to_string()
            }),
        );
        assert!(Matrix::<Fr>::vandermonde(&vec_to_elements(vec![0, 1, 2, 3, 4]), 4).is_ok());

        let actual = Matrix::<Fr>::vandermonde_unchecked(&vec_to_elements(vec![0, 1, 2, 3, 4]), 4);
        #[rustfmt::skip]
        let expected = Matrix::from_vec_vec(mat_to_elements(vec![
            vec![1, 1, 1,  1,  1],
            vec![0, 1, 2,  3,  4],
            vec![0, 1, 4,  9, 16],
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

    #[test]
    fn rank() {
        let mut rng = ark_std::test_rng();

        for n in 1..=20 {
            assert_eq!(Matrix::<Fr>::identity(n).rank(), n);
        }

        for _ in 0..20 {
            let m = Matrix::<Fr>::random(7, 13, &mut rng);
            assert_eq!(m.rank(), m.transpose().rank());
        }

        let m = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 0, 0],
            vec![0, 2, 0],
            vec![0, 0, 3],
        ]))
        .unwrap();
        assert_eq!(m.rank(), 3);

        let m = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 0, 0],
            vec![0, 2, 0],
            vec![0, 0, 3],
            vec![0, 0, 3],
        ]))
        .unwrap();
        assert_eq!(m.rank(), 3);

        let m = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![1, 0, 0],
            vec![0, 2, 0],
            vec![0, 0, 0],
        ]))
        .unwrap();
        assert_eq!(m.rank(), 2);

        let m = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![0, 0, 0],
            vec![0, 0, 0],
            vec![0, 0, 0],
        ]))
        .unwrap();
        assert_eq!(m.rank(), 0);

        let m = Matrix::<Fr>::from_vec_vec(mat_to_elements(vec![
            vec![0, 0, 1, 0],
            vec![1, 0, 0, 1],
            vec![0, 1, 0, 1],
            vec![0, 1, 1, 0],
            vec![1, 0, 0, 0],
        ]))
        .unwrap();
        let rank = m.rank();
        assert!(
            rank <= m.height.min(m.width),
            "rank should be less than {}, got {}",
            m.height.min(m.width),
            rank
        );
    }
}
