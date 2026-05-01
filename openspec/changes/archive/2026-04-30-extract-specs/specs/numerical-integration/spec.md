## ADDED Requirements

### Requirement: Algorithm — adaptive Simpson's rule
`integrate_numerical(f, a, b)` SHALL compute the definite integral ∫[a,b] f(x) dx using adaptive Simpson's rule with recursive interval subdivision.

The algorithm SHALL:
1. Compute Simpson's rule estimate over the full interval
2. Split the interval at the midpoint and compute estimates on each half
3. If the error estimate is within tolerance, return the refined estimate
4. Otherwise, recurse on each half-interval up to `MAX_DEPTH`

#### Scenario: Well-behaved function
- **WHEN** integrating `x^2` over `[0, 1]`
- **THEN** the result is within `1e-8` of the exact value `1/3`

#### Scenario: Trigonometric function
- **WHEN** integrating `sin(x)` over `[0, π]`
- **THEN** the result is within `1e-8` of the exact value `2.0`

---

### Requirement: Tolerance constants
The following constants SHALL govern convergence. These are normative — the implementation MUST use exactly these values:

| Constant | Value | Meaning |
|---|---|---|
| `ABS_TOL` | `1e-10` | Absolute error tolerance |
| `REL_TOL` | `1e-8` | Relative error tolerance |
| `MAX_DEPTH` | `50` | Maximum recursion depth |

The algorithm terminates on a sub-interval when:
```
error ≤ 15 × max(tol, REL_TOL × |estimate|)
```
where `tol` starts at `ABS_TOL` and is halved at each recursive depth. The factor of 15 comes from Richardson extrapolation error analysis. The refined estimate returned is:
```
left + right + (left + right - whole) / 15
```

#### Scenario: Convergence within relative tolerance
- **WHEN** integrating a smooth function where the relative error drops below `1e-8`
- **THEN** the algorithm terminates and returns the current estimate

#### Scenario: Max depth reached
- **WHEN** the function requires more than 50 levels of subdivision
- **THEN** the algorithm returns its best estimate at depth 50 without further recursion

---

### Requirement: Always returns a value
`integrate_numerical` SHALL always return an `f64`. It MUST NOT panic. For functions with singularities or discontinuities in `[a, b]`, the returned value may be `NaN` or `Inf`, but no error is raised.

#### Scenario: Singularity in interval
- **WHEN** integrating `1/x` over `[-1, 1]`
- **THEN** the function returns a value (may be `NaN` or `Inf`) without panicking

---

### Requirement: Reversed bounds
When `a > b`, the integral SHALL be computed as the negative of `∫[b,a] f`:

```
∫[b,a] f = -∫[a,b] f
```

#### Scenario: Reversed interval
- **WHEN** integrating `x^2` over `[1, 0]`
- **THEN** the result is the negation of the result for `[0, 1]`

---

### Requirement: Equal bounds
When `a == b`, `integrate_numerical` SHALL return `0.0`.

#### Scenario: Zero-width interval
- **WHEN** integrating any function over `[c, c]`
- **THEN** the result is `0.0`

---

### Requirement: Independence from symbolic result
Numerical integration is always computed independently of `integrate_symbolic`. The public `integrate(expr_str, a, b)` function SHALL compute `numerical` even when `symbolic` succeeds.

#### Scenario: Both symbolic and numerical computed
- **WHEN** calling `integrate("x^2", 0.0, 1.0)`
- **THEN** `result.symbolic` is `Some(...)` AND `result.numerical` is a finite value
