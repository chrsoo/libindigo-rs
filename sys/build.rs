use std::fs::{self, File};
use std::io::{prelude::*, Result};
use std::env;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use rev_buf_reader::RevBufReader;
use semver::Version;
use regex::Regex;

fn join_paths(base: &PathBuf, path: &str) -> String {
    let p = base.join(path);
    let s = p.to_str().expect("path not found");
    String::from(s)
}

/// Compile a c source file to a binary object file.
fn compile(c_file: PathBuf, include_path: &PathBuf) -> PathBuf {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut o_file = out_path.join("indigo").join(c_file.file_name().unwrap());
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

    println!("{}", String::from_utf8(output.stdout).unwrap());
    println!();
    println!("{}", String::from_utf8(output.stderr).unwrap());

    if !output.status.success() {
        panic!("could not compile object file");
    }
    o_file
}

/// Parse the INDIGO Makefile and extract version and build numbers.
fn _parse_indigo_version<'a>(indigo_root: &PathBuf) -> std::io::Result<Version> {
    // TODO extract build from indigo_config.h - what about version?
    // let m = indigo_root.join("indigo_libs/indigo/indigo_config.h");
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
fn _ensure_build_version(indigo_root: &PathBuf) -> std::io::Result<()>{
    let indigo_version = _parse_indigo_version(indigo_root)?;

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

fn _taillog(file: &str, limit: usize, err: bool) {
    // let file = root.join(file);
    let file = File::open(file).unwrap();

    // https://stackoverflow.com/a/74282737/51016
    let buf = RevBufReader::new(file);
    let lines: Vec<String> = buf.lines().take(limit).map(|l| l.expect("Could not parse line")).collect();

    for line in lines {
        if err { eprintln!("{line}") } else { println!("{line}") } // ugly...
    }
}

fn main() -> std::io::Result<()> {

    let include = if let Ok(envar) = env::var("INDIGO_SOURCE") { // build from source dir
        let indigo_source = PathBuf::from(envar);
        let indigo_source = indigo_source.canonicalize().expect("cannot canonicalize path");
        build_indigo(&indigo_source)?
    } else if let Ok(submodule) = Path::new("externals/indigo").canonicalize() { // build from submodule
        build_indigo(&submodule)?
    } else if Path::new("/usr/include/indigo/indigo_version.h").is_file() { // use system libraries
        let include_dir = PathBuf::from("/usr/include");

        let lib_dir = Path::new("/usr/lib");
        output_cargo_link_search(&lib_dir);
        let libindigo = lib_dir.join("libindigo.a");
        println!("cargo:rustc-link-lib={}", libindigo.to_str().expect("could not find /usr/lib/libindigo.a"));

        include_dir
    } else { // last ditch effort, checkout and build submodule
        let outcome = std::process::Command::new("git")
            .arg("submodule")
            .arg("update")
            .arg("--init")
            .arg("--recursive")
            // .current_dir(&indigo_root.join("indigo_libs"))
            // .stdout(stdin)
            // .stderr(stderr)
            .status()
            .expect("could not spawn `make`")
            ;
        if !outcome.success() {
            panic!("could not checkout externals/indigo");
        }

        let submodule = Path::new("externals/indigo").canonicalize()?;
        build_indigo(&submodule)?
    };

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .clang_arg(format!("-I{}", include.to_str().expect("path not found")))
        .header(join_paths(&include, "indigo/indigo_names.h"))
        .header(join_paths(&include, "indigo/indigo_bus.h"))
        .header(join_paths(&include, "indigo/indigo_client.h"))
        .header(join_paths(&include, "indigo/indigo_driver.h"))
        .header(join_paths(&include, "indigo/indigo_config.h"))
        .header(join_paths(&include, "indigo/indigo_timer.h"))
        .header(join_paths(&include, "indigo/indigo_token.h"))
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

// Tell cargo to look for shared (system) libraries in `/usr/lib`
// search the `${indigo_root}/indigo/build/libs` for native (static) librarires
fn output_cargo_link_search(lib_dir: &Path) {
    let lib_dir = lib_dir.to_str().expect("could not find library dir");
    println!("cargo:rustc-link-search={}", lib_dir);
    println!("cargo:rustc-link-search=native={}", lib_dir);
}

/// compile source in rust
fn build_indigo(indigo_root: &PathBuf) -> Result<PathBuf> {
    // ensure_build_version(&indigo_root)?;

    let source_dir = indigo_root.join("indigo_libs");

    fs::read_dir(&source_dir)?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|p| p.is_file() && p.ends_with(".c"))
        .map(|c| compile(c, &source_dir))
        .for_each(drop) // noop
        ;

    let indigo_libs = source_dir.join("indigo_libs");
    output_cargo_link_search(&indigo_libs);

    let libindgo = source_dir.join("build/lib/libindigo.a");
    println!("cargo:rustc-link-lib={}", libindgo.to_str().expect("could not find build/lib/libindigo.a"));

    Ok(indigo_libs)
}

/// compile source code using external make
fn _make_indigo(indigo_root: &PathBuf) -> Result<()>{
    _ensure_build_version(&indigo_root)?;

    let log = File::create("libindigo-sys.log").unwrap();
    let err = File::create("libindigo-sys.err").unwrap();
    let stdin = std::process::Stdio::from(log);
    let stderr = std::process::Stdio::from(err);

    // run make and write to build.out - panic if it fails
    let status = std::process::Command::new("make")
        .arg("all")
        .current_dir(&indigo_root.join("indigo_libs"))
        .stdout(stdin)
        .stderr(stderr)
        .status()
        .expect("could not spawn `make`");

    if !status.success() {
        println!("libindigo-sys.log:\n...");
        _taillog("libindigo-sys.log", 10, false);
        println!("---");
        eprintln!("libindigo-sys.err:\n...");
        _taillog("libindigo-sys.err", 10, true);
        eprintln!("---");
        panic!("could not make {}", indigo_root.to_str().expect("indigo root not found"));
    }

    Ok(())
}
