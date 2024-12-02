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

/// used for building INDIGO from source when defined
const INDIGO_SOURCE_ENVAR: &str = "INDIGO_SOURCE";

/// used for building INDIGO from source when checked out
const GIT_SUBMODULE: &str = "externals/indigo";

/// used to detect Linux system libraries
const LINUX_INDIGO_VERSION_HEADER: &str = "/usr/include/indigo/indigo_version.h";
const LINUX_INCLUDE: &str = "/usr/include";
const LINUX_LIB: &str = "/usr/lib";


fn main() -> std::io::Result<()> {

    let include = if let Ok(envar) = env::var(INDIGO_SOURCE_ENVAR) {
        eprintln!("building INDIGO from source dir {:?} found in {:?}", envar, INDIGO_SOURCE_ENVAR);
        let indigo_source = PathBuf::from(envar);
        let indigo_source = indigo_source.canonicalize().expect("cannot canonicalize path");
        make_indigo(&indigo_source)?

    } else if let Ok(submodule) = Path::new(GIT_SUBMODULE).canonicalize() {
        eprintln!("building INDIGO from submodule {:?}", GIT_SUBMODULE);
        make_indigo(&submodule)?

    } else if Path::new(LINUX_INDIGO_VERSION_HEADER).is_file() {
        eprintln!("using system libraries");
        let lib_dir = Path::new(LINUX_LIB).to_str().expect("could not find /usr/lib");
        println!("cargo:rustc-link-search=native={}", lib_dir);
        println!("cargo:rustc-link-lib=libindigo");
        PathBuf::from(LINUX_INCLUDE)

    } else { // last ditch effort
        eprintln!("checking out git submodule");
        let submodule = init_indigo_submodule()?;
        eprintln!("building INDIGO from submodule {}", submodule.to_str().expect("expected git submodule"));
        make_indigo(&submodule)?
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

fn init_indigo_submodule() -> Result<PathBuf>{
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
    Path::new(GIT_SUBMODULE).canonicalize()
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

fn taillog(file: &str, limit: usize, err: bool) {
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
struct _Compilation {
    source: PathBuf,
    target: PathBuf,
}

impl Display for _Compilation {
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

impl Debug for _Compilation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Compilation").field("source", &self.source).field("target", &self.target).finish()
    }
}

impl std::error::Error for _Compilation { }
struct _Build<'a> {
    source: &'a PathBuf,
    targets: Vec<PathBuf>,
}

macro_rules! args {
    ($a:expr) => {
        {
            let mut v = Vec::new();
            $a.split(" ").map(|arg| OsString::from(arg)).for_each(|a| v.push(a));
            v
        }
    }
}

// macro_rules! args {
//     ($($a:expr),*) => {
//         [
//             $(
//                 OsStr::new($a),
//             )*
//         ]
//     }
// }

impl<'a> _Build<'a> {
    fn _new(source: &'a PathBuf) -> Self {
        _Build {
            source,
            targets: Vec::new(),
        }
    }

    fn _source_dir(&self) -> PathBuf{
        self.source.join("indigo_libs")
    }

    fn _source_paths(&self) -> Vec<PathBuf> {

        let mut source_paths = Vec::new();

        let source_dir = self._source_dir();
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
    fn _include_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        let source_dir = self._source_dir();
        paths.push(source_dir.clone());

        fs::read_dir(&source_dir.join("externals")).unwrap().into_iter()
            .filter_map(|r| if r.is_ok() { Some(r.unwrap().path()) } else { None })
            .filter(|p| p.is_dir())
            .for_each(|d| paths.push(d))
            ;
        paths.push(source_dir.join("externals").join("libtiff").join("libtiff"));
        paths
    }

    fn _include_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();
        for path in self._include_paths() {
            args.push(OsString::from("-I"));
            args.push(path.into_os_string());
        }
        args
    }

    fn _target(name: &PathBuf) -> PathBuf {
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        out_path.join(name.file_name().unwrap())
    }

    /// Compile a c source file to a binary object file.
    fn _compile(mut self, source: &PathBuf) -> Result<Self> {
        assert!(source.exists(), "source file does not exist");
        // assert!(source.ends_with(".c"), "source file does not have a .c suffix");

        /*
                INDIGO_CUDA =
                ifeq ($(findstring arm64e,$(shell file $(CC))),arm64e)
                        MAC_ARCH = -arch x86_64 -arch arm64
                else
                        MAC_ARCH = -arch x86_64
                endif
         */
        // let arch_args = match std::env::consts::ARCH {
        //     // * `"x86"`
        //     // * `"x86_64"`
        //     // * `"arm"`
        //     // * `"aarch64"`

        //     ""  => { () }
        //     _   => panic!("unsupported architecture: '{}'", std::env::consts::OS)
        // };

        let os_args = match std::env::consts::OS {
            "macos" =>  args!("-g -arch x86_64 -arch arm64 -mmacosx-version-min=10.10 -fPIC -O3 -std=gnu11 -DINDIGO_MACOS -Duint=unsigned"),
            "linux" =>  args!("-DINDIGO_LINUX"),
            "windows" =>  args!("-DINDIGO_WINDOWS"),
            _   => panic!("unsupported OS: '{}'", std::env::consts::OS)
        };

        let target = _Build::_target(&source).with_extension("o");
        // -mmacosx-version-min=10.10 -fPIC -O3
        let output = std::process::Command::new("/usr/bin/clang")
            .args(os_args)
            .args(self._include_args())
            .arg("-c")
            .arg("-o")
            .arg(&target)
            .arg(&source)
            .output()
            .expect("could not spawn `clang`")
            ;

        let comp = _Compilation { source: source.clone(), target };

        if output.status.success() {
            assert!(comp.target.exists(), "target files was not comppiled");
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

    fn _link(&self) -> Result<PathBuf> {
        assert!(self.targets.len() > 0, "target object files missing");

        let lib_path = _Build::_target(&PathBuf::from("libindigo").with_extension("a"));

        // MACOS: ar_flags = "-static -o"
        // LINUX: ar_flags = "-rv"

        let _ld_args = args!("-arch x86_64 -arch arm64 -headerpad_max_install_names -framework Cocoa -mmacosx-version-min=10.10 -framework CoreFoundation -framework IOKit -framework ImageCaptureCore -framework IOBluetooth -lobjc");
        let output = std::process::Command::new("/usr/bin/libtool")
            // .arg("link")
            .arg("-static")
            .arg("-o")
            .arg(&lib_path)
            // .args(ld_args)
            .args(self.targets.as_slice())
            .output()?
            ;
        // let output = std::process::Command::new("ar")
        //     .arg("rcs")
        //     .arg(&lib_path)
        //     .args(self.targets.as_slice())
        //     .output()?
        //     ;
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
fn _build_indigo(indigo_root: &PathBuf) -> Result<PathBuf> {
    // ensure_build_version(&indigo_root)?;

    let source_dir = indigo_root.join("indigo_libs");
    let build = _Build::_new(&indigo_root);

    // eprintln!("INCLUDE='{:?}'", build.include_args());

    let libindigo = build._source_paths().iter()
        .try_fold(build, |build, c| build._compile(c))?
        ._link()?
        ;

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR envar not defined"));
    out_dir.canonicalize().expect("could not find OUT_DIR");
    println!("cargo:rustc-link-search={}", out_dir.to_str().expect("could not conver OStr"));

    assert_eq!(libindigo.file_name().unwrap(), "libindigo.a", "`indigo` library filename must be `libindigo.a`");
    println!("cargo:rustc-link-lib=indigo");

    Ok(source_dir)
}

/// compile source code using external make
fn make_indigo(indigo_root: &PathBuf) -> Result<PathBuf>{
    // _ensure_build_version(&indigo_root)?;

    let log = File::create("libindigo-sys.log").unwrap();
    let err = File::create("libindigo-sys.err").unwrap();
    let stdin = std::process::Stdio::from(log);
    let stderr = std::process::Stdio::from(err);

    // run make and write to build.out - panic if it fails
    let status = std::process::Command::new("make")
        .arg("all")
        .current_dir(indigo_root)
        .stdout(stdin)
        .stderr(stderr)
        .status()
        .expect("could not spawn `make`");

    if !status.success() {
        println!("libindigo-sys.log:\n...");
        taillog("libindigo-sys.log", 10, false);
        println!("---");
        eprintln!("libindigo-sys.err:\n...");
        taillog("libindigo-sys.err", 10, true);
        eprintln!("---");
        panic!("could not make {}", indigo_root.to_str().expect("indigo root not found"));
    }

    let build_dir = indigo_root.join("build/lib/").canonicalize().expect("could not find build/lib/ dir");
    assert_eq!(build_dir.join("libindigo.a").file_name().unwrap(), "libindigo.a", "make did not build libindigo.a");

    println!("cargo:rustc-link-search={}", build_dir.to_str().expect("could not create string from OsStr"));
    println!("cargo:rustc-link-lib=indigo");

    Ok(indigo_root.join("indigo_libs")) // return include dir containing the header files
}
