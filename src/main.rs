/// A command-line tool that searches for the latest version of a crate that is compatible with a specified version of the Rust toolchain.
use std::{env, process::exit};

use crates_io_api::SyncClient;
use volsungr::{query_package, PackageCompatMatchType};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("usage: volsungr <target rustc version> <package name> ");
        exit(1);
    }

    let rustc_version = args.get(1).expect("Failed to get param 1");
    let rustc_version = if let Some(version) = rustc_version.strip_prefix("v") {
        version
    } else {
        rustc_version.as_str()
    };
    let pkg_name = args.get(2).expect("Failed to get param 2");

    let client = SyncClient::new(
        "volsungr (me@fivefivefive.com)",
        std::time::Duration::from_millis(1000),
    )
    .expect("Cannot create crates.io client");

    println!(
        "Searching {} versions compatible with rust {}...",
        pkg_name, rustc_version
    );

    match query_package(&client, pkg_name, rustc_version).expect("Failed to query crates.io") {
        PackageCompatMatchType::Exact(versions) => {
            println!(
                "Latest matching version of {} = {:?}",
                pkg_name, versions[0]
            )
        }
        PackageCompatMatchType::Previous(versions) => {
            println!(
                "Latest compatible version of {} = {:?}",
                pkg_name, versions[0]
            )
        }
        PackageCompatMatchType::NoMatch(versions) => {
            println!(
                "No versions of {} are compatible with rust version {} (checked versions {:?})",
                pkg_name, rustc_version, versions
            );
        }
        PackageCompatMatchType::Unknown(version) => {
            println!("{} {} does not specify a rust version", pkg_name, version);
        }
    }
}
