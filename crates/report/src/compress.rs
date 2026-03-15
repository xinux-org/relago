use anyhow::anyhow;
use anyhow::Context;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{copy, BufReader};
use std::path::Path;

// TODO: expect will panic, better use `?` or `.context("message")?`
pub fn compress(path: impl AsRef<Path>, dest: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    let dest = dest.as_ref();

    let filename = path
        .file_name()
        .map(|f| format!("{}.zlib", f.to_string_lossy()))
        .unwrap_or_else(|| "compressed.zlib".to_string());
    let output_path = dest.join(&filename);

    let input_file = File::open(path).expect("Failed to open input file");
    let output_file = File::create(&output_path).expect("Failed to create output file");
    let mut encoder = ZlibEncoder::new(output_file, Compression::fast());
    let mut reader = BufReader::new(input_file);
    copy(&mut reader, &mut encoder).expect("Zlib compression failed");
    encoder.finish().expect("Failed to finish Zlib compression");

    println!("Compressed to: {}", output_path.display());
    Ok(())
}
