use std::process::{Command, Stdio};

pub fn open_url(url: &str) {
    let mut cmd;
    if cfg!(windows) {
        cmd = Command::new("cmd.exe");
        cmd.args(["/C", "start", ""]);
    } else if cfg!(target_os = "macos") {
        cmd = Command::new("open");
    } else if cfg!(target_os = "linux") {
        cmd = Command::new("xdg-open");
    } else {
        eprintln!("warning: --open not yet implemented on this platform"); // TODO
        return;
    }
    cmd.arg(url);
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()); // might cause invalid handle errors if it tries to inherit handles from a VSC virtual console?
    assert!(cmd.status().unwrap().success());
}
