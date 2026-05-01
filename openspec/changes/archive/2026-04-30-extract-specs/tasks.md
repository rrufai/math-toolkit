## 1. Input Language Spec

- [x] 1.1 Review `specs/input-language/spec.md` against `crates/integrator/src/parser.rs` — verify grammar, token types, and implicit multiplication rules are accurate
- [x] 1.2 Verify parse error conditions against parser test cases in `crates/integrator/src/lib.rs`
- [x] 1.3 Confirm `log` = `ln` alias behaviour in tokeniser
- [x] 1.4 Confirm `τ` (tau) constant handling in tokeniser and evaluator

## 2. Symbolic Calculus Spec

- [x] 2.1 Review `specs/symbolic-calculus/spec.md` against `crates/integrator/src/integrator.rs` — verify all supported integration rules
- [x] 2.2 Verify the `tan(x)` antiderivative form matches the implementation
- [x] 2.3 Verify `a^x` antiderivative (constant base) is correctly specified
- [x] 2.4 Verify all unsupported patterns — confirm each returns `Err` from `integrate_symbolic`
- [x] 2.5 Review differentiation rules against `differentiate_symbolic` implementation — verify chain, product, quotient rule forms
- [x] 2.6 Verify `Abs` rejection in differentiation spec matches implementation
- [x] 2.7 Verify canonicalisation behaviour (`x*x*x` → `x^3`) in spec matches code

## 3. Numerical Integration Spec

- [x] 3.1 Review `specs/numerical-integration/spec.md` against adaptive Simpson's implementation in `integrator.rs`
- [x] 3.2 Confirm tolerance constant values (`ABS_TOL=1e-10`, `REL_TOL=1e-8`, `MAX_DEPTH=50`) match source
- [x] 3.3 Verify reversed-bounds and equal-bounds behaviour in the spec

## 4. Plotting Spec

- [x] 4.1 Review `specs/plotting/spec.md` against `crates/integrator/src/plot.rs`
- [x] 4.2 Confirm all dimension constants (`W`, `H`, `ML`, `MR`, `MT`, `MB`, `N`, `COLS`, `ROWS`) match source
- [x] 4.3 Verify y-range padding calculation (5%) and fallback to `[-1, 1]` match implementation
- [x] 4.4 Verify tick formatting rules match `nice_ticks` / formatting functions in `plot.rs`
- [x] 4.5 Verify `PlotKind::Integrate` vs `PlotKind::Differentiate` behaviour differences in spec

## 5. CLI Interface Spec

- [x] 5.1 Review `specs/cli-interface/spec.md` against `crates/cli/src/main.rs`
- [x] 5.2 Verify the full list of 14 demo expressions matches `run_demo()` in `main.rs`
- [x] 5.3 Verify the 7 verification expressions match `run_verification()` in `main.rs`
- [x] 5.4 Verify SVG output path (`integrate.svg`) and that demo mode skips SVG
- [x] 5.5 Confirm exit codes match implementation

## 6. Web API Spec

- [x] 6.1 Review `specs/web-api/spec.md` against `crates/web/src/controllers/integrate.rs`
- [x] 6.2 Verify all 5 routes are listed and match the controller registration in `app.rs`
- [x] 6.3 Verify JSON response shape for `/api/integrate` matches `ComputedResult` struct
- [x] 6.4 Verify JSON response shape for `/api/differentiate` matches implementation
- [x] 6.5 Verify error response format (HTTP 200 with `{"error": ...}`) matches implementation
- [x] 6.6 Verify HTML escaping spec covers all four characters handled by the escape function
