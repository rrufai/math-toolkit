# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build                           # compile all crates
cargo test --workspace                # run all tests
cargo test -p integrator-core         # run lib tests only (112 tests)
cargo test -p rust-integrator-cli     # run CLI unit + integration tests
cargo test <name>                     # run a single test by name (e.g. cargo test test_sym_sin)
cargo run -p rust-integrator-cli -- --demo     # run the CLI demo
cargo run -p rust-integrator-cli -- "x^2" 0 3  # integrate via CLI
cargo run -p rust-integrator-web -- start      # start the web server (port 5150)
cargo clippy                          # lint
```

## Architecture

Cargo workspace with three crates under `crates/`:

### `crates/integrator` — lib crate (`integrator-core`)

**`src/parser.rs`** — string → AST
- `tokenize()` converts the input string to `Vec<Token>`
- `Parser` (recursive descent) produces an `Expr` AST
- `Expr` is a public enum with variants: `Num`, `Var`, `Neg`, `Add`, `Sub`, `Mul`, `Div`, `Pow`, `Sin`, `Cos`, `Tan`, `Exp`, `Ln`, `Sqrt`, `Abs`
- `Expr::eval(x)` evaluates at a point; `Expr::to_string_repr()` pretty-prints it
- Grammar (low → high precedence): `expr → term → unary → power → primary`; `^` is right-associative

**`src/integrator.rs`** — AST → integral
- `integrate_symbolic(expr)` — pattern-matches to produce a closed-form antiderivative; returns `Err` when no rule matches
- `integrate_numerical(f, a, b)` — adaptive Simpson's rule; tolerances `ABS_TOL=1e-10`, `REL_TOL=1e-8`, `MAX_DEPTH=50`
- `integrate(expr_str, a, b)` — convenience wrapper returning `IntegrationResult`
- `simplify()`, `canonicalize()`, `is_const()` are `pub(crate)` (tested inside the same crate)

**`src/plot.rs`** — SVG and ASCII plots
- `render_svg(...)` → `String` — full SVG markup for inline embedding
- `render_ascii_string(...)` → `String` — ASCII plot for terminal output
- `write_svg(...)` — writes SVG to file; `print_ascii(...)` — prints ASCII to stdout

**`src/lib.rs`** — re-exports all public API + `pub fn first_var(expr)` + 112 unit tests

### `crates/cli` — binary crate (`integrate`)

`src/main.rs` depends on `integrator-core`. Contains `demo()`, `run_demo()`, `run_verification()`, `parse_args()`, `parse_bound()`, `main()`. Uses `enum PlotMode { AsciiOnly, AsciiAndSvg }` — demo calls use `AsciiOnly`, user expressions use `AsciiAndSvg`. `tests/cli.rs` are subprocess integration tests using `CARGO_BIN_EXE_integrate`.

### `crates/web` — Loco web server (`web-server`)

Uses loco-rs 0.16.4 (`default-features = false, features = ["cli"]` — no database). Runs on port 5150.

- `src/app.rs` — `App` struct implementing `Hooks`; registers routes via `controllers::integrate::routes()`
- `src/controllers/integrate.rs` — three routes:
  - `GET /` — static HTML form
  - `POST /integrate` — HTML result with ASCII plot + inline SVG
  - `GET /api/integrate?expr=&a=&b=` — JSON response `{symbolic, numerical, ascii_plot, svg}`

Config at `config/development.yaml` (also duplicated at `crates/web/config/development.yaml`); Loco resolves it from the process CWD.

## Symbolic integration limitations

Only handles direct forms: power rule, trig (`sin(x)`, `cos(x)`, `tan(x)`), `exp(x)`, `ln(x)`, `sqrt(x)`, `1/x`, `a^x`, linear combinations, and products where exactly one factor is constant. Composite arguments (e.g. `sin(2*x)`) fall back to numerical only.
