# rust-integrator

A Rust workspace implementing symbolic and numerical calculus tools — integration, differentiation, root-finding, and plotting — exposed via a CLI, a web server, and a library API.

## Workspace layout

```
crates/
  parser/       — expression parser (string → AST)
  integrator/   — symbolic & numerical integration, plotting
  diff/         — symbolic differentiation CLI
  solver/       — root-finding (Brent's method)
  cli/          — integrate / solve binaries
  web/          — Loco web server (REST + HTML UI)
```

## Quick start

```bash
cargo build                                        # compile everything
cargo run -p rust-integrator-cli -- "x^2" 0 3     # integrate x² over [0, 3]
cargo run -p rust-integrator-cli -- --demo         # built-in demo suite
cargo run -p rust-integrator-cli --bin solve -- "x^2 - 2" 1 2   # find root
cargo run -p rust-integrator-diff -- "x^3" 0 2    # differentiate x³ over [0, 2]
cargo run -p rust-integrator-web -- start          # web server on port 5150
```

## CLI — `integrate`

```
integrate <expr> [<a> <b>]    Integrate expr over [a, b] (default: [0, 1])
integrate --demo               Run built-in demo and verification suite
integrate --help
```

**Examples**

```bash
integrate "x^2 + sin(x)" 0 3.14
integrate "4*x^3 - 3*x^2"
integrate --demo
```

## CLI — `solve`

```
solve <equation> <a> <b>    Find root of equation in [a, b]
solve --help
```

**Examples**

```bash
solve "x^2 - 2" 1 2
solve "x^2 = 2" 1 2
solve "sin(x)" 3 4
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

Runs on **port 5150**.

### HTML endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | HTML form UI (integrate, differentiate, solve) |
| `POST` | `/integrate` | Integrate — returns HTML result with inline SVG |
| `POST` | `/differentiate` | Differentiate — returns HTML result with inline SVG |
| `POST` | `/solve` | Root-finding — returns HTML result with inline SVG |

Form fields: `POST /integrate` and `POST /differentiate` use `expr`, `a`, `b`. `POST /solve` uses `equation`, `a`, `b`.

### JSON API endpoints

| Method | Path | Response |
|--------|------|----------|
| `GET` | `/api/integrate?expr=&a=&b=` | `{"symbolic": "...\|null", "numerical": 0.0, "svg": "..."}` |
| `GET` | `/api/differentiate?expr=&a=&b=` | `{"derivative": "...", "svg": "..."}` |
| `GET` | `/api/solve?equation=&a=&b=` | `{"root": 0.0, "residual": 0.0, "iterations": 0, "svg": "..."}` |

On error, JSON endpoints return `{"error": "..."}` with status `200`. Bad numeric parameters return `422` from the framework.

**Examples**

```bash
curl "http://localhost:5150/api/integrate?expr=x%5E2&a=0&b=3"
curl "http://localhost:5150/api/differentiate?expr=x%5E3&a=0&b=2"
curl "http://localhost:5150/api/solve?equation=x%5E2-2&a=1&b=2"
```

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
