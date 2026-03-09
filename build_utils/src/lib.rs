use regex::Regex;
use semver::Version;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;

/// Extract INDIGO version and build number from the INDIGO Makefile.
///
/// The Makefile contains lines like:
/// ```makefile
/// INDIGO_VERSION = 2.0
/// INDIGO_BUILD = 300
/// ```
///
/// This function parses these values and returns them as a semver Version
/// where major.minor come from INDIGO_VERSION and patch comes from INDIGO_BUILD.
///
/// # Arguments
/// * `indigo_root` - Path to the INDIGO source root directory
///
/// # Returns
/// * `Ok(Version)` - The parsed version (e.g., 2.0.300)
/// * `Err(io::Error)` - If the Makefile cannot be read or parsed
pub fn parse_indigo_version(indigo_root: &Path) -> Result<Version> {
    let makefile = indigo_root.join("Makefile");

    let mut version = Version::new(0, 0, 0);
    let re_version = Regex::new(r"^INDIGO_VERSION\s*=\s*(\d+)\.(\d+)\s*$").unwrap();
    let re_build = Regex::new(r"^INDIGO_BUILD\s*=\s*(\d+)\s*$").unwrap();

    let file = File::open(&makefile)?;
    let buf = BufReader::new(file);

    for line in buf.lines() {
        let s = line?;
        if let Some(v) = re_version.captures(&s) {
            version.major = v[1].parse::<u64>().unwrap();
            version.minor = v[2].parse::<u64>().unwrap();
        } else if let Some(v) = re_build.captures(&s) {
            version.patch = v[1].parse::<u64>().unwrap();
        }
    }

    if version.major == 0 && version.minor == 0 && version.patch == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Could not parse INDIGO version from {}", makefile.display()),
        ));
    }

    Ok(version)
}

/// Format INDIGO version as SemVer build metadata.
///
/// Converts a version like 2.0.300 to "INDIGO.2.0.300"
/// which can be appended to a crate version as build metadata.
///
/// # Arguments
/// * `version` - The INDIGO version
///
/// # Returns
/// * String in format "INDIGO.{major}.{minor}.{patch}"
pub fn format_indigo_build_metadata(version: &Version) -> String {
    format!(
        "INDIGO.{}.{}.{}",
        version.major, version.minor, version.patch
    )
}

/// Generate Rust constants for INDIGO version information.
///
/// Creates constant definitions that can be included in generated code.
///
/// # Arguments
/// * `version` - The INDIGO version
///
/// # Returns
/// * String containing Rust constant definitions
pub fn generate_version_constants(version: &Version) -> String {
    format!(
        r#"/// INDIGO library version (major component)
pub const INDIGO_VERSION_MAJOR: u32 = {};

/// INDIGO library version (minor component)
pub const INDIGO_VERSION_MINOR: u32 = {};

/// INDIGO library build number
pub const INDIGO_BUILD: u32 = {};

/// INDIGO library version string (e.g., "2.0.300")
pub const INDIGO_VERSION: &str = "{}.{}.{}";
"#,
        version.major, version.minor, version.patch, version.major, version.minor, version.patch
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_build_metadata() {
        let version = Version::new(2, 0, 300);
        assert_eq!(format_indigo_build_metadata(&version), "INDIGO.2.0.300");
    }

    #[test]
    fn test_generate_version_constants() {
        let version = Version::new(2, 0, 300);
        let constants = generate_version_constants(&version);
        assert!(constants.contains("pub const INDIGO_VERSION_MAJOR: u32 = 2;"));
        assert!(constants.contains("pub const INDIGO_VERSION_MINOR: u32 = 0;"));
        assert!(constants.contains("pub const INDIGO_BUILD: u32 = 300;"));
        assert!(constants.contains(r#"pub const INDIGO_VERSION: &str = "2.0.300";"#));
    }
}
