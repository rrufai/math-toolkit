# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                                        # compile all crates
cargo test --workspace                             # run all tests (~282 tests)
cargo test -p integrator-core                      # lib tests only (150 tests)
cargo test -p rust-integrator-cli                  # CLI unit + integration tests
cargo test <name>                                  # run a single test by name
cargo run -p rust-integrator-cli -- --demo         # run the CLI demo
cargo run -p rust-integrator-cli -- "x^2" 0 3     # integrate via CLI
cargo run -p rust-integrator-cli --bin solve -- "x^2 - 2" 1 2  # root-finding via CLI
cargo run -p rust-integrator-diff -- "x^3" 0 2    # differentiate via CLI
cargo run -p rust-integrator-web -- start          # start the web server (port 5150)
cargo clippy --workspace -- -D warnings            # lint (warnings are errors)
cargo llvm-cov --workspace --summary-only          # coverage report (floor: 96% per file)
```

## Architecture

Cargo workspace with six crates under `crates/`:

```
crates/
  parser/       — expression parser (string → AST)
  integrator/   — symbolic & numerical integration + differentiation + plotting
  solver/       — root-finding (Brent's method)
  cli/          — integrate and solve binaries
  diff/         — differentiate binary
  web/          — Loco web server (REST + HTML UI)
```

### `crates/parser` — lib crate (`parser-core`)

`src/lib.rs` — string → AST
- `tokenize()` converts the input string to `Vec<Token>`
- `Parser` (recursive descent) produces an `Expr` AST
- `Expr` is a public enum with variants: `Num`, `Var`, `Neg`, `Add`, `Sub`, `Mul`, `Div`, `Pow`, `Sin`, `Cos`, `Tan`, `Exp`, `Ln`, `Sqrt`, `Abs`
- `Expr::eval(x)` evaluates at a point; `Expr::to_string_repr()` pretty-prints it
- Grammar (low → high precedence): `expr → term → unary → power → primary`; `^` is right-associative

### `crates/integrator` — lib crate (`integrator-core`)

Depends on `parser-core`. Re-exports the full public API via `src/lib.rs`.

**`src/integrator.rs`** — symbolic & numerical integration + differentiation
- `integrate_symbolic(expr)` — pattern-matches to produce a closed-form antiderivative; returns `Err` when no rule matches
- `integrate_numerical(f, a, b)` — adaptive Simpson's rule; tolerances `ABS_TOL=1e-10`, `REL_TOL=1e-8`, `MAX_DEPTH=50`
- `integrate(expr_str, a, b)` — convenience wrapper returning `IntegrationResult`
- `differentiate_symbolic(expr)` — symbolic differentiation; returns `Err` for `abs()`
- `simplify()`, `canonicalize()`, `is_const()` are `pub(crate)`

**`src/plot.rs`** — SVG and ASCII plots
- `render_svg(...)` / `render_svg_diff(...)` → `String` — full SVG markup for inline embedding
- `render_ascii_string(...)` / `render_ascii_string_diff(...)` → `String` — ASCII plot for terminal
- `write_svg(...)` — writes SVG to file; `print_ascii(...)` — prints ASCII to stdout
- `PlotKind` enum: `Integrate` (shaded area + antiderivative) vs `Differentiate` (derivative overlay)

**`src/lib.rs`** — re-exports all public API + `pub fn first_var(expr)` + 150 unit tests

### `crates/solver` — lib crate (`solver-core`)

Depends on `parser-core` only. NOT re-exported through `integrator-core` — consumers import directly.

`src/lib.rs`
- `pub fn solve(equation: &str, a: f64, b: f64) -> Result<SolveResult, String>`
- `pub struct SolveResult { root, iterations, residual }`
- Implements Brent's method; tolerances `BRENT_TOL=1e-10`, `BRENT_MAX_ITER=100`
- Accepts `f(x)` (zero-finding) or `f(x) = g(x)` (rewritten as `f - g = 0`)

### `crates/cli` — binary crates (`integrate`, `solve`)

**`src/main.rs`** → binary `integrate`
- Depends on `integrator-core`
- `demo()`, `run_demo()`, `run_verification()`, `parse_args()`, `parse_bound()`, `main()`
- `enum PlotMode { AsciiOnly, AsciiAndSvg }` — demo uses `AsciiOnly`, user expressions use `AsciiAndSvg`
- `tests/cli.rs` — subprocess integration tests using `CARGO_BIN_EXE_integrate` and `CARGO_BIN_EXE_solve`

**`src/solve.rs`** → binary `solve`
- Depends on `integrator-core` (for plotting) and `solver-core`
- `run_solve_with_svg(equation, a, b, svg_path)` — core logic
- Accepts `<equation> <a> <b>` or `--help` / `-h` / no args

### `crates/diff` — binary crate (`differentiate`)

`src/main.rs` depends on `integrator-core`. Differentiates symbolically and plots `f(x)` + `f'(x)`.

### `crates/web` — Loco web server (`web-server`)

Uses loco-rs 0.16.4 (`default-features = false, features = ["cli"]` — no database). Runs on port 5150.

- `src/app.rs` — `App` struct implementing `Hooks`; registers routes via `controllers::integrate::routes()`
- `src/controllers/integrate.rs` — seven routes:

| Method | Path | Response |
|--------|------|----------|
| `GET` | `/` | Static HTML form (integrate, differentiate, solve) |
| `POST` | `/integrate` | HTML result with inline SVG |
| `GET` | `/api/integrate?expr=&a=&b=` | JSON `{symbolic, numerical, svg}` |
| `POST` | `/differentiate` | HTML result with inline SVG |
| `GET` | `/api/differentiate?expr=&a=&b=` | JSON `{derivative, svg}` |
| `POST` | `/solve` | HTML result with inline SVG |
| `GET` | `/api/solve?equation=&a=&b=` | JSON `{root, residual, iterations, svg}` |

JSON error shape: `{"error": "..."}` with status `200`. Bad numeric query params → framework `422`.

Config at `config/development.yaml` (also at `crates/web/config/development.yaml`); Loco resolves from CWD.

## Symbolic integration limitations

Only handles direct forms: power rule, trig (`sin(x)`, `cos(x)`, `tan(x)`), `exp(x)`, `ln(x)`, `sqrt(x)`, `1/x`, `a^x`, linear combinations, and products where exactly one factor is constant. Composite arguments (e.g. `sin(2*x)`) fall back to numerical only.

## Symbolic differentiation limitations

All rules supported including chain rule, product rule, quotient rule. `abs(x)` returns `Err` (not differentiable symbolically).

## Development practices

- All warnings must be clean: `cargo clippy --workspace -- -D warnings`
- Minimum 96% line coverage per file: `cargo llvm-cov --workspace --summary-only`
- Test-first: new functionality requires tests in the same commit
