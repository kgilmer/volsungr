use crates_io_api::{Error, SyncClient};
use semver::Version;
use std::fmt;

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
            LibError::InvalidVersion(v) => write!(f, "Invalid version format: {}", v),
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

/// Parse a version string into a semver Version
fn parse_version(version_str: &str) -> Result<Version, LibError> {
    Version::parse(version_str).map_err(|_| LibError::InvalidVersion(version_str.to_string()))
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