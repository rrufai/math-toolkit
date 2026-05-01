use integrator_core::{integrator, plot, first_var, parse};
use solver_core::solve;

enum PlotMode { AsciiOnly, AsciiAndSvg }

fn demo(label: &str, expr: &str, a: f64, b: f64, plot_mode: PlotMode) {
    demo_with_svg(label, expr, a, b, plot_mode, "integrate.svg");
}

fn demo_with_svg(label: &str, expr: &str, a: f64, b: f64, plot_mode: PlotMode, svg_path: &str) {
    match integrator::integrate(expr, a, b) {
        Err(e) => println!("  ERROR: {}", e),
        Ok(r) => {
            let parsed = parse(expr).ok();
            let var = parsed.as_ref()
                .and_then(first_var)
                .unwrap_or_else(|| "x".to_string());
            println!("┌─ {} ─────────────────────────────────────────", label);
            println!("│  f({})          = {}", var, expr);
            if let Some(ref sym) = r.symbolic {
                println!("│  ∫f({})d{}       = {} + C", var, var, sym);
            } else {
                println!("│  ∫f({})d{}       = (no closed form found)", var, var);
            }
            println!("│  ∫[{}, {}] f d{}  ≈ {:.10}", a, b, var, r.numerical);
            println!("└──────────────────────────────────────────────");
            println!();

            if matches!(plot_mode, PlotMode::AsciiOnly | PlotMode::AsciiAndSvg) {
                if let Some(ref ast) = parsed {
                    let anti_fn: Option<Box<dyn Fn(f64) -> f64>> =
                        integrator::integrate_symbolic(ast).ok().map(|anti| {
                            Box::new(move |x| anti.eval(x)) as Box<dyn Fn(f64) -> f64>
                        });
                    plot::print_ascii(expr, &var, a, b, &|x| ast.eval(x), anti_fn.as_deref());
                }
            }

            if matches!(plot_mode, PlotMode::AsciiAndSvg) {
                if let Some(ref ast) = parsed {
                    let anti_fn: Option<Box<dyn Fn(f64) -> f64>> =
                        integrator::integrate_symbolic(ast).ok().map(|anti| {
                            Box::new(move |x| anti.eval(x)) as Box<dyn Fn(f64) -> f64>
                        });
                    match plot::write_svg(
                        svg_path, expr, &var, a, b,
                        &|x| ast.eval(x),
                        anti_fn.as_deref(),
                        r.numerical,
                    ) {
                        Err(e) => eprintln!("Warning: could not write SVG: {}", e),
                        Ok(()) => println!("  Plot saved → {}", svg_path),
                    }
                }
            }
        }
    }
}

fn run_solve(equation: &str, a: f64, b: f64) -> Result<(), String> {
    run_solve_with_svg(equation, a, b, "solve.svg")
}

fn run_solve_with_svg(equation: &str, a: f64, b: f64, svg_path: &str) -> Result<(), String> {
    match solve(equation, a, b) {
        Err(e) => return Err(e),
        Ok(r) => {
            println!("┌─ solve ──────────────────────────────────────────");
            println!("│  f(x)          = {}", equation);
            println!("│  root          ≈ {:.10}", r.root);
            println!("│  f(root)       ≈ {:e}", r.residual);
            println!("│  iterations    = {}", r.iterations);
            println!("└──────────────────────────────────────────────");
            println!();

            // Build the expression string (lhs - rhs) for plotting.
            let plot_expr = if equation.contains('=') {
                let parts: Vec<&str> = equation.splitn(2, '=').collect();
                format!("({}) - ({})", parts[0].trim(), parts[1].trim())
            } else {
                equation.to_string()
            };

            if let Ok(ast) = parse(&plot_expr) {
                let var = first_var(&ast).unwrap_or_else(|| "x".to_string());
                plot::print_ascii(&plot_expr, &var, a, b, &|x| ast.eval(x), None);

                match plot::write_svg(
                    svg_path,
                    &plot_expr,
                    &var,
                    a,
                    b,
                    &|x| ast.eval(x),
                    None,
                    r.residual,
                ) {
                    Err(e) => eprintln!("Warning: could not write SVG: {}", e),
                    Ok(()) => println!("  Plot saved → {}", svg_path),
                }
            }
        }
    }
    Ok(())
}


fn print_help(prog: &str) {
    println!("Usage:");
    println!("  {} --help                       Show this help message", prog);
    println!("  {} --demo                       Run built-in demo and verification suite", prog);
    println!("  {} <expr> [<a> <b>]             Integrate expr over [a, b] (default: [0, 1])", prog);
    println!("  {} --solve <equation> <a> <b>   Find root of equation in [a, b]", prog);
    println!();
    println!("Supported syntax:");
    println!("  Variables : any identifier (x, t, u, z, ...)");
    println!("  Constants : pi, e");
    println!("  Operators : + - * / ^ (right-associative)");
    println!("  Functions : sin, cos, tan, exp, ln, log, sqrt, abs");
    println!("  Implicit multiplication: 3x^2, 2t^3, 4sin(u)");
    println!();
    println!("Examples:");
    println!("  {} \"x^2 + sin(x)\" 0 3.14", prog);
    println!("  {} \"4*x^3 - 3*x^2\"", prog);
    println!("  {} --solve \"x^2 - 2\" 1 2", prog);
    println!("  {} --demo", prog);
    println!();
    println!("For differentiation, use the `differentiate` binary.");
}

fn run_demo() {
    demo("constant",     "5",               0.0, 3.0, PlotMode::AsciiOnly);
    demo("linear",       "x",               0.0, 2.0, PlotMode::AsciiOnly);
    demo("quadratic",    "x^2",             0.0, 3.0, PlotMode::AsciiOnly);
    demo("cubic",        "x^3 - 2*x + 1",  0.0, 2.0, PlotMode::AsciiOnly);
    demo("high degree",  "3*x^4 - x^2 + 7", 1.0, 4.0, PlotMode::AsciiOnly);
    demo("sin(x)",  "sin(x)",         0.0, std::f64::consts::PI,        PlotMode::AsciiOnly);
    demo("cos(x)",  "cos(x)",         0.0, std::f64::consts::FRAC_PI_2, PlotMode::AsciiOnly);
    demo("tan(x)",  "tan(x)",         0.0, 1.0,                         PlotMode::AsciiOnly);
    demo("exp(x)",  "exp(x)",         0.0, 1.0,                         PlotMode::AsciiOnly);
    demo("ln(x)",   "ln(x)",          1.0, std::f64::consts::E,         PlotMode::AsciiOnly);
    demo("a^x",     "2^x",            0.0, 3.0,                         PlotMode::AsciiOnly);
    demo("sqrt(x)", "sqrt(x)",        0.0, 4.0,                         PlotMode::AsciiOnly);
    demo("1/x",     "1/x",            1.0, std::f64::consts::E,         PlotMode::AsciiOnly);
    demo("x^2 + sin(x)",     "x^2 + sin(x)",     0.0, 2.0, PlotMode::AsciiOnly);
    demo("2*x^3 - 3*cos(x)", "2*x^3 - 3*cos(x)", 0.0, 1.0, PlotMode::AsciiOnly);
    demo("5*exp(x) - x^2",   "5*exp(x) - x^2",   0.0, 2.0, PlotMode::AsciiOnly);

    println!("═══════════════════════════════════════════════════");
    println!("   Verification: F(b) - F(a) matches numerical");
    println!("═══════════════════════════════════════════════════");
    println!();

    if !run_verification() {
        std::process::exit(1);
    }
    println!();
}

fn run_verification() -> bool {
    let tests: &[(&str, f64, f64)] = &[
        ("x^2",              0.0, 3.0),
        ("x^3 - 2*x + 1",    0.0, 2.0),
        ("sin(x)",            0.0, std::f64::consts::PI),
        ("cos(x)",            0.0, std::f64::consts::FRAC_PI_2),
        ("exp(x)",            0.0, 1.0),
        ("ln(x)",             1.0, std::f64::consts::E),
        ("sqrt(x)",           0.0, 4.0),
    ];
    run_verification_with(tests)
}

fn run_verification_with(tests: &[(&str, f64, f64)]) -> bool {
    let mut all_ok = true;
    for (expr_str, a, b) in tests {
        let expr = parse(expr_str).unwrap();
        let anti = integrator::integrate_symbolic(&expr).unwrap();
        let sym_val  = anti.eval(*b) - anti.eval(*a);
        let num_val  = integrator::integrate_numerical(&|x| expr.eval(x), *a, *b);
        let diff = (sym_val - num_val).abs();
        let ok = diff < 1e-6;
        if !ok { all_ok = false; }
        println!(
            "  {} {:22} on [{:.4}, {:.4}]  sym={:.8}  num={:.8}  diff={:.2e}  [{}]",
            if ok { "✓" } else { "✗" },
            expr_str, a, b, sym_val, num_val, diff,
            if ok { "PASS" } else { "FAIL" }
        );
    }
    println!();
    if all_ok {
        println!("  All verification tests PASSED ✓");
    } else {
        println!("  Some tests FAILED ✗");
    }
    all_ok
}

fn parse_bound(s: &str) -> Result<f64, String> {
    s.parse::<f64>().map_err(|_| format!("Error: '{}' is not a valid number", s))
}

fn parse_args(args: &[String]) -> Result<(&str, f64, f64), String> {
    if args.len() != 2 && args.len() != 4 {
        return Err(format!(
            "Usage: {} <expr> [<a> <b>]\n       {} --help for more information",
            args[0], args[0]
        ));
    }
    let expr = args[1].as_str();
    let (a, b) = if args.len() == 4 {
        (parse_bound(&args[2])?, parse_bound(&args[3])?)
    } else {
        (0.0, 1.0)
    };
    Ok((expr, a, b))
}

fn run(args: &[String]) -> Result<(), String> {
    let prog = &args[0];

    match args.get(1).map(String::as_str) {
        Some("--help") | Some("-h") => {
            print_help(prog);
        }
        Some("--demo") => {
            run_demo();
        }
        Some("--solve") => {
            if args.len() != 5 {
                return Err(format!("Usage: {} --solve <equation> <a> <b>", prog));
            }
            let equation = &args[2];
            let a = parse_bound(&args[3])?;
            let b = parse_bound(&args[4])?;
            run_solve(equation, a, b)?;
        }
        Some(_) => {
            let (expr, a, b) = parse_args(args)?;
            demo(expr, expr, a, b, PlotMode::AsciiAndSvg);
        }
        None => {
            print_help(prog);
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run(&args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_bound_valid() {
        assert_eq!(parse_bound("3.14").unwrap(), 3.14);
        assert_eq!(parse_bound("0").unwrap(), 0.0);
        assert_eq!(parse_bound("-1.5").unwrap(), -1.5);
    }

    #[test]
    fn test_parse_bound_invalid() {
        assert!(parse_bound("abc").is_err());
        assert!(parse_bound("").is_err());
        assert!(parse_bound("1.2.3").is_err());
    }

    #[test]
    fn test_parse_args_with_bounds() {
        let args = make_args(&["prog", "x^2", "0", "3"]);
        let (expr, a, b) = parse_args(&args).unwrap();
        assert_eq!(expr, "x^2");
        assert_eq!(a, 0.0);
        assert_eq!(b, 3.0);
    }

    #[test]
    fn test_parse_args_default_bounds() {
        let args = make_args(&["prog", "x^2"]);
        let (expr, a, b) = parse_args(&args).unwrap();
        assert_eq!(expr, "x^2");
        assert_eq!(a, 0.0);
        assert_eq!(b, 1.0);
    }

    #[test]
    fn test_parse_args_wrong_count() {
        assert!(parse_args(&make_args(&["prog", "x^2", "0"])).is_err());
        assert!(parse_args(&make_args(&["prog", "x^2", "0", "1", "extra"])).is_err());
    }

    #[test]
    fn test_parse_args_invalid_bound() {
        assert!(parse_args(&make_args(&["prog", "x^2", "abc", "1"])).is_err());
        assert!(parse_args(&make_args(&["prog", "x^2", "0", "xyz"])).is_err());
    }

    #[test]
    fn test_run_verification_passes() {
        assert!(run_verification());
    }

    #[test]
    fn test_demo_ascii_only() {
        // Exercises demo() with AsciiOnly — covers the inner if-let-Some(ast) block
        // including the closing braces on lines 31/32 that subprocess tests miss.
        demo("unit-test", "x^2", 0.0, 1.0, PlotMode::AsciiOnly);
    }

    #[test]
    fn test_demo_ascii_and_svg() {
        // Exercises demo() with AsciiAndSvg — covers the SVG write block (lines 34-51).
        // write_svg writes to "integrate.svg" relative to cwd; that is fine in tests.
        demo("unit-test", "x^2", 0.0, 1.0, PlotMode::AsciiAndSvg);
    }

    #[test]
    fn test_demo_no_closed_form() {
        // Exercises the "no closed form found" branch in demo() (line 18).
        demo("unit-test", "sin(x^2)", 0.0, 1.0, PlotMode::AsciiOnly);
    }

    #[test]
    fn test_demo_parse_error() {
        // Exercises the Err branch of integrate() in demo() (line 7).
        demo("unit-test", "sin(x +", 0.0, 1.0, PlotMode::AsciiOnly);
    }

    #[test]
    fn test_print_help_runs() {
        print_help("integrate");
    }

    #[test]
    fn test_run_solve_success() {
        // Exercises the Ok branch of run_solve — should print box and not panic.
        assert!(run_solve("x^2 - 2", 1.0, 2.0).is_ok());
    }

    #[test]
    fn test_run_solve_two_sided_equation() {
        // Exercises the equation.contains('=') branch in run_solve (plot_expr rewrite).
        assert!(run_solve("x^2 = 2", 1.0, 2.0).is_ok());
    }

    #[test]
    fn test_run_solve_error() {
        // Exercises the Err branch of run_solve (no sign change).
        let result = run_solve("x^2", 1.0, 3.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no sign change"));
    }

    #[test]
    fn test_run_help() {
        assert!(run(&make_args(&["prog", "--help"])).is_ok());
        assert!(run(&make_args(&["prog", "-h"])).is_ok());
    }

    #[test]
    fn test_run_no_args() {
        assert!(run(&make_args(&["prog"])).is_ok());
    }

    #[test]
    fn test_run_demo() {
        assert!(run(&make_args(&["prog", "--demo"])).is_ok());
    }

    #[test]
    fn test_run_solve_wrong_arg_count() {
        let result = run(&make_args(&["prog", "--solve", "x^2 - 2"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--solve"));
    }

    #[test]
    fn test_run_solve_bad_lower_bound() {
        let result = run(&make_args(&["prog", "--solve", "x^2 - 2", "bad", "2"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a valid number"));
    }

    #[test]
    fn test_run_solve_bad_upper_bound() {
        let result = run(&make_args(&["prog", "--solve", "x^2 - 2", "1", "bad"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a valid number"));
    }

    #[test]
    fn test_run_expr_ok() {
        assert!(run(&make_args(&["prog", "x^2", "0", "3"])).is_ok());
    }

    #[test]
    fn test_run_expr_wrong_count() {
        let result = run(&make_args(&["prog", "x^2", "0"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_run_verification_fail_msg() {
        // run_verification() always passes for valid expressions.
        assert!(run_verification());
    }

    #[test]
    fn test_run_verification_with_failing_case() {
        // ln(x) on [0, E] has a singularity at x=0: the numerical integrator
        // gets NaN/garbage, sym_val=0, so |sym-num| is not < 1e-6 → FAILED path.
        let result = run_verification_with(&[("ln(x)", 0.0, std::f64::consts::E)]);
        assert!(!result, "expected verification to fail for ln(x) on [0, e]");
    }

    #[test]
    fn test_demo_svg_write_error() {
        // Write to "." (a directory) to exercise the Err branch of write_svg in demo().
        demo_with_svg("unit-test", "x^2", 0.0, 1.0, PlotMode::AsciiAndSvg, ".");
    }

    #[test]
    fn test_run_solve_svg_write_error() {
        // Write to "." (a directory) to exercise the Err branch of write_svg in run_solve().
        assert!(run_solve_with_svg("x^2 - 2", 1.0, 2.0, ".").is_ok());
    }
}
