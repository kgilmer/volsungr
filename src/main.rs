use std::env;
use std::process::exit;

use crates_io_api::SyncClient;
use volsungr::{query_package, PackageCompatMatchType};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("usage: volsungr <target rustc version> <package name>");
        exit(1);
    }

    let rustc_version = args.get(1).unwrap();
    let rustc_version = if rustc_version.starts_with('v') {
        &rustc_version[1..]
    } else {
        rustc_version.as_str()
    };
    let pkg_name = args.get(2).unwrap();

    let client = SyncClient::new(
        "volsungr/0.1.0 (https://github.com/kgilmer/volsungr)",
        std::time::Duration::from_millis(1000),
    )
    .unwrap();

    match query_package(&client, pkg_name, rustc_version) {
        Ok(PackageCompatMatchType::Exact(versions)) => {
            println!(
                "Latest matching version of {} = {:?}",
                pkg_name, versions[0]
            );
        }
        Ok(PackageCompatMatchType::Previous(versions)) => {
            println!(
                "Latest compatible version of {} = {:?}",
                pkg_name, versions[0]
            );
        }
        Ok(PackageCompatMatchType::NoMatch(invalid)) => {
            println!(
                "No versions of {} are compatible with rust version {} (checked versions {:?})",
                pkg_name,
                rustc_version,
                invalid
                    .iter()
                    .map(|(v, _)| v.as_str())
                    .collect::<Vec<_>>()
            );
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    }
}