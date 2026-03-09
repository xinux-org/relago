use anyhow::anyhow;
use dbus::arg::Append;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::copy;
use std::io::BufReader;
use std::path::PathBuf;


pub fn compress(path: &str, dest: &PathBuf) -> anyhow::Result<()> {
    let input_file = File::open(path).expect("Failed to open input file");

    // FIXME: maybe take from param ?
    let output_file = File::create( dest.to_owned().join("log.zlib")).expect("Failed to create output file");
    let mut encoder = ZlibEncoder::new(output_file, Compression::fast());
    let mut reader = BufReader::new(input_file);
    copy(&mut reader, &mut encoder).expect("Zlib compression failed");
    encoder.finish().expect("Failed to finish Zlib compression");


    Ok(())
}
