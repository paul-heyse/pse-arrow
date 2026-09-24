---
title: Repair surviving test contracts and retire orphaned rule execution
status: in-progress
date: 2026-09-24
adrs: [ADR-0086]
---

# Test contract repair

## Scope

Implement the failure review's six repairs within Plan 14's target. The starting
workspace nextest run reproduced 1771 tests: 1698 passed and 73 failed, with
`--workspace --locked --no-fail-fast --features pse-relations/force-validate`.
The required final baseline is zero failures. This repair does not close M21 or M22.

## Implementation

- CPU admission uses effective session partitions capped by deployment workers.
  Production and test fixtures attach the same admission extension. Nested execution
  borrows the owner's permit; cancellation does not release another live owner.
- Delete the orphaned rule compiler/executor, exclusive declarations and projections,
  fixture tests, benchmark arm and recipe. Keep the invariant checker and its shared
  row-key projection helper. Preserve authored foreign-key checks.
- Separate fitting mathematics from executable adapter admission. A zero-coordinate
  fit has `Route::Constant`; pure derivative tests use mathematical preparation.
  Dynamic integration tests require their adapter, with explicit unavailable controls.
- Bind SQL validation explicitly at application fixture boundaries. Remove hidden
  test-only default installation. Test both refusal and explicit binding.
- Supply the complete empty product support closure and current source profile in
  durability fixtures; assert the current bounded diagnostic report shape.
- Use finite common publication resources at 128 MiB with one output writer. Hold
  output admission during provisioning and write settlement. A parent releases its
  provisioning slot before settling child writers and reacquires for its write. Recovery still returns
  before input execution; dependent writers settle before parent output admission.
- Retain only the typed fields consumed by local write-completion checks. Full native
  dependency receipts stay durable; the local optimization no longer copies them or
  retains their JSON decoding allowance. Use generated `HeapUsage` before allocation.
- Parse diagnostic implementations and unsafe allowances using pinned syn, retaining
  negative controls and compiler checks. Keep num-dual 0.14.2 and reconcile the
  blueprint under proposed ADR-0086. Regenerate all affected contracts and fixtures.

## Verification

- **Tested:** `cargo nextest run --no-fail-fast --workspace --locked --features
  pse-relations/force-validate --status-level fail --final-status-level fail`:
  **1695 passed, zero failed, zero skipped** (default nextest profile, 120 binaries).
  Required baseline: zero failures. The reduction from 1771 includes deletion of
  exclusive custom-executor tests, retention of live invariant tests, adapter-specific
  numerical test gating and new negative/regression controls.
- **Tested:** `cargo test --doc --workspace --exclude pse-py --locked --features
  pse-relations/force-validate`: **22 passed, zero failed, zero ignored**. The Python
  cdylib has no Cargo doctest target; this follows the repository's doctest scope.
- **Tested:** linked native scope, `cargo nextest run --no-fail-fast -p pse-runtime
  -p pse-relations --lib --locked --features
  pse-runtime/native-solvers,pse-relations/force-validate -E
  'test(workflow::dynamics::) or test(workflow::fitting::)' --status-level fail
  --final-status-level fail`: **9 passed, zero failed**, 61 filtered out. Native math
  and solver environments were sourced, the container runner unset, Ipopt libraries
  on the loader path, and native math thread counts set to one.
- **Tested:** `cargo xtask codegen --check` matched all three schema targets and
  Ipopt bindings, including generated Python annotation checks. `just check` compiled
  all targets. `just docs`, ADR static lint (86 records), and register lint (33 rows)
  passed. These commands are repair evidence; aggregate Plan 14 closure guards remain
  intact.
- **Implemented, not lint-qualified:** workspace Clippy with all targets, force-validation
  and `-D warnings` stopped on 591 generated quantity-code findings (582 unseparated
  identity literals and nine long helper functions). Their owner is the physical
  fixture generator; generated files must not be patched directly. The zero baseline
  remains required for merge. A narrower `cargo clippy --no-deps -p pse-engine
  -p pse-rules -p pse-runtime -p pse-tests-governance --all-targets --locked --features
  pse-relations/force-validate -- -D warnings` passed the revised governance parser
  after a boolean-expression cleanup, then stopped on four existing flight-cache
  findings: identical match arms, type complexity and two missing error sections.

## Remaining work

Complete the outstanding static qualification and formal ADR/design PR. M21/M22
acceptance remains separate; this document does not establish Python, performance or
full scientific/runtime qualification.

## Outcome

**Implemented and Tested:** the surviving default Rust suite reaches its zero-failure
baseline. Private DML now inherits the enclosing command's live CPU admission while
retaining its actual bindings. Quantity validation fixtures install their SQL engine
explicitly. Exact source publication and cold reopening pass at the original 128 MiB
limit. Orphaned inference machinery and exclusive contracts were deleted; live
invariant checking remains.

**Mistake corrected:** bounding publication concurrency alone did not satisfy the
128 MiB test. Local completion proofs still retained full dependency receipts and
their JSON decoding allowance. Retaining only the typed fields actually compared
resolved the failure without raising the pool limit or weakening durable accounting.

**Deliberate deviations:** scoped commands executed the user-authorized repair checks
while M21's aggregate closure guards remained intact. The ADR remains proposed until
its formal decision/design PR. Workspace static qualification remains open as described
above; passing tests do not close Plan 14.
