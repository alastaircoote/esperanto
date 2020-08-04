extern crate cbindgen;

use cbindgen::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(format!("{}.h", package_name))
        .display()
        .to_string();

    // println!("{}", target_dir().to_str().unwrap());

    let config = Config {
        language: cbindgen::Language::C,
        // namespace: Some(String::from("ffi")),
        export: cbindgen::ExportConfig {
            // include: vec!["JSCValue".to_string()],
            ..Default::default()
        },
        parse: cbindgen::ParseConfig {
            // parse_deps: true,
            // include: Some(vec!["esperanto_javascriptcore".to_string()]),
            ..Default::default()
        },
        function: cbindgen::FunctionConfig {
            // swift_name_macro: Some("CF_SWIFT_NAME".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}

/// Find the location of the `target/` directory. Note that this may be
/// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR`
/// variable.
fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_BUILD_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
