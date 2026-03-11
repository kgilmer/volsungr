use std::path::PathBuf;
use std::process::exit;
use std::sync::Once;

use argh::FromArgs;
use crates_io_api::SyncClient;
use volsungr::{parse_cargo_toml, query_package, PackageCompatMatchType};

static MAJOR_WARNING_ONCE: Once = Once::new();
static MINOR_WARNING_ONCE: Once = Once::new();

#[derive(FromArgs)]
/// Search for the latest crate version compatible with a specified Rust toolchain version.
struct Cli {
    /// target rustc version (e.g., 1.70.0 or v1.70.0)
    #[argh(option, short = 'r')]
    rustc_version: String,

    /// package name to query (can be specified multiple times)
    #[argh(option, short = 'p')]
    package: Vec<String>,

    /// directory containing a Cargo.toml file to extract dependencies from
    #[argh(option, short = 'd')]
    directory: Option<PathBuf>,

    /// suppress warnings and only print package name and compatible version
    #[argh(switch, short = 'b')]
    brief: bool,
}

/// Parse and validate the rustc version from CLI, printing warnings if needed
fn parse_cli_rustc_version(version: &str, brief: bool) -> String {
    let version = version.strip_prefix('v').unwrap_or(version);
    let parts: Vec<&str> = version.split('.').collect();

    match parts.len() {
        1 => {
            // Only major version provided
            if !brief {
                MAJOR_WARNING_ONCE.call_once(|| {
                    eprintln!(
                        "Warning: Assuming version {}.0.0 (minor and patch segments omitted)",
                        parts[0]
                    );
                });
            }
        }
        2 => {
            // Major and minor provided
            if !brief {
                MINOR_WARNING_ONCE.call_once(|| {
                    eprintln!(
                        "Warning: Assuming version {}.0 (patch segment omitted)",
                        version
                    );
                });
            }
        }
        _ => {}
    }

    version.to_string()
}

fn main() {
    let cli: Cli = argh::from_env();

    // Validate that either -p or -d is provided, but not both
    let has_packages = !cli.package.is_empty();
    let has_directory = cli.directory.is_some();

    if !has_packages && !has_directory {
        eprintln!("Error: either --package (-p) or --directory (-d) must be provided");
        exit(1);
    }

    if has_packages && has_directory {
        eprintln!("Error: --package (-p) and --directory (-d) cannot be used together");
        exit(1);
    }

    let rustc_version = parse_cli_rustc_version(&cli.rustc_version, cli.brief);

    let client = SyncClient::new(
        "volsungr/0.1.0 (https://github.com/kgilmer/volsungr)",
        std::time::Duration::from_millis(1000),
    )
    .unwrap();

    // Collect package names to query
    let mut packages_to_query = cli.package.clone();

    // If directory is provided, parse Cargo.toml and extract dependencies
    if let Some(dir) = &cli.directory {
        let cargo_toml_path = dir.join("Cargo.toml");
        
        if !cargo_toml_path.exists() {
            eprintln!("Error: Cargo.toml not found at {:?}", cargo_toml_path);
            exit(1);
        }

        match parse_cargo_toml(&cargo_toml_path) {
            Ok(deps) => {
                if deps.is_empty() {
                    eprintln!("Warning: No dependencies found in {:?}", cargo_toml_path);
                } else {
                    packages_to_query.extend(deps);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
    }

    for pkg_name in &packages_to_query {
        match query_package(&client, pkg_name, &rustc_version) {
            Ok(PackageCompatMatchType::Exact(versions)) => {
                if cli.brief {
                    println!("{} = \"{}\"", pkg_name, versions[0]);
                } else {
                    println!(
                        "For rustc version \"{}\", the latest compatible version: {} = \"{}\"",
                        rustc_version, pkg_name, versions[0]
                    );
                }
            }
            Ok(PackageCompatMatchType::Previous(versions)) => {
                if cli.brief {
                    println!("{} = \"{}\"", pkg_name, versions[0]);
                } else {
                    println!(
                        "For rustc version \"{}\", the latest compatible version: {} = \"{}\"",
                        rustc_version, pkg_name, versions[0]
                    );
                }
            }
            Ok(PackageCompatMatchType::NoMatch(invalid)) => {
                if cli.brief {
                    println!(
                        "{} = \"<none>\"",
                        pkg_name
                    );
                } else {
                    println!(
                        "No versions of {} are compatible with rustc version \"{}\" (checked versions {:?})",
                        pkg_name,
                        rustc_version,
                        invalid.iter().map(|(v, _)| v.as_str()).collect::<Vec<_>>()
                    );
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                exit(1);
            }
        }
    }
}
