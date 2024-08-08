use std::{fs, path::Path};

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn emtpy_folder() -> Result<(), Box<dyn std::error::Error>> {
    let folder = assert_fs::TempDir::new().unwrap();
    let dir_path = folder.path();
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(dir_path);

    cmd.arg("init");
    cmd.assert().success();

    assert_eq!(Path::new(".git").exists(), true);
    assert_eq!(Path::new(".git/objects").exists(), true);
    assert_eq!(Path::new(".git/refs").exists(), true);
    assert_eq!(Path::new(".git/HEAD").exists(), true);

    let head_content = fs::read_to_string(".git/HEAD")?;
    assert_eq!(head_content, "ref: refs/heads/main\n");

    folder.close()?;
    Ok(())
}

#[test]
fn already_has_git_folder() -> Result<(), Box<dyn std::error::Error>> {
    let folder = assert_fs::TempDir::new().unwrap();
    Command::new("git")
        .current_dir(folder.path())
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(folder.path());

    cmd.arg("init");
    cmd.assert().success().stdout(predicate::str::contains(
        "Reinitialized existing Git repository in",
    ));

    folder.close()?;
    Ok(())
}
