# rust-integrator

A Rust workspace implementing symbolic and numerical calculus tools — integration, differentiation, root-finding, and plotting — exposed via a CLI, a web server, and a library API.

## Workspace layout

```
crates/
  parser/       — expression parser (string → AST)
  integrator/   — symbolic & numerical integration, plotting
  diff/         — symbolic differentiation CLI
  solver/       — root-finding (Brent's method)
  cli/          — integrate / --solve CLI binary
  web/          — Loco web server (REST + HTML UI)
```

## Quick start

```bash
cargo build                                        # compile everything
cargo run -p rust-integrator-cli -- "x^2" 0 3     # integrate x² over [0, 3]
cargo run -p rust-integrator-cli -- --demo         # built-in demo suite
cargo run -p rust-integrator-diff -- "x^3" 0 2    # differentiate x³ over [0, 2]
cargo run -p rust-integrator-web -- start          # web server on port 5150
```

## CLI — `integrate`

```
integrate <expr> [<a> <b>]             Integrate expr over [a, b] (default: [0, 1])
integrate --solve <equation> <a> <b>   Find root of equation in [a, b]
integrate --demo                       Run built-in demo and verification suite
integrate --help
```

**Examples**

```bash
integrate "x^2 + sin(x)" 0 3.14
integrate "4*x^3 - 3*x^2"
integrate --solve "x^2 - 2" 1 2
```

## CLI — `differentiate`

```
differentiate <expr> [<a> <b>]    Differentiate expr and plot over [a, b] (default: [0, 1])
differentiate --help
```

**Examples**

```bash
differentiate "x^3 + sin(x)" 0 3.14
differentiate "exp(x^2)"
```

## Web server

```bash
cargo run -p rust-integrator-web -- start
```

Runs on **port 5150**. Three endpoints:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | HTML form UI |
| `POST` | `/integrate` | HTML result with ASCII plot + SVG |
| `GET` | `/api/integrate?expr=&a=&b=` | JSON: `{symbolic, numerical, ascii_plot, svg}` |

## Supported syntax

| Feature | Examples |
|---------|---------|
| Variables | `x`, `t`, `u`, `z`, ... |
| Constants | `pi`, `e` |
| Operators | `+` `-` `*` `/` `^` (right-associative) |
| Functions | `sin`, `cos`, `tan`, `exp`, `ln`, `log`, `sqrt`, `abs` |
| Implicit multiplication | `3x^2`, `2t^3`, `4sin(u)` |

## Symbolic integration

Handles direct forms: power rule, `sin(x)`, `cos(x)`, `tan(x)`, `exp(x)`, `ln(x)`, `sqrt(x)`, `1/x`, `a^x`, linear combinations, and products where exactly one factor is constant. Composite arguments (e.g. `sin(2*x)`) fall back to numerical integration.

## Testing

```bash
cargo test --workspace    # 268 tests across all crates
cargo clippy --workspace -- -D warnings
```

## Output

Both CLIs produce an ASCII plot in the terminal and write an SVG file (`integrate.svg` / `differentiate.svg`) to the current directory.
