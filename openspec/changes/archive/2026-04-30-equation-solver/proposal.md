## Why

The system can integrate and differentiate expressions but cannot solve equations — a fundamental operation users expect alongside calculus. Adding a solver completes the core mathematical toolkit.

## What Changes

- New `solve` function in `integrator-core` supporting both `f(x) = 0` and `f(x) = g(x)` input forms
- Numerical root-finding using Brent's method (robust, no derivative required) over a user-supplied interval `[a, b]`
- Optional symbolic pre-processing: rewrite `f(x) = g(x)` as `f(x) - g(x) = 0` before solving
- New `--solve` mode in the CLI binary
- New `POST /solve` and `GET /api/solve` endpoints in the web server
- ASCII plot showing the function and the located root
- SVG plot (CLI and web) highlighting the root

## Capabilities

### New Capabilities

- `equation-solver`: Core solver API — input forms, algorithm, result type, error conditions, and root refinement behaviour
- `cli-solver`: CLI `--solve` mode — argument schema, output format, plot behaviour, exit codes
- `web-solver`: Web endpoints for equation solving — routes, request/response shapes, error format

### Modified Capabilities

- `cli-interface`: Add `--solve` invocation form to the argument schema table
- `web-api`: Add `/solve` and `/api/solve` routes to the routes table

## Impact

- `crates/integrator/src/integrator.rs` — new `solve()` public function and `SolveResult` type
- `crates/integrator/src/lib.rs` — re-export `solve`, `SolveResult`
- `crates/cli/src/main.rs` — new `--solve` argument branch
- `crates/web/src/controllers/integrate.rs` — two new route handlers; route registration updated in `app.rs`
- No new dependencies — Brent's method is pure numerical code requiring only `std`
- No breaking changes to existing APIs
