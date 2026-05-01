use integrator_core::{plot, first_var, parse};
use solver_core::solve;

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

fn parse_bound(s: &str) -> Result<f64, String> {
    s.parse::<f64>().map_err(|_| format!("Error: '{}' is not a valid number", s))
}

fn print_help(prog: &str) {
    println!("Usage:");
    println!("  {} --help                 Show this help message", prog);
    println!("  {} <equation> <a> <b>     Find root of equation in [a, b]", prog);
    println!();
    println!("Supported syntax:");
    println!("  Variables : any identifier (x, t, u, z, ...)");
    println!("  Constants : pi, e");
    println!("  Operators : + - * / ^ (right-associative)");
    println!("  Functions : sin, cos, tan, exp, ln, log, sqrt, abs");
    println!("  Implicit multiplication: 3x^2, 2t^3, 4sin(u)");
    println!();
    println!("Examples:");
    println!("  {} \"x^2 - 2\" 1 2", prog);
    println!("  {} \"x^2 = 2\" 1 2", prog);
    println!("  {} \"sin(x)\" 3 4", prog);
}

fn run(args: &[String]) -> Result<(), String> {
    let prog = &args[0];
    match args.get(1).map(String::as_str) {
        Some("--help") | Some("-h") => {
            print_help(prog);
        }
        Some(_) => {
            if args.len() != 4 {
                return Err(format!("Usage: {} <equation> <a> <b>\n       {} --help for more information", prog, prog));
            }
            let equation = &args[1];
            let a = parse_bound(&args[2])?;
            let b = parse_bound(&args[3])?;
            run_solve(equation, a, b)?;
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

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // parse_bound
    #[test]
    fn test_parse_bound_valid() {
        assert_eq!(parse_bound("1.5").unwrap(), 1.5);
        assert_eq!(parse_bound("0").unwrap(), 0.0);
        assert_eq!(parse_bound("-3").unwrap(), -3.0);
    }

    #[test]
    fn test_parse_bound_invalid() {
        assert!(parse_bound("abc").is_err());
        assert!(parse_bound("").is_err());
    }

    // print_help — just verify it doesn't panic
    #[test]
    fn test_print_help_runs() {
        print_help("solve");
    }

    // run() — help / no-args
    #[test]
    fn test_run_help_long() {
        assert!(run(&args(&["solve", "--help"])).is_ok());
    }

    #[test]
    fn test_run_help_short() {
        assert!(run(&args(&["solve", "-h"])).is_ok());
    }

    #[test]
    fn test_run_no_args() {
        assert!(run(&args(&["solve"])).is_ok());
    }

    // run() — wrong arg count
    #[test]
    fn test_run_wrong_arg_count_too_few() {
        let err = run(&args(&["solve", "x^2 - 2"])).unwrap_err();
        assert!(err.contains("Usage"));
    }

    #[test]
    fn test_run_wrong_arg_count_too_many() {
        let err = run(&args(&["solve", "x^2 - 2", "1", "2", "extra"])).unwrap_err();
        assert!(err.contains("Usage"));
    }

    // run() — bad bounds
    #[test]
    fn test_run_bad_lower_bound() {
        let err = run(&args(&["solve", "x^2 - 2", "abc", "2"])).unwrap_err();
        assert!(err.contains("not a valid number"));
    }

    #[test]
    fn test_run_bad_upper_bound() {
        let err = run(&args(&["solve", "x^2 - 2", "1", "xyz"])).unwrap_err();
        assert!(err.contains("not a valid number"));
    }

    // run_solve — success (f(x)=0 form)
    #[test]
    fn test_run_solve_success() {
        assert!(run_solve("x^2 - 2", 1.0, 2.0).is_ok());
    }

    // run_solve — success (f(x)=g(x) two-sided form)
    #[test]
    fn test_run_solve_two_sided_equation() {
        assert!(run_solve("x^2 = 2", 1.0, 2.0).is_ok());
    }

    // run_solve — error (no sign change)
    #[test]
    fn test_run_solve_no_sign_change() {
        let err = run_solve("x^2", 1.0, 3.0).unwrap_err();
        assert!(err.contains("no sign change"));
    }

    // run_solve — SVG write error branch (write to "." which is a directory)
    #[test]
    fn test_run_solve_svg_write_error() {
        assert!(run_solve_with_svg("x^2 - 2", 1.0, 2.0, ".").is_ok());
    }
}

