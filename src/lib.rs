use crates_io_api::{Error, SyncClient};
use std::fmt;
use std::fs;
use std::path::Path;

/// Simple semver representation (MAJOR.MINOR.PATCH)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl SemVer {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        SemVer { major, minor, patch }
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Specifies possible types of version matching from package to target rustc version
pub enum PackageCompatMatchType {
    Exact(Vec<String>),
    Previous(Vec<String>),
    NoMatch(Vec<(String, String)>),
}

/// Custom error type for the library
#[derive(Debug)]
pub enum LibError {
    InvalidVersion(String),
    ApiError(Error),
}

impl fmt::Display for LibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibError::InvalidVersion(v) => write!(
                f,
                "Invalid version format: '{}'. Version must be in MAJOR.MINOR.PATCH format (e.g., 1.70.0)",
                v
            ),
            LibError::ApiError(e) => write!(f, "API error: {}", e),
        }
    }
}

impl std::error::Error for LibError {}

impl From<Error> for LibError {
    fn from(err: Error) -> Self {
        LibError::ApiError(err)
    }
}

/// Helper function to check if a dependency is a path or git dependency
fn is_path_or_git_dependency(value: &toml::Value) -> bool {
    match value {
        toml::Value::Table(table) => {
            table.contains_key("path") || table.contains_key("git")
        }
        _ => false,
    }
}

/// Parse a Cargo.toml file and extract all dependency names
pub fn parse_cargo_toml(path: &Path) -> Result<Vec<String>, LibError> {
    let content = fs::read_to_string(path).map_err(|e| {
        LibError::InvalidVersion(format!("Failed to read Cargo.toml: {}", e))
    })?;

    let manifest: toml::Value = toml::from_str(&content).map_err(|e| {
        LibError::InvalidVersion(format!("Failed to parse Cargo.toml: {}", e))
    })?;

    let mut dependencies = std::collections::BTreeSet::new();

    // Helper closure to process a dependencies table
    let mut process_deps = |deps_table: Option<&toml::Value>| {
        if let Some(toml::Value::Table(deps)) = deps_table {
            for (name, value) in deps {
                // Skip path and git dependencies as they can't be queried from crates.io
                if !is_path_or_git_dependency(value) {
                    dependencies.insert(name.clone());
                }
            }
        }
    };

    // Collect all dependency names from different sections
    if let Some(table) = manifest.as_table() {
        process_deps(table.get("dependencies"));
        process_deps(table.get("dev-dependencies").or_else(|| table.get("dev_dependencies")));
        process_deps(table.get("build-dependencies").or_else(|| table.get("build_dependencies")));
    }

    Ok(dependencies.into_iter().collect())
}

/// Parse a version string into a SemVer
/// If MINOR or PATCH is omitted, they are assumed to be 0 (no warnings printed)
pub fn parse_version(version_str: &str) -> Result<SemVer, LibError> {
    let parts: Vec<&str> = version_str.split('.').collect();

    match parts.len() {
        1 => {
            // Only major version provided, assume minor and patch are 0
            let major: u64 = parts[0].parse().map_err(|_| LibError::InvalidVersion(version_str.to_string()))?;
            Ok(SemVer::new(major, 0, 0))
        }
        2 => {
            // Major and minor provided, assume patch is 0
            let major: u64 = parts[0].parse().map_err(|_| LibError::InvalidVersion(version_str.to_string()))?;
            let minor: u64 = parts[1].parse().map_err(|_| LibError::InvalidVersion(version_str.to_string()))?;
            Ok(SemVer::new(major, minor, 0))
        }
        _ => {
            // Full version or more than 3 parts, parse first 3 parts
            let major: u64 = parts[0].parse().map_err(|_| LibError::InvalidVersion(version_str.to_string()))?;
            let minor: u64 = parts[1].parse().map_err(|_| LibError::InvalidVersion(version_str.to_string()))?;
            let patch: u64 = parts[2].parse().map_err(|_| LibError::InvalidVersion(version_str.to_string()))?;
            Ok(SemVer::new(major, minor, patch))
        }
    }
}

/// Determine the best compatible match for the given package and target rustc version
pub fn query_package(
    client: &SyncClient,
    pkg_name: &str,
    rustc_version: &str,
) -> Result<PackageCompatMatchType, LibError> {
    let ct = client.get_crate(pkg_name)?;

    // Parse the target rustc version once
    let target_version = parse_version(rustc_version)?;

    let mut exact_matches = vec![];
    let mut earlier_matches = vec![];
    let mut invalid_matches = vec![];

    for version in ct.versions {
        if let Some(rv) = version.rust_version {
            match parse_version(&rv) {
                Ok(rv_version) => {
                    if rv_version == target_version {
                        exact_matches.push(version.num);
                    } else if rv_version < target_version {
                        earlier_matches.push(version.num);
                    } else {
                        invalid_matches.push((version.num, rv));
                    }
                }
                Err(_) => {
                    // Skip versions with invalid semver format
                    invalid_matches.push((version.num, rv));
                }
            }
        } else {
            // Package version doesn't specify rust-version requirement
            // Treat as compatible (add to earlier_matches)
            earlier_matches.push(version.num);
        }
    }

    Ok(if !exact_matches.is_empty() {
        PackageCompatMatchType::Exact(exact_matches)
    } else if !earlier_matches.is_empty() {
        PackageCompatMatchType::Previous(earlier_matches)
    } else {
        PackageCompatMatchType::NoMatch(invalid_matches)
    })
}
