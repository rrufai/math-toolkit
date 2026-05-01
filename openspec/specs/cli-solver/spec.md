## Requirements

### Requirement: Standalone solve binary
Root-finding is exposed as a dedicated binary named `solve` (not a flag on `integrate`). The binary is defined in `crates/cli/src/solve.rs` and registered as `[[bin]] name = "solve"` in `crates/cli/Cargo.toml`.

### Requirement: Argument schema
The `solve` binary SHALL accept the following argument forms:

| Invocation | Behaviour |
|---|---|
| `solve <equation> <a> <b>` | Find root of equation in `[a, b]` |
| `solve --help` | Print usage information |
| `solve -h` | Print usage information |
| No arguments | Print usage information |

Where `<equation>` is either `f(x)` (finds root of `f(x) = 0`) or `f(x) = g(x)` (finds root of `f(x) - g(x) = 0`), and `<a>` `<b>` are the bracketing interval bounds parsed as `f64`.

#### Scenario: Solve with f(x) = 0 form
- **WHEN** invoked as `solve "x^2 - 2" 1 2`
- **THEN** the solver finds the root near `1.41421356` and prints it

#### Scenario: Solve with f(x) = g(x) form
- **WHEN** invoked as `solve "x^2 = 2" 1 2`
- **THEN** the solver finds the same root

#### Scenario: Missing bounds
- **WHEN** invoked as `solve "x^2 - 2"` (no bounds)
- **THEN** the program exits with code `1` and prints a usage error

#### Scenario: No arguments — shows help
- **WHEN** invoked with no arguments
- **THEN** usage information is printed and exit code is `0`

#### Scenario: --help flag
- **WHEN** invoked with `--help` or `-h`
- **THEN** usage information is printed and exit code is `0`

---

### Requirement: Output format
The `solve` output SHALL follow the box format:

```
┌─ solve ──────────────────────────────────────────
│  f(x)          = <equation>
│  root          ≈ <root value, 10 decimal places>
│  f(root)       ≈ <residual, scientific notation>
│  iterations    = <count>
└──────────────────────────────────────────────
```

Followed by an ASCII plot of `f(x)` over `[a, b]`. SVG SHALL be written to `solve.svg` in the current working directory.

#### Scenario: Successful solve output
- **WHEN** `solve "x^2 - 2" 1 2` is called
- **THEN** output contains the root value, residual, and iteration count

#### Scenario: SVG written on success
- **WHEN** `solve` succeeds
- **THEN** a file named `solve.svg` is created in the current directory

---

### Requirement: Error output
If solving fails (no sign change, parse error), the CLI SHALL print the error message to stderr and exit with code `1`.

#### Scenario: No sign change error
- **WHEN** `solve "x^2" 1 3` is called (no root in interval)
- **THEN** stderr contains `"no sign change"` and exit code is `1`

#### Scenario: Parse error
- **WHEN** `solve "sin(x +" 0 4` is called
- **THEN** stderr contains the parse error and exit code is `1`

#### Scenario: Invalid bound
- **WHEN** invoked as `solve "x^2 - 2" abc 2`
- **THEN** the program exits with a non-zero code and prints `"not a valid number"`

---

### Requirement: Exit codes

| Condition | Exit code |
|---|---|
| Successful solve | `0` |
| `--help` / `-h` / no args | `0` |
| No sign change | `1` |
| Parse error | `1` |
| Invalid bounds | `1` |
| Wrong number of arguments | `1` |

---

### Requirement: Library dependency
`solve` uses `solver_core::solve` directly (not via `integrator_core`). `solver-core` is a peer library crate; it is not re-exported through `integrator_core`.
