use core::str;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, Write};
use std::{env, io};
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

// TODO source INDIGO version and build information from INDIGO source code
// NOTE create aux crate for build related functionality shared with the sys crate

/// used for cloning the INDIGO git repository when source is retrieved from a crate
const INDIGO_GIT_REPOSITORY: &str = "https://github.com/indigo-astronomy/indigo";

/// used for building INDIGO from source when checked out
const INDIGO_GIT_SUBMODULE: &str = "sys/externals/indigo";

// used to detect Linux system libraries
// const LINUX_INDIGO_VERSION_HEADER: &str = "/usr/include/indigo/indigo_version.h";
// const LINUX_INCLUDE: &str = "/usr/include";
// const LINUX_LIB: &str = "/usr/lib";

// #define INFO_PROPERTY_NAME										"INFO"

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^#define\s+(?<name>\w+)_NAME\s+"(?<value>.+)"\s*$"#).unwrap());
fn main() -> std::io::Result<()> {

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let submodule = Path::new(INDIGO_GIT_SUBMODULE);
    if !submodule.exists() {
        init_indigo_submodule()?;
    }

    let mut names = BTreeMap::new();

    let include = submodule.join("indigo_libs/indigo/indigo_names.h").canonicalize()?;
    for line in read_lines(include)? {
        if let Some(cap) = RE.captures(&line?) {
            names.insert(cap["name"].to_string(), cap["value"].to_string());
        }
    }

    let names_path = out_dir.join("name.rs");
    let mut names_file = File::create(names_path).expect("props.rs file creation failed");
    for prop in names {
        writeln!(names_file, r#"pub const {}: &str = "{}";"#, prop.0, prop.1)?;
    }

    let include = submodule.join("indigo_libs").canonicalize()?;
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", include.to_str().expect("path not found")))
        .header(join_paths(&include, "indigo/indigo_bus.h"))
        .derive_debug(true)
        .allowlist_item("indigo_device_interface")
        .bitfield_enum("indigo_device_interface")
        .prepend_enum_name(false)
        .translate_enum_integer_types(true)
        .generate_cstr(true)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("interface.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}

fn join_paths(base: &Path, path: &str) -> String {
    let p = base.join(path);
    let s = p.to_str().expect("path not found");
    String::from(s)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn init_indigo_submodule() -> std::io::Result<PathBuf>{
    // check if we are in a crate package or if this a git repository
    let outcome = if PathBuf::from(".git").exists() {
        std::process::Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .arg("--recursive")
        .status()
        .expect("could not spawn `git`")
    } else {
        std::process::Command::new("git")
        .arg("clone")
        .arg(INDIGO_GIT_REPOSITORY)
        .arg("externals/indigo")
        .status()
        .expect("could not spawn `git`")
    };

    if !outcome.success() {
        panic!("could not clone or checkout git submodule externals/indigo");
    }
    Path::new(INDIGO_GIT_SUBMODULE).canonicalize()
}
