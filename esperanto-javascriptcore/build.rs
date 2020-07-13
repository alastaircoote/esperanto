extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let headers = vec![
        "JSBase.h",
        "JSContextRef.h",
        "JSStringRef.h",
        "JSObjectRef.h",
        "JSTypedArray.h",
        "JSValueRef.h",
    ];

    println!("cargo:rustc-link-lib=javascriptcore");

    let mut builder = bindgen::Builder::default();

    for header in headers {
        println!("cargo:rerun-if-changed=headers/{}", header);
        builder = builder.header(format!("headers/{}", header));
    }
    // The input header we would like to generate
    // bindings for.
    let bindings = builder
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
