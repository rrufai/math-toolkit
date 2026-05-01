## 1. Core Solver — integrator-core

- [x] 1.1 Add `SolveResult` struct (`root: f64`, `iterations: u32`, `residual: f64`) to `crates/integrator/src/integrator.rs`
- [x] 1.2 Implement `solve(equation: &str, a: f64, b: f64) -> Result<SolveResult, String>` — parse input, handle `=` split, delegate to Brent
- [x] 1.3 Implement Brent's method loop (bisection + secant + inverse quadratic interpolation, `ABS_TOL=1e-10`, max 100 iterations)
- [x] 1.4 Handle early-exit cases: same-sign bracket → `Err`, `f(a)==0` → return `a`, `f(b)==0` → return `b`
- [x] 1.5 Re-export `solve` and `SolveResult` from `crates/integrator/src/lib.rs`

## 2. Core Solver — Tests

- [x] 2.1 Test `solve("x^2 - 2", 1.0, 2.0)` — root within `1e-10` of `√2`, residual `< 1e-10`
- [x] 2.2 Test `solve("x^2 = 2", 1.0, 2.0)` — same result as single-expression form
- [x] 2.3 Test `solve("sin(x)", 3.0, 4.0)` — root within `1e-10` of `π`
- [x] 2.4 Test `solve("sin(x) = cos(x)", 0.5, 1.0)` — root within `1e-10` of `π/4`
- [x] 2.5 Test `solve("x^2", 1.0, 3.0)` — returns `Err` containing `"no sign change"`
- [x] 2.6 Test `solve("sin(x +", 0.0, 1.0)` — returns `Err` with parse error message
- [x] 2.7 Test `solve("x = 1 = 2", 0.0, 3.0)` — multiple `=` signs returns `Err`
- [x] 2.8 Test `solve("x^2 - 4", 1.0, 3.0)` — root within `1e-10` of `2.0`
- [x] 2.9 Verify `integrator_core::solve` and `integrator_core::SolveResult` are accessible (re-export works)

## 3. CLI — --solve mode

- [x] 3.1 Add `--solve <equation> <a> <b>` branch to `parse_args()` in `crates/cli/src/main.rs`
- [x] 3.2 Print usage error and exit code `1` when `--solve` is missing bounds
- [x] 3.3 Implement solve output box (box-drawing characters, root to 10 dp, residual in scientific notation, iteration count)
- [x] 3.4 Call `render_ascii_string` for ASCII plot of `f(x)` over `[a, b]` with root marked; write `solve.svg` via `write_svg`
- [x] 3.5 On solver error (no sign change, parse error) print to stderr and exit code `1`

## 4. CLI — Tests

- [x] 4.1 Integration test: `integrate --solve "x^2 - 2" 1 2` — stdout contains root value `1.4142135623`
- [x] 4.2 Integration test: `integrate --solve "x^2" 1 3` — stderr contains `"no sign change"`, exit code `1`
- [x] 4.3 Integration test: `integrate --solve "sin(x +" 0 4` — stderr contains parse error, exit code `1`
- [x] 4.4 Integration test: `integrate --solve "x^2 - 2"` (no bounds) — exit code `1` with usage error

## 5. Web — Route handlers

- [x] 5.1 Add `solve_form` handler (`POST /solve`) accepting `application/x-www-form-urlencoded` with fields `equation`, `a`, `b` (parsed as strings then `f64`) — returns HTML
- [x] 5.2 Add `api_solve` handler (`GET /api/solve`) accepting serde query params `equation: String`, `a: f64`, `b: f64` — returns JSON
- [x] 5.3 Implement shared solve logic: call `integrator_core::solve`, generate SVG via `render_svg`, build HTML result page with equation, interval, root (10 dp), residual, iteration count, inline SVG
- [x] 5.4 HTML error page for `POST /solve` failures (no sign change, parse error)
- [x] 5.5 JSON error response for `GET /api/solve` failures: `{"error": "..."}` with status `200`
- [x] 5.6 Register `POST /solve` and `GET /api/solve` routes in `crates/web/src/app.rs`
- [x] 5.7 Update `GET /` index page HTML to include Solve section with form `action="/solve"` and inputs `equation`, `a`, `b`

## 6. Web — Tests

- [x] 6.1 Test `POST /solve` route exists — `200 OK` HTML (not `404`)
- [x] 6.2 Test `GET /api/solve?equation=x%5E2-2&a=1&b=2` route exists — `200 OK` JSON
- [x] 6.3 Test `POST /solve` with `equation=x^2-2&a=1&b=2` — HTML contains root `1.4142135623`
- [x] 6.4 Test `GET /api/solve` success — `root` within `1e-10` of `√2`, `residual < 1e-10`, response has `svg` field
- [x] 6.5 Test `GET /api/solve?equation=x%5E2&a=1&b=3` — response body `{"error": "no sign change..."}` with status `200`
- [x] 6.6 Test `GET /api/solve?equation=sin(x%2B&a=0&b=4` — response body `{"error": "<parse error>"}` with status `200`
- [x] 6.7 Test `GET /` — HTML contains `action="/solve"` and input named `equation`
