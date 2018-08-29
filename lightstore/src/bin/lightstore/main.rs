#[allow(unused_imports)]
use unwrap::*;
#[allow(unused)]
use clap::{Arg, App, SubCommand, AppSettings};
use git2::Repository;
use lightstore::git::RepositoryExt;
use lightstore::daemon::Daemon;
use futures::future;

fn main() {
    let matches = {
        App::new("lightstore")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::GlobalVersion)
        .subcommand({
            SubCommand::with_name("create")
            .about("Creates a repository in the cloud")
            .arg({
                Arg::with_name("remote")
                .short("r")
                .help("Add the created address as a remote")
                .takes_value(true)
            })
        })
        .subcommand({
            SubCommand::with_name("list")
            .about("List lightstore addresses of this repository")
        })
        .subcommand({
            SubCommand::with_name("daemon")
            .about("Start the lightstore daemon")
        })
        .get_matches()
    };

    match unwrap!(matches.subcommand_name()) {
        "create" => {
            let sub_matches = unwrap!(matches.subcommand_matches("create"));
            let repo = unwrap!(Repository::open_from_env());
            let keypair = unwrap!(repo.create_lightstore_key());
            let url = keypair.public.to_url();
            match sub_matches.value_of("remote") {
                Some(remote) => {
                    unwrap!(repo.remote(remote, &url));
                    println!("added remote {} {}", remote, url);
                },
                None => {
                    println!("created remote {}", url);
                },
            }
        },
        "list" => {
            let repo = unwrap!(Repository::open_from_env());
            let keys = unwrap!(repo.get_all_lightstore_keys());
            for keypair in keys {
                let url = keypair.public.to_url();
                println!("{}", url);
            }
        },
        "daemon" => {
            let (_daemon, addr) = unwrap!(Daemon::start());
            println!("Daemon running at {}", addr);
            tokio::run(future::empty());
        },
        _ => unreachable!(),
    }
}

