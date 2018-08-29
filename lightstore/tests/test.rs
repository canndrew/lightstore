use lightstore::priv_prelude::*;
use std::process::Command;
use crate::util::TestRepo;
use std::str;

mod util;

fn enter_test_env() {
    let mut dir = unwrap!(std::env::current_dir());
    dir.push("target");
    dir.push("debug");

    let dir = unwrap!(dir.to_str());

    let path = unwrap!(std::env::var("PATH"));
    let path = format!("{}:{}", dir, path);
    std::env::set_var("PATH", path.clone());
}

#[test]
fn lightstore_create() {
    enter_test_env();
    let _test_repo = TestRepo::enter();

    let output = unwrap!(
        Command::new("lightstore")
        .args(&["create"])
        .output()
    );
    assert!(output.status.success());
    let output = unwrap!(str::from_utf8(&output.stdout));
    let prefix = "created remote lsd://";
    let postfix = "/\n";
    assert!(output.starts_with(prefix));
    assert!(output.ends_with(postfix));
    let len = output.len();
    let pk = &output[prefix.len()..(len - postfix.len())];
    let _pk = unwrap!(base32::decode(base32::Alphabet::Crockford, pk));

    let path = PathBuf::from(format!(".git/info/lightstore-keys/{}", pk));
    assert!(path.exists());
}

