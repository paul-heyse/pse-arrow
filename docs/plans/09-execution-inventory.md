---
title: Native caching implementation and qualification inventory
status: done
date: 2026-09-17
adrs: [ADR-0068, ADR-0069, ADR-0070]
phase: 1
---

# Native caching implementation and qualification inventory

This is the completed source/caller/fixture and qualification ledger for
[Plan 09](09-native-caching-and-pivot-completion.md). All carried Plan 08 E00–E10
obligations and C00–C12 are closed for the existing local architecture.

## Final C12 qualification

**Complete — 2026-09-18.** C00–C12, including the carried Plan 08 E00–E10
scope and legacy deletions, are implemented and qualified for the existing local
process-model architecture. The [acceptance review](../design_review/reviews/design_review_native-cache-pivot-acceptance_2026-09-17.md)
assesses Q01–Q14, K01–K11 and G1–G7 independently. The
[execution inventory](09-execution-inventory.md) and
[measurements](09-cache-measurements.md) preserve commands, source boundaries,
failures repaired during qualification, and the limits of the claims.

**Tested, baseline zero:** the exact union of the initial Rust campaign and affected
continuations covers **1,002/1,002 ordinary tests** (993 original, nine added, none
removed). This is aggregate coverage across recorded source boundaries, not a claim
that one untouched final full-suite run passed. Final affected checks include 268
schema/relation/rule/compiler cases and 253 provider/source cases. The last kernel
case passes in **309.658 seconds** under the unchanged **360-second** CI deadline,
with all 390 outputs, exact unit assertions and Delta publication/reopen retained.
The [Rust ledger](../design_review/evidence/native-cache-rust-qualification-2026-09-18.json)
identifies every accepted test and archived failure.

**Tested and Measured:** four complete heater/mixer publications and independent
Rust/Python inspection, 72 final Python unit/component tests, 12 linked solver
cases, 84 feature checks, and the 84-record native cache matrix have passing receipts.
The final extension independently reopens all four engineering publications.
Formatting, strict Clippy in both modes, generated equality, Python quality,
environment checks, rustdoc and book checks pass. Earlier source boundaries for
solver/features/doctests/benchmarks and Rust engineering publications remain explicit;
none is silently relabeled as a rerun on the final source.

No timeout, retry allowance, runtime budget, scientific assertion or legacy pathway
was added to obtain a pass. No new simulator function, numerical IDAES parity,
remote destructive coordination or universal speedup is claimed. Proposed ADRs
remain proposed; this completion record does not change their status or the blueprint.

### Final repair and verification boundary

The final production archive is `build/plan09-acceptance-20260918-25`.
`/tmp/pse-plan09-declarations97.log` records 268 schema/relation/rule/compiler
passes in 21.884 seconds. `/tmp/pse-plan09-provider-final99.log` records all
253 catalog/source cases passing under the existing CI profile and force-validation.
`/tmp/pse-plan09-kernel98.log` records the complete kernel journey, 309.658 seconds;
its 31.321-second composition and 231.847-second execution are measured observations,
with concurrent checks and one bounded debugger sample, not a controlled speedup.
`/tmp/pse-plan09-py-test25.log` records 72 passes in 7.12 seconds, 32 workers.
All have zero failures against baseline zero. Exact test names and UUIDs are in the
Rust ledger; original failures and source boundaries are retained.

Late repairs use native projections and immutable providers to execute construction
and witness conditions once; preserve one owner per shared expanded producer;
move native Delta metadata into the unchanged receipt decoder; and borrow the
registry's existing exact declaration projection. External declarations with forged
matching IDs/fingerprints still refuse. No alternate cache authority or receipt
format was introduced. The DataFusion capability scan reports only two tiny,
planning-only test SessionContexts; no production gap was found in the changed
scope. The Delta capability scan reports no findings.

The receipts below are chronological history, including failures subsequently
repaired. Their local pending/open wording does not supersede this final status.

### Historical qualification repairs


**2026-09-18 first complete engineering publication:** continuation 18 plans 390
heater FTPx outputs in 29.319 seconds, publishes in 398.611 seconds and reopens in
Rust in 7.804 seconds, with all independent model assertions intact. Python's first
attempt stopped before opening because the fixture supplied a nonexistent explicit
spill directory. The fixture now creates that directory and uses the existing exact
cache/export ownership assertions. Reusing this already successful publication,
the same separate-process pytest command passes **1/1, zero failures, baseline zero,
integration mode**, 9.48 seconds (`/tmp/pse-plan09-cold-heater-ftpx79.log`). Final
reserved bytes equal the reported 139,212,024 cache-owned bytes; no external pins,
active loads or in-flight bytes remain, and all persisted-file hashes are unchanged.
The ongoing Rust selection also exposed one stale unsigned depth expectation;
the fixture now checks the declared signed value `Cell::I64(1)` with the same
ancestor/depth assertion. Its affected rerun is `/tmp/pse-plan09-signed-depth80.log`.

**2026-09-18 complete provenance:** continuation 17 completed member writes but
whole-publication admission refused 239 duplicate derivation identities. Repeated
rule invocations had concatenated the same head with equal or additional support.
`native/evidence.rs` now uses native aggregate, flatten, distinct and sort to combine
the support set by the complete derivation head. Conflicting scalar head fields remain
separate and therefore remain subject to the unchanged primary-key constraint. Empty
support stays empty. `/tmp/pse-plan09-compiler-units78.log` records **29/29 passes,
zero failures, baseline zero**, default/force-validation, 3.293 seconds; the new
overlap/empty/conflict regression passes in 1.350 seconds. A read-only inspection of
383 persisted declared key projections found duplicates only in this relation.
Continuation 18 records the repaired compiler source and reruns full publication.

**2026-09-18 exact durable decoding:** continuation 16 and both repaired element-molar
cases advanced past alias resolution, then exposed an Arrow cast visiting binary bytes
hidden beneath null parents. Native `StructArray::flatten` now propagates struct masks;
native `take` removes only null-container child ranges before the exact semantic cast.
An internal nullable comparison view permits native inverse-cast placeholders while
the admitted/output field declaration and visible-value refusal remain exact. The new
regressions cover structs, lists, fixed-size lists and maps, each with a malformed
visible-value attack. `/tmp/pse-plan09-visible-containers70.log` records **4/4 passes,
zero failures, baseline zero**, default/force-validation, 0.022 seconds. The preceding
failed reproduction and failed engine receipt remain recorded; they are not passes.
Current-source strict Clippy passes in both modes (`/tmp/pse-plan09-clippy75.log`),
pure generation equality passes (`/tmp/pse-plan09-codegen-final9.log`), and the refreshed
extension passes **72 Python unit/component tests, zero failures, baseline zero,
32 workers, 6.50 seconds** (`/tmp/pse-plan09-py-test-final18.log`).

**2026-09-18 publication completion repairs:** the remaining Rust selection exposed
the same native nested-projection alias collision in both element-molar conservation
cases. A small in-memory regression reproduced the failure before the repair.
Nested occurrence queries now allocate source-disjoint internal field names;
quantity, source-span and reference consumers bind the actual output column.
Parent masks, NULLs and bag multiplicity remain checked. The focused native units
pass **12/12, zero failures, baseline zero, 4.118 seconds**, default profile with
force-validation (`/tmp/pse-plan09-nested59.log`). The affected native storage/DML/
publication/reuse group passes **32/32, zero failures, 30.234 seconds**, same mode
(`/tmp/pse-plan09-storage60.log`). Full engineering and the two conservation repairs
remain under qualification.

Engineering continuations 13–15 were interrupted after debugger samples identified
three preparation paths expanding shared graphs: conservative projection discovery
continued after finding an extension; durable/publication builders performed SQL
name normalization despite already resolved fields; effect isolation and deferred
namespace rewrites each walked the expanded tree. The first now stops at the native
decision, publication uses native typed Projection/Aggregate/flat Union constructors,
and effect preparation performs one accounted native DAG traversal. No timeout,
memory budget or domain assertion is increased or relaxed. Deep shared-producer
construction and no-effect units are part of the regression boundary. Continuation
13 passed doctests, **12 linked solver tests**, Python quality and **72 Python tests**
(baseline zero, 32-worker unit/component mode, 6.72 seconds), plus the complete
**84-record cache matrix** before the interrupted engineering gate.

**Measured:** the 256-callback vector fixture reports 32,111 prepared reserved bytes,
32,415 maximum observed reserved bytes, 22,507 microseconds for its callback loop,
and full release. Command: `just test-package pse-numerics --test native_expressions
-E '"test(repeated_callback_vectors_keep_contiguous_values_and_release_their_reservations)"'
--success-output immediate`; **one pass, zero failures, baseline zero**, default/
force-validation (`/tmp/pse-plan09-callback61.log`). These are actual reservation
observations, not a count of every allocation inside upstream libraries.

**Current repair boundary:** store-binding reservations now appear in the shared
cache statistics relation, with actual hit/miss/bypass counts. The Python release
checks compare pool reservations against reported cache ownership; exported Arrow
owners must retain either additional reservations or a positive cache pin. The
previous seven failures all exposed the same omitted 66,846-byte binding extent.
The accounting/reuse/drop regression passes in
`/tmp/pse-plan09-role-diagnostic36.log` (CI/force-validation, baseline zero).
That two-case run also identifies the heater's failure precisely:
`inferred.topology_edges` was requested under an obsolete role name; **one pass,
one failure**, 148.466 seconds. P5 tear selection and P6 guard/instance queries now
use the predicate inventory's typed lookup, preserving the caller's input bindings.
The next run, `/tmp/pse-plan09-typed-inventory37.log`, passed **16 focused cache/round
units** and advanced beyond the role failure. Its heater was interrupted at
465.953 seconds after profiling native support-union optimization and policy
traversal; the engine case is not a pass. Native rule/compiler union construction
now uses DataFusion's multi-input `Union::try_new_with_loose_types`, preserving the
binary builder's coercion and bag contract without a deep binary spine. Effect
discovery also uses charged actual-node identities. The independent NULL/duplicate/
mixed-integer union regression and the heater run are in
`/tmp/pse-plan09-wide-support38.log`. Strict workspace/all-target Clippy passes in both
default and no-default modes in `/tmp/pse-plan09-clippy51.log`.

Continuation 10 stopped during compilation of an intermediate inventory borrow;
it did not run terminal gates. The async inventory borrow is now exclusive, matching
its reservation ownership and retaining the required `Send` future contract.
Continuation `build/plan09-acceptance-20260917-11 --start-at py-sync` passed the
extension rebuild, quality and **72 Python tests, zero failures, baseline zero,
unit/component selection, 32 xdist workers, 6.60 seconds**. Its cache matrix also
completed. Engineering inspection and later static gates remain in progress.
The union correction followed the campaign's initial source archive and requires
its own final source verification; that archive cannot certify the later edits.
Earlier Rust gate failures remain open until the repaired target is requalified.

**Implemented and focused-Tested — shared support values:** the flat union run 38
passed seven lineage/union units, then its heater diagnostic was interrupted at
396.721 seconds while preparing 69-, 68- and 95-branch native witness queries.
The process reached approximately 13 GiB RSS; no engine pass is claimed. The
separate engineering-inspection process in continuation 11 was also interrupted
while this shared cause was repaired. Witness recovery now retains one lazy native
cache producer per unchanged value-side node within the borrowed query. Aggregate,
window and join witnesses still carry all original source branches; round ownership
and reset remain enforced by the existing native cache/session implementation.
`just test-package pse-rules --profile ci -E 'test(plan::trace::tests::) | binary(stratified_fixed_point) | test(every_native_rule_binds)'`
passes **14 tests, zero failures, baseline zero, 63 skipped**, force-validation,
4.955 seconds of execution (`/tmp/pse-plan09-shared-support39.log`). This includes
every declared rule binding plus independent cyclic support, aggregate, window,
outer/anti-join, NULL and duplicate witnesses. `just clippy` passes both modes in
`/tmp/pse-plan09-clippy53.log`. The current engine qualification is
`/tmp/pse-plan09-shared-engine40.log` (heater FcTP and cyclic parallel topology).

That run exposed a stack overflow in the recursive witness-plan walk (both engine
cases failed; debugger receipt `/tmp/pse-plan09-cyclic-stack40.log`). Recovery and
absence-scope traversal now use bounded post-order work lists keyed by the actual
borrowed node and source alias. A 256-projection regression preserves the three
independently expected source witnesses: **one pass, zero failures, baseline zero,
0.028 seconds**, CI/force-validation (`/tmp/pse-plan09-deep-support42.log`). The
fourteen rule/stratum checks also pass after this change. Their paired engine cases
then reached a schema-refinement refusal instead of overflowing. Native optimization
can refine a producer's nullability; repeated references must compare the original
schemas, not an optimized schema against a later original reference. The corrected
cyclic-topology journey passes in `/tmp/pse-plan09-producer-schema43.log`:
**one engine pass, zero engine failures, baseline zero, 140.168 seconds**, existing
CI/force-validation profile. That run's separate new unit had an incorrect assertion
about its parent union's nullability; the parent legitimately retains its declaration.
The corrected unit and cache/round group pass **14/14, baseline zero, 0.098 seconds**
in `/tmp/pse-plan09-cache-identity45.log`. An adversarial same-schema rewrite from
literal 3 to 7 now gets its own cache identity and returns both actual values.
That heater rerun, `/tmp/pse-plan09-heater-completion46.log`, failed after
501.117 seconds at the 32 GiB pool limit. Many retained logical observations were
128 MiB each. Profiling located native pretty-JSON serialization, not model-row
execution. The diagnostic cut now calls DataFusion's public `PgJsonVisitor` once
per actual node, retaining its operator details and output names with explicit
ordered graph edges. This removes deep indentation and repeated shared branches
without retaining live plans/providers or truncating evidence. The superseded
cache-only diagnostic clone and duplicate `RuleOutcome` explain string are deleted.
The first four diagnostic units passed three and caught one omitted native output
field setting. The corrected graph/subquery/cancellation checks and the heater pass
in `/tmp/pse-plan09-graph-heater48.log`: **six passes, zero failures, baseline zero,
531.582 seconds**, CI/force-validation. The heater produces 390 outputs, with
38.733469 seconds of composition and 484.017591 seconds of execution. This is a
passing current-function receipt, not a whole-campaign result.

**Implemented and focused-Tested — immutable declarations and effect discovery:**
samples from that heater identified repeated codec field construction and expression
cloning during freshness classification. The four immutable registry-derived codec
fields now share exact Arrow owners; arguments are still validated on every call.
Freshness discovery uses actual extension owners and borrowed native expressions,
shares immutable node facts across sibling outputs, and keeps each root's effect
contracts separate. Scratch is charged to the actual scope/native pool and observes
cancellation. Admission, closure and diagnostic-function inspection also borrow
native expressions instead of cloning them merely to visit them. The initial eight
freshness/codec/diagnostic units pass in `/tmp/pse-plan09-freshness50.log`:
**eight passes, zero failures, baseline zero, 1.122 seconds**, CI/force-validation.
Subsequent cleanup and the final cache-factory scope binding require the final
current-source gates. The Python continuation immediately before these last two
repairs passes **72/72, baseline zero, 6.47 seconds**, 32-worker unit/component mode
(`/tmp/pse-plan09-py-test-final6.log`). Full architecture acceptance remains open.

The final runner now explicitly uses Nextest's existing `ci` profile, matching
`.github/workflows/rust.yml`, with force-validation and no retries. Checked-in
timeouts are unchanged; actual timings remain in the receipts. Earlier default-profile
campaigns remain historical failures, not retroactive passes under another profile.

**Current continuation:** the second broad Rust campaign in
`build/plan09-acceptance-20260917-04` was stopped after its remaining cases repeatedly
hit planning defects already being repaired: **976/982 run, 937 passed, 22 failed
(including the interrupted case), 17 timed out, six not run**, 2715.882 seconds,
default/force-validation, baseline zero, no retries. It exposed source-scope, codec,
constraint and execution-time failures and is not a passing receipt. Subsequent
independent gates continue from doctests in `build/plan09-acceptance-20260917-05`;
that receipt cannot certify the Rust gate or earlier commands.

**Implemented and focused-Tested:** recursive dependency discovery now recognizes
only work tables bound by the actual enclosing native recursive query, using the
same name/schema admission as executable plans. Extracted work tables still refuse.
Diagnostic codecs retain exact selected providers through native `TableScanBuilder`
and opaque descriptors during protobuf construction, then restore the actual owned
provider before admission. All three codec tests pass in
`/tmp/pse-plan09-codec-ddl19.log` (CI/force-validation, zero failures in that binary).
Arbitrary native UDF objects remain executable; diagnostic name-based serialization
still refuses an unrelated implementation with a matching registered name.

**Implemented and Tested:** the native deferred CREATE adapter retains declared
constraints when its child layout is unchanged. DataFusion 55's generic
`with_new_exprs` clears them; losing the PK had allowed a duplicate private insert.
The strengthened default/constraint/generation test and bounded admission test pass
in `/tmp/pse-plan09-ddl-budget20.log`: 2 passes, zero failures, 235 filtered,
CI/force-validation. Admission now explicitly refuses zero control-memory headroom
and succeeds with 8 KiB without re-decoding a large immutable value.

**Implemented, engine qualification pending:** the bounded field-admission cache
also keys by complete native Arrow field equality, retaining nested fields and all
metadata. This avoids revalidating equal freshly allocated expression fields.
The 12 admission/command units pass in `/tmp/pse-plan09-scope-units20.log`
(default/force-validation, zero failures, 124 filtered, 0.856 seconds).
The older heater diagnostics were interrupted after stack sampling; neither is a
passing receipt. The complete-equality cache uses cheap bucket hashes followed by
Arrow's full equality, avoiding repeated recursive metadata hashing. Private argument
workspaces now derive all sibling native fields together and reuse one discovered
source inventory for selected dependencies. Their focused admission/cache selection
passes in `/tmp/pse-plan09-admission-batch22.log`: **21 passed, zero failed, baseline
zero, 115 filtered**, default/force-validation, 0.906 seconds. In the subsequent
`/tmp/pse-plan09-heater-batched22.log`, the compiler unit passed and the heater
diagnostic was interrupted after profiling; neither interrupted diagnostic certifies
the engine journey.

**Implemented; current representative pending:** declared invariant queries and
diagnostic branches now share batch admission within the exact immutable session.
The seven focused rules/compiler/budget checks pass in
`/tmp/pse-plan09-invariant-batch23.log` (CI/force-validation, baseline zero); the
heater diagnostic was interrupted after profiling and is not a pass. That profile
also exposed loss of native extension sharing: cloned `LogicalPlan` enum wrappers
retained one extension Arc, but restoration keyed their distinct wrapper addresses.
Admission now keys those nodes by the actual immutable extension owner, retaining
scope distinctions and refusing foreign sources. A regression checks that repeated
admission preserves one producer across 32 cloned roots. Tuple projections use
native `Projection::try_new` for already-resolved fields, avoiding repeated SQL
USING-column discovery over the producer graph. Current checks run in
`/tmp/pse-plan09-shared-owner24.log`.

**Implemented and focused-Tested:** field restoration now preserves unchanged
native nodes and carries DataFusion's `Transformed` state through shared children.
Recursive metadata transport keeps an existing correctly declared projection instead
of adding another wrapper on every admission. The 18 selected catalog/rules/compiler
checks pass in `/tmp/pse-plan09-idempotent25.log` (CI/force-validation, baseline zero);
the full heater representative is still pending. The preceding 24 diagnostic was
interrupted after sampling, not passed.

**Implemented; Python qualification pending:** Python's one-thread blocking pool
deadlocked the pinned Delta kernel's nested `block_in_place`/`spawn_blocking` path.
Its native executor explicitly documents that failure mode. Python now retains
Tokio's native blocking capacity; application CPU workers and cache/load admission
limits remain explicit. The Python gate in continuation 05 was interrupted with
57 passes and one stale expectation that zero spill capacity must be rejected.
The fixture now verifies zero spill capacity is preserved, matching the native
DiskManager contract. Continuation `build/plan09-acceptance-20260917-06` passed
`py-sync` and quality, then stopped compiling the fixture runner while field-admission
repairs were incomplete. Continuation 07 also stopped at compilation during those
edits. Neither continuation supplies a passing Python journey.

**Implemented and focused-Tested:** private cache nodes retain a disposable structural
admission record for their exact immutable input, registry and actual provider owners.
Foreign providers or registries cannot use it; reconstruction clears it. Native SQL
binding uses temporary schema-only boundaries around those admitted producers and
restores their exact original owners before returning a query. This avoids DataFusion's
otherwise unconditional extension reconstruction on a no-op SQL rewrite. No execution,
row-validation or authorization result is cached by this mechanism. Reservations cover
the retained record and transient discovery. The owner/refusal/rewrite/release unit
passes, including an adversarial caller cache factory and a no-op native rewrite.
The complete focused selection in `/tmp/pse-plan09-query-boundary28.log` passed
**29 tests, zero assertion failures**, CI/force-validation, baseline zero; its heater
representative was interrupted after profiling and is not a passing receipt.

**Implemented; representative qualification pending:** source discovery now visits
each actual shared node once per recursive scope with charged temporary storage.
The regression constructs a 20-level shared graph and checks 21 unique nodes per
scope, separate scope admission and reservation release. It and recursive worktable
admission pass in `/tmp/pse-plan09-source-discovery29.log`: two passes, baseline zero,
CI/force-validation. The heater diagnostic was interrupted at 137.798 seconds after
profiling identified repeated SQL USING-column discovery in diagnostic sorting.
Already resolved diagnostic projections, filters, ordering and metadata transport now
use native plan constructors; unresolved SQL continues through native binding.
Qualification of this source runs in `/tmp/pse-plan09-native-constructors30.log`.

**Tested / Measured, partial journey:** the native-constructor selection passed
30 focused checks (CI/force-validation, baseline zero); its heater diagnostic was
interrupted after profiling at 80.666 seconds. Reusing unchanged producers' intrinsic
closed-scope proof then allowed the next heater to finish composition in **38.535982
seconds** in `/tmp/pse-plan09-closed-producers31.log`. Eighteen focused checks passed
in that run. Execution remained open: profiling found repeated whole-graph command
target discovery, and the representative was interrupted at 139.840 seconds.
Command target discovery now visits each actual immutable node once with charged
temporary storage; its current qualification is `/tmp/pse-plan09-command-targets32.log`.
Composition timing is not a completed-engine timing or a terminal pass.

**Implemented and focused-Tested:** physical substitution now preserves each actual
shared cache producer once. DataFusion also receives each closed producer as an
independently planned native computation; private physical boundaries prevent its
whole-tree scalar-subquery discovery and optimizer from multiplying shared inputs.
No producer executes during planning. Public logical plans still contain all sources,
requirements and operations. Native reset eligibility explicitly checks the retained
producer; reset uses DataFusion's `reset_plan_states` on that separate graph. The
completed-owner/epoch regression and cached/uncached empty-to-nonempty hash/cross-join
regression pass in `/tmp/pse-plan09-producer-physical34.log` (13 focused passes,
CI/force-validation, baseline zero). The heater now reaches domain execution and
fails at 176.626 seconds with a duplicate `authored.scopes` predicate-input role.
Private predicate inventories now use their own role namespace; original argument
bindings and policies remain intact. That repair is qualifying in
`/tmp/pse-plan09-predicate-scope35.log`.

**Implemented; Python qualification continuing:** Python now obtains its factory from
`SharedRuntime::session_factory`, retaining the deployment's cache service and native
planner. The direct factory construction had omitted that service. Continuation 08
ran 72 Python tests in 6.75 seconds: 70 passed and two failed (resident hits and the
one-byte fresh-process budget). Both cases pass after this connection in continuation
09, which reports 65 passes and seven failures in 6.79 seconds. All seven failures
were assertions that reader close must release the shared cache's retained memory.
The tests now require zero reader pins, active loads and inflight allocations, and
require the remaining reservation to equal reported cache capacity/live ownership.
Export-lifetime assertions separately require a reader pin or non-cache reservation.
The common process fixture declares one cache budget for all publication readers.
These assertion changes are qualifying in `/tmp/pse-plan09-python-ownership10.log`.

**Tested:** `just clippy` passes both default and no-default workspace/all-target modes
in `/tmp/pse-plan09-clippy47.log`, baseline zero, before the command-discovery edit.
`git diff --check`, `just lint-agents` and the 142-file Delta source verification also
pass at this repair boundary. Continuation 08 starts at `py-sync`; its results remain
separate from the open Rust representative and preceding campaign receipts.

**Interface-checked:** Cargo fingerprint tracing in
`/tmp/pse-plan09-shared-owner24.log` identified `CARGO_MANIFEST_DIR` leaking from
`cargo run` through the acceptance runner into nested recipes. That toggled ring's
build-script fingerprint and invalidated its DataFusion dependants. The runner now
removes that injected package variable. The next focused build took 23.24 seconds,
compiling project crates only; this is build evidence, not engine-runtime evidence.
Further tracing in `/tmp/pse-plan09-producer-physical34.log` found the same issue
for `CARGO_PKG_NAME` (`xtask` versus absent). The runner now removes inherited
`CARGO_PKG_*` and package-manifest variables from child recipes while retaining
the caller's target directory, toolchain and ordinary Cargo configuration.

**Implemented:** registry-bound admission now visits actual shared native plan nodes once
per operation and recursive scope, retaining their owners and charging temporary memo
storage. It reuses immutable Arrow field owners, preserves cancellation, validates
foreign scans and recursive bindings, and no longer constructs a discarded schema or
reconstructs an unchanged schema twice. Native recursive terms explicitly project the
anchor's schema metadata; Rust anchor nullability and SQL widening are both admitted.

**Implemented:** `ArtifactPlan::execute_outputs` shares only the actual native cache
nodes within one artifact invocation. Completed physical owners keep completion cells
alive until all siblings finish; the next invocation has a fresh cache store. A counting
algorithm confirms one execution for two siblings and another execution on the next call.

**Implemented:** native extension `Debug` uses each node's existing native EXPLAIN
formatter. DataFusion renders the children separately. Derived recursive debug payloads
had produced gigabytes of duplicate JSON, exhausting the query budget. Diagnostic string
buffers now grow amortized and reserve their actual capacity, with exact-size admission
when spare capacity is unavailable. Native JSON now renders each actual cache producer
once and marks later occurrences with plan-local references. Every producer definition
and every reference remains visible; original and executable plans are unchanged.
This uses DataFusion's renderer over a diagnostic clone, avoiding an exponentially
expanded tree of the same shared producers.

**Tested, baseline zero:** the force-validation/default-profile admission/observation
selection in `/tmp/pse-plan09-admission-observation-units18.log` passes 13 tests, zero
failures (117 filtered). Two registry-wide native durable CHECK and physical storage
projection tests pass in `/tmp/pse-plan09-all-durable-projections.log` (3.157 seconds,
129 filtered). These are focused receipts, not complete engine qualification.

**Implemented:** logical-type Arrow storage is compiled once when the immutable registry
is assembled. Field admission compares those native declarations; JSON remains the
catalog interchange representation. Completed cache producers retain weak physical
owners for reuse within the current invocation and epoch. Native cache producers are
being optimized once through private, temporary boundaries, then expanded again before
admission, physical planning and public observation. Sibling outputs share a bounded
preparation scope; this is still undergoing engine qualification.

**Tested, baseline zero:** the focused cache/admission/failure selection via
`just unit-package pse-catalog 'test(cache::) | test(admission::) | test(failure::)'`
passes 49 tests, zero failures, 85 filtered, default profile with force validation
(6.568 seconds). A native projection regression reproduced the Float64 source-span
failure. Adding a native subquery alias scopes nested occurrences correctly; that
regression and three nested-value checks pass (4 tests, 1.637 seconds).

**Tested, baseline zero:** `just unit-package pse-catalog 'test(session::observation) | test(cache::logical)'`
passes 5 tests, zero failures, 131 filtered, default profile with force validation
(0.024 seconds). This proves once-per-producer optimization, correct complete versus
extracted correlated-query scope, bounded diagnostic growth, complete native producer
definitions and resolvable references. A 12-level shared graph renders under 128 KiB
and releases its reservation on drop. The next representative engine run is pending.

**Implemented and Tested:** enum-type references now target unique registry-derived
`reference.schema_enum_types` declarations. Enum members remain in
`reference.schema_enums`, with an explicit parent reference. `just codegen` and
`just conformance-fixtures` regenerated their consumers. The fresh
`just inspection-fixture build/plan09-source-probe-20260917-e` succeeds, including
complete member publication and reopening. The former ambiguous enum-member target
and source-span projection failures no longer block this fixture. Python tests have
not yet been rerun against the rebuilt extension.

**Implemented:** immutable artifact outputs are admitted together at construction.
Sibling preparation validates each newly optimized producer once, materializes its
field guards before expanding shared children, and retains complete public plans.
Pure plans skip mutation-only rewrite passes; EXPLAIN still applies its effect guards.
Completed producers become native `StreamingTableExec` leaf readers over their actual
successful, current-epoch cache values. Physical optimization no longer rewalks their
already executed producer trees. Readers retain memory/spill ownership and refuse a
stale execution epoch.

**Tested, baseline zero:** `/tmp/pse-plan09-cache-admission-boundaries15-units.log`
has 23 passes, zero failures, 257 filtered (CI/force-validation), covering admission,
sibling preparation, caches and mutation-free EXPLAIN. The subsequent
`just unit-package pse-catalog 'package(pse-catalog) & test(session::cache)'`
has 10 passes, zero failures, 125 filtered (default/force-validation, 0.049 seconds),
including childless native replay, stale-epoch refusal and retained Arrow ownership.

**Tested and Measured, representative only:**
`PSE_TEST_PHASE_TIMINGS=1 just test --profile ci --nocapture -E "'package(pse-tests-engine) & test(lexical_binding_leaves)'"`
passes in 101.674 seconds, zero failures, 48 filtered, CI/force-validation
(`/tmp/pse-plan09-engine-cache-leaf16.log`). Composition is 33.955 seconds and the
206-output execution is 64.773 seconds. The preceding successful run took 349.989
seconds before the native leaf-reader change. These overlapping-host observations
are not a latency guarantee. No timeout or scientific assertion changed.

**Tested, baseline zero:** `just native-solver-test` passes all 12 linked tests,
zero failures/skips, force-validation in the pinned solver container, 18.521 seconds
(`/tmp/pse-plan09-native-solver2.log`). Complete Rust/Python/engineering qualification
continues in `build/plan09-acceptance-20260917-04`; the representative is not a full
engine-suite pass.

**Measured:** the corrected `just bench-cache` full matrix completes with 84 JSON
records in `/tmp/pse-plan09-cache-matrix4.log`. Checkpoint classification now uses
actual filenames; 1,000-commit IO remains 12 requests / 40,245 bytes versus 2,001 /
11,367,542 without checkpoints. [Measurements and defaults](09-cache-measurements.md)
records dimensions, selected bounded defaults, native limitations and exact evidence.


**Tested, baseline zero:** campaign `build/plan09-acceptance-20260917-03`,
`just architecture-acceptance ... --start-at test`, default-profile force-validation,
no retries: 963 tests, 826 passed, 136 failed, one 120-second timeout. The campaign
stopped at this gate. Targeted selections are used for repairs; counts from overlapping
selections are not summed into a purported complete-suite pass.

**Tested:** `/tmp/pse-plan09-admission-cache-units14.log` records the exact CI-profile,
force-validation nextest filter: 17 tests passed, zero failed, 215 filtered. This covers
the native cache/reuse lifecycle with cache off/on, field-admission cases and both
conformance fixture tests. `/tmp/pse-plan09-contract-repairs12.log` establishes native
maintenance lifecycle and deferred DML repairs (two passed, one then-open cache rebind
failure subsequently fixed). `/tmp/pse-plan09-contract-repairs11.log` has 26 passed,
three subsequently repaired failures and one engine timeout at 360.131 seconds.

**Implemented; qualification in progress:** recursive work-table field admission;
native round reset after complete exhaustion; actual expanded-view source lineage
and rebinding; CDF metadata decoding and truthful native pushdown; full-range timestamp
tick layout; deferred native write hooks behind requirements, including nonmutating
EXPLAIN; current typed domain fixtures and registry-driven conformance fixtures.
Native kernel CRC seed/advance/corruption fallback has a passing isolated test. An
ordinary delta-rs snapshot without an eligible CRC seed cannot bootstrap checksum
state at this pin; this remains an explicit native fallback, not a repeated data write.

The engine timeout occurs during logical composition, before model execution. Live
stack samples identified repeated immutable Arrow-field validation. Admission now
retains actual Field Arcs in a budgeted temporary cache, reuses native child declarations
and shares that scope across sibling artifact outputs. Source ownership, expression
transfer, invalid-field rejection and registry identity remain checked. The isolated
cache rejection/release/bypass unit passes; final engine timing is still open. Diagnostic
runs interrupted after stack sampling are not passing qualification receipts. No timeout
configuration was increased. Remove temporary production debugging before closure.

**Measured, partial:** the first `just bench-cache` release run completed the 3/8/16-node
strata cases and generated a real 1,000-commit Delta fixture; it stopped at a fixture
missing durable field descriptors. That fixture now uses the generated durable schema.
The full matrix is running again; defaults and terminal architecture acceptance remain
unresolved until its complete results are assessed.

## Implementation ownership and deletions

Paths in this table are relative to `crates/` unless prefixed with another directory.

| Package / prior obligations | Producer and consumers | Replacement/deletion and qualification |
|---|---|---|
| C00 / E00 | ADR-0070; AGENTS.md; plan index; this ledger; caching capability map | Active execution is Plan 09. No blueprint or accepted ADR is edited; proposed decisions remain proposed. Historical scoped receipts stay in Plan 08. |
| C01 / E05/E06/E09 | `pse-catalog/src/session/config.rs`, `config/resource.rs`, session factory/profile, artifact dependencies/attempts | One conservative setting classification. Resource ceilings and spill paths do not become result identity. Unknown options and actual implementation generations remain semantic. Invocation identity is separate. |
| C02 / E01/E02 | `pse-schema/src/{catalog/cache.rs,model/algorithm.rs,model/relation.rs,catalog/publication.rs,codegen/rust/arguments.rs}`; generated relations; runtime budgets; Python settings | Complete tagged dependency evidence replaces nullable evidence combinations; registry-owned argument consumption drives generic projection, reflection and identity. No hard-coded algorithm roster remains in the generator. Existing algorithms retain Whole unless access is enforced; a declared extension proves restricted access. |
| C03 / E03/E05/E09 | `pse-catalog/src/cache_service/`; runtime env/report; `session/cache/{value.rs,tests.rs}`; `pse-ids/src/owned_buffer.rs`; Delta attempt receipts | One shared service, native DefaultCache envelopes, actual store/root namespaces, bounded live single-flight coordination and one shared fill semaphore. Snapshot/resident reservations survive eviction. Arrow extents deduplicate allocation ownership; exported buffers retain owners. Read and write JSON receipt staging reserve before decode/serialization. |
| C04 | `tooling/delta-native-seams.patch`, `scripts/vendor-delta.py`, Cargo source metadata, generated vendor provenance, Delta skill overlay, governance source hashes | Six narrow upstream source seams: CDF cache injection, CRC replay configuration and snapshot accounting/reference/checksum access. No private replay implementation; no edits to cargo's immutable checkout. `just delta-source` verifies all 142 selected files. |
| C05 / E02/E04/E07 | `pse-catalog/src/session/round.rs`, preparation/cache execution; `pse-rules/src/strata/{rounds.rs,native_state.rs,mod.rs}` | Stable unknown-statistics round providers and one prepared native stratum. Native reset eligibility checks actual operator/physical-expression types. Per-round SQL/analyzer/optimizer/physical planning and old `merge_round` are removed. Failures poison the retained computation. |
| C06 / E06/E09 | `pse-catalog/src/cache_service/snapshot.rs`, `delta/{provider,publication,publish,attempt,write,changes,dependencies,maintenance}.rs` | Every product native open uses the shared opener. Only it and its cache-disabled native fallback load a table. Exact native latest checks, capability-specific keys, forward native seed, returned committed states, declared checkpoints and pre-destruction fences replace repeated fresh opens. Ambiguous writes reconcile native receipts and never replay input for cache/checksum failure. |
| C07 / E03/E05/E08 | `cache_service/resident.rs`; exact selected providers; inspection/TableReader; Python publication streams | Lazy selected-result provider integrates ordinary reads. Full exact selection below residual projection/filter preserves multiplicity/NULLs and truthful partitioning. Idle Arrow/cache entries do not hold file leases. Every new consumer takes a fresh lease and validates uncached native generation. |
| C08 / E06/E07/E09 | `artifact/{consumption,dependencies,reuse}.rs`; compiler `passes/argument.rs`, native execution and generated projections; `delta/changes.rs` | Native projection analysis before simplifying optimization; keys, filters, absence and membership remain dependencies. Complete bounded CDF or exact endpoint multiset comparison establishes equivalence. Rebinding retains actual implementation/operation ownership. Reuse runs in the requesting session under current budget/policy; unknown cold generations refuse. |
| C09 / E05/E08 | `cache_service/{inspection,details,metrics}.rs`; `delta/leased.rs`; runtime/Python report types | Native inspection table functions, optional unknown counters, guarded inventory reservation and per-reader predicate ceilings. Namespaced CDF and ordinary reads share metadata caches. Resource telemetry is excluded from semantic inputs. |
| C10 / E01–E09 | All current schema/authoring/compiler/numerical/provider/Python consumers and generators | Complete variants, signed bounded ordinals, coherent expression/vector/Jacobian rows, typed keys/refs and common native admission remain authoritative. Removed the last production authoring/compiler/rule relational Cell round trip. Parser rows, generated external codecs, numerical FFI and useful transient graph algorithms are retained boundaries. |
| C11 / E08/E10 | `xtask/src/architecture_acceptance.rs`, bench/test manifests, generated outputs/stubs, this ledger | Campaign records exact source hashes, tracked diff, untracked source archive, Cargo/dependency provenance and each command. Fixtures and measurements use current target code with cache-off/on controls. C12 starts only after implementation, deletions and static closure. |

### Source audit dispositions

- **Interface-checked:** source search for StageKey, StageId, StageMemo, ProducerRecord,
  ChangeEnvelope, RuleExpr, RulePlan, SnapshotStore and ManifestStore finds no production
  predecessor authority. Removed controller/store/memo modules stay absent. Generated
  schemas come exclusively from registry generators.
- **Interface-checked:** remaining `DeltaTableBuilder`/`load_version` production sites
  are the common `delta/provider.rs` construction/fallback and `cache_service/snapshot.rs`
  actual loader. Maintenance protected-version checks use this same opener.
- **Interface-checked:** no production `CellCodec`, `encode_columns` or `decode_columns`
  remains in authoring/compiler/rules. Source parsing is an explicit boundary, and
  generated typed Arrow codecs remain useful. Native metadata field restoration,
  declared-output restoration and the preserve-field UDF have distinct semantic roles.
- Signed storage preserves explicit finite domain bounds: a former UInt16 ordinal can
  still be a declared Int64 value bounded by 65535. That bound is not a legacy storage
  path. Numerical dimensions use their registered bounds and exact sparse-coordinate
  coverage; native unsigned Arrow sources remain eligible at the external boundary.
- The DataFusion/Delta skill project scans are syntax-based leads. Fixture sessions
  without deployment pools, deliberate bounded collection, unknown changing statistics,
  residual filters above materialized values and exact-schema writes are intentional.
  Delta application transaction records are witnesses, **not automatic deduplication**
  at this pin. Ordinary mutation retry authority is the explicit receipt reconciliation.
  Maintenance retention is the declared native policy plus protected versions; it does
  not silently lower the native safety threshold.
- Long native relational assemblies carry local `expect` explanations where splitting
  an expression only to satisfy a line count would hide its dependency order. These are
  per-function, compiler-checked expectations, not a workspace lint baseline. Ownership,
  borrowing, large futures, enum size and import findings are corrected in code.

## Target oracles and acceptance crosswalk

These fixtures are implemented; execution status is separate in the receipts below.

| Obligations | Named target oracle / command |
|---|---|
| Q01/Q02/Q08 | `pse-schema` field_contract/native_field_declarations/reference_contracts; catalog captured_native_layouts/tagged_delta_values/native_value_contracts/unified_delta_contracts; conformance generated invariant fixtures |
| Q03/Q04 | catalog row_keys/composite_reference_checks; rules plan support units and relational_execution/stratified_fixed_point; engine source_projection_admission and unified_sources |
| Q05/Q06 | mathir relation_roundtrip; numerics scalar_math/native_expressions; backend-native planning/native_solve; catalog numerical projection; `just native-solver-test` |
| Q07/Q09 | catalog provider_contracts/unified_delta_dml/unified_delta_contracts; session mutation/provider/admission units; native_default and extension fixtures within these targets |
| Q10/Q13 | catalog native_artifact_lifecycle and native_cache_lifecycle; lifecycle publication_each_object with checkpoint interval 1 and CRC acceleration enabled; native member/root failure and retained-history fixtures |
| Q11 | engine native_normalization/native_template_graph/native_engineering_workflows/incremental_equals_clean_p0_p3; native compiler extension and requirement composition; authoring native parser/edit/rename tests |
| Q12 | `just engineering-inspection` emits four current engineering publications then reads them in a separate Python process; Python test_native_registry/test_native_caches/test_engineering_inspection and descriptor/stream ownership fixtures |
| Q14/K11 | compiler registered extension projection unit and native extension execution; deleted path audit; selected Delta source/governance hash verification; architecture campaign; independent G1–G7 review |
| K01 | catalog session::config::tests, actual current-session artifact reuse in native_cache_lifecycle; implementation-generation refusal fixtures |
| K02/K03/K09 | cache_service units (excluding delta_journeys), store_tests, cache buffer ownership/spill units, native inspection functions; native Parquet/CDF instrumentation in bench-cache |
| K04 | session::round::tests::prepared_joins_observe_empty_then_nonempty_epochs_without_replanning; fixed-point and support oracles; bench prepared_rounds exact preparation/reset counters |
| K05/K10 | cache_service::delta_journeys::exact_versions_load_classes_and_seeded_refresh_match_fresh_native_files (**integration, not unit**); publication fault fixture; delta_open, delta_incremental_open, crc_load_class measurements |
| K06/K07 | native_cache_lifecycle::selected_cache_and_projected_reuse_match_fresh_native_results; Python exported-array fixture; artifact::consumption::tests::projected_changes_preserve_nulls_and_bag_multiplicity; cache-off/on benchmark with fresh output oracle |
| K08 | native_cache_lifecycle executes actual child-process Optimize with idle parent caches and reopens exact old selections; native_artifact_lifecycle covers active leases/protected versions/reclamation; flight and invalidation units cover late fills/cancellation |

## Measurements and conditions

`just bench-cache` runs only in C12. Ordinary `cargo test --benches` uses smoke sizes.
The matrix emits JSON lines; assertion failures fail the command, not a timing threshold.

- Prepared native rounds: 16/1024/16384 rows, alternating empty/nonempty epochs,
  preparation time, per-round time and exact physical-plan/reset counts. Actual cyclic
  strata additionally exercise 3/8/16-node graphs, two rules and one head, with an
  independent all-pairs closure oracle. Native tracing separates stratum preparation,
  candidates/support execution, head merging and support/epoch advancement.
- Delta opening: 1/8/32 distinct member roots, selected versions after 1/100/1000
  commits, fresh cache, warm cache and separate fresh process. Fixture setup and
  copying immutable generated native logs are timed separately. OS cache is uncontrolled.
- Delta refresh/load classes: native incremental update, metadata versus full query
  replay, missing/present CRC. CRC summaries never stand in for the full Add set.
- Parquet scans: 16 nullable columns, 4096 rows, selective versus complete scan,
  predicate capacities 0/1 MiB, reader concurrency 1/4, actual reader/operator metrics
  and log/checkpoint/data IO through a validator-preserving ObjectStore decorator.
- CDF: native insert/delete/update-preimage/update-postimage counts, fresh/warm cache
  on/off, shared metadata injection. Gapped history refusal is the product CDF fixture.
- Residency/dependencies/pressure: 16/1024/16384 rows and value-only versus key edits;
  fresh recomputation oracle, repeated reads, cache capacities below/near/above the
  retained selection and observable spill/bypass/pin behavior. Rust and Python
  ownership fixtures retain buffers through close/eviction.
- Maintenance/failure qualification: actual native Optimize/checkpoint/vacuum and
  protected data/log versions, independent process fences, empty/interrupted attempts
  and post-commit checkpoint/CRC failures. Current engineering outcomes use the
  existing four heater/mixer × FTPx/FcTP fixtures; no new simulator outcome is claimed.

Native deployment-wide replay-action and decoded-row counters are unavailable and
remain NULL. Actual instrumented IO and physical-reader metrics are reported instead.
Staging budgets bound admission/fanout, not every allocation inside upstream replay.
Process RSS remains a separate measured envelope.

## Historical C11 verification receipts

**Tested:** `just unit-package pse-catalog 'test(cache_service::) | test(session::round::tests::) | test(session::cache::tests::) | test(session::config::tests::) | test(delta::attempt::tests::)'`
ran before `cache_service::delta_journeys` was added: **27 passed, 0 failed, baseline 0,
91 filtered**, dev/test with `pse-relations/force-validate`. It includes receipt encode
budget refusal and round hash/cross joins/cancellation. Future isolated commands must
exclude `delta_journeys`; it performs storage integration.

**Tested:** `just unit-package pse-compiler 'test(passes::argument::tests::)'`:
**1 passed, 0 failed, baseline 0, 28 filtered**, force-validation; restricted extension
arguments cannot recover omitted fields or the full relation owner.

**Tested:** `just unit-package pse-catalog 'test(artifact::consumption::tests::)'`:
**1 passed, 0 failed, baseline 0, 119 filtered**, force-validation. The native projection
regression covers a nested field also named `value`, NULLs and bag multiplicity. Its
first run exposed the alias ambiguity; the successful receipt follows the actual fix.

**Interface-checked:** `just delta-source <pinned-upstream-checkout>` verifies 142 files;
`just codegen-contracts` generates source/Python/docs without physical package execution.
Pure Rust/Python/docs generator equality and compiled API stub checks passed. Four new
generated Rust files needed Git intent-to-add registration; the generator outputs
already matched. The first required feature compile rejected the host's Ipopt 3.11;
the recipe now uses the same immutable solver interface as `native-solver-test`.
`just clippy` passes the whole workspace/all targets with `-D warnings` in both default
and no-default modes after the final benchmark fixtures. `just family-check`,
`just fmt-check`, `just lint-repo` and `just python-stubs --check` pass. Required feature
compilation passes: 53 depth-two feature combinations and 31 no-default package checks.
Python quality passed, including its 14 setup unit tests
(baseline zero). Cargo separately reports the pinned upstream `proc-macro-error2`
2.0.1 future-compatibility notice; no current project Clippy warning is accepted.
These are the historical C11 receipts. Final C12 acceptance and the completed cost
matrix are recorded above. Do not infer
current correctness from an earlier source boundary.

### Historical C12 target corrections

The first campaign passed format/family checks and stopped at physical fixture
generation. The stoichiometry phase-closure query now uses native joins and explicit
presence/UNKNOWN handling instead of nested correlated subqueries that lost an alias.
Its independent policy regression passed **1/1, 0 failed, baseline 0, 12 filtered**
under `just unit-package pse-rules 'test(stoichiometry_phase_policy_preserves_defaults_restrictions_and_missing_members)'`,
force-validation, 0.747 seconds of test execution.

Regeneration then exposed lost ordering through `ExecutionContractExec`: its physical
properties forwarded ordering but it omitted `maintains_input_order`. The contract now
preserves the value child's order and identifies its requirement child as a barrier.
`just test-package pse-catalog --test native_output` passed **4/4, 0 failed, baseline 0**,
force-validation, 3.097 seconds, including an explicit sorted-result assertion.
`just codegen` subsequently regenerated all three targets and pinned Ipopt bindings.
The next campaign was `build/plan09-acceptance-20260917-02`; no preceding full
Rust, engine, solver or Python suite had run at that checkpoint.

The second campaign passed format, family, complete generated equality and workspace
compilation; it stopped at a test-only primitive-sort lint. That correction and the
new continuation CLI pass strict Clippy in both modes. The remaining gates now run
with `just architecture-acceptance build/plan09-acceptance-20260917-03 --start-at test`.
Its receipt identifies the selected gate range and cannot certify skipped earlier
commands. The first broad Rust suite is `just test --no-fail-fast --success-output final`,
force-validation, with unchanged timeouts and zero automatic retries. Its completed
results and subsequent repairs are recorded in the current checkpoint above.
