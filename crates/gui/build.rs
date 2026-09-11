fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let locales_dir = format!("{}/resources/locales", env!("CARGO_MANIFEST_DIR"));

    fluent_zero_build::generate_static_cache(&locales_dir);

    relm4_icons_build::bundle_icons(
        // Name of the file that will be generated at `OUT_DIR`
        "icon_names.rs",
        // Optional app ID
        Some("org.relago.Reporter"),
        // Custom base resource path:
        // * defaults to `/com/example/myapp` in this case if not specified explicitly
        // * or `/org/relm4` if app ID was not specified either
        None::<&str>,
        // Directory with custom icons (if any)
        None::<&str>,
        // List of icons to include
        ["plus", "question-round-outline"],
    );
}
