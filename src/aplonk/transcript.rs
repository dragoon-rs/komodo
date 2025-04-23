use ark_ec::pairing::{Pairing, PairingOutput};
use ark_ff::PrimeField;
use ark_serialize::{CanonicalSerialize, Compress, SerializationError};
use rs_merkle::{algorithms::Sha256, Hasher};

/// initialize the transcript of IPA
///
/// this can be found in [aPlonk from [Ambrona et al.]][aPlonK] as $ts := (C_G, r, P)$ in
/// - page **15**. in IPA.Prove.1.
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
pub(super) fn initialize<E: Pairing>(
    c_g: PairingOutput<E>,
    r: E::ScalarField,
    p: E::G1,
) -> Result<Vec<u8>, SerializationError> {
    let mut ts = vec![];
    c_g.serialize_with_mode(&mut ts, Compress::Yes)?;
    r.serialize_with_mode(&mut ts, Compress::Yes)?;
    p.serialize_with_mode(&mut ts, Compress::Yes)?;

    Ok(ts)
}

/// reset the transcript of IPA
///
/// this can be found in [aPlonk from [Ambrona et al.]][aPlonK] as $ts := u_j$ in
/// - page **15**. in IPA.Prove.5.
/// - page **15**. in IPA.Verify.3.
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
pub(super) fn reset<E: Pairing>(u: E::ScalarField) -> Result<Vec<u8>, SerializationError> {
    let mut ts = vec![];
    u.serialize_with_mode(&mut ts, Compress::Yes)?;

    Ok(ts)
}

/// hash curve elements into the transcript of IPA
///
/// this can be found in [aPlonk from [Ambrona et al.]][aPlonK] as
/// $ts := \text{hash}(L_G^j, R_G^j, L_r^j, R_r^j, ts)$ in
/// - page **15**. in IPA.Prove.5.
/// - page **15**. in IPA.Verify.3.
///
/// [aPlonk]: https://eprint.iacr.org/2022/1352.pdf
pub(super) fn hash<E: Pairing>(
    l_g_j: PairingOutput<E>,
    r_g_j: PairingOutput<E>,
    l_r_j: E::G1,
    r_r_j: E::G1,
    ts: &[u8],
) -> Result<E::ScalarField, SerializationError> {
    let mut bytes = vec![];
    l_g_j.serialize_with_mode(&mut bytes, Compress::Yes)?;
    r_g_j.serialize_with_mode(&mut bytes, Compress::Yes)?;
    l_r_j.serialize_with_mode(&mut bytes, Compress::Yes)?;
    r_r_j.serialize_with_mode(&mut bytes, Compress::Yes)?;
    bytes.extend(ts);

    Ok(E::ScalarField::from_le_bytes_mod_order(&Sha256::hash(
        bytes.as_slice(),
    )))
}
