use std::{fs, path::Path, thread::sleep, time::Duration};

use assert_cmd::Command;

#[test]
fn emtpy_folder() -> Result<(), Box<dyn std::error::Error>> {
    let folder = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    cmd.current_dir(folder.path());

    cmd.arg("init");
    cmd.assert().success();

    sleep(Duration::from_secs(15));

    assert_eq!(Path::new(".git").exists(), true);
    assert_eq!(Path::new(".git/objects").exists(), true);
    assert_eq!(Path::new(".git/refs").exists(), true);
    assert_eq!(Path::new(".git/HEAD").exists(), true);

    let head_content = fs::read_to_string(".git/HEAD")?;
    assert_eq!(head_content, "ref: refs/heads/main\n");

    folder.close()?;
    Ok(())
}
