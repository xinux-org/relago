fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let locales_dir = format!("{}/resources/locales", env!("CARGO_MANIFEST_DIR"));

    fluent_zero_build::generate_static_cache(&locales_dir);
}
