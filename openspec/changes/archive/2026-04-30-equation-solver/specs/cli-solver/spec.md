## ADDED Requirements

### Requirement: --solve argument form
The `integrate` CLI SHALL accept a new invocation form:

```
integrate --solve <equation> <a> <b>
```

Where `<equation>` is either `f(x)` (finds root of `f(x) = 0`) or `f(x) = g(x)` (finds root of `f(x) - g(x) = 0`), and `<a>` `<b>` are the bracketing interval bounds parsed as `f64`.

The updated argument schema:

| Invocation | Behaviour |
|---|---|
| `integrate --demo` | Run 16 demo expressions + 7 verification tests |
| `integrate <expr>` | Integrate `<expr>` over default bounds `[0.0, 1.0]` |
| `integrate <expr> <a> <b>` | Integrate `<expr>` over `[a, b]` |
| `integrate --solve <equation> <a> <b>` | Find root of equation in `[a, b]` |
| `integrate --help` | Print usage information |
| No arguments | Print usage information |

#### Scenario: Solve with f(x) = 0 form
- **WHEN** invoked as `integrate --solve "x^2 - 2" 1 2`
- **THEN** the solver finds the root near `1.41421356` and prints it

#### Scenario: Solve with f(x) = g(x) form
- **WHEN** invoked as `integrate --solve "x^2 = 2" 1 2`
- **THEN** the solver finds the same root

#### Scenario: Missing bounds for --solve
- **WHEN** invoked as `integrate --solve "x^2 - 2"` (no bounds)
- **THEN** the program exits with code `1` and prints a usage error

---

### Requirement: --solve output format
The `--solve` output SHALL follow the same box format as integration:

```
┌─ solve ──────────────────────────────────────────
│  f(x)          = <equation>
│  root          ≈ <root value, 10 decimal places>
│  f(root)       ≈ <residual, scientific notation>
│  iterations    = <count>
└──────────────────────────────────────────────
```

Followed by an ASCII plot of `f(x)` over `[a, b]` with the root marked. SVG SHALL be written to `solve.svg` in the current working directory.

#### Scenario: Successful solve output
- **WHEN** `integrate --solve "x^2 - 2" 1 2` is called
- **THEN** output contains the root value, residual, and iteration count

#### Scenario: SVG written on success
- **WHEN** `--solve` succeeds
- **THEN** a file named `solve.svg` is created in the current directory

---

### Requirement: --solve error output
If solving fails (no sign change, parse error), the CLI SHALL print the error message to stderr and exit with code `1`.

#### Scenario: No sign change error
- **WHEN** `integrate --solve "x^2" 1 3` is called (no root in interval)
- **THEN** stderr contains `"no sign change"` and exit code is `1`

#### Scenario: Parse error
- **WHEN** `integrate --solve "sin(x +" 0 4` is called
- **THEN** stderr contains the parse error and exit code is `1`
