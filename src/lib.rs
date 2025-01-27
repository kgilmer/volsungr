use crates_io_api::{Error, SyncClient};

/// Specifies possible types of version matching from package to target rustc version
pub enum PackageCompatMatchType {
    Exact(Vec<String>),
    Previous(Vec<String>),
    NoMatch(Vec<(String, String)>),
    Unknown(String),
}

/// Determine the best compatible match for the given package and target rustc version
pub fn query_package(
    client: &SyncClient,
    pkg_name: &str,
    rustc_version: &str,
) -> Result<PackageCompatMatchType, Error> {
    let ct = client.get_crate(pkg_name)?;

    let mut exact_matches = vec![];
    let mut earlier_matches = vec![];
    let mut invalid_matches = vec![];

    for version in ct.versions {
        if let Some(rv) = version.rust_version {
            if rv == rustc_version {
                exact_matches.push(version.num);
            } else if rv.as_str() < rustc_version {
                earlier_matches.push(version.num);
            } else {
                invalid_matches.push((version.num, rv));
            }
        } else {
            return Ok(PackageCompatMatchType::Unknown(version.num));
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
