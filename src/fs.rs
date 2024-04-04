//! interact with the filesystem, read from and write to it
use std::{
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

use anyhow::Result;

use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use rs_merkle::{algorithms::Sha256, Hasher};
use tracing::info;

use crate::Block;

/// dump any serializable object to the disk
///
/// - `dumpable` can be anything that is _serializable_
/// - if `filename` is provided, then it will be used as the filename as is
/// - otherwise, the hash of the _dumpable_ will be computed and used as the
///   filename
///
/// this function will return the name of the file the _dumpable_ has been
/// dumped to.
pub fn dump(
    dumpable: &impl CanonicalSerialize,
    dump_dir: &Path,
    filename: Option<&str>,
    compress: Compress,
) -> Result<String> {
    info!("serializing the dumpable");
    let mut serialized = vec![0; dumpable.serialized_size(compress)];
    dumpable.serialize_with_mode(&mut serialized[..], compress)?;

    let filename = match filename {
        Some(filename) => filename.to_string(),
        None => Sha256::hash(&serialized)
            .iter()
            .map(|x| format!("{:x}", x))
            .collect::<Vec<_>>()
            .join(""),
    };

    let dump_path = dump_dir.join(&filename);

    info!("dumping dumpable into `{:?}`", dump_path);
    let mut file = File::create(&dump_path)?;
    file.write_all(&serialized)?;

    Ok(filename)
}

/// dump a bunch of blocks to the disk and return a JSON / NUON compatible table
/// of all the hashes that have been dumped
pub fn dump_blocks<F: PrimeField, G: CurveGroup<ScalarField = F>>(
    blocks: &[Block<F, G>],
    block_dir: &PathBuf,
    compress: Compress,
) -> Result<String> {
    info!("dumping blocks to `{:?}`", block_dir);
    let mut hashes = vec![];
    std::fs::create_dir_all(block_dir)?;
    for block in blocks.iter() {
        let hash = dump(block, block_dir, None, compress)?;
        hashes.push(hash);
    }

    let mut formatted_output = String::from("[");
    for hash in &hashes {
        formatted_output.push_str(&format!("{:?},", hash));
    }
    formatted_output.push(']');

    Ok(formatted_output)
}

/// read blocks from a list of block hashes
pub fn read_blocks<F: PrimeField, G: CurveGroup<ScalarField = F>>(
    block_hashes: &[String],
    block_dir: &Path,
    compress: Compress,
    validate: Validate,
) -> Result<Vec<(String, Block<F, G>)>> {
    block_hashes
        .iter()
        .map(|f| {
            let filename = block_dir.join(f);
            let s = std::fs::read(filename)?;
            Ok((
                f.clone(),
                Block::<F, G>::deserialize_with_mode(&s[..], compress, validate)?,
            ))
        })
        .collect()
}
