## Context

The integrator-core crate already provides numerical evaluation of arbitrary `Expr` ASTs via `Expr::eval(x)`. Root-finding only requires a scalar function `f64 → f64`, so the solver can be built entirely on top of the existing `eval` infrastructure with no new dependencies. The parser already handles both single-expression and will handle the `f(x) = g(x)` form by rewriting it as `f(x) - g(x)` before solving.

## Goals / Non-Goals

**Goals:**
- Find a single root of `f(x) = 0` or `f(x) = g(x)` within a user-supplied bracketing interval `[a, b]`
- Use Brent's method: guaranteed convergence when `f(a)` and `f(b)` have opposite signs
- Expose the solver through all three surfaces: core library, CLI (`integrate --solve`), and web (`POST /solve`, `GET /api/solve`)
- Parse `f(x) = g(x)` by splitting on `=` and forming `f - g` internally

**Non-Goals:**
- Finding all roots (only one root per call, within the bracket)
- Symbolic root-finding (e.g. solving polynomial equations algebraically)
- Complex roots or roots outside the supplied interval
- Systems of equations (multiple unknowns)

## Decisions

### Brent's method over bisection or Newton-Raphson

**Decision**: Use Brent's method as the single algorithm.

**Rationale**: Brent's method combines bisection, secant, and inverse quadratic interpolation. It guarantees convergence (like bisection) when a sign change exists, and converges superlinearly in practice. Newton-Raphson requires a derivative (expensive to compute symbolically for all cases) and can diverge. Bisection alone is too slow.

**Alternatives considered**:
- Bisection: simple but O(log n) convergence, ~53 iterations for f64 precision. Rejected as primary method (but Brent degrades to bisection in worst case, so coverage is identical).
- Newton-Raphson: requires `differentiate_symbolic`, fails for `abs()` and unsupported patterns. Rejected.

---

### Input parsing: split on `=` character

**Decision**: Accept either a single expression string (treated as `f(x) = 0`) or a string containing exactly one `=` character (treated as `f(x) = g(x)`, rewritten to `f(x) - g(x)`).

**Rationale**: The existing `parse()` function handles arbitrary expressions. Splitting on `=` is unambiguous — the parser does not produce `=` tokens. The rewrite `f - g` is algebraically correct and requires no changes to the AST.

**Alternatives considered**: A dedicated two-expression API (`solve_two(lhs, rhs, a, b)`). Rejected — a single string input is more natural for CLI and web users.

---

### Tolerances

**Decision**: Absolute tolerance `1e-10`, maximum 100 iterations.

**Rationale**: Matches the precision of the numerical integrator (`ABS_TOL = 1e-10`). 100 iterations is more than sufficient for Brent's method to reach machine epsilon within any reasonable bracket.

---

### CLI surface: `integrate --solve`

**Decision**: Add `--solve` to the existing `integrate` binary rather than creating a new `solve` binary.

**Rationale**: Consistent with how `integrate` and `differentiate` are separate binaries — but a solver is closely related to integration (finding zeros of functions is a common pre-step). Adding it to `integrate` keeps the surface minimal. A separate binary can be added later if needed.

---

### Web surface: `/solve` routes, not nested under `/integrate`

**Decision**: Add `POST /solve` and `GET /api/solve` as top-level routes.

**Rationale**: The solver is a distinct operation with a different result shape (`root` instead of `symbolic`/`numerical`). Nesting under `/integrate` would be misleading.

## Risks / Trade-offs

- **No sign change in interval** → `solve()` returns `Err("no sign change in [a, b]")`. User must supply a better bracket. Mitigation: clear error message.
- **Multiple roots in interval** → Brent's method finds one root (unspecified which). Mitigation: document this limitation in the spec.
- **Function not continuous** → e.g. `tan(x)` near π/2 may have a sign change that is a pole, not a root. Mitigation: document; caller should inspect the returned root value by evaluating `f(root)` ≈ 0.
- **`=` appears in expression string** → e.g. a user accidentally passing `x^2` with no `=` is fine; `x^2 = sin(x)` splits correctly; two `=` signs produce an error. Mitigation: validate split count before parsing.
