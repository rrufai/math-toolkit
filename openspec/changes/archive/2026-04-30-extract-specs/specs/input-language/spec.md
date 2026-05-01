## ADDED Requirements

### Requirement: Expression grammar
The system SHALL accept mathematical expressions conforming to the following grammar (low to high precedence):

```
expr    → term (('+' | '-') term)*
term    → unary (('*' | '/') unary)*
unary   → '-' unary | power
power   → primary ('^' unary)?        ← right-associative (one ^ per node, recursion through unary)
primary → NUMBER | IDENT | IDENT '(' expr ')' | '(' expr ')'
```

Operator `^` is right-associative: `x^y^z` parses as `x^(y^z)`.

#### Scenario: Addition and subtraction are left-associative
- **WHEN** the input is `a - b - c`
- **THEN** it parses as `(a - b) - c`

#### Scenario: Power is right-associative
- **WHEN** the input is `x^y^z`
- **THEN** it parses as `x^(y^z)`

#### Scenario: Unary minus binds tighter than power
- **WHEN** the input is `-x^2`
- **THEN** it parses as `-(x^2)`, not `(-x)^2`

---

### Requirement: Supported tokens
The tokeniser SHALL recognise:

| Token type | Examples |
|---|---|
| Integer literal | `0`, `42`, `100` |
| Decimal literal | `3.14`, `0.5`, `.25` |
| Scientific notation | `1e2`, `1.5e-3`, `2e+4` |
| Identifier | `x`, `t`, `sin`, `pi` |
| Unicode constant | `π` (pi), `τ` (tau) — tokenised directly as numeric literals |
| Operators | `+` `-` `*` `/` `^` |
| Parentheses | `(` `)` |
| Whitespace | spaces, tabs, newlines — ignored |

Any character not in the above set SHALL produce a parse error.

#### Scenario: Integer literal
- **WHEN** the input is `42`
- **THEN** it evaluates to `42.0`

#### Scenario: Scientific notation — negative exponent
- **WHEN** the input is `1.5e-3`
- **THEN** it evaluates to `0.0015`

#### Scenario: Unknown character
- **WHEN** the input contains `@` or `#`
- **THEN** the parser returns an error

---

### Requirement: Reserved identifiers — functions
The following identifiers SHALL be treated as built-in functions requiring a parenthesised argument:

`sin`, `cos`, `tan`, `exp`, `ln`, `log`, `sqrt`, `abs`

`log` is an alias for `ln`.

#### Scenario: Function call with argument
- **WHEN** the input is `sin(x)`
- **THEN** it parses as the sine function applied to `x`

#### Scenario: log is alias for ln
- **WHEN** the input is `log(x)`
- **THEN** it is equivalent to `ln(x)`

---

### Requirement: Reserved identifiers — constants
The following identifiers SHALL be treated as numeric constants:

| Identifier | Aliases | Value |
|---|---|---|
| `pi` | `PI` | π ≈ 3.141592653589793 |
| `e` | `E` | e ≈ 2.718281828459045 |
| `τ` (tau) | — | τ = 2π ≈ 6.283185307179586 (unicode character only) |

The unicode characters `π` and `τ` are tokenised directly by the tokeniser (not via the identifier path). `PI` and `E` (uppercase) are equivalent aliases for `pi` and `e`.

#### Scenario: pi resolves to π
- **WHEN** the input is `pi`
- **THEN** it evaluates to `3.141592653589793...`

#### Scenario: PI uppercase alias
- **WHEN** the input is `PI`
- **THEN** it evaluates to the same value as `pi`

#### Scenario: e resolves to Euler's number
- **WHEN** the input is `e`
- **THEN** it evaluates to `2.718281828459045...`

#### Scenario: Unicode π token
- **WHEN** the input contains the unicode character `π`
- **THEN** it is tokenised as the numeric value of π directly

---

### Requirement: Implicit multiplication
The parser SHALL insert an implicit `*` operator in the following cases:

- Number immediately followed by identifier: `3x` → `3*x`
- Number immediately followed by `(`: `3(x+1)` → `3*(x+1)`
- Identifier immediately followed by `(` where identifier is not a known function: `x(x+1)` → `x*(x+1)`
- Function result immediately followed by `(` is NOT supported and SHALL produce a parse error or incorrect parse

#### Scenario: Coefficient before variable
- **WHEN** the input is `3x`
- **THEN** it is equivalent to `3*x`

#### Scenario: Coefficient before function
- **WHEN** the input is `2sin(x)`
- **THEN** it is equivalent to `2*sin(x)`

#### Scenario: Number before parenthesised group
- **WHEN** the input is `3(x+1)`
- **THEN** it is equivalent to `3*(x+1)`

---

### Requirement: Variable names
Any identifier that is not a reserved function name or constant SHALL be treated as a variable. The system currently supports a single-variable model — `first_var()` extracts the first variable name found and uses it as the integration/differentiation variable.

#### Scenario: Arbitrary variable name
- **WHEN** the input is `t^2`
- **THEN** `t` is the variable; expression evaluates to `t²`

#### Scenario: Multiple identifiers — first wins
- **WHEN** the expression contains multiple distinct non-reserved identifiers
- **THEN** the first one encountered in a depth-first traversal is used as the variable

---

### Requirement: Parse error conditions
The parser SHALL return an error for:

| Condition | Example |
|---|---|
| Empty input | `""` |
| Unmatched open parenthesis | `"(x + 1"` |
| Trailing close parenthesis | `"x + 1)"` |
| Leading binary operator | `"* x"` |
| Unknown character | `"x @ 1"` |
| Truncated function call | `"sin(x +"` |

#### Scenario: Empty input
- **WHEN** the input is an empty string
- **THEN** the parser returns an error

#### Scenario: Unmatched parenthesis
- **WHEN** the input is `(x + 1`
- **THEN** the parser returns an error

#### Scenario: Trailing operator produces error
- **WHEN** the input is `x +`
- **THEN** the parser returns an error

---

### Requirement: Expression evaluation
`Expr::eval(x: f64)` SHALL evaluate the AST at the given value. All variable identifiers in the expression are substituted with `x`. The function SHALL always return an `f64` — it MUST NOT panic. Results may be `NaN` or `Inf` for undefined operations.

#### Scenario: Well-defined evaluation
- **WHEN** evaluating `x^2` at `x = 3.0`
- **THEN** returns `9.0`

#### Scenario: Division by zero — returns Inf
- **WHEN** evaluating `1/x` at `x = 0.0`
- **THEN** returns `Inf` (no panic, no error)

#### Scenario: sqrt of negative — returns NaN
- **WHEN** evaluating `sqrt(x)` at `x = -1.0`
- **THEN** returns `NaN` (no panic, no error)

---

### Requirement: Expression pretty-printing
`Expr::to_string_repr()` SHALL produce a human-readable string. The following formatting rules apply:

- Integer-valued floats printed without decimal: `3.0` → `3`
- π multiples formatted with unicode: `6.283...` → `2π`, `1.5707...` → `π/2`
- `e` constant formatted as `e`
- Simple fractions reduced: `2/4` → `1/2` (GCD reduction for denominators ≤ 99)
- Minimal parenthesisation based on operator precedence
- Unicode `π` and `τ` used for display

#### Scenario: Integer-valued float
- **WHEN** the expression is `Num(3.0)`
- **THEN** `to_string_repr()` returns `"3"`

#### Scenario: Pi multiple
- **WHEN** the expression evaluates to `2π`
- **THEN** `to_string_repr()` returns `"2π"`
