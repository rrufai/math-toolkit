## Context

The rust-integrator codebase consists of three crates sharing a common `integrator-core` library. No specifications exist — all behavioural contracts are implicit in the implementation and its 112 unit tests. This change creates a spec layer without altering any code.

## Goals / Non-Goals

**Goals:**
- Capture all observable system behaviour as written, testable specifications
- Establish 6 capability spec files that can serve as the baseline for future changes
- Make implicit rules (operator precedence, fallback behaviour, tolerance constants) explicit and auditable

**Non-Goals:**
- Changing any code
- Specifying behaviour that doesn't exist yet
- Resolving ambiguities by changing what the code does — ambiguities are documented as-is

## Decisions

### One spec file per domain, not per crate

**Decision**: Group specs by mathematical/interface domain, not by Rust crate.

**Rationale**: The CLI and web server both use `integrator-core`; a per-crate split would duplicate the integration and differentiation rules. Domain grouping gives each spec a clear, stable identity.

**Alternative considered**: One spec per crate (`integrator-core.md`, `cli.md`, `web.md`). Rejected because it would force `integrator-core.md` to be enormous and wouldn't align specs with the concepts users think in.

---

### Symbolic calculus as a single spec

**Decision**: Combine symbolic integration and symbolic differentiation into one `symbolic-calculus` spec rather than two separate files.

**Rationale**: Both share the same `Expr` AST, the same `is_const` / `canonicalize` helpers, and the same "supported patterns" mental model. Separating them would require duplicating the shared context.

**Alternative considered**: `symbolic-integration.md` + `symbolic-differentiation.md`. Rejected due to duplication.

---

### Extract exact constants verbatim from source

**Decision**: Tolerance values (`ABS_TOL = 1e-10`, `REL_TOL = 1e-8`, `MAX_DEPTH = 50`) and plot dimensions are copied exactly from source code, not rounded or approximated.

**Rationale**: These are normative constants — the spec is the source of truth for what the implementation should do. If the code ever diverges, the spec wins.

---

### Spec format: requirements + scenarios

**Decision**: Use `### Requirement` / `#### Scenario` format with SHALL/MUST language throughout.

**Rationale**: Matches OpenSpec schema expectations; scenarios are directly testable.

## Risks / Trade-offs

- **Spec staleness** → Mitigation: spec files live in the repo; any PR changing observable behaviour should update the relevant spec.
- **Incomplete coverage** → Mitigation: specs are extracted from the full codebase read including all 112 unit tests and CLI/web integration tests. Edge cases documented in tests are included.
- **Ambiguity in "supported patterns"** → Some symbolic integration edge cases (e.g., `x^0`, `0*f(x)`) are handled by simplification, not by direct integration rules. These are documented under the simplification sub-section of `symbolic-calculus`.
