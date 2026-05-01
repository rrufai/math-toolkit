use std::process::Command;

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_integrate"));
    // Propagate LLVM coverage profiling env var so subprocess coverage is captured.
    if let Ok(profile_file) = std::env::var("LLVM_PROFILE_FILE") {
        cmd.env("LLVM_PROFILE_FILE", profile_file);
    }
    cmd
}

fn run(args: &[&str]) -> (String, String, i32) {
    let out = bin().args(args).output().expect("failed to run binary");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let code   = out.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

// ── Help ──────────────────────────────────────────────────────

#[test]
fn test_help_flag() {
    let (out, _, code) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("--help"));
    assert!(out.contains("--demo"));
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

// ── Demo ─────────────────────────────────────────────────────

#[test]
fn test_demo_flag() {
    let (out, _, code) = run(&["--demo"]);
    assert_eq!(code, 0);
    assert!(out.contains("PASS"));
    assert!(out.contains("All verification tests PASSED"));
}

// ── Expression integration ───────────────────────────────────

#[test]
fn test_expr_with_bounds() {
    let (out, _, code) = run(&["x^2", "0", "3"]);
    assert_eq!(code, 0);
    assert!(out.contains("x^3/3"));
    assert!(out.contains("9.0000000000"));
}

#[test]
fn test_expr_default_bounds() {
    let (out, _, code) = run(&["x^2"]);
    assert_eq!(code, 0);
    assert!(out.contains("x^3/3"));
    // ∫[0,1] x^2 = 1/3
    assert!(out.contains("0.3333333333"));
}

#[test]
fn test_expr_sin() {
    let (out, _, code) = run(&["sin(x)", "0", "3.14159265"]);
    assert_eq!(code, 0);
    assert!(out.contains("-cos(x)"));
    assert!(out.contains("2.0000000000"));
}

#[test]
fn test_expr_no_closed_form() {
    let (out, _, code) = run(&["sin(x^2)", "0", "1"]);
    assert_eq!(code, 0);
    assert!(out.contains("no closed form found"));
}

#[test]
fn test_expr_implicit_mul() {
    let (out, _, code) = run(&["4x^3", "0", "1"]);
    assert_eq!(code, 0);
    assert!(out.contains("x^4"));
    assert!(out.contains("1.0000000000"));
}

// ── --solve ──────────────────────────────────────────────────

#[test]
fn test_solve_x_squared_minus_2() {
    let (out, _, code) = run(&["--solve", "x^2 - 2", "1", "2"]);
    assert_eq!(code, 0);
    // √2 ≈ 1.41421356237... — check first 10 significant digits
    assert!(out.contains("1.414213562"), "out={}", out);
}

#[test]
fn test_solve_no_sign_change() {
    let (_, err, code) = run(&["--solve", "x^2", "1", "3"]);
    assert_ne!(code, 0);
    assert!(err.contains("no sign change"), "err={}", err);
}

#[test]
fn test_solve_parse_error() {
    let (_, err, code) = run(&["--solve", "sin(x +", "0", "4"]);
    assert_ne!(code, 0);
    assert!(!err.is_empty(), "expected parse error in stderr");
}

#[test]
fn test_solve_missing_bounds() {
    let (_, err, code) = run(&["--solve", "x^2 - 2"]);
    assert_ne!(code, 0);
    assert!(err.contains("Usage") || err.contains("--solve"), "err={}", err);
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
    let (out, _, code) = run(&["sin(x +", "0", "1"]);
    assert_eq!(code, 0); // parse error → ERROR printed via demo()
    assert!(out.contains("ERROR"));
}
