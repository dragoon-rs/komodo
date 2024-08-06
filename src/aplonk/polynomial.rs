use ark_ec::pairing::Pairing;
use ark_poly::DenseUVPolynomial;
use ark_std::One;
use std::ops::{Div, Mul};

/// convert a number to it's binary representation with 0-padding to the right
fn to_binary(number: usize, width: usize) -> Vec<u8> {
    format!("{:0width$b}", number, width = width)
        .bytes()
        .rev()
        .map(|byte| byte - b'0')
        .collect()
}

/// compute the polynomial *g(X)* in [aPlonk from [Ambrona et al.]][aPlonk]
///
/// *g(X)* can be found, at
/// - page **13**. in *open.7*
/// - page **13**. in *check.5*
/// - page **15**. in *IPA.verify.4*
///
/// it's theoretical formula is the following (modified version):  
/// *g(X) = \Pi_{j=1}^{\kappa = log_2(k)}(u_j^{-1} + u_j X^{2^j})*
///
/// however this formula is not very convenient, so let's expand this and
/// compute all the coefficients!
/// when we do that on small examples:
/// - *\kappa = 1*: *
///     g(X) = (u_0^{-1} + u_0 X)
///          = u_0^{-1} +
///            u_0 X
/// *
/// - *\kappa = 2*: *
///     g(X) = (u_0^{-1} + u_0 X)(u_1^{-1} + u_1 X^2)
///          = u_1^{-1} u_0^{-1}     +
///            u_1^{-1} u_0        X +
///            u_1      u_0^{-1} X^2 +
///            u_1      u_0      X^3
/// *
/// - *\kappa = 3*: *
///     g(X) = (u_0^{-1} + u_0 X)(u_1^{-1} + u_1 X^2)(u_2^{-1} + u_2 X^2)
///          = u_2^{-1} u_1^{-1} u_0^{-1}     +
///            u_2^{-1} u_1^{-1} u_0        X +
///            u_2^{-1} u_1      u_0^{-1} X^2 +
///            u_2^{-1} u_1      u_0      X^3 +
///            u_2      u_1^{-1} u_0^{-1} X^4 +
///            u_2      u_1^{-1} u_0      X^5 +
///            u_2      u_1      u_0^{-1} X^6 +
///            u_2      u_1      u_0      X^7
/// *
///
/// we can see that the *j*-the coefficient of *g(X)* for a given *\kappa* is
/// a product of a combination of *(u_i)* and their inverse elements directly
/// related to the binary representation of the *j* polynomial power, e.g.
/// - with *\kappa = 3* and *j = 6*, the binary is *110* and the coefficient is
///   *u_0 \times u_1 \times u_2^{-1}*
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
pub(super) fn compute_g<E, P>(
    k: usize,
    kappa: usize,
    u: &[E::ScalarField],
    u_inv: &[E::ScalarField],
) -> P
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let coefficients = (0..k)
        .map(|j| {
            to_binary(j, kappa)
                .iter()
                .enumerate()
                .map(|(i, bit)| if *bit == 1u8 { u[i] } else { u_inv[i] })
                .fold(E::ScalarField::one(), |acc, it| acc.mul(it))
        })
        .collect::<Vec<P::Point>>();

    P::from_coefficients_vec(coefficients)
}

#[cfg(test)]
mod tests {
    use std::ops::Div;
    use std::ops::Mul;

    use ark_bls12_381::Bls12_381;
    use ark_ec::pairing::Pairing;
    use ark_ff::Field;
    use ark_poly::univariate::DensePolynomial;
    use ark_poly::DenseUVPolynomial;
    use ark_std::test_rng;
    use ark_std::UniformRand;

    use super::compute_g;
    use super::to_binary;

    type UniPoly381 = DensePolynomial<<Bls12_381 as Pairing>::ScalarField>;

    #[test]
    fn to_binary_conversion() {
        assert_eq!(to_binary(2, 4), vec![0, 1, 0, 0]);
        assert_eq!(to_binary(10, 4), vec![0, 1, 0, 1]);
        assert_eq!(to_binary(5, 8), vec![1, 0, 1, 0, 0, 0, 0, 0]);
    }

    fn g_poly_computation_template<E, P>()
    where
        E: Pairing,
        P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
        for<'a, 'b> &'a P: Div<&'b P, Output = P>,
    {
        let rng = &mut test_rng();

        let u = [
            E::ScalarField::rand(rng),
            E::ScalarField::rand(rng),
            E::ScalarField::rand(rng),
            E::ScalarField::rand(rng),
        ];
        let u_inv: Vec<E::ScalarField> = u.iter().map(|u_i| u_i.inverse().unwrap()).collect();

        let expected_coefficients = vec![u_inv[0], u[0]];
        assert_eq!(
            compute_g::<E, P>(2, 1, &u[..1], &u_inv[..1]),
            P::from_coefficients_vec(expected_coefficients),
            "computation of *g(X)* failed for k = {}",
            1
        );

        #[rustfmt::skip]
        let expected_coefficients = vec![
            u_inv[0].mul(u_inv[1]),
                u[0].mul(u_inv[1]),
            u_inv[0].mul(    u[1]),
                u[0].mul(    u[1]),
        ];
        assert_eq!(
            compute_g::<E, P>(4, 2, &u[..2], &u_inv[..2]),
            P::from_coefficients_vec(expected_coefficients),
            "computation of *g(X)* failed for k = {}",
            2
        );

        #[rustfmt::skip]
        let expected_coefficients = vec![
            u_inv[0].mul(u_inv[1]).mul(u_inv[2]),
                u[0].mul(u_inv[1]).mul(u_inv[2]),
            u_inv[0].mul(    u[1]).mul(u_inv[2]),
                u[0].mul(    u[1]).mul(u_inv[2]),
            u_inv[0].mul(u_inv[1]).mul(    u[2]),
                u[0].mul(u_inv[1]).mul(    u[2]),
            u_inv[0].mul(    u[1]).mul(    u[2]),
                u[0].mul(    u[1]).mul(    u[2]),
        ];
        assert_eq!(
            compute_g::<E, P>(8, 3, &u[..3], &u_inv[..3]),
            P::from_coefficients_vec(expected_coefficients),
            "computation of *g(X)* failed for k = {}",
            3
        );

        #[rustfmt::skip]
        let expected_coefficients = vec![
            u_inv[0].mul(u_inv[1]).mul(u_inv[2]).mul(u_inv[3]),
                u[0].mul(u_inv[1]).mul(u_inv[2]).mul(u_inv[3]),
            u_inv[0].mul(    u[1]).mul(u_inv[2]).mul(u_inv[3]),
                u[0].mul(    u[1]).mul(u_inv[2]).mul(u_inv[3]),
            u_inv[0].mul(u_inv[1]).mul(    u[2]).mul(u_inv[3]),
                u[0].mul(u_inv[1]).mul(    u[2]).mul(u_inv[3]),
            u_inv[0].mul(    u[1]).mul(    u[2]).mul(u_inv[3]),
                u[0].mul(    u[1]).mul(    u[2]).mul(u_inv[3]),
            u_inv[0].mul(u_inv[1]).mul(u_inv[2]).mul(    u[3]),
                u[0].mul(u_inv[1]).mul(u_inv[2]).mul(    u[3]),
            u_inv[0].mul(    u[1]).mul(u_inv[2]).mul(    u[3]),
                u[0].mul(    u[1]).mul(u_inv[2]).mul(    u[3]),
            u_inv[0].mul(u_inv[1]).mul(    u[2]).mul(    u[3]),
                u[0].mul(u_inv[1]).mul(    u[2]).mul(    u[3]),
            u_inv[0].mul(    u[1]).mul(    u[2]).mul(    u[3]),
                u[0].mul(    u[1]).mul(    u[2]).mul(    u[3]),
        ];
        assert_eq!(
            compute_g::<E, P>(16, 4, &u[..4], &u_inv[..4]),
            P::from_coefficients_vec(expected_coefficients),
            "computation of *g(X)* failed for k = {}",
            4
        );
    }

    #[test]
    fn g_poly_computation() {
        g_poly_computation_template::<Bls12_381, UniPoly381>();
    }
}
