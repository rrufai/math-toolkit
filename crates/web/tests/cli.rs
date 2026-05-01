use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_web-server"))
}

fn run(args: &[&str]) -> (String, String, i32) {
    let out = bin().args(args).output().expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let code = out.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

#[test]
fn test_help_flag() {
    let (out, _, code) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("start") || out.contains("Usage"));
}

#[test]
fn test_version_command() {
    // `version` is a built-in Loco CLI sub-command that prints the app version
    // and exits cleanly — this covers the main() entry point (main.rs lines 5-7).
    let (_, _, code) = run(&["version"]);
    assert_eq!(code, 0);
}
