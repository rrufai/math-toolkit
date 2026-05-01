use integrator_core::{plot, first_var, parse};
use solver_core::solve;

fn run_solve(equation: &str, a: f64, b: f64) -> Result<(), String> {
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
                    "solve.svg",
                    &plot_expr,
                    &var,
                    a,
                    b,
                    &|x| ast.eval(x),
                    None,
                    r.residual,
                ) {
                    Err(e) => eprintln!("Warning: could not write SVG: {}", e),
                    Ok(()) => println!("  Plot saved → solve.svg"),
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
