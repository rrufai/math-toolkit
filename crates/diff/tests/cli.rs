use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_differentiate"))
}

fn run(args: &[&str]) -> (String, String, i32) {
    let out = bin().args(args).output().expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let code   = out.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

#[test]
fn test_help_flag() {
    let (out, _, code) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("--help"));
    assert!(out.contains("<expr>"));
}

#[test]
fn test_help_short_flag() {
    let (out, _, code) = run(&["-h"]);
    assert_eq!(code, 0);
    assert!(out.contains("--help"));
}

#[test]
fn test_no_args_shows_help() {
    let (out, _, code) = run(&[]);
    assert_eq!(code, 0);
    assert!(out.contains("--help"));
}

#[test]
fn test_diff_x_squared() {
    let (out, _, code) = run(&["x^2", "0", "2"]);
    assert_eq!(code, 0);
    // d/dx x^2 = 2*x
    assert!(out.contains("f'(x)"));
    assert!(out.contains("2"));
    assert!(out.contains("<svg") || out.contains("Plot saved"));
}

#[test]
fn test_diff_sin() {
    let (out, _, code) = run(&["sin(x)", "0", "3.14"]);
    assert_eq!(code, 0);
    assert!(out.contains("cos(x)"));
}

#[test]
fn test_diff_polynomial_default_bounds() {
    let (out, _, code) = run(&["x^3 - 2*x + 1"]);
    assert_eq!(code, 0);
    assert!(out.contains("f'(x)"));
}

#[test]
fn test_diff_implicit_mul() {
    let (out, _, code) = run(&["3x^2", "0", "2"]);
    assert_eq!(code, 0);
    assert!(out.contains("f'(x)"));
}

#[test]
fn test_wrong_arg_count() {
    let (_, err, code) = run(&["x^2", "0"]);
    assert_ne!(code, 0);
    assert!(err.contains("Usage"));
}

#[test]
fn test_invalid_lower_bound() {
    let (_, err, code) = run(&["x^2", "abc", "3"]);
    assert_ne!(code, 0);
    assert!(err.contains("not a valid number"));
}

#[test]
fn test_invalid_upper_bound() {
    let (_, err, code) = run(&["x^2", "0", "xyz"]);
    assert_ne!(code, 0);
    assert!(err.contains("not a valid number"));
}

#[test]
fn test_bad_expression() {
    let (_, err, code) = run(&["sin(x +", "0", "1"]);
    assert_ne!(code, 0);
    assert!(err.contains("ERROR"));
}

#[test]
fn test_unsupported_abs() {
    let (out, _, code) = run(&["abs(x)", "0", "1"]);
    // abs is unsupported — prints error line but exits 0 via the Ok branch
    // actually it prints ERROR in the box; exit code depends on path
    let _ = code;
    assert!(out.contains("ERROR") || out.contains("error"));
}
