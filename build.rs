use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    let src = [
        "vendor/src/CDtoc.cpp",
        "vendor/src/arrays.cpp",
        "vendor/src/compress.cpp",
        "vendor/src/extracts.cpp",
        "vendor/src/iconv.cpp",
        "vendor/src/id3v2.cpp",
        "vendor/src/main.cpp",
        "vendor/src/metalist.cpp",
        "vendor/src/parsley.cpp",
        "vendor/src/sha1.cpp",
        "vendor/src/util.cpp",
        "vendor/src/uuid.cpp",
    ];

    cc::Build::new()
        .cpp(true)
        .flag("-Wno-everything")
        .files(src.iter())
        .define("PACKAGE_VERSION", "\"20211003.181952.0\"")
        .define("BUILD_INFO", "\"90ad66d789bf55aa3738e3d3f7e21436ac04b59c\"")
        .compile("atomicparsley");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .blocklist_function("main")
        .clang_arg("-xc++")
        .clang_arg("-std=c++11")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let target_os = env::var("CARGO_CFG_TARGET_OS");
    if let Ok("macos") = target_os.as_deref() {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=IOKit");
    };
}
