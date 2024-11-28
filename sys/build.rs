use core::str;
use std::ffi::OsString;
use std::fmt::{Debug, Display};
use std::fs::{self, File};
use std::io::{self, prelude::*, ErrorKind, Result};
use std::env;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use rev_buf_reader::RevBufReader;
use semver::Version;
use regex::Regex;

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

        let lib_dir_search = lib_dir.to_str().expect("could not find /usr/lib");
        println!("cargo:rustc-link-search={}", lib_dir_search);
        println!("cargo:rustc-link-search=native={}", lib_dir_search);

        let libindigo = lib_dir.join("libindigo.a");
        println!("cargo:rustc-link-lib=static={}", libindigo.to_str().expect("could not find /usr/lib/libindigo.a"));

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
            .expect("could not spawn `git`")
            ;
        if !outcome.success() {
            panic!("could not checkout git submodule externals/indigo");
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

fn join_paths(base: &PathBuf, path: &str) -> String {
    let p = base.join(path);
    let s = p.to_str().expect("path not found");
    String::from(s)
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

/// Assumes success until first failure.
struct Compilation {
    source: PathBuf,
    target: PathBuf,
}

impl Display for Compilation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.source.to_string_lossy())?;
        f.write_str("->")?;
        f.write_str(&self.target.to_string_lossy())?;
        // writeln!(f)?;
        // f.write_str(str::from_utf8(&self.output.stdout).expect("could not write stdout"))?;
        // writeln!(f)?;
        // f.write_str(str::from_utf8(&self.output.stderr).expect("could not write stderr"))?;
        // writeln!(f)?;
        // writeln!(f, "compilation exited with status: '{}'", self.output.status)?;
        Ok(())
    }
}

impl Debug for Compilation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Compilation").field("source", &self.source).field("target", &self.target).finish()
    }
}

impl std::error::Error for Compilation { }
struct Build<'a> {
    source: &'a PathBuf,
    targets: Vec<PathBuf>,
}

impl<'a> Build<'a> {
    fn new(source: &'a PathBuf) -> Self {
        Build {
            source,
            targets: Vec::new(),
        }
    }

    fn source_dir(&self) -> PathBuf{
        self.source.join("indigo_libs")
    }

    fn source_paths(&self) -> Vec<PathBuf> {

        let mut source_paths = Vec::new();

        let source_dir = self.source_dir();
        source_paths.push(source_dir.join("indigo_base64.c"));
        source_paths.push(source_dir.join("indigo_bus.c"));
        source_paths.push(source_dir.join("indigo_bus.c"));
        source_paths.push(source_dir.join("indigo_client_xml.c"));
        source_paths.push(source_dir.join("indigo_io.c"));
        source_paths.push(source_dir.join("indigo_service_discovery.c"));
        source_paths.push(source_dir.join("indigo_token.c"));
        source_paths.push(source_dir.join("indigo_xml.c"));
        source_paths.push(source_dir.join("indigo_md5.c"));
        source_paths.push(source_dir.join("indigo_fits.c"));
        source_paths.push(source_dir.join("indigo_version.c"));

        source_paths
    }

    // TODO recurse into all subdirs
    fn include_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        let source_dir = self.source_dir();
        paths.push(source_dir.clone());

        fs::read_dir(&source_dir.join("externals")).unwrap().into_iter()
            .filter_map(|r| if r.is_ok() { Some(r.unwrap().path()) } else { None })
            .filter(|p| p.is_dir())
            .for_each(|d| paths.push(d))
            ;
        paths.push(source_dir.join("externals").join("libtiff").join("libtiff"));
        paths
    }

    fn include_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();
        for path in self.include_paths() {
            args.push(OsString::from("-I"));
            args.push(path.into_os_string());
        }
        args
    }

    fn target(name: &PathBuf) -> PathBuf {
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        out_path.join(name.file_name().unwrap())
    }

    /// Compile a c source file to a binary object file.
    fn compile(mut self, source: &PathBuf) -> Result<Self> {
        assert!(source.exists(), "source file does not exist");
        eprint!("{:?}", source);
        // assert!(source.ends_with(".c"), "source file does not have a .c suffix");

        let os = match std::env::consts::OS {
            "macos" =>  "-DINDIGO_MACOS",
            "linux" =>  "-DINDIGO_LINUX",
            "windows" =>  "-DINDIGO_WINDOWS",
            _   => panic!("unsupported OS: '{}'", std::env::consts::OS)
        };

        let target = Build::target(&source).with_extension("o");
        let output = std::process::Command::new("clang")
            .arg(os)
            .args(self.include_args())
            .arg("-c")
            .arg("-o")
            .arg(&target)
            .arg(&source)
            .output()
            .expect("could not spawn `clang`")
            ;

        // assert!(target.exists(), "target files was not comppiled");

        let comp = Compilation { source: source.clone(), target };

        if output.status.success() {
            eprintln!("compiled {:?}", comp.source);
            self.targets.push(comp.target);
            Ok(self)
        } else {
            println!("{}", str::from_utf8(&output.stdout).expect("could not print stdout"));
            eprintln!("{}", str::from_utf8(&output.stderr).expect("could not print stdout"));
            let msg = format!("could not compile '{:?}'", comp.source);
            Err(io::Error::new(ErrorKind::Other, msg))
        }
    }

    fn link(&self) -> Result<PathBuf> {
        assert!(self.targets.len() > 0, "target object files missing");

        let lib_path = Build::target(&PathBuf::from("libindigo").with_extension("a"));
        let output = std::process::Command::new("ar")
            .arg("rcs")
            .arg(&lib_path)
            .args(self.targets.as_slice())
            .output()?
            ;
        if output.status.success() {
            Ok(lib_path)
        } else {
            println!("{}", str::from_utf8(&output.stdout).expect("could not print stdout"));
            eprintln!("{}", str::from_utf8(&output.stderr).expect("could not print stdout"));
            let msg = format!("could not link targets: command exited with {:?}", output.status);
            Err(io::Error::new(ErrorKind::Other, msg))
        }
    }
}

/// compile source in rust and return the include dir containing the INDIGO header files.
fn build_indigo(indigo_root: &PathBuf) -> Result<PathBuf> {
    // ensure_build_version(&indigo_root)?;

    let source_dir = indigo_root.join("indigo_libs");
    let build = Build::new(&indigo_root);

    // eprintln!("INCLUDE='{:?}'", build.include_args());

    let mut libindigo = build.source_paths().iter()
        .try_fold(build, |build, c| build.compile(c))?
        .link()?
        ;

    libindigo.set_extension("");
    // let libindigo = libindigo.to_str().expect("could not convert OStr");
    let libindigo = libindigo.file_name().unwrap().to_str().expect("could not convert OStr");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("could not find OUT_DIR"));
    println!("cargo:rustc-link-search={}", out_dir.to_str().expect("could not find OUT_DIR"));
    println!("cargo:rustc-link-lib={}", libindigo);

    Ok(source_dir)
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
