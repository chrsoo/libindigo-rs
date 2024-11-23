use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::io::BufReader;
use std::path::PathBuf;
use rev_buf_reader::RevBufReader;
use semver::Version;
use regex::Regex;

fn join_paths(base: &PathBuf, path: &str) -> String {
    let p = base.join(path);
    let s = p.to_str().expect("path not found");
    String::from(s)
}

// TODO find all .c source files in dir
fn _get_sources(src_dir: &PathBuf) -> Vec<PathBuf> {
    vec![src_dir.join("indigo_bus.c")]
}

/// Compile a c source file to a binary object file.
fn _compile(c_file: &PathBuf, include_path: &PathBuf) -> PathBuf {
    let mut o_file = c_file.clone();
    o_file.set_extension("o");

    let output = std::process::Command::new("clang")
        .arg("-I")
        .arg(include_path)
        .arg("-c")
        .arg("-o")
        .arg(&o_file)
        .arg(c_file)
        .output()
        .expect("could not spawn `clang`");

    println!("{}", String::from_utf8(output.stderr).unwrap());
    println!();
    println!("{}", String::from_utf8(output.stdout).unwrap());

    if !output.status.success() {
        panic!("could not compile object file");
    }
    o_file
}

/// Parse the INDIGO Makefile and extract version and build numbers.
fn parse_indigo_version<'a>(indigo_root: &PathBuf) -> std::io::Result<Version> {
    let m = indigo_root.join("Makefile");

    let mut version = Version::new(0, 0, 0);
    let re_version = Regex::new(r"^INDIGO_VERSION *= *(\d+)\.(\d+) *$").unwrap();
    let re_build = Regex::new(r"^INDIGO_BUILD *= *(\d+) *$").unwrap();

    if let Ok(file) = File::open(m) {
        let buf = BufReader::new(file);
        for l in buf.lines() {
            let s = l.unwrap();
            if let Some(v) = re_version.captures(&s) {
                version.major = v[1].parse::<u64>().unwrap();
                version.minor = v[2].parse::<u64>().unwrap();
            } else if let Some(v) = re_build.captures(&s) {
                version.patch = v[1].parse::<u64>().unwrap();
            }
        }
    } else {
        panic!("could not open INDIGO Makefile");
    }
    Ok(version)
}

/// Ensure that the Cargo package version string includes the correct INDIGO build number.
fn ensure_build_version(indigo_root: &PathBuf) -> std::io::Result<()>{
    let indigo_version = parse_indigo_version(indigo_root)?;

    if let Ok(v) = env::var("CARGO_PKG_VERSION") {
        if let Ok(pkg_version) = Version::parse(&v) {
            assert_eq!(indigo_version, pkg_version,
                "the Makefile INDIGO_VERSION and INDIGO_BUILD (left) does not match the cargo package version (right)")
        } else {
            panic!("could not parse package version `{}`", v);
        }
    }
    Ok(())
}

fn taillog(file: &str, limit: usize) -> Vec<String> {
    let file = File::create(file).unwrap();
    // https://stackoverflow.com/a/74282737/51016
    let buf = RevBufReader::new(file);
    buf.lines().take(limit).map(|l| l.expect("Could not parse line")).collect()
}

fn main() -> std::io::Result<()> {
    let indigo_root: PathBuf = if env::var("INDIGO_ROOT").is_ok() {
        PathBuf::from(env::var("INDIGO_ROOT").expect("path defined by INDIGO_ROOT envar not found"))
    } else {
        PathBuf::from("externals/indigo")
    };
    let indigo_root = indigo_root.canonicalize().expect("cannot canonicalize path");
    ensure_build_version(&indigo_root)?;

    // lib source directory in `indigo/indigo_libs`
    let indigo_libs: PathBuf = indigo_root.join("indigo_libs");

    let log = File::create("libindigo-sys.log").unwrap();
    let err = File::create("libindigo-sys.err").unwrap();
    let stdin = std::process::Stdio::from(log);
    let stderr = std::process::Stdio::from(err);

    // run make and write to build.out - panic if it fails
    let status = std::process::Command::new("make")
        .arg("all")
        .current_dir(&indigo_root)
        .stdout(stdin)
        .stderr(stderr)
        .status()
        .expect("could not spawn `make`");

    if !status.success() {
        println!("libindigo-sys.log:\n...");
        taillog("libindigo-sys.log", 10);
        eprintln!("libindigo-sys.err:\n...");
        taillog("libindigo-sys.err", 10);
        panic!("could not make {}", indigo_root.to_str().expect("indigo root not found"));
    }

    let indigo_build = indigo_root.join("build");
    let indigo_build_libs = indigo_build.join("lib");

    // Tell cargo to look for shared (system) libraries in `indigo/build/libs`
    println!("cargo:rustc-link-search={}", indigo_build_libs.to_str().expect("path not found"));

    // search the `indigo/build/libs` for native (static) librarires
    println!("cargo::rustc-link-search=native={}", indigo_build_libs.to_str().expect("path not found"));

    // bind with the `libindigo.a` static library in the `indigo/build/libs` directory
    println!("cargo:rustc-link-lib=indigo");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .clang_arg(format!("-I{}", indigo_libs.to_str().expect("path not found")))
        .header(join_paths(&indigo_libs, "indigo/indigo_names.h"))
        .header(join_paths(&indigo_libs, "indigo/indigo_bus.h"))
        .header(join_paths(&indigo_libs, "indigo/indigo_client.h"))
        .header(join_paths(&indigo_libs, "indigo/indigo_driver.h"))
        .header(join_paths(&indigo_libs, "indigo/indigo_config.h"))
        .header(join_paths(&indigo_libs, "indigo/indigo_timer.h"))
        .header(join_paths(&indigo_libs, "indigo/indigo_token.h"))
        .no_copy(".*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
