use lightstore::priv_prelude::*;
use tempdir::TempDir;
use std::process;

pub struct TestRepo {
    temp_dir: TempDir,
}

impl TestRepo {
    pub fn new() -> TestRepo {
        let mut tar_path = unwrap!(env::current_dir());
        tar_path.push("tests");
        tar_path.push("util");
        tar_path.push("test-repo.tar.gz");

        let temp_dir = unwrap!(TempDir::new("test"));

        let command_res = {
            process::Command::new("tar")
            .arg("xzf")
            .arg(tar_path)
            .current_dir(temp_dir.path())
            .status()
        };
        assert!(unwrap!(command_res).success());

        TestRepo {
            temp_dir,
        }
    }

    pub fn enter() -> TestRepo {
        let ret = TestRepo::new();
        unwrap!(env::set_current_dir(&ret.path()));
        ret
    }

    pub fn path(&self) -> PathBuf {
        let mut ret = PathBuf::from(self.temp_dir.path());
        ret.push("test-repo");
        ret
    }
}

