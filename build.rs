use core::str;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, Write};
use std::{env, io};
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

/// used for cloning the INDIGO git repository when source is retrieved from a crate
const INDIGO_GIT_REPOSITORY: &str = "https://github.com/indigo-astronomy/indigo";

/// used for building INDIGO from source when checked out
const GIT_SUBMODULE: &str = "sys/externals/indigo";

// used to detect Linux system libraries
// const LINUX_INDIGO_VERSION_HEADER: &str = "/usr/include/indigo/indigo_version.h";
// const LINUX_INCLUDE: &str = "/usr/include";
// const LINUX_LIB: &str = "/usr/lib";

// #define INFO_PROPERTY_NAME										"INFO"

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct IndigoName {
    name: String,
    value: String,
}

impl Display for IndigoName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"pub const {}: &str = "{}";"#, self.name, self.value)
    }
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^#define\s+(?<name>\w+)_NAME\s+"(?<value>.+)"\s*$"#).unwrap());
fn main() -> std::io::Result<()> {

    let submodule = Path::new(GIT_SUBMODULE);
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

    let names_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("name.rs");
    let mut names_file = File::create(names_path).expect("props.rs file creation failed");
    for prop in names {
        writeln!(names_file, r#"pub const {}: &str = "{}";"#, prop.0, prop.1)?;
    }

    Ok(())
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
    Path::new(GIT_SUBMODULE).canonicalize()
}
