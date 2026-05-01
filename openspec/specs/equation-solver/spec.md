## ADDED Requirements

### Requirement: Solve function signature
The `solver-core` crate SHALL expose a public function:

```rust
pub fn solve(equation: &str, a: f64, b: f64) -> Result<SolveResult, String>
```

and a public result type:

```rust
pub struct SolveResult {
    pub root: f64,
    pub iterations: u32,
    pub residual: f64,   // |f(root)|
}
```

`solver-core` is a standalone library crate. It is NOT re-exported through `integrator_core`. Callers use `solver_core::solve` directly.

#### Scenario: Successful solve
- **WHEN** `solve("x^2 - 2", 1.0, 2.0)` is called
- **THEN** `result.root` is within `1e-10` of `√2 ≈ 1.41421356`

#### Scenario: Result fields populated
- **WHEN** a root is found
- **THEN** `result.root` is the located root, `result.residual` is `|f(root)|`, and `result.iterations` is the iteration count

---

### Requirement: Input form — f(x) = 0
When the `equation` string contains no `=` character, it SHALL be treated as `f(x) = 0` — i.e., find `x` such that `f(x) = 0`.

#### Scenario: Single expression
- **WHEN** `solve("x^2 - 2", 1.0, 2.0)` is called
- **THEN** the solver finds a root of `x^2 - 2 = 0`

#### Scenario: Trigonometric zero
- **WHEN** `solve("sin(x)", 3.0, 4.0)` is called
- **THEN** `result.root` is within `1e-10` of `π`

---

### Requirement: Input form — f(x) = g(x)
When the `equation` string contains exactly one `=` character, it SHALL be split on `=` to produce `lhs` and `rhs`, then internally rewritten as `lhs - rhs = 0`.

#### Scenario: Two-sided equation
- **WHEN** `solve("x^2 = 2", 1.0, 2.0)` is called
- **THEN** the solver treats it as `x^2 - 2 = 0` and returns the same root as the single-expression form

#### Scenario: Equal expressions on both sides
- **WHEN** `solve("sin(x) = cos(x)", 0.5, 1.0)` is called
- **THEN** `result.root` is within `1e-10` of `π/4`

#### Scenario: Multiple = signs is an error
- **WHEN** the equation string contains two or more `=` characters
- **THEN** `solve` returns `Err` with a descriptive message

---

### Requirement: Algorithm — Brent's method
`solve` SHALL use Brent's method (combining bisection, secant, and inverse quadratic interpolation) for root-finding.

Algorithm constants:
| Constant | Value |
|---|---|
| Absolute tolerance | `1e-10` |
| Maximum iterations | `100` |

The algorithm SHALL:
1. Evaluate `f(a)` and `f(b)`
2. If `f(a)` and `f(b)` have the same sign, return `Err("no sign change in [a, b]: f(a)=... f(b)=...")`
3. If `f(a) == 0.0`, return `a` immediately
4. If `f(b) == 0.0`, return `b` immediately
5. Apply Brent iterations until `|bracket| < 1e-10` or `|f(root)| < 1e-10` or max iterations reached

#### Scenario: Convergence within tolerance
- **WHEN** solving `x^2 - 4` over `[1, 3]`
- **THEN** `result.root` is within `1e-10` of `2.0` and `result.residual < 1e-10`

#### Scenario: Max iterations reached
- **WHEN** a pathological function requires more than 100 Brent iterations
- **THEN** the best estimate so far is returned (no panic, no error)

---

### Requirement: No sign change error
If `f(a)` and `f(b)` have the same sign, `solve` SHALL return `Err` with a message including the values of `f(a)` and `f(b)`.

#### Scenario: Same-sign bracket
- **WHEN** `solve("x^2", 1.0, 3.0)` is called (f is always positive)
- **THEN** `solve` returns `Err` containing `"no sign change"`

---

### Requirement: Parse errors propagated
If the equation string cannot be parsed (after splitting on `=` and forming the difference expression), `solve` SHALL return `Err` with the parse error message.

#### Scenario: Malformed expression
- **WHEN** `solve("sin(x +", 0.0, 1.0)` is called
- **THEN** `solve` returns `Err` with the parse error

---

### Requirement: Crate boundary
`solver-core` is a peer library alongside `integrator-core`. It is NOT a dependency of `integrator-core` and is NOT re-exported through it. Each consumer crate (`cli`, `web`) that needs solving must declare `solver-core` as a direct dependency and import via `use solver_core::solve`.

#### Scenario: Direct import
- **WHEN** a binary crate needs root-finding
- **THEN** it imports `use solver_core::solve` directly, not via `integrator_core`
