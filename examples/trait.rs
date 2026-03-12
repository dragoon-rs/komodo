use ark_ec::pairing::Pairing;
use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_std::ops::Div;

use komodo::{algebra, error::KomodoError, fec, semi_avid};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

#[cfg(feature = "aplonk")]
use komodo::aplonk;
#[cfg(feature = "kzg")]
use komodo::kzg;

macro_rules! start_timer {
    ($msg:expr) => {{
        eprint!("{}", $msg);
        std::time::Instant::now()
    }};
}

macro_rules! end_timer {
    ($t:expr) => {{
        eprintln!("done in {:?}", $t.elapsed());
    }};
    ($t:expr, $msg:expr) => {{
        eprintln!("done in {:?} ({})", $t.elapsed(), $msg);
    }};
}

fn run_protocol<E, P, T>(
    protoc: T,
    max_degree: usize,
    bytes: &[u8],
    shards: &[T::Shard],
    rng: &mut impl Rng,
) -> Result<(), KomodoError>
where
    T: komodo::Protocol,
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let t = start_timer!("        SETUP... ");
    let (setup, vk) = protoc.setup(max_degree, rng)?;
    end_timer!(t);

    let t = start_timer!("        COMMIT... ");
    let commitment = protoc.commit(bytes, &setup)?;
    end_timer!(t);

    let t = start_timer!("        PROVE... ");
    let proofs = protoc.prove(bytes, &commitment, shards, &setup)?;
    end_timer!(t);

    let t = start_timer!("        VERIFY... ");
    let mut ok = true;
    for (shard, proof) in shards.iter().zip(proofs.iter()) {
        if !protoc.verify(&commitment, shard, proof, &vk)? {
            ok = false;
        }
    }
    end_timer!(t, ok);

    Ok(())
}

fn example<E, P>(
    bytes: &[u8],
    (k, n): (usize, usize),
    rng: &mut impl Rng,
) -> Result<(), KomodoError>
where
    E: Pairing,
    P: DenseUVPolynomial<E::ScalarField, Point = E::ScalarField>,
    for<'a, 'b> &'a P: Div<&'b P, Output = P>,
{
    let t = start_timer!(format!("({},{})-encoding {} bytes... ", k, n, bytes.len()));
    let points = &(0..n)
        .map(|i| E::ScalarField::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect::<Vec<_>>();
    let encoding_mat = algebra::linalg::Matrix::vandermonde_unchecked(points, k);
    let mut shards = fec::encode(bytes, &encoding_mat).unwrap();
    end_timer!(t);

    let ff_byte_size = E::ScalarField::MODULUS_BIT_SIZE as usize / 8;
    let m = bytes.len() / ff_byte_size / k;

    eprintln!("    SEMI-AVID");
    run_protocol::<E, P, _>(
        semi_avid::SemiAVID::<E::ScalarField, E::G1, P>::new(k),
        m - 1,
        bytes,
        &shards,
        rng,
    )?;

    #[cfg(feature = "kzg")]
    {
        eprintln!("    KZG");
        run_protocol::<E, P, _>(kzg::Kzg::<E, P>::new(k), k - 1, bytes, &shards, rng)?;
    }

    #[cfg(feature = "aplonk")]
    {
        eprintln!("    APLONK");
        run_protocol::<E, P, _>(
            aplonk::Aplonk::<E, P>::new(k, m),
            k - 1,
            bytes,
            &shards,
            rng,
        )?;
    }

    // gather k random shards
    shards.shuffle(rng);
    shards = shards[..k].to_vec();

    let t = start_timer!("decoding... ");
    assert_eq!(bytes, fec::decode(&shards).unwrap());
    end_timer!(t);

    Ok(())
}

fn main() {
    let (k, n) = (3, 6);
    let bytes = include_bytes!("../assets/dragoon_133x133.png");

    // NOTE: aPlonK requires the size of the data to be a "power of 2" multiple of the field element size
    let ff_bit_size = <ark_bls12_381::Bls12_381 as Pairing>::ScalarField::MODULUS_BIT_SIZE;
    let ff_byte_size = ff_bit_size as usize / 8;
    let nb_bytes = k * 8 * ff_byte_size;

    example::<
        ark_bls12_381::Bls12_381,
        DensePolynomial<<ark_bls12_381::Bls12_381 as Pairing>::ScalarField>,
    >(&bytes[0..nb_bytes], (k, n), &mut StdRng::seed_from_u64(0))
    .unwrap();
}
