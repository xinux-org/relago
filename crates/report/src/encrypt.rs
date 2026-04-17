use anyhow::{Context, Result};
use pgp::composed::{Deserializable, MessageBuilder, SignedPublicKey};
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::types::KeyDetails;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use rand::thread_rng;
use std::path::Path;

pub fn encrypt_file(
    file_path: impl AsRef<Path>,
    public_key_path: impl AsRef<Path>,
) -> Result<std::path::PathBuf> {
    let file_path = file_path.as_ref();
    let public_key_path = public_key_path.as_ref();

    let key_file = File::open(public_key_path).context("Failed to open public key file")?;
    let (public_key, _) = SignedPublicKey::from_reader_single(BufReader::new(key_file))
        .context("Failed to parse public key")?;

    let mut file_data = Vec::new();
    File::open(file_path)
        .context("Failed to open file to encrypt")?
        .read_to_end(&mut file_data)
        .context("Failed to read file data")?;

    let encryption_subkey = public_key
        .public_subkeys
        .iter()
        .find(|sk| sk.algorithm().can_encrypt())
        .context("No encryption-capable subkey found")?;

    // FUTURE: v2 would be the better choice for future-proofing(follow the latest standard)
    let mut builder = MessageBuilder::from_bytes("", file_data)
        .seipd_v1(thread_rng(), SymmetricKeyAlgorithm::AES256);

    builder
        .encrypt_to_key(thread_rng(), encryption_subkey)
        .context("Failed to encrypt to key")?;

    let encrypted = builder
        .to_vec(thread_rng())
        .context("Failed to finalize encrypted message")?;

    let output_path = file_path.with_extension("zip.pgp");
    fs::write(&output_path, &encrypted).context("Failed to write encrypted file")?;

    println!(
        "Encrypted: {} -> {}",
        file_path.display(),
        output_path.display()
    );
    Ok(output_path)
}
