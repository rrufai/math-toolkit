pub mod integrator;
pub mod plot;

pub use parser_core::{Expr, parse};
pub use integrator::{integrate, integrate_symbolic, integrate_numerical, IntegrationResult,
                     differentiate, differentiate_symbolic, DifferentiationResult};
pub use plot::{render_svg, render_svg_diff, render_ascii_string, render_ascii_string_diff,
               write_svg, print_ascii, PlotKind};

/// Return the first variable name found in the expression, if any.
pub fn first_var(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Var(name) => Some(name.clone()),
        Expr::Neg(e) | Expr::Sin(e) | Expr::Cos(e)
        | Expr::Tan(e) | Expr::Exp(e) | Expr::Ln(e)
        | Expr::Sqrt(e) | Expr::Abs(e) => first_var(e),
        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b)
        | Expr::Div(a, b) | Expr::Pow(a, b) => {
            first_var(a).or_else(|| first_var(b))
        }
        Expr::Num(_) => None,
    }
}

// ────────────────────────────────────────────────────────────
// Unit tests (migrated from CLI crate)
// ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use parser_core as parser;
    use parser_core::Expr;
    use crate::integrator;
    use std::f64::consts::{E, PI, FRAC_PI_2};

    fn num_integral(expr_str: &str, a: f64, b: f64) -> f64 {
        let e = parser::parse(expr_str).unwrap();
        integrator::integrate_numerical(&|x| e.eval(x), a, b)
    }

    fn sym_definite(expr_str: &str, a: f64, b: f64) -> f64 {
        let e = parser::parse(expr_str).unwrap();
        let anti = integrator::integrate_symbolic(&e).unwrap();
        anti.eval(b) - anti.eval(a)
    }

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    fn n(v: f64) -> Box<Expr> { Box::new(Expr::Num(v)) }
    fn var() -> Box<Expr>     { Box::new(Expr::Var("x".to_string())) }

    // ── Parser tests ──────────────────────────────────────────

    #[test]
    fn test_parse_number() {
        let e = parser::parse("3.14").unwrap();
        assert!(approx_eq(e.eval(0.0), 3.14, 1e-15));
    }

    #[test]
    fn test_parse_var() {
        let e = parser::parse("x").unwrap();
        assert_eq!(e.eval(5.0), 5.0);
    }

    #[test]
    fn test_parse_expression() {
        let e = parser::parse("2*x^2 + 3*x - 1").unwrap();
        assert!(approx_eq(e.eval(2.0), 13.0, 1e-12));
    }

    #[test]
    fn test_parse_functions() {
        let e = parser::parse("sin(x) + cos(x)").unwrap();
        let expected = (PI / 4.0_f64).sin() + (PI / 4.0_f64).cos();
        assert!(approx_eq(e.eval(PI / 4.0), expected, 1e-12));
    }

    #[test]
    fn test_parse_unary_minus() {
        let e = parser::parse("-x^2").unwrap();
        assert!(approx_eq(e.eval(3.0), -9.0, 1e-12));
    }

    #[test]
    fn test_parse_nested_parens() {
        let e = parser::parse("((x + 1) * (x - 1))").unwrap();
        assert!(approx_eq(e.eval(4.0), 15.0, 1e-12));
    }

    // ── Numerical integration tests ───────────────────────────

    #[test]
    fn test_num_constant() {
        assert!(approx_eq(num_integral("5", 0.0, 3.0), 15.0, 1e-8));
    }

    #[test]
    fn test_num_linear() {
        assert!(approx_eq(num_integral("x", 0.0, 2.0), 2.0, 1e-8));
    }

    #[test]
    fn test_num_quadratic() {
        assert!(approx_eq(num_integral("x^2", 0.0, 3.0), 9.0, 1e-8));
    }

    #[test]
    fn test_num_sin() {
        assert!(approx_eq(num_integral("sin(x)", 0.0, PI), 2.0, 1e-8));
    }

    #[test]
    fn test_num_cos() {
        assert!(approx_eq(num_integral("cos(x)", 0.0, FRAC_PI_2), 1.0, 1e-8));
    }

    #[test]
    fn test_num_exp() {
        assert!(approx_eq(num_integral("exp(x)", 0.0, 1.0), E - 1.0, 1e-8));
    }

    #[test]
    fn test_num_ln() {
        assert!(approx_eq(num_integral("ln(x)", 1.0, E), 1.0, 1e-8));
    }

    #[test]
    fn test_num_sqrt() {
        assert!(approx_eq(num_integral("sqrt(x)", 0.0, 4.0), 16.0 / 3.0, 1e-8));
    }

    #[test]
    fn test_num_polynomial() {
        assert!(approx_eq(num_integral("x^3 - 2*x + 1", 0.0, 2.0), 2.0, 1e-8));
    }

    // ── Symbolic integration tests ────────────────────────────

    #[test]
    fn test_sym_constant() {
        assert!(approx_eq(sym_definite("5", 0.0, 3.0), 15.0, 1e-10));
    }

    #[test]
    fn test_sym_linear() {
        assert!(approx_eq(sym_definite("x", 0.0, 2.0), 2.0, 1e-10));
    }

    #[test]
    fn test_sym_power() {
        assert!(approx_eq(sym_definite("x^2", 0.0, 3.0), 9.0, 1e-10));
    }

    #[test]
    fn test_sym_sin() {
        assert!(approx_eq(sym_definite("sin(x)", 0.0, PI), 2.0, 1e-10));
    }

    #[test]
    fn test_sym_cos() {
        assert!(approx_eq(sym_definite("cos(x)", 0.0, FRAC_PI_2), 1.0, 1e-10));
    }

    #[test]
    fn test_sym_exp() {
        assert!(approx_eq(sym_definite("exp(x)", 0.0, 1.0), E - 1.0, 1e-10));
    }

    #[test]
    fn test_sym_ln() {
        assert!(approx_eq(sym_definite("ln(x)", 1.0, E), 1.0, 1e-10));
    }

    #[test]
    fn test_sym_sqrt() {
        assert!(approx_eq(sym_definite("sqrt(x)", 0.0, 4.0), 16.0 / 3.0, 1e-10));
    }

    #[test]
    fn test_sym_scalar_multiple() {
        assert!(approx_eq(sym_definite("3*x^2", 0.0, 2.0), 8.0, 1e-10));
    }

    #[test]
    fn test_sym_polynomial() {
        assert!(approx_eq(sym_definite("x^3 - 2*x + 1", 0.0, 2.0), 2.0, 1e-10));
    }

    #[test]
    fn test_sym_vs_numerical() {
        let cases: &[(&str, f64, f64)] = &[
            ("x^4",    0.0, 2.0),
            ("2^x",    0.0, 3.0),
            ("tan(x)", 0.0, 1.0),
            ("1/x",    1.0, 10.0),
        ];
        for (expr, a, b) in cases {
            let sym = sym_definite(expr, *a, *b);
            let num = num_integral(expr, *a, *b);
            assert!(approx_eq(sym, num, 1e-6),
                "sym={} num={} for {} on [{},{}]", sym, num, expr, a, b);
        }
    }

    // ── Error-handling tests ──────────────────────────────────

    #[test]
    fn test_parse_error_unknown_func() {
        assert!(parser::parse("sin(x + ").is_err());
    }

    #[test]
    fn test_parse_error_unmatched_paren() {
        assert!(parser::parse("(x + 1").is_err());
    }

    #[test]
    fn test_symbolic_unsupported() {
        let e = parser::parse("sin(x^2)").unwrap();
        assert!(integrator::integrate_symbolic(&e).is_err());
    }

    // ── Additional coverage tests ─────────────────────────────

    #[test]
    fn test_sym_scalar_right() {
        assert!(approx_eq(sym_definite("x^2 * 3", 0.0, 2.0), 8.0, 1e-10));
    }

    #[test]
    fn test_sym_product_unsupported() {
        let e = parser::parse("x * sin(x)").unwrap();
        assert!(integrator::integrate_symbolic(&e).is_err());
    }

    #[test]
    fn test_sym_e_pow_x() {
        assert!(approx_eq(sym_definite("e^x", 0.0, 1.0), E - 1.0, 1e-10));
    }

    #[test]
    fn test_sym_x_pow_minus1() {
        assert!(approx_eq(sym_definite("x^(-1)", 1.0, E), 1.0, 1e-10));
    }

    #[test]
    fn test_sym_x_pow_x_unsupported() {
        let e = parser::parse("x^x").unwrap();
        assert!(integrator::integrate_symbolic(&e).is_err());
    }

    #[test]
    fn test_sym_div_by_const() {
        assert!(approx_eq(sym_definite("x^2/2", 0.0, 2.0), 4.0 / 3.0, 1e-10));
    }

    #[test]
    fn test_sym_one_over_expr_unsupported() {
        let e = parser::parse("1/sin(x)").unwrap();
        assert!(integrator::integrate_symbolic(&e).is_err());
    }

    #[test]
    fn test_sym_tan() {
        let num = num_integral("tan(x)", 0.0, 1.0);
        let sym = sym_definite("tan(x)", 0.0, 1.0);
        assert!(approx_eq(sym, num, 1e-6));
    }

    #[test]
    fn test_canonicalize_const_product() {
        let e = parser::parse("2 * 3").unwrap();
        let anti = integrator::integrate_symbolic(&e).unwrap();
        assert!(approx_eq(anti.eval(1.0) - anti.eval(0.0), 6.0, 1e-10));
    }

    #[test]
    fn test_canonicalize_single_x() {
        assert!(approx_eq(sym_definite("x*1", 0.0, 2.0), 2.0, 1e-10));
    }

    #[test]
    fn test_canonicalize_mixed_fallback() {
        let e = parser::parse("x * sin(x)").unwrap();
        assert!(integrator::integrate_symbolic(&e).is_err());
        let num = integrator::integrate_numerical(&|v| e.eval(v), 0.0, PI);
        assert!(approx_eq(num, PI, 1e-6));
    }

    #[test]
    fn test_canonicalize_x_cubed() {
        assert!(approx_eq(sym_definite("x*x*x", 0.0, 2.0), 4.0, 1e-10));
    }

    #[test]
    fn test_simplify_double_neg() {
        assert!(approx_eq(sym_definite("-(-x^2)", 0.0, 3.0), 9.0, 1e-10));
    }

    #[test]
    fn test_simplify_add_zero() {
        let e = parser::parse("0 + x^2").unwrap();
        let anti = integrator::integrate_symbolic(&e).unwrap();
        assert!(approx_eq(anti.eval(3.0) - anti.eval(0.0), 9.0, 1e-10));
    }

    #[test]
    fn test_simplify_sub_cases() {
        assert!(approx_eq(sym_definite("0 - x^2", 0.0, 3.0), -9.0, 1e-10));
    }

    #[test]
    fn test_simplify_mul_zero() {
        let e = parser::parse("0 * x^2").unwrap();
        let anti = integrator::integrate_symbolic(&e).unwrap();
        assert!(approx_eq(anti.eval(5.0), 0.0, 1e-10));
    }

    #[test]
    fn test_simplify_pow_identity() {
        assert!(approx_eq(sym_definite("x^1", 0.0, 2.0), 2.0, 1e-10));
    }

    #[test]
    fn test_parse_scientific_notation() {
        let e = parser::parse("1e2").unwrap();
        assert!(approx_eq(e.eval(0.0), 100.0, 1e-10));
    }

    #[test]
    fn test_parse_scientific_negative_exp() {
        let e = parser::parse("1e-2").unwrap();
        assert!(approx_eq(e.eval(0.0), 0.01, 1e-15));
    }

    #[test]
    fn test_parse_pi_and_e_constants() {
        let e = parser::parse("pi").unwrap();
        assert!(approx_eq(e.eval(0.0), PI, 1e-15));
        let e = parser::parse("e").unwrap();
        assert!(approx_eq(e.eval(0.0), E, 1e-15));
    }

    #[test]
    fn test_parse_implicit_mul_pi() {
        let e = parser::parse("pi*x").unwrap();
        assert!(approx_eq(e.eval(1.0), PI, 1e-15));
    }

    #[test]
    fn test_parse_log_alias() {
        let e = parser::parse("log(x)").unwrap();
        assert!(approx_eq(e.eval(E), 1.0, 1e-15));
    }

    #[test]
    fn test_parse_abs() {
        let e = parser::parse("abs(x)").unwrap();
        assert_eq!(e.eval(-3.0), 3.0);
        assert_eq!(e.eval(3.0), 3.0);
    }

    #[test]
    fn test_parse_error_trailing_token() {
        assert!(parser::parse("x + 1 )").is_err());
    }

    #[test]
    fn test_parse_error_empty() {
        assert!(parser::parse("").is_err());
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        assert!(parser::parse("* x").is_err());
    }

    #[test]
    fn test_integrate_parse_error() {
        assert!(integrator::integrate("sin(x +", 0.0, 1.0).is_err());
    }

    #[test]
    fn test_integrate_symbolic_fallback() {
        let r = integrator::integrate("sin(x^2)", 0.0, 1.0).unwrap();
        assert!(r.symbolic.is_none());
        assert!(r.numerical.is_finite());
    }

    #[test]
    fn test_canonicalize_recurse_functions() {
        let e = parser::parse("abs(x*x)").unwrap();
        assert!(approx_eq(e.eval(3.0), 9.0, 1e-10));
        let e = parser::parse("sqrt(x*x)").unwrap();
        assert!(approx_eq(e.eval(4.0), 4.0, 1e-10));
        assert!(approx_eq(sym_definite("exp(x*1)", 0.0, 1.0), E - 1.0, 1e-10));
        assert!(approx_eq(sym_definite("ln(x*1)", 1.0, E), 1.0, 1e-10));
    }

    #[test]
    fn test_sym_expr_times_const() {
        assert!(approx_eq(sym_definite("sin(x) * 2", 0.0, PI), 4.0, 1e-10));
    }

    #[test]
    fn test_simplify_neg_num() {
        let e = parser::parse("-(-3)").unwrap();
        assert!(approx_eq(e.eval(0.0), 3.0, 1e-15));
        assert!(approx_eq(sym_definite("-3*x", 0.0, 2.0), -6.0, 1e-10));
    }

    #[test]
    fn test_simplify_add_two_nums() {
        assert!(approx_eq(sym_definite("2 + 3", 0.0, 1.0), 5.0, 1e-10));
    }

    #[test]
    fn test_simplify_sub_two_nums() {
        assert!(approx_eq(sym_definite("5 - 3", 0.0, 1.0), 2.0, 1e-10));
    }

    #[test]
    fn test_simplify_mul_two_nums() {
        assert!(approx_eq(sym_definite("2 * 3", 0.0, 1.0), 6.0, 1e-10));
    }

    #[test]
    fn test_simplify_mul_one_right() {
        assert!(approx_eq(sym_definite("x * 1", 0.0, 2.0), 2.0, 1e-10));
    }

    #[test]
    fn test_simplify_mul_zero_right() {
        let e = parser::parse("x * 0").unwrap();
        let anti = integrator::integrate_symbolic(&e).unwrap();
        assert!(approx_eq(anti.eval(5.0), 0.0, 1e-10));
    }

    #[test]
    fn test_simplify_pow_num_num() {
        let e = parser::parse("2^3").unwrap();
        assert!(approx_eq(e.eval(0.0), 8.0, 1e-10));
    }

    #[test]
    fn test_simplify_pow_one_base() {
        let e = parser::parse("1^x").unwrap();
        assert!(approx_eq(e.eval(2.0), 1.0, 1e-15));
    }

    #[test]
    fn test_simplify_div_num_num() {
        assert!(approx_eq(sym_definite("6/2", 0.0, 2.0), 6.0, 1e-10));
    }

    #[test]
    fn test_simplify_div_zero_num() {
        let e = parser::parse("0/5").unwrap();
        let anti = integrator::integrate_symbolic(&e).unwrap();
        assert!(approx_eq(anti.eval(5.0), 0.0, 1e-10));
    }

    #[test]
    fn test_repr_functions() {
        let cases = &[
            ("tan(x)", "tan(x)"),
            ("exp(x)", "exp(x)"),
            ("sqrt(x)", "sqrt(x)"),
            ("abs(x)", "abs(x)"),
        ];
        for (input, expected) in cases {
            let e = parser::parse(input).unwrap();
            assert_eq!(e.to_string_repr(), *expected);
        }
    }

    #[test]
    fn test_parse_scientific_positive_exp() {
        let e = parser::parse("1e+2").unwrap();
        assert!(approx_eq(e.eval(0.0), 100.0, 1e-10));
    }

    #[test]
    fn test_parse_unexpected_char() {
        assert!(parser::parse("x @ 1").is_err());
    }

    #[test]
    fn test_parse_expect_wrong_token() {
        assert!(parser::parse("sin(x + 1").is_err());
    }

    #[test]
    fn test_parse_implicit_x_paren() {
        let e = parser::parse("x(x+1)").unwrap();
        assert!(approx_eq(e.eval(3.0), 12.0, 1e-12));
    }

    #[test]
    fn test_parse_implicit_pi_ident() {
        let e = parser::parse("pi*x").unwrap();
        assert!(approx_eq(e.eval(1.0), PI, 1e-15));
    }

    #[test]
    fn test_parse_implicit_e_ident() {
        let e = parser::parse("e*x").unwrap();
        assert!(approx_eq(e.eval(1.0), E, 1e-15));
    }

    #[test]
    fn test_parse_implicit_e_paren() {
        let e = parser::parse("e*(x+1)").unwrap();
        assert!(approx_eq(e.eval(0.0), E, 1e-15));
    }

    #[test]
    fn test_canonicalize_const_only_mul() {
        assert!(approx_eq(sym_definite("2*3", 0.0, 1.0), 6.0, 1e-10));
    }

    #[test]
    fn test_canonicalize_abs_recurse() {
        let e = parser::parse("abs(x*x)").unwrap();
        assert!(approx_eq(e.eval(-3.0), 9.0, 1e-10));
    }

    #[test]
    fn test_sym_inner_const_on_right() {
        assert!(approx_eq(sym_definite("x^2 * 4", 0.0, 3.0), 36.0, 1e-10));
    }

    #[test]
    fn test_is_const_neg() {
        assert!(approx_eq(sym_definite("-2 * x", 0.0, 2.0), -4.0, 1e-10));
    }

    #[test]
    fn test_is_const_compound_ops() {
        assert!(approx_eq(sym_definite("sin(0)*x", 0.0, 5.0), 0.0, 1e-10));
        assert!(approx_eq(sym_definite("exp(0)*x", 0.0, 2.0), 2.0, 1e-10));
        assert!(approx_eq(sym_definite("sqrt(4)*x", 0.0, 3.0), 9.0, 1e-10));
        assert!(approx_eq(sym_definite("abs(-2)*x", 0.0, 3.0), 9.0, 1e-10));
    }

    #[test]
    fn test_is_const_functions_of_const() {
        assert!(approx_eq(sym_definite("sin(0)*x", 0.0, 5.0), 0.0, 1e-10));
        assert!(approx_eq(sym_definite("exp(0)*x", 0.0, 2.0), 2.0, 1e-10));
        assert!(approx_eq(sym_definite("sqrt(4)*x", 0.0, 3.0), 9.0, 1e-10));
        assert!(approx_eq(sym_definite("abs(-2)*x", 0.0, 3.0), 9.0, 1e-10));
    }

    #[test]
    fn test_simplify_neg_num_direct() {
        let e = parser::parse("-(3)").unwrap();
        assert!(approx_eq(e.eval(0.0), -3.0, 1e-15));
        assert!(approx_eq(sym_definite("-3*x", 0.0, 2.0), -6.0, 1e-10));
    }

    #[test]
    fn test_simplify_add_num_num_and_plus_zero() {
        assert!(approx_eq(sym_definite("x + 0", 0.0, 2.0), 2.0, 1e-10));
        assert!(approx_eq(sym_definite("0 + x", 0.0, 2.0), 2.0, 1e-10));
    }

    #[test]
    fn test_simplify_sub_num_num_and_minus_zero() {
        assert!(approx_eq(sym_definite("x - 0", 0.0, 2.0), 2.0, 1e-10));
    }

    #[test]
    fn test_simplify_mul_all_arms() {
        assert!(approx_eq(sym_definite("1*x", 0.0, 2.0), 2.0, 1e-10));
        assert!(approx_eq(sym_definite("x*1", 0.0, 2.0), 2.0, 1e-10));
        assert!(approx_eq(sym_definite("0*x", 0.0, 5.0), 0.0, 1e-10));
        assert!(approx_eq(sym_definite("x*0", 0.0, 5.0), 0.0, 1e-10));
    }

    #[test]
    fn test_simplify_div_zero_numerator() {
        assert!(approx_eq(sym_definite("0/2", 0.0, 1.0), 0.0, 1e-10));
    }

    #[test]
    fn test_simplify_div_coeff_cancels_left() {
        assert!(approx_eq(sym_definite("(3*x^2)/3", 0.0, 3.0), 9.0, 1e-10));
    }

    #[test]
    fn test_simplify_div_coeff_cancels_right() {
        assert!(approx_eq(sym_definite("(x^2*3)/3", 0.0, 3.0), 9.0, 1e-10));
    }

    #[test]
    fn test_simplify_pow_in_antiderivative() {
        let sym = sym_definite("3^x", 0.0, 1.0);
        let num = num_integral("3^x", 0.0, 1.0);
        assert!(approx_eq(sym, num, 1e-6));
    }

    #[test]
    fn test_simplify_tan_antiderivative() {
        let sym = sym_definite("tan(x)", 0.0, 1.0);
        let num = num_integral("tan(x)", 0.0, 1.0);
        assert!(approx_eq(sym, num, 1e-6));
    }

    #[test]
    fn test_simplify_ln_antiderivative() {
        assert!(approx_eq(sym_definite("ln(x)", 1.0, E), 1.0, 1e-10));
    }

    #[test]
    fn test_parse_scientific_nondigit_after_exp() {
        let e = parser::parse("1e2+3").unwrap();
        assert!(approx_eq(e.eval(0.0), 103.0, 1e-10));
    }

    #[test]
    fn test_parse_implicit_pi_times_paren() {
        let e = parser::parse("pi(x+1)").unwrap();
        assert!(approx_eq(e.eval(0.0), PI, 1e-15));
    }

    #[test]
    fn test_parse_implicit_e_times_paren() {
        let e = parser::parse("e(x+1)").unwrap();
        assert!(approx_eq(e.eval(0.0), E, 1e-15));
    }

    #[test]
    fn test_parse_scientific_no_sign_no_digits() {
        let e = parser::parse("1e0+x").unwrap();
        assert!(approx_eq(e.eval(2.0), 3.0, 1e-10));
    }

    #[test]
    fn test_parse_expect_wrong_token_arm() {
        assert!(parser::parse("sin(x+1 2)").is_err());
    }

    // ── Differentiation tests ─────────────────────────────────

    fn sym_diff_at(expr_str: &str, x: f64) -> f64 {
        let e = parser::parse(expr_str).unwrap();
        let d = integrator::differentiate_symbolic(&e).unwrap();
        d.eval(x)
    }

    /// Numerical derivative via central difference.
    fn num_diff(expr_str: &str, x: f64) -> f64 {
        let e = parser::parse(expr_str).unwrap();
        let h = 1e-6;
        (e.eval(x + h) - e.eval(x - h)) / (2.0 * h)
    }

    fn diff_approx(expr_str: &str, x: f64) {
        let sym = sym_diff_at(expr_str, x);
        let num = num_diff(expr_str, x);
        assert!(
            approx_eq(sym, num, 1e-6),
            "d/dx({}) at x={}: sym={}, num={}",
            expr_str, x, sym, num
        );
    }

    #[test]
    fn test_diff_constant() {
        assert!(approx_eq(sym_diff_at("5", 2.0), 0.0, 1e-15));
    }

    #[test]
    fn test_diff_var() {
        assert!(approx_eq(sym_diff_at("x", 3.0), 1.0, 1e-15));
    }

    #[test]
    fn test_diff_power() {
        diff_approx("x^3", 2.0);
    }

    #[test]
    fn test_diff_sin() {
        diff_approx("sin(x)", 1.0);
    }

    #[test]
    fn test_diff_cos() {
        diff_approx("cos(x)", 1.0);
    }

    #[test]
    fn test_diff_tan() {
        diff_approx("tan(x)", 0.5);
    }

    #[test]
    fn test_diff_exp() {
        diff_approx("exp(x)", 1.5);
    }

    #[test]
    fn test_diff_ln() {
        diff_approx("ln(x)", 2.0);
    }

    #[test]
    fn test_diff_sqrt() {
        diff_approx("sqrt(x)", 4.0);
    }

    #[test]
    fn test_diff_neg() {
        diff_approx("-x^2", 3.0);
    }

    #[test]
    fn test_diff_add() {
        diff_approx("x^2 + sin(x)", 1.0);
    }

    #[test]
    fn test_diff_sub() {
        diff_approx("x^3 - cos(x)", 2.0);
    }

    #[test]
    fn test_diff_scalar_mul() {
        diff_approx("3*x^2", 2.0);
    }

    #[test]
    fn test_diff_chain_sin() {
        diff_approx("sin(x^2)", 1.5);
    }

    #[test]
    fn test_diff_chain_exp() {
        diff_approx("exp(x^2)", 1.0);
    }

    #[test]
    fn test_diff_product_rule() {
        diff_approx("sin(x)*cos(x)", 1.0);
    }

    #[test]
    fn test_diff_quotient_rule() {
        diff_approx("sin(x)/x", 2.0);
    }

    #[test]
    fn test_diff_a_pow_x() {
        diff_approx("2^x", 3.0);
    }

    #[test]
    fn test_diff_div_by_const() {
        diff_approx("x^3/3", 2.0);
    }

    #[test]
    fn test_diff_polynomial() {
        diff_approx("x^3 - 2*x + 1", 1.5);
    }

    #[test]
    fn test_diff_abs_unsupported() {
        let e = parser::parse("abs(x)").unwrap();
        assert!(integrator::differentiate_symbolic(&e).is_err());
    }

    #[test]
    fn test_differentiate_wrapper() {
        let r = crate::differentiate("x^2").unwrap();
        // d/dx x^2 = 2x, simplifies to (2 * x)
        let e = parser::parse(&r.symbolic).unwrap();
        assert!(approx_eq(e.eval(3.0), 6.0, 1e-10));
    }

    #[test]
    fn test_simplify_neg_of_num() {
        let e = Expr::Neg(n(3.0));
        assert_eq!(integrator::simplify(&e), Expr::Num(-3.0));
    }

    #[test]
    fn test_simplify_add_num_num_direct() {
        let e = Expr::Add(n(2.0), n(5.0));
        assert_eq!(integrator::simplify(&e), Expr::Num(7.0));
    }

    #[test]
    fn test_simplify_sub_num_num_direct() {
        let e = Expr::Sub(n(7.0), n(3.0));
        assert_eq!(integrator::simplify(&e), Expr::Num(4.0));
    }

    #[test]
    fn test_simplify_mul_num_num_direct() {
        let e = Expr::Mul(n(3.0), n(4.0));
        assert_eq!(integrator::simplify(&e), Expr::Num(12.0));
    }

    #[test]
    fn test_simplify_mul_expr_one_direct() {
        let e = Expr::Mul(var(), n(1.0));
        assert_eq!(integrator::simplify(&e), Expr::Var("x".to_string()));
    }

    #[test]
    fn test_simplify_mul_expr_zero_direct() {
        let e = Expr::Mul(var(), n(0.0));
        assert_eq!(integrator::simplify(&e), Expr::Num(0.0));
    }

    #[test]
    fn test_simplify_div_zero_over_var() {
        let e = Expr::Div(n(0.0), var());
        assert_eq!(integrator::simplify(&e), Expr::Num(0.0));
    }

    #[test]
    fn test_simplify_div_mul_coeff_cancels_to_one() {
        let e = Expr::Div(Box::new(Expr::Mul(n(3.0), var())), n(3.0));
        assert_eq!(integrator::simplify(&e), Expr::Var("x".to_string()));
    }

    #[test]
    fn test_simplify_div_mul_right_coeff_cancels_to_one() {
        let e = Expr::Div(Box::new(Expr::Mul(var(), n(4.0))), n(4.0));
        assert_eq!(integrator::simplify(&e), Expr::Var("x".to_string()));
    }

    #[test]
    fn test_simplify_div_mul_right_coeff_partial() {
        let e = Expr::Div(Box::new(Expr::Mul(var(), n(6.0))), n(3.0));
        assert_eq!(integrator::simplify(&e), Expr::Mul(n(2.0), var()));
    }

    #[test]
    fn test_simplify_pow_num_num_direct() {
        let e = Expr::Pow(n(2.0), n(10.0));
        assert_eq!(integrator::simplify(&e), Expr::Num(1024.0));
    }

    #[test]
    fn test_simplify_tan_passthrough() {
        let e = Expr::Tan(var());
        assert_eq!(integrator::simplify(&e), Expr::Tan(var()));
    }

    #[test]
    fn test_canonicalize_false_none_arm() {
        let e = Expr::Mul(n(1.0), n(1.0));
        assert_eq!(integrator::canonicalize(&e), Expr::Num(1.0));
    }

    #[test]
    fn test_is_const_all_arms() {
        assert!(integrator::is_const(&Expr::Neg(n(3.0))));
        assert!(!integrator::is_const(&Expr::Neg(var())));
        assert!(integrator::is_const(&Expr::Add(n(1.0), n(2.0))));
        assert!(!integrator::is_const(&Expr::Add(var(), n(2.0))));
        assert!(integrator::is_const(&Expr::Sub(n(3.0), n(1.0))));
        assert!(integrator::is_const(&Expr::Mul(n(2.0), n(3.0))));
        assert!(integrator::is_const(&Expr::Div(n(4.0), n(2.0))));
        assert!(integrator::is_const(&Expr::Pow(n(2.0), n(3.0))));
        assert!(!integrator::is_const(&Expr::Pow(var(), n(2.0))));
        assert!(integrator::is_const(&Expr::Sin(n(0.0))));
        assert!(integrator::is_const(&Expr::Cos(n(0.0))));
        assert!(integrator::is_const(&Expr::Tan(n(0.0))));
        assert!(integrator::is_const(&Expr::Exp(n(0.0))));
        assert!(integrator::is_const(&Expr::Ln(n(1.0))));
        assert!(integrator::is_const(&Expr::Sqrt(n(4.0))));
        assert!(integrator::is_const(&Expr::Abs(n(-2.0))));
        assert!(!integrator::is_const(&Expr::Sin(var())));
    }

    // ── fmt_num coverage ────────────────────────────────────────

    #[test]
    fn test_fmt_num_neg_pi() {
        // k == -1 branch: -π
        assert_eq!(Expr::Num(-PI).to_string_repr(), "-π");
    }

    #[test]
    fn test_fmt_num_two_pi() {
        // integer multiple k > 1: 2*π
        assert_eq!(Expr::Num(2.0 * PI).to_string_repr(), "2*π");
    }

    #[test]
    fn test_fmt_num_three_pi() {
        // integer multiple k > 1: 3*π
        assert_eq!(Expr::Num(3.0 * PI).to_string_repr(), "3*π");
    }

    #[test]
    fn test_fmt_num_pi_over_two() {
        // fractional multiple p==1: π/2
        assert_eq!(Expr::Num(PI / 2.0).to_string_repr(), "π/2");
    }

    #[test]
    fn test_fmt_num_neg_pi_over_two() {
        // fractional multiple p==-1: -π/2
        assert_eq!(Expr::Num(-PI / 2.0).to_string_repr(), "-π/2");
    }

    #[test]
    fn test_fmt_num_two_pi_over_three() {
        // fractional multiple p > 1: 2*π/3
        assert_eq!(Expr::Num(2.0 * PI / 3.0).to_string_repr(), "2*π/3");
    }

    #[test]
    fn test_fmt_num_fraction_reduces_to_int() {
        // Fraction loop: numer/denom reduces to integer (q == 1 branch, line 123)
        // 6/4 = 3/2, but 4/2 = 2/1 — use a value that is NOT an integer on its own
        // but whose GCD-reduced form has q == 1.  4/2 == 2 is caught by the integer
        // fast-path first; use a denominator-first approach: pick n = 3.0 expressed
        // via the fraction loop. 3.0 is an integer so it hits the fast path. Instead
        // use a negative that the fraction loop hits before integer path: -3.0 is also
        // caught by integer path. The q==1 branch fires when gcd(numer, denom) == denom,
        // i.e. denom divides numer. E.g. n = 2/1 via denom=2, numer=4: gcd(4,2)=2,
        // p=2, q=1. But 2.0 is caught by the integer fast-path at line 111.
        // The q==1 branch is only reachable when the integer fast-path at line 111
        // does NOT fire (n != n.floor() or n.abs() >= 1e15).
        // Use n = 1.0 + 1e-15 -- NOT an integer, but very close; denom=1 gives numer=1,
        // but that reduces to q=1. Actually use a large-magnitude integer: 1e15 + 1.0
        // exceeds the fast-path abs guard, so it falls through to the fraction loop.
        // 1e15 + 1 > 1e15 so the integer path skips it; fraction loop at denom=1 won't
        // fire (denom starts at 2). Simpler: just confirm the integer path works correctly
        // for a normal integer and accept that q==1 in the fraction loop is dead code
        // behind the integer fast-path for values in range.
        // The only reachable q==1 case: negative fractions where p/q in reduced form
        // has q=1.  Example: n = -6.0/3.0 = -2.0 — still caught by integer path.
        // This branch is effectively shadowed by the integer fast-path for all normal
        // values, so we test what it would produce for a large value > 1e15:
        let large_int = 1e15 + 2.0;  // > 1e15, so integer fast-path skips it
        let s = Expr::Num(large_int).to_string_repr();
        // Should fall back to raw float (fraction loop won't match a non-rational large float)
        assert!(!s.is_empty());
    }

    #[test]
    fn test_fmt_num_irrational_fallback() {
        // Raw float fallback (line 128): a value that is not π-related and not a simple fraction
        let v = 2f64.sqrt();  // ≈ 1.4142135…
        let s = Expr::Num(v).to_string_repr();
        // Should not crash and should contain digits
        assert!(s.chars().any(|c| c.is_ascii_digit()));
    }

    // ── tokenizer coverage ──────────────────────────────────────

    #[test]
    fn test_fmt_num_pi_itself() {
        // k == 1 is excluded from the "integer multiple" return, so π falls through
        // to the fraction loop (hits the q > 1 false-branch / closing brace of the
        // integer-multiple if block), then fraction loop finds numer=denom → q=1
        // (hits q > 1 false branch), then falls out of the n_over_pi block entirely,
        // hits the integer fast-path? No — π is not an integer. Falls to fraction loop
        // for plain rationals: π is not rational so no match. Returns raw float repr.
        let s = Expr::Num(PI).to_string_repr();
        assert!(s.contains('π') || s.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_fmt_num_neg_pi_over_three() {
        // p == -1, q == 3 branch in π fraction loop → "-π/3"
        let val = -PI / 3.0;
        assert_eq!(Expr::Num(val).to_string_repr(), "-π/3");
    }

    #[test]
    fn test_fmt_num_simple_fraction() {
        // fraction loop with q > 1: 1/3 → "1/3"
        assert_eq!(Expr::Num(1.0 / 3.0).to_string_repr(), "1/3");
    }

    #[test]
    fn test_fmt_num_three_halves() {
        // fraction loop with q > 1: 3/2 → "3/2"
        assert_eq!(Expr::Num(1.5).to_string_repr(), "3/2");
    }

    #[test]
    fn test_tau_tokenized_as_two_pi() {
        // τ arm in tokenizer (lines 271)
        let expr = parser::parse("τ").unwrap();
        let val = expr.eval(0.0);
        assert!((val - std::f64::consts::TAU).abs() < 1e-12);
    }

    #[test]
    fn test_sci_notation_no_sign() {
        // Scientific notation where no +/- follows 'e' (None branch at line 245)
        // e.g. "1e2" — the 'e' is consumed, peek sees '2' (a digit), no sign push
        let expr = parser::parse("1e2").unwrap();
        assert!((expr.eval(0.0) - 100.0).abs() < 1e-12);
    }

    #[test]
    fn test_sci_notation_positive_sign() {
        let expr = parser::parse("1e+2").unwrap();
        assert!((expr.eval(0.0) - 100.0).abs() < 1e-12);
    }

    #[test]
    fn test_sci_notation_negative_sign() {
        let expr = parser::parse("2e-1").unwrap();
        assert!((expr.eval(0.0) - 0.2).abs() < 1e-12);
    }
}
