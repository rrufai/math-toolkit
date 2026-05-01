use integrator_core::{integrator, plot, first_var, parse};

fn run_diff(expr: &str, a: f64, b: f64) -> Result<(), String> {
    let parsed = parse(expr)?;
    let var = first_var(&parsed).unwrap_or_else(|| "x".to_string());

    println!("┌─ {} ─────────────────────────────────────────", expr);
    println!("│  f({})          = {}", var, expr);
    match integrator::differentiate_symbolic(&parsed) {
        Err(e) => {
            println!("│  f'({})         = ERROR: {}", var, e);
            println!("└──────────────────────────────────────────────");
        }
        Ok(deriv) => {
            println!("│  f'({})         = {}", var, deriv.to_string_repr());
            println!("└──────────────────────────────────────────────");
            println!();

            let ascii = plot::render_ascii_string_diff(
                expr, &var, a, b,
                &|x| parsed.eval(x),
                Some(&|x| deriv.eval(x)),
            );
            print!("{}", ascii);

            let svg_path = "differentiate.svg";
            let svg = plot::render_svg_diff(
                expr, &var, a, b,
                &|x| parsed.eval(x),
                Some(&|x| deriv.eval(x)),
            );
            match std::fs::write(svg_path, svg) {
                Err(e) => eprintln!("Warning: could not write SVG: {}", e),
                Ok(()) => println!("  Plot saved → {}", svg_path),
            }
        }
    }
    Ok(())
}

fn print_help(prog: &str) {
    println!("Usage:");
    println!("  {} --help           Show this help message", prog);
    println!("  {} <expr> [<a> <b>] Differentiate expr and plot over [a, b] (default: [0, 1])", prog);
    println!();
    println!("Supported syntax:");
    println!("  Variables : any identifier (x, t, u, z, ...)");
    println!("  Constants : pi, e");
    println!("  Operators : + - * / ^ (right-associative)");
    println!("  Functions : sin, cos, tan, exp, ln, log, sqrt");
    println!("  Implicit multiplication: 3x^2, 2t^3, 4sin(u)");
    println!();
    println!("Examples:");
    println!("  {} \"x^3 + sin(x)\" 0 3.14", prog);
    println!("  {} \"exp(x^2)\"", prog);
}

fn parse_bound(s: &str) -> Result<f64, String> {
    s.parse::<f64>().map_err(|_| format!("'{}' is not a valid number", s))
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
        Some("--help") | Some("-h") => print_help(prog),
        Some(_) => {
            let (expr, a, b) = parse_args(args)?;
            run_diff(expr, a, b)?;
        }
        None => print_help(prog),
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run(&args) {
        eprintln!("ERROR: {}", e);
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
    fn test_run_diff_x_squared() {
        assert!(run_diff("x^2", 0.0, 1.0).is_ok());
    }

    #[test]
    fn test_run_diff_abs_unsupported() {
        // abs(x) parses fine but differentiation is unsupported — exercises the Err branch.
        assert!(run_diff("abs(x)", 0.0, 1.0).is_ok());
    }

    #[test]
    fn test_run_diff_parse_error() {
        // Malformed expression — exercises the parse Err return path.
        let result = run_diff("sin(x +", 0.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_print_help_runs() {
        print_help("differentiate");
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
    fn test_run_expr_ok() {
        assert!(run(&make_args(&["prog", "x^2", "0", "3"])).is_ok());
    }

    #[test]
    fn test_run_expr_parse_error() {
        let result = run(&make_args(&["prog", "sin(x +"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_run_expr_wrong_count() {
        let result = run(&make_args(&["prog", "x^2", "0"]));
        assert!(result.is_err());
    }
}
