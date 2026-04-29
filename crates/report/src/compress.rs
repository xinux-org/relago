use anyhow::{anyhow, Context};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{copy, BufReader};
use std::path::{Path, PathBuf};
use utils::config::CONFIG;
use zip_archive::Archiver;

pub fn compress(path: impl AsRef<Path>, dest: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let dest = dest.as_ref();

    let filename = path
        .file_name()
        .map(|f| format!("{}.zlib", f.to_string_lossy()))
        .unwrap_or_else(|| "compressed.zlib".to_string());
    let output_path = dest.join(&filename);

    let input_file = File::open(path).context("Failed to open input file")?;
    let output_file = File::create(&output_path).context("Failed to create output file")?;
    let mut encoder = ZlibEncoder::new(output_file, Compression::fast());
    let mut reader = BufReader::new(input_file);
    copy(&mut reader, &mut encoder).context("Zlib compression failed")?;
    encoder
        .finish()
        .context("Failed to finish Zlib compression")?;

    println!("Compressed to: {}", output_path.display());
    Ok(())
}

pub fn compress_zip(origin: impl AsRef<Path>, dest: impl AsRef<Path>) -> anyhow::Result<()> {
    let parallel_compression = CONFIG.get().parallel_compression;
    let origin = PathBuf::from(origin.as_ref());
    let dest = PathBuf::from(dest.as_ref());

    let mut archiver = Archiver::new();

    archiver.push(origin);
    archiver.set_destination(dest);
    archiver.set_thread_count(parallel_compression);
    // println!("Compressed to: {}", output_path.display());

    archiver
        .archive()
        .map_err(|err| anyhow!("Cannot archive the directory! {}", err))
}
