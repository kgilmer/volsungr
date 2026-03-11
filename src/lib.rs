use crates_io_api::{Error, SyncClient};
use semver::Version;
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::Path;

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

/// Structures for parsing Cargo.toml
#[derive(Deserialize, Debug)]
pub struct CargoManifest {
    pub dependencies: Option<std::collections::BTreeMap<String, Dependency>>,
    pub dev_dependencies: Option<std::collections::BTreeMap<String, Dependency>>,
    pub build_dependencies: Option<std::collections::BTreeMap<String, Dependency>>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(DependencyDetails),
}

#[derive(Deserialize, Debug)]
pub struct DependencyDetails {
    pub version: Option<String>,
    pub path: Option<String>,
    pub git: Option<String>,
}

/// Parse a Cargo.toml file and extract all dependency names
pub fn parse_cargo_toml(path: &Path) -> Result<Vec<String>, LibError> {
    let content = fs::read_to_string(path).map_err(|e| {
        LibError::InvalidVersion(format!("Failed to read Cargo.toml: {}", e))
    })?;

    let manifest: CargoManifest = toml::from_str(&content).map_err(|e| {
        LibError::InvalidVersion(format!("Failed to parse Cargo.toml: {}", e))
    })?;

    let mut dependencies = std::collections::BTreeSet::new();

    // Collect all dependency names from different sections
    if let Some(deps) = manifest.dependencies {
        for (name, dep) in deps {
            // Skip path and git dependencies as they can't be queried from crates.io
            match dep {
                Dependency::Simple(_) => {
                    dependencies.insert(name);
                }
                Dependency::Detailed(details) => {
                    if details.path.is_none() && details.git.is_none() {
                        dependencies.insert(name);
                    }
                }
            }
        }
    }

    if let Some(deps) = manifest.dev_dependencies {
        for (name, dep) in deps {
            match dep {
                Dependency::Simple(_) => {
                    dependencies.insert(name);
                }
                Dependency::Detailed(details) => {
                    if details.path.is_none() && details.git.is_none() {
                        dependencies.insert(name);
                    }
                }
            }
        }
    }

    if let Some(deps) = manifest.build_dependencies {
        for (name, dep) in deps {
            match dep {
                Dependency::Simple(_) => {
                    dependencies.insert(name);
                }
                Dependency::Detailed(details) => {
                    if details.path.is_none() && details.git.is_none() {
                        dependencies.insert(name);
                    }
                }
            }
        }
    }

    Ok(dependencies.into_iter().collect())
}

/// Parse a version string into a semver Version
/// If MINOR or PATCH is omitted, they are assumed to be 0 (no warnings printed)
pub fn parse_version(version_str: &str) -> Result<Version, LibError> {
    let parts: Vec<&str> = version_str.split('.').collect();

    match parts.len() {
        1 => {
            // Only major version provided, assume minor and patch are 0
            Version::parse(&format!("{}.0.0", parts[0]))
                .map_err(|_| LibError::InvalidVersion(version_str.to_string()))
        }
        2 => {
            // Major and minor provided, assume patch is 0
            Version::parse(&format!("{}.0", version_str))
                .map_err(|_| LibError::InvalidVersion(version_str.to_string()))
        }
        _ => {
            // Full version or more than 3 parts, try to parse as-is
            Version::parse(version_str)
                .map_err(|_| LibError::InvalidVersion(version_str.to_string()))
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
