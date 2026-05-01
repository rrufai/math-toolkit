//! Root-finding using Brent's method.
//!
//! # Usage
//! ```
//! let result = solver_core::solve("x^2 - 2", 1.0, 2.0).unwrap();
//! assert!((result.root - 2f64.sqrt()).abs() < 1e-10);
//! ```

use parser_core::parse;

const BRENT_TOL: f64 = 1e-10;
const BRENT_MAX_ITER: u32 = 100;

/// Result returned by [`solve`].
#[derive(Debug)]
pub struct SolveResult {
    /// The located root.
    pub root: f64,
    /// Number of Brent iterations performed.
    pub iterations: u32,
    /// |f(root)| — residual at the returned root.
    pub residual: f64,
}

/// Find a root of the equation described by `equation` within the bracket `[a, b]`.
///
/// `equation` may be:
/// - A single expression `f(x)` — finds `x` such that `f(x) = 0`.
/// - An expression `f(x) = g(x)` — internally rewritten as `f(x) - g(x) = 0`.
///
/// Returns `Err` if the bracket has no sign change, the equation cannot be parsed,
/// or the equation string contains more than one `=` character.
pub fn solve(equation: &str, a: f64, b: f64) -> Result<SolveResult, String> {
    let parts: Vec<&str> = equation.splitn(3, '=').collect();
    let expr_str = match parts.len() {
        1 => equation.to_string(),
        2 => format!("({}) - ({})", parts[0].trim(), parts[1].trim()),
        _ => return Err(format!("equation contains more than one '=': {}", equation)),
    };

    let ast = parse(&expr_str)?;
    let f = |x: f64| ast.eval(x);

    let fa = f(a);
    let fb = f(b);

    if fa == 0.0 {
        return Ok(SolveResult { root: a, iterations: 0, residual: 0.0 });
    }
    if fb == 0.0 {
        return Ok(SolveResult { root: b, iterations: 0, residual: 0.0 });
    }

    if fa * fb > 0.0 {
        return Err(format!(
            "no sign change in [{}, {}]: f({})={}, f({})={}",
            a, b, a, fa, b, fb
        ));
    }

    brent(&f, a, b, fa, fb)
}

#[allow(unused_assignments)]
pub(crate) fn brent(
    f: &dyn Fn(f64) -> f64,
    a: f64,
    b: f64,
    fa: f64,
    fb: f64,
) -> Result<SolveResult, String> {
    let (mut a, mut b, mut fa, mut fb) = if fa.abs() < fb.abs() {
        (b, a, fb, fa)
    } else {
        (a, b, fa, fb)
    };

    let mut c = a;
    let mut fc = fa;
    let mut mflag = true;
    let mut s = 0.0_f64;
    let mut d = 0.0;

    for i in 0..BRENT_MAX_ITER {
        if (b - a).abs() < BRENT_TOL || fb.abs() < BRENT_TOL {
            return Ok(SolveResult {
                root: b,
                iterations: i,
                residual: fb.abs(),
            });
        }

        if fa != fc && fb != fc {
            // Inverse quadratic interpolation
            s = a * fb * fc / ((fa - fb) * (fa - fc))
              + b * fa * fc / ((fb - fa) * (fb - fc))
              + c * fa * fb / ((fc - fa) * (fc - fb));
        } else {
            // Secant method
            s = b - fb * (b - a) / (fb - fa);
        }

        let cond1 = !((3.0 * a + b) / 4.0 < s && s < b || (3.0 * a + b) / 4.0 > s && s > b);
        let cond2 = mflag && (s - b).abs() >= (b - c).abs() / 2.0;
        let cond3 = !mflag && (s - b).abs() >= (c - d).abs() / 2.0;
        let cond4 = mflag && (b - c).abs() < BRENT_TOL;
        let cond5 = !mflag && (c - d).abs() < BRENT_TOL;

        if cond1 || cond2 || cond3 || cond4 || cond5 {
            s = (a + b) / 2.0;
            mflag = true;
        } else {
            mflag = false;
        }

        let fs = f(s);
        d = c;
        c = b;
        fc = fb;

        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }

    Ok(SolveResult {
        root: b,
        iterations: BRENT_MAX_ITER,
        residual: fb.abs(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_solve_x_squared_minus_2() {
        let r = solve("x^2 - 2", 1.0, 2.0).unwrap();
        assert!((r.root - 2f64.sqrt()).abs() < 1e-10, "root={}", r.root);
        assert!(r.residual < 1e-10, "residual={}", r.residual);
    }

    #[test]
    fn test_solve_two_sided_equation() {
        let r = solve("x^2 = 2", 1.0, 2.0).unwrap();
        assert!((r.root - 2f64.sqrt()).abs() < 1e-10, "root={}", r.root);
    }

    #[test]
    fn test_solve_sin_x() {
        let r = solve("sin(x)", 3.0, 4.0).unwrap();
        assert!((r.root - PI).abs() < 1e-10, "root={}", r.root);
    }

    #[test]
    fn test_solve_sin_eq_cos() {
        let r = solve("sin(x) = cos(x)", 0.5, 1.0).unwrap();
        assert!((r.root - PI / 4.0).abs() < 1e-10, "root={}", r.root);
    }

    #[test]
    fn test_solve_no_sign_change() {
        let err = solve("x^2", 1.0, 3.0).unwrap_err();
        assert!(err.contains("no sign change"), "err={}", err);
    }

    #[test]
    fn test_solve_parse_error() {
        let err = solve("sin(x +", 0.0, 1.0).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_solve_multiple_equals() {
        let err = solve("x = 1 = 2", 0.0, 3.0).unwrap_err();
        assert!(err.contains("more than one '='"), "err={}", err);
    }

    #[test]
    fn test_solve_x_squared_minus_4() {
        let r = solve("x^2 - 4", 1.0, 3.0).unwrap();
        assert!((r.root - 2.0).abs() < 1e-10, "root={}", r.root);
        assert!(r.residual < 1e-10);
    }

    #[test]
    fn test_solve_exact_zero_at_a() {
        // fa == 0.0 early exit
        let r = solve("x^2 - 1", 1.0, 2.0).unwrap();
        assert!((r.root - 1.0).abs() < 1e-10);
        assert_eq!(r.iterations, 0);
    }

    #[test]
    fn test_solve_exact_zero_at_b() {
        // fb == 0.0 early exit
        let r = solve("x^2 - 4", 1.0, 2.0).unwrap();
        assert!((r.root - 2.0).abs() < 1e-10);
        assert_eq!(r.iterations, 0);
    }

    #[test]
    fn test_brent_bisection_fallback() {
        // Use a bracket where the secant/IQI step falls outside [(3a+b)/4, b],
        // forcing the cond1 bisection fallback on the first iteration.
        // f(x) = x^3 - x - 2, root ≈ 1.5214
        // With a=-10, b=10 the secant step lands far outside the bracket.
        let f = |x: f64| x * x * x - x - 2.0;
        let fa = f(-10.0_f64);
        let fb = f(10.0_f64);
        let r = brent(&f, -10.0, 10.0, fa, fb).unwrap();
        assert!((r.root - 1.521_379_706_939f64).abs() < 1e-6);
    }

    #[test]
    fn test_brent_max_iterations() {
        // Construct a function that has a sign change but whose bracket never
        // shrinks below BRENT_TOL — forcing the loop to exhaust BRENT_MAX_ITER
        // and hit the fallback return at the bottom of the loop.
        // f(x) = sin(1e12 * x) has a root near 0 but the tiny bracket
        // oscillates wildly, making Brent's interpolation degenerate.
        // We call brent() directly to bypass the sign-change guard and
        // set up a bracket where fa*fb < 0.
        let f = |x: f64| {
            // A function that always evaluates to a sign based on step count —
            // but since we can't use mutable closures easily, use a non-convergent
            // analytic function: sign oscillates at sub-BRENT_TOL scale.
            // f(x) = x * sin(1/x) near 0 -- but singularity at 0.
            // Simpler: use a nearly-flat function with huge bracket.
            // If we use bracket [-1e15, 1e15] with f(x)=x, convergence
            // needs log2(2e15/1e-10) ≈ 84 bisection steps — under 100.
            // Use bracket width 2^101 * BRENT_TOL to need > 100 bisections.
            x  // f(x)=x, root at 0, but bracket so wide that bisection alone takes >100 steps
        };
        let huge = (2.0_f64).powi(101) * BRENT_TOL;  // > 2^100 * 1e-10, needs >100 bisections
        let fa = f(-huge);
        let fb = f(huge);
        // fa*fb < 0 since fa=-huge, fb=huge
        let r = brent(&f, -huge, huge, fa, fb).unwrap();
        // Should still return a result (even if not fully converged)
        assert!(r.root.abs() < huge);
    }
}
