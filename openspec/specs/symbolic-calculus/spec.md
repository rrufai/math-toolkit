## ADDED Requirements

### Requirement: Symbolic integration — supported rules
`integrate_symbolic(expr)` SHALL produce a closed-form antiderivative for the following patterns. All results omit the constant of integration `+C`.

| Input pattern | Antiderivative | Condition |
|---|---|---|
| `c` (constant) | `c·x` | `c` is free of all variables |
| `x^n` | `x^(n+1) / (n+1)` | `n ≠ -1` |
| `x^(-1)` or `1/x` | `ln(x)` | — |
| `sin(x)` | `-cos(x)` | — |
| `cos(x)` | `sin(x)` | — |
| `tan(x)` | `-ln(abs(cos(x)))` | — |
| `exp(x)` | `exp(x)` | — |
| `a^x` | `a^x / ln(a)` | `a` is a constant |
| `ln(x)` | `x·ln(x) - x` | — |
| `sqrt(x)` | `(2/3)·x^(3/2)` | — |
| `c·f(x)` | `c·∫f(x) dx` | `c` is constant |
| `f(x) + g(x)` | `∫f dx + ∫g dx` | linearity |
| `f(x) - g(x)` | `∫f dx - ∫g dx` | linearity |
| `f(x) / c` | `(1/c)·∫f(x) dx` | `c` is constant |

#### Scenario: Power rule
- **WHEN** the expression is `x^3`
- **THEN** the antiderivative is `x^4/4`

#### Scenario: Reciprocal rule
- **WHEN** the expression is `1/x`
- **THEN** the antiderivative is `ln(x)`

#### Scenario: Sine
- **WHEN** the expression is `sin(x)`
- **THEN** the antiderivative is `-cos(x)`

#### Scenario: Cosine
- **WHEN** the expression is `cos(x)`
- **THEN** the antiderivative is `sin(x)`

#### Scenario: Tangent
- **WHEN** the expression is `tan(x)`
- **THEN** the antiderivative is `-ln(abs(cos(x)))`

#### Scenario: Exponential
- **WHEN** the expression is `exp(x)`
- **THEN** the antiderivative is `exp(x)`

#### Scenario: Natural log
- **WHEN** the expression is `ln(x)`
- **THEN** the antiderivative is `x·ln(x) - x`

#### Scenario: Square root
- **WHEN** the expression is `sqrt(x)`
- **THEN** the antiderivative is `(2/3)·x^(3/2)`

#### Scenario: Scalar multiple
- **WHEN** the expression is `3*sin(x)`
- **THEN** the antiderivative is `-3*cos(x)`

#### Scenario: Sum linearity
- **WHEN** the expression is `x^2 + cos(x)`
- **THEN** the antiderivative is `x^3/3 + sin(x)`

---

### Requirement: Symbolic integration — unsupported patterns
`integrate_symbolic(expr)` SHALL return `Err` for any pattern not listed above. Specifically:

| Unsupported pattern | Example |
|---|---|
| Non-constant product | `x*sin(x)`, `x^2*exp(x)` |
| Non-constant divisor | `1/sin(x)`, `x/(x+1)` |
| Composite argument (non-linear) | `sin(x^2)`, `exp(x^2)` |
| Variable base and variable exponent | `x^x` |
| `abs(x)` | `abs(x)` |

#### Scenario: Unsupported — product of two non-constant terms
- **WHEN** the expression is `x*sin(x)`
- **THEN** `integrate_symbolic` returns `Err`

#### Scenario: Unsupported — composite function
- **WHEN** the expression is `sin(x^2)`
- **THEN** `integrate_symbolic` returns `Err`

#### Scenario: Unsupported — abs
- **WHEN** the expression is `abs(x)`
- **THEN** `integrate_symbolic` returns `Err`

---

### Requirement: Symbolic integration — fallback behaviour
When `integrate_symbolic` returns `Err`, the public `integrate()` function SHALL silently fall back to numerical integration. The `IntegrationResult.symbolic` field SHALL be `None`; `IntegrationResult.numerical` SHALL always be populated.

#### Scenario: Fallback on unsupported expression
- **WHEN** calling `integrate("sin(x^2)", 0.0, 1.0)`
- **THEN** `result.symbolic` is `None` and `result.numerical` is a finite value

---

### Requirement: Canonicalisation before integration
Before symbolic integration is attempted, the expression SHALL be canonicalised:

- `x*x*x` → `x^3` (repeated variable products → power)
- `t*t` → `t^2`
- Nested multiplications are flattened before conversion

#### Scenario: Triple product becomes power
- **WHEN** the expression is `x*x*x`
- **THEN** it is canonicalised to `x^3` before integration, yielding `x^4/4`

---

### Requirement: Simplification of integration result
After integration, the result SHALL be simplified using constant folding and algebraic identities:

- `0 + x` → `x`
- `x * 1` → `x`
- `x * 0` → `0`
- `x ^ 0` → `1`
- `x ^ 1` → `x`
- Constant sub-expressions are folded to a single number

#### Scenario: Constant folding in result
- **WHEN** integration of a constant expression produces `0 + c*x`
- **THEN** the simplified result is `c*x`

---

### Requirement: Symbolic differentiation — supported rules
`differentiate_symbolic(expr)` SHALL compute derivatives using the following rules:

| Pattern | Derivative |
|---|---|
| `c` (constant) | `0` |
| `x` (variable) | `1` |
| `c·f(x)` | `c·f'(x)` |
| `f + g` | `f' + g'` |
| `f - g` | `f' - g'` |
| `f * g` | `f'·g + f·g'` (product rule) |
| `f / g` | `(f'·g - f·g') / g^2` (quotient rule) |
| `f ^ n` (n constant) | `n·f^(n-1)·f'` (chain rule) |
| `sin(f)` | `cos(f)·f'` |
| `cos(f)` | `-sin(f)·f'` |
| `tan(f)` | `f' / cos(f)^2` |
| `exp(f)` | `exp(f)·f'` |
| `ln(f)` | `f' / f` |
| `sqrt(f)` | `f' / (2·sqrt(f))` |
| `-f` | `-f'` |

Chain rule is automatically applied for all composite functions.

#### Scenario: Power rule
- **WHEN** the expression is `x^3`
- **THEN** the derivative is `3*x^2`

#### Scenario: Chain rule — sin of composite
- **WHEN** the expression is `sin(x^2)`
- **THEN** the derivative is `cos(x^2)*2*x`

#### Scenario: Product rule
- **WHEN** the expression is `x*sin(x)`
- **THEN** the derivative is `sin(x) + x*cos(x)`

#### Scenario: Quotient rule
- **WHEN** the expression is `sin(x)/x`
- **THEN** the derivative is `(cos(x)*x - sin(x)) / x^2`

---

### Requirement: Symbolic differentiation — abs constraint
`differentiate_symbolic(expr)` SHALL return `Err` when the expression contains `abs()`.

#### Scenario: abs is not differentiable
- **WHEN** the expression is `abs(x)`
- **THEN** `differentiate_symbolic` returns `Err`

---

### Requirement: Differentiation variable
`differentiate_symbolic` SHALL differentiate with respect to the first variable found in the expression (via `var_name_of()`). All other identifiers are treated as constants.

#### Scenario: Single variable
- **WHEN** the expression is `x^2`
- **THEN** differentiation is with respect to `x`, returning `2*x`
