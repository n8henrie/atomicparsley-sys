use chrono::prelude::*;
use git2::{Diff, Repository, RepositoryState};
use std::{env, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
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
    let repo = Repository::open("./vendor")
        .expect("Unable to open repo from ./vendor");
    let commit = repo.head()?.peel_to_commit()?;
    let hash = commit.id().to_string();
    let timestamp = {
        let time = commit.time();
        time.seconds()
            - time.offset_minutes() as i64 * 60 * {
                match time.sign() {
                    '-' => -1,
                    '+' => 1,
                    s => panic!("sign: {}", s),
                }
            }
    };

    let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);
    let utc_datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    let formatted_timestamp = utc_datetime.format("%Y%m%d.%H%M%S.0");

    let patch = r#"
diff --git a/src/main.cpp b/src/main.cpp
index 002fa32..5c06fcc 100644
--- a/src/main.cpp
+++ b/src/main.cpp
@@ -4449,7 +4449,7 @@ int wmain(int argc, wchar_t *arguments[]) {
 
 #ifdef __MINGW32__
 
-int main() {
+int mingw_main() {
   int argc;
   wchar_t **argv = CommandLineToArgvW(GetCommandLineW(), &argc);
   return wmain(argc, argv);
@@ -4459,7 +4459,7 @@ int main() {
 
 #else // defined __CYGWIN__
 
-int main(int argc, char *argv[]) {
+int cygwin_main(int argc, char *argv[]) {
   size_t name_len = strlen(argv[0]);
   if (name_len >= 5 && (strcmp(argv[0] + (name_len - 5), "-utf8") == 0 ||
                         strcmp(argv[0] + (name_len - 5), "-UTF8") == 0)) {
@@ -4474,7 +4474,7 @@ int main(int argc, char *argv[]) {
 
 #else
 
-int main(int argc, char *argv[]) { return real_main(argc, argv); }
+int other_main(int argc, char *argv[]) { return real_main(argc, argv); }
 
 #endif
 
"#;

    let diff_from_patch = Diff::from_buffer(patch.as_bytes())?;

    if let RepositoryState::Clean = repo.state() {
        repo.apply(&diff_from_patch, git2::ApplyLocation::WorkDir, None)?;
    };

    cc::Build::new()
        .cpp(true)
        .flag("-Wno-everything")
        .files(src.iter())
        .define(
            "PACKAGE_VERSION",
            format!("\"{}\"", formatted_timestamp).as_ref(),
        )
        .define("BUILD_INFO", format!("\"{}\"", hash).as_ref())
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
    Ok(())
}
