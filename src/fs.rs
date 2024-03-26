use std::{
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

use anyhow::Result;

use ark_ec::pairing::Pairing;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use rs_merkle::{algorithms::Sha256, Hasher};
use tracing::info;

use crate::Block;

const COMPRESS: Compress = Compress::Yes;
const VALIDATE: Validate = Validate::Yes;

pub fn dump(
    dumpable: &impl CanonicalSerialize,
    dump_dir: &Path,
    filename: Option<&str>,
) -> Result<PathBuf> {
    info!("serializing the dumpable");
    let mut serialized = vec![0; dumpable.serialized_size(COMPRESS)];
    dumpable.serialize_with_mode(&mut serialized[..], COMPRESS)?;

    let filename = match filename {
        Some(filename) => filename.to_string(),
        None => Sha256::hash(&serialized)
            .iter()
            .map(|x| format!("{:x}", x))
            .collect::<Vec<_>>()
            .join(""),
    };

    let dump_path = dump_dir.join(filename);

    info!("dumping dumpable into `{:?}`", dump_path);
    let mut file = File::create(&dump_path)?;
    file.write_all(&serialized)?;

    Ok(dump_path)
}

pub fn dump_blocks<E: Pairing>(blocks: &[Block<E>], block_dir: &PathBuf) -> Result<String> {
    info!("dumping blocks to `{:?}`", block_dir);
    let mut hashes = vec![];
    std::fs::create_dir_all(block_dir)?;
    for block in blocks.iter() {
        let filename = dump(block, block_dir, None)?;
        hashes.push(filename);
    }

    let mut formatted_output = String::from("[");
    for hash in &hashes {
        formatted_output = format!("{}{:?},", formatted_output, hash);
    }
    formatted_output = format!("{}{}", formatted_output, "]");

    Ok(formatted_output)
}

pub fn read_blocks<E: Pairing>(
    block_hashes: &[String],
    block_dir: &Path,
) -> Result<Vec<(String, Block<E>)>> {
    block_hashes
        .iter()
        .map(|f| {
            let filename = block_dir.join(f);
            let s = std::fs::read(filename)?;
            Ok((
                f.clone(),
                Block::<E>::deserialize_with_mode(&s[..], COMPRESS, VALIDATE)?,
            ))
        })
        .collect()
}
