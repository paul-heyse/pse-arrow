# Integrated test architecture and assurance

**Interface-checked · 2026-11-07 · source `3419aca2a356a700a351c1706ba629c8ca61bfbc`.**
The [audit index](README.md) records the baseline, evidence limits and finding ownership.

## How the current system composes

### Selection, execution and evidence

The managed host resolves a selected package, target, feature and filter against the checkout.
The correctness adapter verifies Arrow validation in the admitted Cargo unit graph. The host
builds and lists the selected binaries, authenticates the captured nextest, binds the applicable
fixture declarations to the observed population, and executes from that capture. It distinguishes
prerequisite refusal, no selection, test failure and incomplete execution. Native result readers
project the captured report and physical settlement. Separate Cargo entry points support generic
correctness and doctests. In the managed route, a pure selection can reach a test without launching
a fixture or preparing a scientific provider. [Host test workflow](../../../crates/pse-host/src/test_workflow/mod.rs) ·
[Correctness adapter](../../../crates/pse-host/src/correctness.rs) ·
[Validation commands](../../dev/validation-assessment.md)

The convergence is strongest inside one captured invocation. Qualification-only declarations,
name-based scheduling and the distinct generic runner still leave gaps between the test a caller
requests, its actual prerequisites, and the evidence returned. The native nextest configuration
does assign the scientific group to selected runtime cases. That establishes an exclusion policy
for those matches within its effective invocation scope. Changing a test name or reaching an
equivalent pool-owning journey through another selection can change that scope. [DTA-015](evidence-routes-and-execution.md#dta-015) ·
[DTA-050](evidence-routes-and-execution.md#dta-050) · [DTA-063](test-execution-system.md#dta-063)

### Database and process ownership

The split now follows the effect being tested. SurrealKit executes generated-schema and actor
contracts against a test-owned endpoint. Rust tests retain typed callers, physical migration and
reopen, service failure and original child drain. The SurrealKit control checks exact case results
and challenges bad assertion values, setup finality, interruption and drift paths. The host distinguishes a borrowed
logical database, a target-owned service and prepared inputs. Some target-owned constructors still
restate environment-to-spec defaults. A borrowed fixture also changes the executor used by
protected-result cleanup, so that lane's evidence is scoped to the fixture-owned path.
[SurrealKit caller](../../../crates/pse-operations/tests/native_service/surrealkit.rs) ·
[Logical fixture owner](../../../crates/pse-operations/src/native_fixture/logical.rs) ·
[DTA-011](doubles-and-coupling.md#dta-011) · [DTA-059](doubles-and-coupling.md#dta-059)

### Scientific evidence

Four retained parity exchanges have an actual native recipe, and authored reference packages
have a native conformance route. The ordinary Proto reference path sends scoped inputs to an
isolated oracle and compares independently generated values. Evidence still depends on what enters
the comparator. Selected covariance and sensitivity allowances come from the candidate. A native
receipt or a passing comparison does not establish population coverage across supported models.
Trace each claim to its units, authored bounds, expected-result derivation and required cases.
The accepted database and fixture cases recorded by Plan67c and the remaining population and
assembled scientific acceptance in Plan66l have distinct scopes. [DTA-049](oracles-and-acceptance.md#dta-049) ·
[DTA-061](oracles-and-acceptance.md#dta-061) · [Plan67c](../../plans/67c-surrealkit-and-rust-test-consolidation.md) ·
[Plan66l](../../plans/66l-required-population-and-assembled-acceptance.md)

## Representative change and failure scenarios

- **Add a scientific case.** Source-built inventory and the unit-graph proof answer which
  executable and compiler closure the host admitted. The case still needs the right fixture role,
  group membership and independent expectation. A new declaration or a renamed test can miss a
  name filter even when the case remains compilable.
- **Repeat a focused check or inspect a retained population.** Plan67 removes routine capsule
  construction from the selected ordinary runtime and PID populations, and the managed route uses
  one build/list capture for its later execution. Group command construction verifies the entire
  inventory for each group. A retained page can verify the same binary for each selected row.
  The same identity premise is examined repeatedly within a single operation. A workspace
  membership control also requests the complete dependency graph even when it uses only the member
  set. These are source-visible work boundaries. Their elapsed impact is unmeasured.
  [DTA-039](execution-efficiency.md#dta-039) · [DTA-071](execution-efficiency.md#dta-071)
- **Fail after readiness.** A service or process can be ready while an assertion or child drain
  remains outstanding. SurrealKit and the selected native service controls observe separate exit,
  verdict and disposition facts. The sampled supervisor tests challenge a real child with an
  independent foreign-process control and use PIDFD exit, typed outcomes and `waitid` reaping as
  observations. The park control has a bounded absence check and a separately collected helper
  that returns when no fixture marker is supplied. Its parent exercises the actual fault path.
  [Supervisor tests](../../../crates/pse-diagnostics/src/supervised_child/tests.rs) ·
  [DTA-050](evidence-routes-and-execution.md#dta-050) · [DTA-057](evidence-routes-and-execution.md#dta-057)

## Contrasting strengths

The `pse-testkit` boundary and resolved dependency governance protect focused engine tests from
runtime and storage dependency reach. Real-caller tests remain where isolation would change the
claim. For example, xtask's documentation checks use explicit temporary trees and controlled
lychee responses to test the diagnostic merger, and a separate document-owner check reads the
actual repository. The controlled responses establish interpretation, while the real repository
case checks wiring. In xtask's conformance controls, authored fixtures and an unknown-declaration
refusal exercise selection, and a real Arrow report checks publication and overwrite behavior.
[Native boundary governance](../../../tests/governance/tests/native_engine_boundaries.rs) ·
[Documentation controls](../../../xtask/src/documentation/check/tests.rs) ·
[Document owner controls](../../../xtask/src/documentation/ownership/tests.rs) ·
[Conformance controls](../../../xtask/src/development/conformance_tests.rs)

## Assessment limits

These are representative source traces. The [audit index](README.md#scope-strengths-and-limits)
distinguishes current examination, inherited evidence and unresolved runtime questions.
