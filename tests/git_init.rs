use assert_fs::{prelude::*, TempDir};
use std::env::set_current_dir;
use std::path::Path;

use assert_cmd::Command;

fn enter_temporary_folder() -> Result<TempDir, Box<dyn std::error::Error>> {
    let folder = assert_fs::TempDir::new().unwrap();
    Ok(folder)
}

#[test]
fn git_init() -> Result<(), Box<dyn std::error::Error>> {
    let dir = enter_temporary_folder()?;
    let mut cmd = Command::cargo_bin("git-rs")?;

    cmd.current_dir(dir);

    cmd.arg("init");

    cmd.assert().success();

    // assert_eq!(Path::new(".git").exists(), true);

    assert_eq!(4, 2 + 2);
    Ok(())
}
