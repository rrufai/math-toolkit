//! Symbolic and numerical integration of `Expr` ASTs.

use parser_core::Expr;

// ============================================================
// Symbolic integration
// ============================================================

/// Attempt to compute a symbolic antiderivative of `expr` with respect to its variable.
///
/// Returns `Ok(antiderivative)` when a closed-form result is found,
/// or `Err(reason)` when the pattern is not (yet) supported.
pub fn integrate_symbolic(expr: &Expr) -> Result<Expr, String> {
    let expr = canonicalize(expr);
    let var = var_name_of(&expr);
    let result = integrate_sym_inner(&expr, &var)?;
    Ok(simplify(&result))
}

/// Walk the AST and return the first variable name found; fall back to "x".
fn var_name_of(expr: &Expr) -> String {
    match expr {
        Expr::Var(n) => n.clone(),
        Expr::Neg(e) | Expr::Sin(e) | Expr::Cos(e) | Expr::Tan(e)
        | Expr::Exp(e) | Expr::Ln(e) | Expr::Sqrt(e) | Expr::Abs(e) => var_name_of(e),
        Expr::Add(a, b) | Expr::Sub(a, b) | Expr::Mul(a, b)
        | Expr::Div(a, b) | Expr::Pow(a, b) => {
            let from_a = var_name_of(a);
            if from_a != "x" { return from_a; } // found non-default
            // also check b in case a was pure-constant fallback
            let from_b = var_name_of(b);
            if from_b != "x" { from_b } else { from_a }
        }
        Expr::Num(_) => "x".to_string(), // pure constant, no variable present
    }
}

/// Shorthand: construct a `Var` node with the given name.
fn v(name: &str) -> Expr {
    Expr::Var(name.to_string())
}

/// Flatten a `Mul` tree into its leaf factors.
fn collect_mul_factors(expr: &Expr, factors: &mut Vec<Expr>) {
    match expr {
        Expr::Mul(a, b) => {
            collect_mul_factors(a, factors);
            collect_mul_factors(b, factors);
        }
        other => factors.push(canonicalize(other)),
    }
}

/// Rewrite repeated variable products into power form, e.g. `t*t*t` → `t^3`,
/// `4*t*t` → `4*t^2`. Recurses into all nodes.
pub(crate) fn canonicalize(expr: &Expr) -> Expr {
    match expr {
        Expr::Mul(_, _) => {
            let mut factors = Vec::new();
            collect_mul_factors(expr, &mut factors);

            let mut coeff = 1.0_f64;
            let mut var_count = 0u32;
            let mut var_name: Option<String> = None;
            let mut other: Vec<Expr> = Vec::new();

            for f in factors {
                match f {
                    Expr::Num(n)    => coeff *= n,
                    Expr::Var(ref name) => {
                        var_count += 1;
                        if var_name.is_none() {
                            var_name = Some(name.clone());
                        }
                    }
                    e => other.push(e),
                }
            }

            // Only simplify when there are no leftover non-var, non-const factors
            if other.is_empty() {
                let vn = var_name.unwrap_or_else(|| "x".to_string());
                let x_term = match var_count {
                    0 => None,
                    1 => Some(Expr::Var(vn)),
                    n => Some(Expr::Pow(Box::new(Expr::Var(vn)), Box::new(Expr::Num(n as f64)))),
                };
                let has_coeff = coeff != 1.0;
                match (has_coeff, x_term) {
                    (false, None)        => Expr::Num(coeff), // shouldn't happen but safe
                    (false, Some(x))     => x,
                    (true,  None)        => Expr::Num(coeff),
                    (true,  Some(x))     => Expr::Mul(Box::new(Expr::Num(coeff)), Box::new(x)),
                }
            } else {
                // Can't simplify — rebuild as left-associative chain of all factors,
                // putting the coefficient first if present
                let vn = var_name.unwrap_or_else(|| "x".to_string());
                let mut all: Vec<Expr> = Vec::new();
                if coeff != 1.0 { all.push(Expr::Num(coeff)); }
                for _ in 0..var_count { all.push(Expr::Var(vn.clone())); }
                all.extend(other);
                all.into_iter().reduce(|a, b| Expr::Mul(Box::new(a), Box::new(b))).unwrap()
            }
        }
        // Recurse into every other node
        Expr::Neg(e)    => Expr::Neg(Box::new(canonicalize(e))),
        Expr::Add(a, b) => Expr::Add(Box::new(canonicalize(a)), Box::new(canonicalize(b))),
        Expr::Sub(a, b) => Expr::Sub(Box::new(canonicalize(a)), Box::new(canonicalize(b))),
        Expr::Div(a, b) => Expr::Div(Box::new(canonicalize(a)), Box::new(canonicalize(b))),
        Expr::Pow(a, b) => Expr::Pow(Box::new(canonicalize(a)), Box::new(canonicalize(b))),
        Expr::Sin(e)    => Expr::Sin(Box::new(canonicalize(e))),
        Expr::Cos(e)    => Expr::Cos(Box::new(canonicalize(e))),
        Expr::Tan(e)    => Expr::Tan(Box::new(canonicalize(e))),
        Expr::Exp(e)    => Expr::Exp(Box::new(canonicalize(e))),
        Expr::Ln(e)     => Expr::Ln(Box::new(canonicalize(e))),
        Expr::Sqrt(e)   => Expr::Sqrt(Box::new(canonicalize(e))),
        Expr::Abs(e)    => Expr::Abs(Box::new(canonicalize(e))),
        leaf            => leaf.clone(),
    }
}

fn integrate_sym_inner(expr: &Expr, var: &str) -> Result<Expr, String> {
    match expr {
        // ── Constants ──────────────────────────────────────────
        // ∫ c dx = c·x
        Expr::Num(c) => Ok(Expr::Mul(
            Box::new(Expr::Num(*c)),
            Box::new(v(var)),
        )),

        // ── Variable ───────────────────────────────────────────
        // ∫ x dx = x²/2
        Expr::Var(_) => Ok(Expr::Div(
            Box::new(Expr::Pow(Box::new(v(var)), Box::new(Expr::Num(2.0)))),
            Box::new(Expr::Num(2.0)),
        )),

        // ── Negation ───────────────────────────────────────────
        // ∫ -f dx = -(∫ f dx)
        Expr::Neg(inner) => {
            let fi = integrate_sym_inner(inner, var)?;
            Ok(Expr::Neg(Box::new(fi)))
        }

        // ── Sum / Difference ───────────────────────────────────
        // ∫ (f ± g) dx = ∫ f dx ± ∫ g dx
        Expr::Add(a, b) => Ok(Expr::Add(
            Box::new(integrate_sym_inner(a, var)?),
            Box::new(integrate_sym_inner(b, var)?),
        )),
        Expr::Sub(a, b) => Ok(Expr::Sub(
            Box::new(integrate_sym_inner(a, var)?),
            Box::new(integrate_sym_inner(b, var)?),
        )),

        // ── Scalar multiples ───────────────────────────────────
        // ∫ c·f dx = c · ∫ f dx   (c constant, f any)
        // ∫ f·c dx = c · ∫ f dx
        Expr::Mul(a, b) => {
            if is_const(a) {
                let fi = integrate_sym_inner(b, var)?;
                return Ok(Expr::Mul(a.clone(), Box::new(fi)));
            }
            if is_const(b) {
                let fi = integrate_sym_inner(a, var)?;
                return Ok(Expr::Mul(b.clone(), Box::new(fi)));
            }
            Err(format!(
                "Cannot symbolically integrate product of two non-constant factors: {}",
                expr.to_string_repr()
            ))
        }

        // ── Division by constant ───────────────────────────────
        // ∫ f/c dx = (1/c) · ∫ f dx
        Expr::Div(num, den) if is_const(den) => {
            let fi = integrate_sym_inner(num, var)?;
            Ok(Expr::Div(Box::new(fi), den.clone()))
        }

        // ∫ 1/x dx = ln(x)
        Expr::Div(num, den) if matches!(num.as_ref(), Expr::Num(n) if *n == 1.0) => {
            if matches!(den.as_ref(), Expr::Var(_)) {
                return Ok(Expr::Ln(Box::new(v(var))));
            }
            Err(format!(
                "Cannot symbolically integrate: {}",
                expr.to_string_repr()
            ))
        }

        // ── Power rule ─────────────────────────────────────────
        // ∫ x^n dx = x^(n+1) / (n+1)   [n ≠ -1]
        // ∫ x^(-1) dx = ln(x)
        Expr::Pow(base, exp) if matches!(base.as_ref(), Expr::Var(_)) => {
            match exp.as_ref() {
                Expr::Num(n) if *n == -1.0 => Ok(Expr::Ln(Box::new(v(var)))),
                Expr::Num(n) => {
                    let new_exp = n + 1.0;
                    Ok(Expr::Div(
                        Box::new(Expr::Pow(
                            Box::new(v(var)),
                            Box::new(Expr::Num(new_exp)),
                        )),
                        Box::new(Expr::Num(new_exp)),
                    ))
                }
                _ => Err(format!(
                    "Cannot symbolically integrate x^(non-constant): {}",
                    expr.to_string_repr()
                )),
            }
        }

        // ── Trigonometric ──────────────────────────────────────
        // ∫ sin(x) dx = -cos(x)
        Expr::Sin(arg) if matches!(arg.as_ref(), Expr::Var(_)) => {
            Ok(Expr::Neg(Box::new(Expr::Cos(Box::new(v(var))))))
        }

        // ∫ cos(x) dx = sin(x)
        Expr::Cos(arg) if matches!(arg.as_ref(), Expr::Var(_)) => {
            Ok(Expr::Sin(Box::new(v(var))))
        }

        // ∫ tan(x) dx = -ln|cos(x)|
        Expr::Tan(arg) if matches!(arg.as_ref(), Expr::Var(_)) => {
            Ok(Expr::Neg(Box::new(Expr::Ln(Box::new(Expr::Abs(Box::new(
                Expr::Cos(Box::new(v(var))),
            )))))))
        }

        // ── Exponential / Logarithm ────────────────────────────
        // ∫ exp(x) dx = exp(x)
        Expr::Exp(arg) if matches!(arg.as_ref(), Expr::Var(_)) => {
            Ok(Expr::Exp(Box::new(v(var))))
        }

        // ∫ e^x = exp(x)  (alias)
        Expr::Pow(base, exp)
            if matches!(base.as_ref(), Expr::Num(n) if (*n - std::f64::consts::E).abs() < 1e-15)
                && matches!(exp.as_ref(), Expr::Var(_)) =>
        {
            Ok(Expr::Exp(Box::new(v(var))))
        }

        // ∫ a^x dx = a^x / ln(a)   [a > 0, a ≠ 1]
        Expr::Pow(base, exp)
            if is_const(base) && matches!(exp.as_ref(), Expr::Var(_)) =>
        {
            Ok(Expr::Div(
                Box::new(Expr::Pow(base.clone(), Box::new(v(var)))),
                Box::new(Expr::Ln(base.clone())),
            ))
        }

        // ∫ ln(x) dx = x·ln(x) - x
        Expr::Ln(arg) if matches!(arg.as_ref(), Expr::Var(_)) => {
            Ok(Expr::Sub(
                Box::new(Expr::Mul(
                    Box::new(v(var)),
                    Box::new(Expr::Ln(Box::new(v(var)))),
                )),
                Box::new(v(var)),
            ))
        }

        // ── Square root ────────────────────────────────────────
        // ∫ sqrt(x) dx = (2/3)·x^(3/2)
        Expr::Sqrt(arg) if matches!(arg.as_ref(), Expr::Var(_)) => {
            Ok(Expr::Mul(
                Box::new(Expr::Div(
                    Box::new(Expr::Num(2.0)),
                    Box::new(Expr::Num(3.0)),
                )),
                Box::new(Expr::Pow(
                    Box::new(v(var)),
                    Box::new(Expr::Num(1.5)),
                )),
            ))
        }

        _ => Err(format!(
            "Cannot symbolically integrate: {}",
            expr.to_string_repr()
        )),
    }
}

/// Returns `true` iff the expression is free of any variable (i.e. a constant).
pub(crate) fn is_const(e: &Expr) -> bool {
    match e {
        Expr::Num(_)    => true,
        Expr::Var(_)    => false,
        Expr::Neg(a)    => is_const(a),
        Expr::Add(a, b)
        | Expr::Sub(a, b)
        | Expr::Mul(a, b)
        | Expr::Div(a, b)
        | Expr::Pow(a, b) => is_const(a) && is_const(b),
        Expr::Sin(a)
        | Expr::Cos(a)
        | Expr::Tan(a)
        | Expr::Exp(a)
        | Expr::Ln(a)
        | Expr::Sqrt(a)
        | Expr::Abs(a)  => is_const(a),
    }
}

/// Basic constant-folding / algebraic simplification pass.
pub(crate) fn simplify(expr: &Expr) -> Expr {
    match expr {
        Expr::Num(n)     => Expr::Num(*n),
        Expr::Var(name)  => Expr::Var(name.clone()),

        Expr::Neg(e) => {
            let e = simplify(e);
            match e {
                Expr::Num(n) => Expr::Num(-n),
                Expr::Neg(inner) => *inner,
                other => Expr::Neg(Box::new(other)),
            }
        }

        Expr::Add(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x + y),
                (Expr::Num(x), _) if *x == 0.0 => b,
                (_, Expr::Num(y)) if *y == 0.0 => a,
                _ => Expr::Add(Box::new(a), Box::new(b)),
            }
        }

        Expr::Sub(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x - y),
                (_, Expr::Num(y)) if *y == 0.0 => a,
                (Expr::Num(x), _) if *x == 0.0 => Expr::Neg(Box::new(b)),
                _ => Expr::Sub(Box::new(a), Box::new(b)),
            }
        }

        Expr::Mul(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x * y),
                (Expr::Num(x), _) if *x == 1.0 => b,
                (_, Expr::Num(y)) if *y == 1.0 => a,
                (Expr::Num(x), _) if *x == 0.0 => Expr::Num(0.0),
                (_, Expr::Num(y)) if *y == 0.0 => Expr::Num(0.0),
                // c * (expr / d)  →  (c/d) * expr, dropping coefficient if it is 1
                (Expr::Num(c), Expr::Div(inner, denom)) if matches!(denom.as_ref(), Expr::Num(_)) => {
                    let Expr::Num(d) = denom.as_ref() else { unreachable!() };
                    let coeff = c / d;
                    let inner = inner.as_ref().clone();
                    if coeff == 1.0 {
                        inner
                    } else {
                        Expr::Mul(Box::new(Expr::Num(coeff)), Box::new(inner))
                    }
                }
                // c * (n * expr)  →  (c*n) * expr  (merge adjacent numeric coefficients)
                (Expr::Num(c), Expr::Mul(n, inner)) if matches!(n.as_ref(), Expr::Num(_)) => {
                    let Expr::Num(n) = n.as_ref() else { unreachable!() };
                    let coeff = c * n;
                    let inner = inner.as_ref().clone();
                    if coeff == 1.0 {
                        inner
                    } else {
                        Expr::Mul(Box::new(Expr::Num(coeff)), Box::new(inner))
                    }
                }
                _ => Expr::Mul(Box::new(a), Box::new(b)),
            }
        }

        Expr::Div(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) if *y != 0.0 => Expr::Num(x / y),
                (_, Expr::Num(y)) if *y == 1.0 => a,
                (Expr::Num(x), _) if *x == 0.0 => Expr::Num(0.0),
                // c * expr / d  →  (c/d) * expr, dropping coefficient if it is 1
                (Expr::Mul(ca, inner), Expr::Num(d)) if matches!(ca.as_ref(), Expr::Num(_)) => {
                    let Expr::Num(c) = ca.as_ref() else { unreachable!() };
                    let coeff = c / d;
                    let inner = inner.as_ref().clone();
                    if coeff == 1.0 {
                        inner
                    } else {
                        Expr::Mul(Box::new(Expr::Num(coeff)), Box::new(inner))
                    }
                }
                // expr * c / d  →  (c/d) * expr, dropping coefficient if it is 1
                (Expr::Mul(inner, cb), Expr::Num(d)) if matches!(cb.as_ref(), Expr::Num(_)) => {
                    let Expr::Num(c) = cb.as_ref() else { unreachable!() };
                    let coeff = c / d;
                    let inner = inner.as_ref().clone();
                    if coeff == 1.0 {
                        inner
                    } else {
                        Expr::Mul(Box::new(Expr::Num(coeff)), Box::new(inner))
                    }
                }
                _ => Expr::Div(Box::new(a), Box::new(b)),
            }
        }

        Expr::Pow(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (Expr::Num(x), Expr::Num(y)) => Expr::Num(x.powf(*y)),
                (_, Expr::Num(y)) if *y == 1.0 => a,
                (_, Expr::Num(y)) if *y == 0.0 => Expr::Num(1.0),
                (Expr::Num(x), _) if *x == 1.0 => Expr::Num(1.0),
                _ => Expr::Pow(Box::new(a), Box::new(b)),
            }
        }

        Expr::Sin(e)  => Expr::Sin(Box::new(simplify(e))),
        Expr::Cos(e)  => Expr::Cos(Box::new(simplify(e))),
        Expr::Tan(e)  => Expr::Tan(Box::new(simplify(e))),
        Expr::Exp(e)  => Expr::Exp(Box::new(simplify(e))),
        Expr::Ln(e)   => Expr::Ln(Box::new(simplify(e))),
        Expr::Sqrt(e) => Expr::Sqrt(Box::new(simplify(e))),
        Expr::Abs(e)  => Expr::Abs(Box::new(simplify(e))),
    }
}

// ============================================================
// Symbolic differentiation
// ============================================================

/// Compute the symbolic derivative of `expr` with respect to its variable.
///
/// Supports all direct forms including chain rule for composite arguments.
/// Returns `Ok(derivative)` always (differentiation is always possible for
/// supported AST nodes), or `Err` for unsupported nodes like `Abs`.
pub fn differentiate_symbolic(expr: &Expr) -> Result<Expr, String> {
    let expr = canonicalize(expr);
    let var = var_name_of(&expr);
    let result = diff_inner(&expr, &var)?;
    Ok(simplify(&result))
}

/// Differentiate `expr` with respect to `var`.
fn diff_inner(expr: &Expr, var: &str) -> Result<Expr, String> {
    match expr {
        // d/dx c = 0
        Expr::Num(_) => Ok(Expr::Num(0.0)),

        // d/dx x = 1,  d/dx y = 0  (other variable treated as constant)
        Expr::Var(name) => {
            if name == var {
                Ok(Expr::Num(1.0))
            } else {
                Ok(Expr::Num(0.0))
            }
        }

        // d/dx (-f) = -(f')
        Expr::Neg(inner) => {
            let di = diff_inner(inner, var)?;
            Ok(Expr::Neg(Box::new(di)))
        }

        // d/dx (f + g) = f' + g'
        Expr::Add(a, b) => Ok(Expr::Add(
            Box::new(diff_inner(a, var)?),
            Box::new(diff_inner(b, var)?),
        )),

        // d/dx (f - g) = f' - g'
        Expr::Sub(a, b) => Ok(Expr::Sub(
            Box::new(diff_inner(a, var)?),
            Box::new(diff_inner(b, var)?),
        )),

        // d/dx (f * g) — scalar-multiple fast path, then product rule
        Expr::Mul(a, b) => {
            if is_const(a) {
                // d/dx (c * f) = c * f'
                let db = diff_inner(b, var)?;
                return Ok(Expr::Mul(a.clone(), Box::new(db)));
            }
            if is_const(b) {
                // d/dx (f * c) = c * f'
                let da = diff_inner(a, var)?;
                return Ok(Expr::Mul(b.clone(), Box::new(da)));
            }
            // General product rule: (f*g)' = f'*g + f*g'
            let da = diff_inner(a, var)?;
            let db = diff_inner(b, var)?;
            Ok(Expr::Add(
                Box::new(Expr::Mul(Box::new(da), b.clone())),
                Box::new(Expr::Mul(a.clone(), Box::new(db))),
            ))
        }

        // d/dx (f / g) — constant-denominator fast path, then quotient rule
        Expr::Div(num, den) if is_const(den) => {
            // d/dx (f / c) = f' / c
            let dn = diff_inner(num, var)?;
            Ok(Expr::Div(Box::new(dn), den.clone()))
        }
        Expr::Div(num, den) => {
            // Quotient rule: (f/g)' = (f'*g - f*g') / g^2
            let df = diff_inner(num, var)?;
            let dg = diff_inner(den, var)?;
            Ok(Expr::Div(
                Box::new(Expr::Sub(
                    Box::new(Expr::Mul(Box::new(df), den.clone())),
                    Box::new(Expr::Mul(num.clone(), Box::new(dg))),
                )),
                Box::new(Expr::Pow(den.clone(), Box::new(Expr::Num(2.0)))),
            ))
        }

        // Power rule with chain rule: d/dx f^n = n * f^(n-1) * f'
        Expr::Pow(base, exp) => {
            match (base.as_ref(), exp.as_ref()) {
                // d/dx x^n = n * x^(n-1)
                (_, Expr::Num(n)) => {
                    let db = diff_inner(base, var)?;
                    let new_exp = n - 1.0;
                    let power = Expr::Mul(
                        Box::new(Expr::Num(*n)),
                        Box::new(Expr::Pow(base.clone(), Box::new(Expr::Num(new_exp)))),
                    );
                    Ok(Expr::Mul(Box::new(power), Box::new(db)))
                }
                // d/dx a^x = a^x * ln(a)   (a constant)
                (_, _) if is_const(base) => {
                    let de = diff_inner(exp, var)?;
                    Ok(Expr::Mul(
                        Box::new(Expr::Mul(
                            Box::new(Expr::Pow(base.clone(), exp.clone())),
                            Box::new(Expr::Ln(base.clone())),
                        )),
                        Box::new(de),
                    ))
                }
                _ => Err(format!(
                    "Cannot differentiate: {}",
                    expr.to_string_repr()
                )),
            }
        }

        // d/dx sin(f) = cos(f) * f'  (chain rule)
        Expr::Sin(inner) => {
            let di = diff_inner(inner, var)?;
            Ok(Expr::Mul(
                Box::new(Expr::Cos(inner.clone())),
                Box::new(di),
            ))
        }

        // d/dx cos(f) = -sin(f) * f'
        Expr::Cos(inner) => {
            let di = diff_inner(inner, var)?;
            Ok(Expr::Neg(Box::new(Expr::Mul(
                Box::new(Expr::Sin(inner.clone())),
                Box::new(di),
            ))))
        }

        // d/dx tan(f) = f' / cos²(f)
        Expr::Tan(inner) => {
            let di = diff_inner(inner, var)?;
            Ok(Expr::Div(
                Box::new(di),
                Box::new(Expr::Pow(
                    Box::new(Expr::Cos(inner.clone())),
                    Box::new(Expr::Num(2.0)),
                )),
            ))
        }

        // d/dx exp(f) = exp(f) * f'
        Expr::Exp(inner) => {
            let di = diff_inner(inner, var)?;
            Ok(Expr::Mul(
                Box::new(Expr::Exp(inner.clone())),
                Box::new(di),
            ))
        }

        // d/dx ln(f) = f' / f
        Expr::Ln(inner) => {
            let di = diff_inner(inner, var)?;
            Ok(Expr::Div(Box::new(di), inner.clone()))
        }

        // d/dx sqrt(f) = f' / (2 * sqrt(f))
        Expr::Sqrt(inner) => {
            let di = diff_inner(inner, var)?;
            Ok(Expr::Div(
                Box::new(di),
                Box::new(Expr::Mul(
                    Box::new(Expr::Num(2.0)),
                    Box::new(Expr::Sqrt(inner.clone())),
                )),
            ))
        }

        Expr::Abs(_) => Err(format!(
            "Cannot differentiate abs() symbolically (not differentiable at 0): {}",
            expr.to_string_repr()
        )),
    }
}

/// Result returned by the high-level `differentiate` function.
pub struct DifferentiationResult {
    /// Symbolic derivative string.
    pub symbolic: String,
}

/// Differentiate the expression `expr_str` symbolically.
pub fn differentiate(expr_str: &str) -> Result<DifferentiationResult, String> {
    let expr = parser_core::parse(expr_str)?;
    let deriv = differentiate_symbolic(&expr)?;
    Ok(DifferentiationResult {
        symbolic: deriv.to_string_repr(),
    })
}

// ============================================================
// Numerical integration  (adaptive Simpson's rule)
// ============================================================

const ABS_TOL: f64 = 1e-10;
const REL_TOL: f64 = 1e-8;
const MAX_DEPTH: u32 = 50;

/// Compute the definite integral of `f` from `a` to `b` using
/// adaptive Simpson's rule.  Tolerances: absolute 1e-10, relative 1e-8.
pub fn integrate_numerical(f: &dyn Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    adaptive_simpsons(f, a, b, ABS_TOL, 0)
}

fn simpsons(f: &dyn Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let mid = (a + b) / 2.0;
    (b - a) / 6.0 * (f(a) + 4.0 * f(mid) + f(b))
}

fn adaptive_simpsons(
    f: &dyn Fn(f64) -> f64,
    a: f64,
    b: f64,
    tol: f64,
    depth: u32,
) -> f64 {
    let mid = (a + b) / 2.0;
    let whole = simpsons(f, a, b);
    let left  = simpsons(f, a, mid);
    let right = simpsons(f, mid, b);
    let err   = (left + right - whole).abs();
    let tol_adaptive = tol.max(REL_TOL * whole.abs());

    if depth >= MAX_DEPTH || err <= 15.0 * tol_adaptive {
        left + right + (left + right - whole) / 15.0  // Richardson extrapolation
    } else {
        let half_tol = tol / 2.0;
        adaptive_simpsons(f, a, mid, half_tol, depth + 1)
            + adaptive_simpsons(f, mid, b, half_tol, depth + 1)
    }
}

// ============================================================
// Convenience wrapper
// ============================================================

/// Result returned by the high-level `integrate` function.
pub struct IntegrationResult {
    /// Symbolic antiderivative string, if available.
    pub symbolic: Option<String>,
    /// Numerical value of the definite integral from `a` to `b`.
    pub numerical: f64,
}

/// Integrate the expression `expr_str` from `a` to `b`.
///
/// Attempts symbolic integration first; always provides a numerical answer.
pub fn integrate(expr_str: &str, a: f64, b: f64) -> Result<IntegrationResult, String> {
    let expr = parser_core::parse(expr_str)?;

    let symbolic = match integrate_symbolic(&expr) {
        Ok(anti) => Some(anti.to_string_repr()),
        Err(_)   => None,
    };

    let numerical = integrate_numerical(&|x| expr.eval(x), a, b);

    Ok(IntegrationResult { symbolic, numerical })
}
