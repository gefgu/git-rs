use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
fn read_blob() -> Result<(), Box<dyn std::error::Error>> {
    let folder = assert_fs::TempDir::new().unwrap();
    let file = folder.child("hello_world.txt");
    file.write_str("hello world")?;

    Command::new("git")
        .current_dir(folder.path())
        .arg("init")
        .unwrap();

    let hash = Command::new("git")
        .current_dir(folder.path())
        .arg("hash-object")
        .arg("-w")
        .arg("hello_world.txt")
        .unwrap()
        .stdout
        .split_last()
        .unwrap()
        .1
        .to_vec();

    let hash = String::from_utf8(hash).unwrap();

    println!("{hash}");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.current_dir(folder.path());

    cmd.arg("cat-file").arg("-p").arg(hash);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("hello world"));

    Ok(())
}
