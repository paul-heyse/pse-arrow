# Test contract repair review — 2026-09-24

## 1. Scope, purpose and coverage

Change-tier target review of CPU admission, fitting admission, SQL validation bindings,
publication resource admission, obsolete inference and governance. The initial executable
review reproduced 73 failures in 1771 tests with workspace nextest, no fail fast and
`pse-relations/force-validate`. Full process-science qualification remains outside this
repair and belongs to Plan 14 M22. The Plan 14 direction governs the binding's known
conflict with the historical custom compiler blueprint.

## 2. Authority and identity map

Registry declarations own relation contracts. Session state owns effective partition
configuration. Deployment workers bound CPU permits. Library mathematics owns fitting
derivatives. Native adapter availability gates executable nonconstant fits. Delta
publication control owns visibility; memory exhaustion grants no success claim.

| Quantity/model element | Unit and basis | Reference/convention and validity | Authority |
|---|---|---|---|
| Fit coordinates | Authored physical port units; complete shared and local variable inventory | Finite initial values, declared bounds and positive scales | Generated model/fit declarations and quantity registry |
| Observations and uncertainty | Observation units converted to output units; sigma uses difference scaling | Finite included values, positive sigma and importance | Quantity conversion and fitting admission |
| Dynamic coordinate control | Authored minute/time coordinates converted through the compiler | Smooth admitted integration and consistent initial-value dependencies | Shared compiled functions and Diffsol binding |

## 3. Contracts and invariants

Applicable DP-01, DP-02, DP-04, DP-07, DP-09, DP-13, DP-16, DP-20 and DP-24 are
supported within the exercised repair scope. Empty required products must exist with
exact schemas; missing members refuse. SQL CHECK requires an explicit engine binding.
Partitions may outnumber workers. Nested work retains the parent's admitted owner.
Zero-variable routing is decided after local states and shared parameters are assembled.
The existing bounds, derivative extent, observation ownership and original-model quality
checks remain the well-posedness boundary for this repair; broad structural and process
qualification remains M22. Other principles have no changed contract in this scope. PS-01, PS-08 and PS-10 require
preserving physical value, derivative and original-model result assertions; their broad
scientific claims are unresolved, not established by these repairs.

## 4. Derivation and execution

Delete orphaned inference and its exclusive declarations and consumers. Retain the
invariant checker used by inspection and physical code generation. Fit mathematics
precedes native route admission; only a zero-coordinate problem evaluates directly.
Publication dependency children settle before a parent occupies an output slot.

| Stage | Formulation and derivatives | Scaling and class | Outcome and postcheck |
|---|---|---|---|
| Mathematical fit preparation | Existing compiled symbolic derivatives; exact steady Hessian or declared transient sensitivity route | Explicit fit scales, finite dense storage, complete coordinates | Typed admission errors before execution |
| Constant fit | Original compiled evaluation with no optimization variables | No native solver required | Fresh predictions and original constraint/bound quality |
| Variable fit | Same oracle and native derivative contract | Ipopt/POUNCE capability admitted after mathematical preparation | Existing native termination plus independent original-model evaluation |

## 5. Journeys

Exercise partitioned catalog operations, invariant validation, all-fixed and variable
fits, linked dynamic sensitivities, and exact source publication with a cold reopen.
Retain negative controls for missing products, unavailable adapters and budget refusal.

## 6. Gates

| Gate | Verdict and scope |
|---|---|
| G1 | Pass at repair scope: blueprint revision 50, exclusive declarations deleted, generated consistency passed; formal ADR acceptance remains pending. |
| G2 | Pass at repair scope: surviving semantic assertions pass in the full default suite. |
| G3 | Pass at repair scope: explicit SQL binding and unavailable-adapter controls pass. |
| G4 | Pass at repair scope: bounded cold source reopening and existing recovery controls pass. |
| G5 | Pass at repair scope: effective CPU admission, nested DML inheritance, cancellation/drop and 128 MiB publication pass. |
| G6 | Pass at repair scope: mathematical fitting and nine linked dynamic/fitting tests pass. |
| G7 | Runtime evidence complete (1695 passed); workspace static qualification remains unresolved. |
| G8 | Pass at design scope: existing DataFusion, Tokio, syn and native routing supply the mechanisms. |
| PS-G1–PS-G3 | Unresolved for full process qualification; M22 remains open. |

## 7. Findings

1. **High — CPU units disagree.** Root queries acquire partition-count permits from a
   worker-count semaphore, refusing valid single-query configurations. Bound admission
   by deployment workers and the effective session partitions; retain nested ownership.
2. **High — orphaned rule contracts.** Forty-six failures refer to removed compiler
   outputs or unsupported rule execution. Reintroducing those schemas would restore a
   second compiler. Retire exclusive machinery and preserve authored FK assertions.
3. **High — premature solver admission.** All-fixed evaluation requires no adapter;
   derivative preparation also has no solver dependency. Separate mathematics and route
   admission, retaining refusal for nonconstant executable fits without an adapter.
4. **Medium — fixture authority gaps.** SQL fixtures lack native bindings, product
   fixtures omit declared support, and one diagnostic test expects a removed wrapper.
   Correct fixtures at their application boundary without weakening production checks.
5. **High — publication admission scope.** Concurrent unbounded provisioning exhausts
   a 128 MiB fixture pool. A targeted rerun also exposed local completion proofs retaining
   full attempt receipts despite comparing only member/identity/input fields. Use common
   finite resources, bound provisioning, and retain only those typed proof fields under
   generated HeapUsage accounting. Keep durable JSON accounting. Recovered writes must
   still return before input execution.
6. **Medium — text-based governance.** Generic diagnostic implementations and narrow
   unsafe allowances are falsely rejected. Use syn structural parsing with scoped
   negative controls; retain actual compiler enforcement and the existing allowlist.

## 8. Library-leverage ledger

Tokio semaphores provide bounded shared admission; DataFusion partitions are tasks,
not dedicated workers. Existing output admission provides write concurrency without a
new scheduler. Existing `Route::Constant` represents direct evaluation. Syn's parsed
items/attributes replace textual matching. Local pinned sources and Context7 syn
ItemImpl/attribute documentation establish interface shape, not runtime acceptance.

## 9. Alternatives

Restoring custom inference has no production consumer. Deleting all of pse-rules
would remove live invariants. Raising the memory limit would hide unbounded concurrency.
Skipping all fitting tests would hide mathematical regressions. Blanket unsafe
allowances would widen the authorized unsafe surface. Existing library mechanisms
and explicit fixture boundaries are the simplest complete repair.

## 10. Verification plan

**Tested:** workspace nextest with `--workspace --locked --no-fail-fast --features
pse-relations/force-validate` ran 1695 tests: all passed, none skipped, against the
required zero-failure baseline. Linked native dynamics/fitting ran nine selected tests:
all passed. Generated consistency, all-target compilation and ADR/docs checks passed.
Exact commands, conditions and outstanding Clippy findings are recorded in
[the implementation outcome](../../plans/14-test-contract-repair.md#verification).

## 11. Authority changes and exceptions

ADR-0086 and blueprint revision 50 document retirement and structural governance.
The ADR remains proposed; formal decision/design PR and review are required before
merge. No memory baseline, skipped failure or compatibility executor is introduced.

## 12. Decision

**Accept-scoped:** the implemented repairs and executor retirement satisfy the exercised
runtime contracts. This verdict does not approve merge while workspace static checks
remain red, accept ADR-0086, or qualify M21/M22. Physical fixture generator and
flight-cache lint findings require resolution before claiming the repository's zero
quality baseline.
