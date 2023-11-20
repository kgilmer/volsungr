use std::{env, process::exit};

use crates_io_api::{SyncClient, Error};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("usage: volsungr <target rustc version> <package name> ");
        exit(1);
    }

    let rustc_version = args.get(1).unwrap();
    let rustc_version = if rustc_version.starts_with("v") {
        rustc_version[1..].as_ref()
    } else {
        rustc_version.as_str()
    };
    let pkg_name = args.get(2).unwrap();

    let client = SyncClient::new(
        "my-user-agent (my-contact@domain.com)",
        std::time::Duration::from_millis(1000),
    )
    .unwrap();

    query_package(&client, pkg_name, rustc_version).expect("Can query crates.io");
}

fn query_package(client: &SyncClient, pkg_name: &str, rustc_version: &str) -> Result<(), Error> {
    println!("Searching {} versions compatible with rust {}...", pkg_name, rustc_version);

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
            // println!("{} {} does not specify a rust version", pkg_name, version.num);
        }
    }

    if !exact_matches.is_empty() {
        println!("Latest matching version of {} = {:?}", pkg_name, exact_matches[0])
    } else if !earlier_matches.is_empty() {
        println!("Latest compatible version of {} = {:?}", pkg_name, earlier_matches[0])
    } else {
        println!("No versions of {} are compatible with rust version {} (checked versions {:?})", pkg_name, rustc_version, invalid_matches);
    }

    Ok(())
}
