use ark_ec::{scalar_mul::fixed_base::FixedBase, CurveGroup, VariableBaseMSM};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{end_timer, rand::RngCore, start_timer};

use crate::error::KomodoError;

/// the representation of a ZK trusted setup
///
/// this is a simple wrapper around a sequence of elements of the curve.
#[derive(Debug, Clone, Default, CanonicalSerialize, CanonicalDeserialize, PartialEq)]
pub struct Powers<F: PrimeField, G: CurveGroup<ScalarField = F>>(Vec<G::Affine>);

impl<F: PrimeField, G: CurveGroup<ScalarField = F>> Powers<F, G> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<F: PrimeField, G: CurveGroup<ScalarField = F>> IntoIterator for Powers<F, G> {
    type Item = G::Affine;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// a ZK commitment, i.e. an evaluatio of a given polynomial on a secret
///
/// this is a simpler wrapper around a single elemenf of the curve.
#[derive(Debug, Clone, Copy, Default, CanonicalSerialize, CanonicalDeserialize, PartialEq)]
pub struct Commitment<F: PrimeField, G: CurveGroup<ScalarField = F>>(pub G::Affine);

/// create a trusted setup of a given size, the expected maximum degree of the data
pub fn setup<R: RngCore, F: PrimeField, G: CurveGroup<ScalarField = F>>(
    max_degree: usize,
    rng: &mut R,
) -> Result<Powers<F, G>, KomodoError> {
    if max_degree < 1 {
        return Err(KomodoError::DegreeIsZero);
    }
    let setup_time = start_timer!(|| format!("setup with degree {}", max_degree));

    let beta = F::rand(rng);
    let g = G::rand(rng);

    let mut powers_of_beta = vec![F::one()];
    let mut cur = beta;
    for _ in 0..max_degree {
        powers_of_beta.push(cur);
        cur *= &beta;
    }

    let window_size = FixedBase::get_mul_window_size(max_degree + 1);
    let scalar_bits = F::MODULUS_BIT_SIZE as usize;

    let g_time = start_timer!(|| "Generating powers of G");
    let g_table = FixedBase::get_window_table(scalar_bits, window_size, g);
    let powers_of_g = FixedBase::msm::<G>(scalar_bits, window_size, &g_table, &powers_of_beta);
    end_timer!(g_time);

    let powers_of_g: Vec<G::Affine> = G::normalize_batch(&powers_of_g);

    end_timer!(setup_time);
    Ok(Powers(powers_of_g))
}

fn check_degree_is_too_large(degree: usize, num_powers: usize) -> Result<(), KomodoError> {
    let num_coefficients = degree + 1;
    if num_coefficients > num_powers {
        Err(KomodoError::TooFewPowersInTrustedSetup(
            num_powers,
            num_coefficients,
        ))
    } else {
        Ok(())
    }
}

fn skip_leading_zeros_and_convert_to_bigints<F: PrimeField, P: DenseUVPolynomial<F>>(
    p: &P,
) -> (usize, Vec<F::BigInt>) {
    let mut num_leading_zeros = 0;
    while num_leading_zeros < p.coeffs().len() && p.coeffs()[num_leading_zeros].is_zero() {
        num_leading_zeros += 1;
    }
    let coeffs = convert_to_bigints(&p.coeffs()[num_leading_zeros..]);
    (num_leading_zeros, coeffs)
}

fn convert_to_bigints<F: PrimeField>(p: &[F]) -> Vec<F::BigInt> {
    let to_bigint_time = start_timer!(|| "Converting polynomial coeffs to bigints");
    let coeffs = ark_std::cfg_iter!(p)
        .map(|s| s.into_bigint())
        .collect::<Vec<_>>();
    end_timer!(to_bigint_time);
    coeffs
}

/// compute a commitment of a polynomial on a trusted setup
pub fn commit<F, G, P>(
    powers: &Powers<F, G>,
    polynomial: &P,
) -> Result<Commitment<F, G>, KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
{
    check_degree_is_too_large(polynomial.degree(), powers.len())?;

    let commit_time = start_timer!(|| format!(
        "Committing to polynomial of degree {} with hiding_bound: {:?}",
        polynomial.degree(),
        hiding_bound,
    ));

    let (num_leading_zeros, plain_coeffs) = skip_leading_zeros_and_convert_to_bigints(polynomial);

    let msm_time = start_timer!(|| "MSM to compute commitment to plaintext poly");
    let commitment = <G as VariableBaseMSM>::msm_bigint(
        &powers.0[num_leading_zeros..],
        // FIXME: this is far from satisfying
        &plain_coeffs.into_iter().collect::<Vec<_>>(),
    );
    end_timer!(msm_time);

    end_timer!(commit_time);
    Ok(Commitment(commitment.into()))
}

// compute the number of elements that a _trusted setup_ should have for data of
// a certain expected size
pub fn nb_elements_in_setup<F: PrimeField>(nb_bytes: usize) -> usize {
    let bytes_per_element = (F::MODULUS_BIT_SIZE as usize) / 8;
    nb_bytes / bytes_per_element
}

#[cfg(test)]
mod tests {
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::CurveGroup;
    use ark_ff::PrimeField;
    use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
    use ark_std::test_rng;

    use crate::error::KomodoError;

    use super::{commit as commit_to_test, setup};

    fn generate_setup_template<F: PrimeField, G: CurveGroup<ScalarField = F>>(nb_bytes: usize) {
        let degree = nb_bytes / (F::MODULUS_BIT_SIZE as usize / 8);

        let rng = &mut test_rng();

        let powers = setup::<_, F, G>(degree, rng).unwrap();

        assert_eq!(
            powers.len(),
            degree + 1,
            "number of powers in the trusted setup does not match the number of coefficients"
        );
    }

    #[test]
    fn generate_setup() {
        for nb_kb in [1, 2, 4, 8, 16, 32, 64] {
            generate_setup_template::<Fr, G1Projective>(nb_kb * 1024);
        }
    }

    fn generate_invalid_setup_template<F: PrimeField, G: CurveGroup<ScalarField = F>>() {
        let rng = &mut test_rng();

        let powers = setup::<_, F, G>(0, rng);
        assert!(
            powers.is_err(),
            "creating a trusted setup for a degree 0 polynomial should NOT work"
        );
        assert_eq!(
            powers.err().unwrap(),
            KomodoError::DegreeIsZero,
            "message should say the degree is zero"
        );
        assert!(
            setup::<_, F, G>(1, rng).is_ok(),
            "creating a trusted setup for any polynomial with degree at least 1 should work"
        );
    }

    #[test]
    fn generate_invalid_setup() {
        generate_invalid_setup_template::<Fr, G1Projective>();
    }

    fn commit_template<F, G, P>(nb_bytes: usize)
    where
        F: PrimeField,
        G: CurveGroup<ScalarField = F>,
        P: DenseUVPolynomial<F>,
    {
        let degree = nb_bytes / (F::MODULUS_BIT_SIZE as usize / 8);

        let rng = &mut test_rng();

        let powers = setup::<_, F, G>(degree, rng).unwrap();

        assert!(
            commit_to_test(&powers, &P::rand(degree - 1, rng)).is_ok(),
            "committing a polynomial with less coefficients than there are powers in the trusted setup should work"
        );
        assert!(
            commit_to_test(&powers, &P::rand(degree, rng)).is_ok(),
            "committing a polynomial with as many coefficients as there are powers in the trusted setup should work"
        );
        assert!(
            commit_to_test(&powers, &P::rand(degree + 1, rng)).is_err(),
            "committing a polynomial with more coefficients than there are powers in the trusted setup should NOT work"
        );
    }

    #[test]
    fn commit() {
        for nb_kb in [1, 2, 4, 8, 16, 32, 64] {
            commit_template::<Fr, G1Projective, DensePolynomial<Fr>>(nb_kb * 1024);
        }
    }
}
