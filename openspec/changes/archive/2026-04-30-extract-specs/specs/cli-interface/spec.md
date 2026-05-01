## ADDED Requirements

### Requirement: Argument schema
The CLI binary (`integrate`) SHALL accept the following argument forms:

| Invocation | Behaviour |
|---|---|
| `integrate --demo` | Run 16 demo expressions + 7 verification tests |
| `integrate <expr>` | Integrate `<expr>` over default bounds `[0.0, 1.0]` |
| `integrate <expr> <a> <b>` | Integrate `<expr>` over `[a, b]` |
| `integrate --help` | Print usage information |
| No arguments | Print usage information |

Note: there is no standalone `--verify` flag. Verification runs automatically as part of `--demo`.

Arguments `<a>` and `<b>` SHALL be parsed as `f64`. Any string that cannot be parsed as `f64` SHALL cause the program to exit with a non-zero code and print an error message.

#### Scenario: Expression with explicit bounds
- **WHEN** invoked as `integrate "x^2" 0 3`
- **THEN** the integral of `x^2` over `[0, 3]` is computed and printed

#### Scenario: Expression with default bounds
- **WHEN** invoked as `integrate "sin(x)"`
- **THEN** the integral over `[0.0, 1.0]` is computed

#### Scenario: Invalid bound
- **WHEN** invoked as `integrate "x^2" 0 abc`
- **THEN** the program exits with a non-zero code and prints an error message

---

### Requirement: Demo mode
`--demo` SHALL run a hardcoded set of 16 demo expressions and print results for each, then automatically run 7 verification tests. Demo mode uses `PlotKind::Integrate` with ASCII-only output (no SVG written to disk). If any verification test fails, the process exits with code `1`.

The 16 demo expressions are:
1. `5` (constant) over `[0, 3]`
2. `x` (linear) over `[0, 2]`
3. `x^2` (quadratic) over `[0, 3]`
4. `x^3 - 2*x + 1` (cubic) over `[0, 2]`
5. `3*x^4 - x^2 + 7` (high degree) over `[1, 4]`
6. `sin(x)` over `[0, π]`
7. `cos(x)` over `[0, π/2]`
8. `tan(x)` over `[0, 1]`
9. `exp(x)` over `[0, 1]`
10. `ln(x)` over `[1, e]`
11. `2^x` (`a^x`) over `[0, 3]`
12. `sqrt(x)` over `[0, 4]`
13. `1/x` over `[1, e]`
14. `x^2 + sin(x)` over `[0, 2]`
15. `2*x^3 - 3*cos(x)` over `[0, 1]`
16. `5*exp(x) - x^2` over `[0, 2]`

#### Scenario: Demo produces output for all expressions
- **WHEN** invoked with `--demo`
- **THEN** output contains results for all 16 expressions and verification results, exit code is 0 if all pass

---

### Requirement: Verification suite (part of demo)
After the 16 demo expressions, `--demo` automatically runs 7 verification tests comparing symbolic `F(b)−F(a)` against numerical integration. The 7 expressions are:

| Expression | Bounds |
|---|---|
| `x^2` | `[0, 3]` |
| `x^3 - 2*x + 1` | `[0, 2]` |
| `sin(x)` | `[0, π]` |
| `cos(x)` | `[0, π/2]` |
| `exp(x)` | `[0, 1]` |
| `ln(x)` | `[1, e]` |
| `sqrt(x)` | `[0, 4]` |

A test passes if `|symbolic - numerical| < 1e-6`. If any test fails, `--demo` exits with code `1`.

#### Scenario: All verifications pass
- **WHEN** invoked with `--demo` on a correct build
- **THEN** verification output indicates all tests passed and exit code is `0`

#### Scenario: Verification failure exits with code 1
- **WHEN** a symbolic and numerical result differ by more than `1e-6`
- **THEN** the program exits with code `1`

---

### Requirement: Standard expression mode output
When integrating a user-supplied expression, the CLI SHALL print:

```
┌─ <label> ─────────────────────────────────────────
│  f(<var>)          = <expr>
│  ∫f(<var>)d<var>   = <antiderivative> + C    (if symbolic available)
│  ∫f(<var>)d<var>   = (no closed form found)  (if no symbolic result)
│  ∫[<a>, <b>] f d<var>  ≈ <numerical value, 10 decimal places>
└──────────────────────────────────────────────
```

Followed by an ASCII plot. SVG output SHALL always be written to `integrate.svg` in the current working directory.

#### Scenario: Symbolic result available
- **WHEN** integrating `x^2` over `[0, 1]`
- **THEN** output includes the symbolic antiderivative and the numerical value `0.3333...`

#### Scenario: No closed form
- **WHEN** integrating `sin(x^2)` over `[0, 1]`
- **THEN** output says "No closed form found" and still prints the numerical result

#### Scenario: SVG always written
- **WHEN** integrating any expression in standard mode
- **THEN** a file named `integrate.svg` is created in the current directory

---

### Requirement: Plot mode selection
- Demo invocations SHALL use `PlotKind::Integrate` with ASCII output only (no SVG file)
- User expression invocations SHALL use `PlotKind::Integrate` with both ASCII and SVG output

#### Scenario: Demo — no SVG file
- **WHEN** running `--demo`
- **THEN** no `integrate.svg` file is created

---

### Requirement: Exit codes

| Condition | Exit code |
|---|---|
| Successful integration (expression mode) | `0` |
| `--demo` with all verifications passing | `0` |
| `--demo` with any verification failing | `1` |
| Parse error on expression | `1` (via `eprintln!` + `process::exit(1)`) |
| Invalid bounds argument | `1` (via `eprintln!` + `process::exit(1)`) |
| Wrong number of arguments | `1` (via `eprintln!` + `process::exit(1)`) |

#### Scenario: Successful expression
- **WHEN** integrating a valid expression
- **THEN** exit code is `0`

#### Scenario: Bad expression
- **WHEN** the expression cannot be parsed
- **THEN** exit code is `1`
