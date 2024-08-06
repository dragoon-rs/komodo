//! a replacement of Arkworks' KZG10 module
use ark_ec::{scalar_mul::fixed_base::FixedBase, CurveGroup, VariableBaseMSM};
use ark_ff::PrimeField;
use ark_poly::DenseUVPolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{end_timer, ops::Div, rand::RngCore, start_timer};

#[cfg(feature = "kzg")]
use ark_ec::pairing::Pairing;
#[cfg(feature = "kzg")]
use ark_poly_commit::kzg10;

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
pub fn setup<F: PrimeField, G: CurveGroup<ScalarField = F>>(
    max_degree: usize,
    rng: &mut impl RngCore,
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

    let commit_time =
        start_timer!(|| format!("Committing to polynomial of degree {}", polynomial.degree(),));

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

/// compute the commitments of a set of polynomials
///
/// this function uses the commit scheme of KZG.
///
/// > **Note**
/// > - `powers` can be generated with functions like [`setup`]
/// > - if `polynomials` has length `n`, then [`commit`] will generate `n`
/// >   commits.
#[allow(clippy::type_complexity)]
#[inline(always)]
pub fn batch_commit<F, G, P>(
    powers: &Powers<F, G>,
    polynomials: &[P],
) -> Result<Vec<Commitment<F, G>>, KomodoError>
where
    F: PrimeField,
    G: CurveGroup<ScalarField = F>,
    P: DenseUVPolynomial<F>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let mut commits = Vec::new();
    for polynomial in polynomials {
        let commit = commit(powers, polynomial)?;
        commits.push(commit);
    }

    Ok(commits)
}

/// compute the number of elements that a _trusted setup_ should have for data of
/// a certain expected size
pub fn nb_elements_in_setup<F: PrimeField>(nb_bytes: usize) -> usize {
    let bytes_per_element = (F::MODULUS_BIT_SIZE as usize) / 8;
    nb_bytes / bytes_per_element
}

/// Specializes the public parameters for a given maximum degree `d` for polynomials
/// `d` should be less that `pp.max_degree()`.
///
/// > see [`ark-poly-commit::kzg10::tests::KZG10`](https://gitlab.isae-supaero.fr/a.stevan/poly-commit/-/blob/19fc0d4ad2bcff7df030c952d09649918dba7ddb/src/kzg10/mod.rs#L513-L538)
#[cfg(feature = "kzg")]
pub fn trim<E: Pairing>(
    pp: kzg10::UniversalParams<E>,
    supported_degree: usize,
) -> (kzg10::Powers<'static, E>, kzg10::VerifierKey<E>) {
    let powers_of_g = pp.powers_of_g[..=supported_degree].to_vec();
    let powers_of_gamma_g = (0..=supported_degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();

    let powers = kzg10::Powers {
        powers_of_g: ark_std::borrow::Cow::Owned(powers_of_g),
        powers_of_gamma_g: ark_std::borrow::Cow::Owned(powers_of_gamma_g),
    };
    let vk = kzg10::VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };

    (powers, vk)
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

        let powers = setup::<F, G>(degree, rng).unwrap();

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

        let powers = setup::<F, G>(0, rng);
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
            setup::<F, G>(1, rng).is_ok(),
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

        let powers = setup::<F, G>(degree, rng).unwrap();

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
