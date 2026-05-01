## ADDED Requirements

### Requirement: Test-first development
All new functionality SHALL be implemented using a test-first (TDD) workflow:
1. Write a failing test that specifies the desired behavior
2. Write the minimum production code to make the test pass
3. Refactor as needed, keeping tests green

This applies to all new features, bug fixes, and behavioral changes across all crates.

#### Scenario: New feature
- **WHEN** a new capability is added to any crate
- **THEN** at least one test covering the new behavior SHALL exist in the same commit as the implementation

#### Scenario: Bug fix
- **WHEN** a bug is fixed
- **THEN** a regression test reproducing the bug SHALL be added before the fix is applied

---

### Requirement: Minimum line coverage
Every source file in the workspace SHALL maintain a minimum of **96% line coverage** as measured by `cargo llvm-cov --workspace --summary-only`.

The `/coverage` slash command SHALL be used to verify coverage before merging changes.

#### Scenario: Coverage check
- **WHEN** `/coverage` is run
- **THEN** no file SHALL appear below 96% in the Lines Cover column
