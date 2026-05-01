## Why

The rust-integrator codebase has no written specifications — behaviour, contracts, and constraints live only in the implementation. Extracting them now creates a stable reference for future changes and makes implicit rules explicit.

## What Changes

- Create 6 new spec files covering the full system behaviour as it exists today
- No code changes — this is a documentation-only change

## Capabilities

### New Capabilities

- `input-language`: Grammar, tokenisation rules, operator precedence, implicit multiplication, supported constants and functions, and parse error conditions
- `symbolic-calculus`: Symbolic integration rules, symbolic differentiation rules, supported/unsupported patterns, fallback behaviour, and the `Abs` constraint
- `numerical-integration`: Adaptive Simpson's rule algorithm, tolerance constants (`ABS_TOL`, `REL_TOL`, `MAX_DEPTH`), convergence behaviour, and edge cases
- `plotting`: SVG and ASCII plot contracts — dimensions, sampling counts, y-range calculation, NaN/Inf handling, tick formatting, and polyline segmentation
- `cli-interface`: Argument schema, operation modes (`--demo`, `--verify`, expression mode), output files, default bounds, and exit codes
- `web-api`: HTTP routes, request/response contracts for HTML and JSON endpoints, error format, port, and HTML escaping guarantees

### Modified Capabilities

## Impact

- Adds `openspec/specs/` tree (6 new spec files)
- No changes to `crates/` source code, `Cargo.toml`, or any tests
