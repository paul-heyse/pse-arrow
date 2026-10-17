---
title: Build, test and development
status: abandoned
superseded-by: https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md
date: 2026-10-11
adrs: [ADR-0174, ADR-0175]
review_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
scenario_sources:
  - docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md
doc_owner: docs/plans/36e-build-test-and-development.md
doc_topics: [architecture, preparation, storage, validation, environment]
---

# Plan 36e: Build, test and development

> **Superseded 2026-10-13 by [Plan 38](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md).** Remaining execution and acceptance obligations now belong to the [new series transfer](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/38-efficiency-and-solver-target.md#plan-36-transfer-and-supersession). The inactive `abandoned` lifecycle value records replacement of this execution plan, not completion or loss of its delivered work. All packet statuses, running notes, decisions and checkpoints below are historical; failed/interrupted evidence is unchanged. Do not resume this plan.


Part of [Plan 36](36-efficiency-alignment.md). The coordinator owns the combined target, accepted rule decisions, finding dispositions and assembled acceptance. This document owns the contracts and progress of its packages. Source baseline: `b01a45d71a5f5ab29991ad193679f5588ac3662f`, inspected 2026-10-11. Diagnoses are **Interface-checked**; targets and acceptance controls are **Proposed**. No product implementation, tests or measurements are established by plan authoring.

## Implementation packets

The maintainer authorized accepted-target implementation on 2026-10-11; the packet table records current progress. Prerequisites refer to delivered, verified consumed slices. A bounded investigation ends with an explicit selected remedy, retained contract or supported limitation and its reasoning; an unsupported optimization remains unselected.

| Packet | Required input | Delivery and acceptance | Status |
|---|---|---|---|
| P-E1 | Authenticated terminal owner and exact resource policy | Per-record metadata, exact identity access, safe migration and bounded cleanup with interrupted/racing controls. | In progress |
| P-E2 | Current command/signal/drain owners | Direct Python dispatch, remove unrelated exit sweep and reconcile runner interruption exactly once. | In progress |
| P-E3 | Production policy/helper owners and real integration fixtures | Direct responsibility tests, retained canonical integration and preflight; decide pooling only from residual need/isolation. | In progress |
| P-E4 | Crate/fallback decision route under P-G0; P-D5 feature declaration | Lower admission crate, coherent surrogate relocation, scoped generators and actual dependency closure. | In progress |
| P-E5 | Accepted RC18 and actual launch admission | Explicit workload classes and widths; keep current host partition until separate evidence-backed decision. | In progress |
| P-E6 | Bounded questions below; relevant earlier mechanism removed first | Decide native rebuild inputs, invocation reuse, features, provenance, parity and host alternatives without broad qualification claims. | In progress |

Implementation checkpoint, 2026-10-11: E1/E2's final bounded source review has no remaining material findings. Exact durable completion acknowledgment remains a host-drain prerequisite; the observer checks physical drain without waiting circularly on its enclosing acknowledgment. Original cancellation clocks and kill reserves constrain registration, authentication, locks, restoration, physical drain and release, including failed-fence prefixes. Nonnative waiting/escalation/reaping remains outside signal handlers. The affected private-registry controls pass with no observed warning. All five actual final native/outer-entry controls now pass: composer, native-owner and outer-entry interruption, uninterrupted durable terminal composition, and an unacknowledged composer killed without a fabricated terminal. Their fixtures retain acknowledgment evidence through exact allocation release. The maintainer explicitly authorized disposal of the two earlier test-owned admission records; their exact identities and physical drain were rechecked, original records archived durably, and only those two rows removed. Their terminal outcomes remain explicitly unproved. An earlier tight-deadline subprocess ResourceWarning remains a separately unresolved observation.

The outer native observer now uses the same environment-boundary manager selection
for observation, stop and mandatory kill. An actual entry-cancellation control passes
for SIGINT and SIGTERM without inherited manager variables, preserving authenticated
terminal evidence and exact child drain; the focused kill/fallback control also passes.
This repairs the manager-unavailable cancellation discovered during Runtime compilation.

E5's actual admitted placement probes now observe functional and compile launches
with four physical lanes, eight logical affinity CPUs, a four-CPU enforced quota,
and four default Cargo/nextest workers. The timing launch observes eight physical
lanes, sixteen logical affinity CPUs, an eight-CPU quota and eight default workers.
These probes use explicit 4 GiB allocations and inspect every actual cgroup quota
ancestor; they establish placement and width, not timing isolation, performance,
native library team sizes or scientific worker-capacity qualification. The selected
host partition remains unchanged.

E6's ordinary owned-native entry now leaves correctness resolution to its preparing observer instead of resolving before that observer resolves again. Its actual selected-root metadata/inventory reuse and ignored-selection transport remain covered by focused controls. This does not claim cross-run reuse or close every orchestration branch. Environment readiness succeeds with the private test registry; the installed extension still differs from generated contracts and needs an actual rebuild. A readiness attempt against the default registry refused the unsupported migration state; no live registry migration was performed.

E6's parity fixture now owns one actual Runtime, immutable original document-byte
snapshots and lazy shared physical admissions for that session. All four scientific
comparisons still build fresh model packages and results. Locked environment sync,
incremental extension build, import/preflight and fail-never-skip behavior remain.
Three ownership/refusal controls are written but unrun while the installed extension
and generated contracts differ. This is source sharing, not scientific-result reuse
or parity qualification.

## Selected target and consequential corrections

Choose four concrete changes: detach unrelated report sweeping from environment exits; use exact per-resource coordination under the existing resource-policy owner; move native document/physical admission below Runtime; and move the surrogate library integration into its actual Runtime consumer. Test policy owners directly, preserve real canonical integration fixtures, and declare launch classes and runner widths using current host admission authority. The investigation below retains the current host partition/quota policy; its named reopening conditions do not delay the selected work.

These changes address different causes. Removing the Python dispatch build hop and unrelated sweep is independently useful; neither depends on a database pool, new crate, new host scheduler or benchmark. A complete physical-admission extraction improves generator and policy-test dependencies; it does not make a scientific Runtime store-free. Surrogate relocation narrows mathematical/compiler roots; it does not eliminate the libraries from surrogate execution. Build and native artifact reuse still needs full relevant dependencies and trust-boundary checks.

The audit supersedes several original causal claims. `Runtime::from_shared` receives an explicit canonical deployment; the fixture creates the store. `Operations::from_store` binds handles without I/O. Lazy construction of that value cannot close F26. Hot metadata already removes disposed records and demotes settled evidence into exact completed records. Native operation collection already unlinks authentically settled records unless a registration/foreground-parent consumer still owns the proof. Directory population alone cannot show leaked history. Host admission polls until its deadline and assigns two ordinary slots disjoint cores; sixteen processes versus a four-core CPU quota is not measured oversubscription harm.

## P-E1: exact metadata with per-record files

The bounded change review's R36M-F01 identified unsafe ordering in the initially proposed
two-publication cutover. Disposition: target corrected in ADR-0174 to durably refuse legacy
writers before v2 activation, with fenced unactivated recovery. Actual-file crash/race
controls remain open under P-E1; this design correction alone does not establish migration
acceptance. Default live legacy metadata remains unchanged during implementation. Targeted
controls use a fresh private `PSE_TEST_RESOURCE_STATE`, preserving shared host admission.

`scripts/test_resources.py::{resource_metadata,register,finalize,borrow,retain_reference,release_reference,reclaim}` owns resource meaning. Keep that owner and its current distinctions: selected invocation/test; terminal authority; service generation/namespace/database; outcome; manual pin/explicit release; references; borrowers; physical drain; cleaner ownership; retention/disposal. A physical backend must not distribute these policies among Rust fixtures, Python fixtures and the runner.

Select per-record files rather than SQLite for this scope. Source inspection did not identify a consumed all-or-nothing transition across several resource records. `borrow`, `pin`, `record_drain`, reference acquisition/release and reclaim mutate one resource. `reference_report` protects the origin with a borrower, validates its sealed receipt, adds the retained reference to that same origin, then releases the borrower. It does not atomically transfer ownership between two resource records. `finalize` currently updates all resources of an invocation under one global lock, but eligibility and removal are then resource-by-resource; no inspected reader requires simultaneous disposition visibility for the entire invocation. It can project an authenticated, frozen terminal result idempotently onto each associated resource.

Registration does have uniqueness across random resource IDs: `(state,database,kind)` for fixtures, and an owned report directory for evidence. Represent that as exact logical-identity lookup and short identity exclusion, rather than scanning unrelated resources. Keep a stable resource lock across active/completed movement. Exact lookup obtains that lock, selects the authoritative active or completed record, and checks identity. An invocation's small membership lookup and a report's directory lookup are access paths to resource authority, not independently editable policy. Explicit reconciliation may enumerate live/unresolved resource records; ordinary exact lookup must not do so. Completed history must remain outside that enumeration.

Registration and terminal freeze share one exact invocation authority. Registration checks the invocation is open and durably establishes its resource membership under that authority; finalization seals new registration before projecting the authenticated frozen terminal result over members. Use short exact invocation exclusion with a documented order relative to identity/resource locks. A registration already accepted before freeze must appear in projection or in retained recoverable pending membership. A post-freeze registration must refuse with any uncertain external effect still owned, or explicitly consume that same frozen outcome under a supported recovery contract; it cannot create an unowned live resource for a completed invocation. This is an exact fence, not a requirement for simultaneous disposition visibility across every resource. Never hold its lock during database creation/drain or other external effects.

Crash behavior matters more than a convenient directory layout. Registration, index publication and cold promotion can be interrupted. A dangling or contradictory identity/index must refuse acquisition and retain uncertainty; absence cannot recreate an old owner, grant permission or prove drain. Make lookup derivable/reconcilable from identified resource authority, and make writes private, atomic and durably ordered as required by the existing recovery contract. Publish pending invocation membership before resource authority, then publish the exact identity index and complete membership; reconcile interrupted prefixes under exact invocation authority before terminal projection can claim completion. Do not add a bespoke general journal. Keep the existing known cold records accessible, including their sealed receipt/manifests and report indexes. Resolve lock-file lifetime so concurrent openers cannot obtain different locks after unlink/recreation.

SQLite remains a technically credible alternative. Context7's SQLite documentation confirms transactional atomic commit and WAL reader/writer concurrency; it does not remove the single-writer constraint, bounded waiting, connection/schema/WAL lifecycle, migration or external-effect reconciliation. SQLite would give unique indexes and library-owned multi-row transactions. It would be preferable if a demonstrated invariant truly requires atomic registration plus terminal fencing across resource/invocation rows, all-or-none bundle references, or inseparable hot/cold/index updates that cannot safely fail closed and reconcile. Merely having several fields or updating several independent dispositions is insufficient. Do not introduce those invariants to justify the library. If implementation controls expose that need, stop the backend package and have the coordinator decide SQLite before consumer migration; retain the same semantic owner either way.

Migration is an owned cutover, not garbage collection. Under authenticated metadata exclusion, import current live/pending/unresolved records and exact known cold records, preserving every field and association. Validate format, identity, references and ambiguity; unknown records remain retained/refused. Publish one activation/version authority only after import and exact lookups agree. All consumers move together; older writers must reject the new version rather than recreating the full ledger. Preserve the old artifact as migration evidence until no current consumer needs it. No age, size or apparent duplication authorizes retirement. Deletion of production content needs its actual owner/retention authority.

Acceptance must discriminate: concurrent duplicate registration; registration versus terminal freeze in both orders; crash after pending membership before resource publication and after resource publication before identity-index/Registered publication; borrower/reference acquisition versus reclaim in both orders; interrupted register/promotion/finalize/cutover; unknown and corrupt records; an exact completed-record read; deferred cleanup after service closure/reopening; selected-terminal-owner refusal; and fixed live population with increasing cold records. Assert bounded exact access and lock waiting using temporary registries. A performance campaign is unnecessary to establish removal of whole-ledger work; claimed latency benefits would need separate measurement.

## P-E2: command outcome, runner interruption and invocation lifetime

Remove `test_resources.reclaim_reports()` from the applicable newly scoped native and non-native exits in `scripts/pse_env.py`. The audit correctly excludes inherited/no-scope `exec` paths, early refusal/explain paths and direct `just --list` from the original blanket statement. Runner completion and an explicit bounded sweep own resource cleanup. Extend that owner to retry eligible database cleanup deferred by service state; an evidence-only sweep does not cover that case. Service admission/configuration/generation must be rechecked before actual database removal. Batch disposal only after each resource is independently claimed and retains its uncertainty/receipt on partial or unknown effects; one connection is an optimization, not a relaxation of eligibility.

Preserve three outcomes: the workload's terminal result, unrelated cleanup result, and drain of the workload's own admitted work. A corrupt unrelated report registry cannot erase a child exit 0 or 7 by converting it to unattributed boundary status 125. Failure to drain the command's own work remains a boundary failure with retained charge and explicit cause. Removing the unrelated sweep must not suppress native operation/host allocation drain checks. Bound metadata lock waits at the actual owner boundary; a five-second sweep clock does not bound an earlier blocking lock.

`scripts/test_run.py::{run_python,run_rust}` compose terminal reports after synchronous child return and lack a local interruption-finally path. Add exactly-once terminal reconciliation by the already selected terminal owner across normal return, signal and exceptions. Preserve authentic selection/JUnit evidence; missing or partial reports yield incomplete/failed outcomes, never inferred passing resources. Keep the runner's signal/exit meaning and separately report reconciliation/drain failures. An enclosing native/observer owner may already own signal forwarding and opaque work drain; do not install competing cancellation or terminal owners. Recovery selects the exact invocation and authenticated outcome, not a numeric process group or an arbitrary newest report.

Invocation JSON, selected catalogs, generated nextest config, JUnit and terminal artifacts may retire only after all association, reconciliation, diagnostics and retained-reference consumers are identified and discharged. A runner returning is insufficient. Record minimal unresolved invocation state until authentic finalization; cold diagnostic retention is distinct from live coordination. Do not prune by age or pass label alone. For per-record finalization, first establish the frozen terminal composition with its selected owner, then idempotently project outcomes; interruption must resume without granting a missing pass or losing a reference. This is the needed lifetime contract, not a general workflow engine.

## P-E3: direct owners with canonical integration preserved

Implementation checkpoint, 2026-10-11: the direct deployment tests now distinguish
the factory's actual consumed memory pool from the incoming wrapper pool that
`EngineFactory::from_builder` normalizes away. Each foreign owner checks its exact
refusal, configured width is independent of currently held permits, and identical
consumed services remain admissible. Real canonical composition and durability
assertions retain fresh fixtures and exact store coordinates. Focused compilation
and direct/canonical controls remain open.

Use asserted responsibility to migrate tests. In `crates/pse-runtime/src/workflow/tests.rs`, `workflow_composition_preserves_custom_same_owner_factory_and_partition_demand` and `workflow_composition_refuses_foreign_consumed_owners_and_width` construct canonical Runtime fixtures to exercise factory ownership/width checks. The production owner already exists: `Runtime::check_deployment(&SharedRuntime,&EngineFactory)`. Tests in that module can exercise it with actual `SharedRuntime`, factory, cache/CPU owners and controlled substitutions, without a canonical deployment. They must still test distinct outer RuntimeEnv wrappers with identical consumed services, foreign pool/disk/cache/store registry, foreign CPU semaphore, wrong configured width and in-flight permits. Keep a smaller real composition integration assertion covering actual canonical wiring.

Similarly use typed decoding, strategy/transition and lease-policy validation owners directly when their assertions do not consume canonical source/publication. There are already direct strategy tests in `math/strategy/tests.rs`; not every test using the Runtime helper is movable. `workflow_durability_selection_uses_this_deployment_and_refuses_zero_intervals` mixes deployment binding with interval policy: split a direct policy assertion from the actual store-binding integration assertion rather than inventing a fake deployment. When a helper needs extraction, make production consume the same operation and explicit typed facts. Do not substitute a fake Runtime, test-only admission or handwritten canonical source.

Scientific authored packages, persistence, interpretation rejection, publication and actual native execution stay on real canonical fixtures even for ephemeral run durability. Keep isolated opaque databases wherever state is asserted; late Rust interpretation checks; original-space scientific checks; fixture borrower/drain/reference ownership; and fail-never-skip behavior. One runner compatibility preflight before fan-out can improve diagnostic locality, but generation/interpretation changes still need late refusal.

Pooling is deliberately conditional. First move proven subject/setup mismatches and observe remaining canonical setup if that affects a pool decision. A reusable initialized database must have an enforceable reset contract covering rows, protections, staged edits, claims, interpretations, contexts and late effects, plus actual previous-owner drain and exclusive next borrower. An unmodified initialized fixture handed once to one test is simpler than resetting a used database, but pre-provisioning does not remove necessary DDL. If reset/isolation cannot be established, retain fresh per-test UUID databases; runner-owned batched provisioning/registration and independent preflight remain viable. No store-free production Runtime is selected.

## P-E4: lower admission, generators and dependency closure

Add one crate, working name `pse-admission`, through the ordinary crate-addition ADR route. Its responsibility is native document intake and physical source admission: move current `pse-runtime/src/authoring_driver/**`, `physical.rs` and its `plans`/`preconditions`, together with registry-generated document adapters. Inputs remain owned document bytes, parse budgets, explicit registry/validation context, engine session/factory, finite allocation and cancellation. Outputs remain admitted checked relation batches, document edit/source ownership and physical inventories with exact units, basis/reference state, explicit Boolean/neutral context, preconditions and retained allocation. The crate does not select scientific runs, publish canonical revisions, own MathService or allocate native solver sessions.

Depend on the existing lower `pse-authoring`, `pse-model`, `pse-modeling` (owned data-document contracts), `pse-quantity`, IDs/diagnostics, schema, relations, columnar, engine and rules owners plus actually used parsing/serialization/data libraries. No dependency on `pse-runtime`, `pse-math`, `pse-compiler`, `pse-operations` or `pse-backend-native`. Keep pure `pse-authoring` pure: moving DataFusion/Arrow admission into it would widen every DSL/model consumer to solve the wrong boundary. Keep the current engine-owned resource/factory implementation instead of duplicating it in a new facade.

`xtask/src/codegen/physical.rs::Sources::load` currently builds `SharedRuntime` only to obtain pool/session construction. `pse_engine::resources::EngineResources::build` and `EngineFactory::new` already provide that capability; `workflow/modeling/pure.rs` uses them without MathService today. Construct actual finite resources, cache/CPU extensions where consumed, native engine profile and validation context directly. Retain explicit package selection, registry fingerprint agreement, declaration construction, package resolution and native invariant execution. Keep original document/allocation/spill ownership until the inventory finishes; narrow inventory partition demand as the existing production physical-context path does. This is production admission, not bypassing the engine with an independent reduced validator.

Migrate Runtime/Python/xtask direct imports together and delete old modules/import paths, without compatibility re-exports. Adjust `pse-codegen/src/codegen/rust/{mod.rs,...}` and its declared generated roots to emit document adapters in the new owner; regenerate, never hand-edit generated output. Generated pure document value types stay in `pse-authoring`. Move tests that exercise only document/physical admission to the new crate; tests composing scientific workflows remain Runtime integration tests. Existing document-admission tests that currently reach workflow PhysicalContext need a lower admitted-inventory assertion plus separate integration coverage, rather than a reverse dependency.

Separate xtask capability features/targets for physical admission from product-document schemas. `xtask/src/codegen/schemas.rs` genuinely derives actual workflow/native/operations serde types and serializes boundary fixtures. Those higher schema consumers cannot all disappear with the admission extraction. Retain their owning types and broad full-schema route; do not copy DTOs or handwrite schemas. Bootstrap registry contracts first where generated consumers need them, then physical admission, actual product schema emission and native ABI generation as separate capability operations. Existing `--only` supports focused targets; the full aggregate still regenerates everything and compares both directions.

Run Ipopt bindgen with the actual selected native headers independently from physical/registry generation. A filename shortcut is insufficient to reuse bindings: transitive headers, bindgen/clang version/options, target/features, missing outputs and selected native generation all affect validity. Initially keep actual bindgen comparison for the aggregate; optional reuse requires a complete owner key. Preserve byte/mtime no-op publication and declared root pruning boundaries.

Independently change `py-test` to `scripts/pse-env --store -- <checkout-python> -m scripts.python_tests` with the existing forwarded args. Delete the `xtask::python_tests` dispatch command/module once all callers migrate. Python selection, native observer requirements, fixture composition, imported extension provenance and signal/exit handling remain owned by their existing Python modules. This removes a Rust dispatch build, not the extension's own required build. Compare selected tests and failed/signal exits through the same mode; no latency probe is needed to choose it.

## Surrogates, workspace fallback and feature closure

Move `pse-math/src/surrogate.rs` into a coherent library-integration module under Runtime, distinct from its existing actor/orchestration module. Migrate all direct consumers: `math/surrogate`, prediction, strategy, starts, derived preparation and tests. `egobox-ego`, `egobox-moe`, `egobox-gp`, argmin and ndarray are used only by that module in the inspected math source; move their normal dependencies to Runtime. Move `egobox-doe` with its tests as a dev-dependency: the original normal-dependency claim was wrong. Preserve retained non-Send library-state ownership, exact correspondence/task identity, proposal versus original scientific acceptance, typed callback terminal cause, bounded work/memory and drain. A feature gate alone cannot establish narrower roots under workspace unification. No new surrogate crate is justified for the currently single consumer. Coordinate with audit A04's snapshot-demand work so relocation does not preserve or duplicate an obsolete copying API; relocation and snapshot semantics have separate acceptance.

Under accepted RC17, select retirement of the workspace-hack fallback when the selected-unit closure confirms its unnecessary broad dependencies; route the concrete removal through the existing ADR process before implementation. The supported toolchain remains the pinned nightly with workspace feature unification. ADR-0122 Outcome 5's support for Cargo ignoring `[unstable]` is the actual rule being changed; the crate-addition ADR rule remains intact. Delete the hack crate, member dependencies, hakari configuration/generation/check recipes and corresponding generated-output exception through the ADR/removal route. Preserve `family-check`, exact locks/pins, opt-in exclusion and explicit force-validation. If the supported pinned toolchain cannot preserve the required feature/type-family/validation behavior without it, reconsider narrowing membership to actual supported fallback consumers and state the remaining closure breadth; do not claim the full remedy. A lexical pruned tree is not feature/profile evidence.

The completed L2 feature investigation below specifies these choices for implementation: DataFusion default file/compression/function features, parquet async/object_store, Arrow IPC compression and BLAKE3 rayon/dead hashing knob. Preserve all consumed authored SQL/invariant functions, file/Parquet document readers and external formats; no blanket `default-features=false` is justified by unused result compression alone. Query actual selected target/host/dev units and pinned source contracts. Declare `serde_json/float_roundtrip` explicitly before dependency pruning can remove its incidental Clarabel activation (F29 belongs to its coordinator owner). `pse-py` normally enables `pse-operations/test-support`; inspected Python initialization uses ordinary CanonicalStore connect/create/disconnect, and no production testing-module use was found in `crates/pse-py/src`. Remove that production opt-in after checking the complete call set; retain genuine Rust dev support and correctness activation in selected-root dev closures. No second Arrow/DataFusion universe is permitted.

## P-E6: bounded development investigations

E-C01/G-07: scope `IPOPT_DIR`/`PKG_CONFIG_PATH` tracking to features that actually consume them, and root-isolation prefix tracking to root isolation. Preserve workspace POUNCE source identity outside linking and all real link/receipt/header inputs. Stable output writes should compare bytes first. Before asserting a warm rebuild benefit, alternate the same no-native feature/profile/target unit plain→native environment→plain on an isolated, quiescent checkout, using fingerprint logs and artifact freshness. Distinguish script execution, output bytes/mtime, backend rustc and downstream rustc. Different unit keys do not establish the proposed same-unit cause; tiny script-only work stays Low priority. Do not clear shared caches or touch concurrent source to manufacture a measurement.

G-05's native sealed-generation verification/discovery belongs with F05/RC05's assurance owner. Keep authenticated borrowing and explicit-prefix/publication trust checks. Repeated collection can move to generation publication/explicit GC only with selection-versus-retirement exclusion. A stat tuple is not content identity. Native records already retire on authentic settlement; classify live/pending/unknown/removable obligations before changing their lifetime. Do not repeat the original historical-leak inference.

The pre-run resolution chain can reuse exact nextest binaries metadata/inventory within one invocation, preserving actual target/host/dev feature closure and selected executable/test association. Select one prepared invocation consumed by validation/unit graph and nextest list/run; do not remove force-validation resolution or trust a stale inventory across source/config/environment changes. No cross-run cache is selected.

Implementation checkpoint, 2026-10-11: standalone native Rust execution now uses
the same prepared invocation as observer execution, carrying binary/Cargo metadata
and the exact selected inventory into nextest run. Run-only output options stay
outside enumeration; ignored tests, filters and profiles remain explicit. All 25
affected runner controls pass, including actual Python worker overlap and the
standalone inventory-reuse control. Parity environment reuse and enclosing native
execution qualification remain open; this is within-invocation reuse only.

Keep conservative `validation_scope.RUST_INPUTS` version 3 and environment provenance as the initial target. An input-scope refinement must show all result-affecting source, scripts, packages, compiler, native, feature/profile and dirty-context dependencies, including absence; otherwise retain invalidation. Full provenance/assessment retention is not a per-test hot-path defect. Retire those artifacts only through the same reference/borrower/manifest owner. Keep dev opt-levels, checkout-local targets/shared sccache, mold, unwind and floating-point settings; no evidence here chooses profile tuning.

The parity investigation retains the existing setup: locked isolated `.venv-parity` sync and native extension development may already be incrementally cheap. Reuse a verified environment/extension only under exact lock/interpreter/source/features/native-prefix identity and current import/provenance checks; retain fail-never-skip solver/reference preflight and independent IDAES/Pyomo oracle/tolerances. Its session-scoped comparison Runtime is already shared. Immutable physical/reference document preparation may be retained within that owner if source admission/lifetime remains sound; no cached scientific result or producer-derived oracle is selected.

## P-E5: explicit launch classes and widths

Implementation checkpoint: an attempted focused native run built its artifacts but
executed zero tests because the outer recipe inherited the smaller functional lane.
The unit recipe wrappers now select the existing scientific class before nesting,
and host classification recognizes the native unit aliases. Explicit caller and
inherited classes still take precedence; host partitions and scientific widths are
unchanged. Focused actual-recipe transport/classification controls pass. A native
observer rerun is required to establish the repaired placement path; the refused run
does not establish test acceptance.

For F28/RC18 use existing explicit recipe-default classes at real workload launches, preserving user override and inherited owner authority. Derive/test Cargo and nextest widths from the admitted operation, naming physical cores, logical affinity and CPU quota separately and preserving store/scientific effect groups. Keep present partitions/quotas/memory admission as the default. The investigation below compared timing isolation, cross-repository interference, borrowing/revocation, finite population and resident-service ownership, and retained current policy. Reopening requires a named constrained operation and a complete interference/revocation contract. Change to weights/borrowing only if the complete model preserves those requirements and improves a named blocked/constrained operation. Fair waiters are needed only if starvation/fairness requirements warrant them; bounded polling can remain deliberate. Optional measurements use unchanged artifacts and quiet admitted host conditions. Arithmetic alone establishes neither harm nor improvement.

## Package sequence and completion

1. Independently remove unrelated exit sweep and direct py-test dispatch, with temporary-registry/status and Python selection controls. These are ready from static evidence.
2. Define terminal composition, exact metadata access/migration and invocation retirement contracts; implement per-record coordination with all fixture/runner/report consumers and interruption/reclaim controls. Preserve unknown obligations and known cold evidence.
3. Migrate demonstrated direct policy tests; keep real canonical integration; implement preflight. Retain fresh UUID databases; used-fixture pooling is unselected for the reasons below.
4. Route crate addition/removal and fallback decision through the ordinary ADR process before topology changes. Extract admission plus generated roots/consumers; separate capability generator targets; relocate surrogate integration and delete superseded modules/dependencies.
5. Implement explicit launch classes/derived widths. Apply the E-C01, resolution-chain, feature, provenance and parity decisions below, then run the actual changed-root acceptance controls. The investigative branches are complete; optional reopening follows their named triggers.
6. At functional scope end, coordinator assembles focused script/owner tests, actual physical/regeneration comparisons, relevant canonical/Python/native integration and applicable hygiene/manual checks. Measure benefits separately only when making quantified claims. Scoped passes do not qualify whole simulator science or unexercised host behavior.

Cross-owner prerequisites: P-D5 explicit float parsing precedes surrogate/feature pruning; F05 assurance defines native verification reuse; A04 supplies surrogate snapshot semantics; generator registry-owner changes precede regenerated consumer migration. Shared edits include justfile, root/member manifests, codegen roots, Runtime import paths and test-resource/runner scripts; the coordinator owns their integration. Completion deletes the obsolete dispatch, old admission modules/adapters, math surrogate module/dependencies and accepted fallback machinery while preserving actual generated outputs and canonical/scientific guarantees.

Library documentation consulted: Context7 Cargo workspace feature unification and build-script change detection, rooted at `https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/unstable.md` and `https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/build-scripts.md`; SQLite transactional/WAL documentation through `/websites/sqlite_docs`, with official entrypoint `https://www.sqlite.org/docs.html`. Current docs explain mechanisms; local pinned source/configuration establishes actual boundaries. No utilization JSONL, capability maps or catalog was used. 

## Investigation decisions, 2026-10-11

The investigation baseline is `2d6bf6ae7`, initially clean. [Development evidence](36-investigation-evidence/development.md) records source observations, executable probes, alternatives and exclusions. These decisions refine the selected target; every implementation packet above remains **Scheduled**. Prototype success does not implement the product.

### Exact metadata and terminal composition

**Select per-record files with invocation-owned pending membership.** The current API's resource policy remains in `test_resources`; a storage adapter implements exact operations without lending a mutable whole-ledger dictionary to consumers. Its operations are `lookup(ResourceId)`, `register(LogicalIdentity, InvocationId?, TestId?, OwnedResourceFacts)`, `update(ResourceId, transition, original_deadline)`, `freeze(InvocationId, authenticated_terminal)`, `project(InvocationId, ResourceId)`, and bounded unresolved enumeration. `ResourceId`, invocation nonce and service generation remain distinct. Exact identity is the existing normalized `(state,database,kind)` or owned report directory; the hash selects an index path, and the stored full identity must agree. Acquisition reads the record under its stable lock and rechecks service generation/admission where consumed.

The required lock order is **invocation → logical identity → resource**. Operations that begin with a resource lock never acquire invocation exclusion. Finalization freezes under invocation exclusion, releases it, then projects onto resources one at a time. No metadata lock covers external database creation, removal, service startup or physical drain. All lock acquisition consumes the caller's original absolute deadline; unknown or exhausted acquisition refuses, retaining effects already issued. Lock files are stable across hot/cold movement and are not unlinked by per-record cleanup. Directory/version retirement requires exclusion of all openers; ordinary compaction cannot prove that premise.

Use one stable `resources/<id>.json` authority and separate `active/<id>`/invocation/identity access paths, rather than two writable hot/cold copies. Active markers contain only resource identity; lookup reads the resource phase under its lock. A stale marker can cause bounded reconciliation work but cannot authorize acquisition or disposal. Cold promotion changes phase and retires the active marker after the authoritative record is durable; an exact cold read remains one identified resource. Migration imports legacy completed records into that same authority while retaining their manifests. Bounded cleanup enumerates active/unresolved access paths, never the historical resources directory. Every private atomic file publication flushes file content before replacement and its containing directory before an acknowledgment that promises crash recovery. The actual-file controls must challenge these durability assumptions.

Registration now has a specific crash protocol: under invocation exclusion verify Open; reserve exact identity; **durably publish pending membership before resource authority**; publish the resource, then its exact identity index; mark membership Registered; return the resource capability. External fixture creation follows accepted registration. A crash before authority is published has an identified metadata-only intent; a crash afterwards has a membership from which recovery locates the exact resource. Freeze resolves every Pending member before claiming projection complete. It can roll back a proven never-published metadata intent; contradictory/missing authority after uncertain publication stays unresolved and pinned. Post-freeze registration refuses before effects. Unassociated bare resources stay incomplete/pinned and cannot inherit an arbitrary terminal outcome. This refines the earlier resource-before-membership recovery sketch, which would otherwise require a scan to discover an orphan.

An accepted registration retains its creation/effect borrower through subsequent external creation and settlement. Freeze may project a terminal disposition while that work remains live, but it cannot mark drain or authorize reclaim; late effects remain charged to the registered owner. Recovery distinguishes a metadata-only Pending intent from an issued external effect, retaining the latter's exact uncertainty until observed settlement. A caller returning or dying is not proof that an accepted resource never existed.

Terminal composition is immutable per invocation and selected reconciler. It records selected catalog identity, workload exit/signal, authentic complete/partial report association, per-test dispositions, reconciliation state and owned drain outcome. Only complete authentic passing test evidence can project Pass. Freeze prevents additions; idempotent projection can resume after interruption and records its exact terminal identity. Report-selection artifacts and membership remain until projection, references and diagnostic retention release them. A frozen complete result is not proof of drain, and an unrelated sweep failure is not the workload's exit code.

Cutover uses the existing legacy `.lock` to exclude old metadata writers while importing and validating identified live/unresolved and known cold authorities. After exact lookups/membership agree, durably publish the v2 refusal sentinel at the old ledger location **before** publishing v2 activation; legacy `validate_ledger` rejects unsupported versions. A crash between these publications leaves a fenced-but-unactivated state: the new owner resumes and validates the same staged imports under legacy exclusion before activation, while old writers refuse. Recovery never serves writable v1. This order was corrected by the bounded metadata change review (R36M-F01). All consumers move together. Do not restore a writable v1 ledger after activation. Preserve ambiguous records and migration evidence; no production deletion is selected. The implementation must exercise interrupted publication/fsync/rename and both race orders on actual files; the ten-case abstract crash-prefix probe establishes the protocol's intended ordering only. SQLite remains unselected: no current consumer requires simultaneous all-resource disposition visibility. A demonstrated indivisible cross-resource invariant reopens the backend choice before migration.

### Commands, fixtures and runner ownership

**Select direct Python dispatch and remove the unrelated scoped-exit sweep.** `scripts.python_tests` already owns marker intersection, native observer selection and signal-status normalization. The new `py-test` recipe calls it through the same store-selected environment and forwards positional arguments unchanged; delete xtask's dispatch after its callers move. Keep native setup/actual operation drain and allocation release in their enclosing owner. Cleanup selected by the runner or explicit sweep retains its own errors and can retry eligible same-generation databases once the service reopens. A failed/unknown own-work drain still retains capacity and a boundary failure; unrelated registry corruption cannot replace child exit 0 or 7.

For direct runs, the runner owns terminal reconciliation in `finally`; inherited observer children consume the selected terminal-owner declaration and do not independently finalize. Signal forwarding stays with the existing enclosing process owner. Preserve the original child result while separately reporting report/reconciliation/drain errors. Missing JUnit, selection or terminal evidence produces incomplete disposition. An authenticated retry freezes/projects the same invocation, never the newest report or a recycled PGID. These controls need actual normal/exception/SIGINT/SIGTERM paths at implementation, not a competing signal handler in every layer.

**Retain fresh isolated UUID canonical databases; do not implement a reset pool.** Source tracing shows fixture handles and delayed interpretation/source/result operations can outlive a caller, while `remove_isolated_fixture` records drain intent and runner disposition. No reset operation currently proves the absence of all late effects or restores canonical interpretations, staged state, claims and source protections. That lifetime gap is sufficient to reject pooling now; setup timing cannot supply it. Move the two factory-affiliation assertions to `Runtime::check_deployment` using actual services and keep one composition integration control. Split interval validation from durability/deployment binding. Runner compatibility preflight is selected for early diagnostics, while late interpretation refusal remains mandatory. One-use pre-provisioned fixtures/batched registration remain possible within the same fresh-database contract; select them only if residual setup is shown material after these migrations.

The carried nextest lead is current: the exclusive filter spells `fixture_connections_preserve_external_and_deployment_databases`, but the source test is `fixture_connections_preserve_deployment_database`. The real test observes deployment-database existence; retain its exclusive **store-effect** classification and correct the exact selected name. Do not infer a need for native scientific serialization or reuse the literal sixteen as a CPU count. Generate effect capacity and runner width together from the admitted invocation, preserving first-match override semantics and verifying the selected inventory.

### Build topology and selected closures

**Select the lower admission extraction and surrogate relocation already described.** The lower crate exposes document loading/editing and `PhysicalInventory` with owned bytes/batches/allocation, parse budget, cancellation, and the actual engine-bound validation context. It consumes the [36d bound model](36d-relational-and-python-boundaries.md#p-d1-bind-and-retain-the-actual-immutable-model); a registry fingerprint alone cannot substitute for that context. `PhysicalInventory::load` consumes an `EngineSession` from the same factory/registry. Keep its quantities/preconditions/compiler-context projection in the lower owner, using existing model/quantity types; it does not depend on a compiler instance. Runtime owns scientific preparation, workflow errors and canonical publication. All source adapters/generated roots/imports migrate together without reverse dependencies or compatibility exports. A lower error may be wrapped by `WorkflowError`; the lower crate cannot depend on that wrapper.

Generator capability routes are registry bootstrap, physical admission, full product schemas, and native ABI. `xtask::codegen::physical::Sources` retains lower documents, session, finite EngineResources and spill ownership instead of SharedRuntime. The full schema route still derives actual workflow/native/operations DTOs. Separate feature/command requirements express these genuine dependencies; a focused physical target cannot merely select a subcommand in a binary whose normal dependencies still include Runtime. Full aggregate comparison continues to include every target/root.

**Select workspace-hack retirement through RC17's ADR route.** Pinned-Cargo unit-graph probes in a disposable manifest workspace found 423→382 engine check units and 285→273 packages after removing hack dependencies; the normal Arrow leaf features stayed equal. Explicit engine correctness selection still enabled `force_validate` on arrow-array/data/cast. Relocating the surrogate manifests reduced the math check graph from 368→264 units and 250→175 packages, removing all four egobox packages. These are **Measured graph populations**, not compilation time or proof the relocated source compiles. External lock source/version/checksum tuples were unchanged. Implementation still verifies all touched target/host/dev/native roots and one type universe before removing the fallback machinery. No evidence chooses a toolchain/profile change.

### Remaining development alternatives

| Question | Investigation decision and concrete consequence | Reopening condition / owner |
|---|---|---|
| E-C01 same-unit native inputs | **Select feature-scoped inputs and byte-preserving writes.** An isolated workspace using the actual unmodified backend build script, pinned build dependencies and a miniature downstream rebuilt both libraries on plain→unused native environment→plain, while output bytes stayed equal and mtimes changed. Warm plain rebuilt neither. Gate IPOPT/PKG_CONFIG tracking by the script's actual native consumers; include root-isolation pkg-config inputs where used. Keep workspace POUNCE source identity independent of linking, and preserve native header/receipt/link dependencies. | P-E6 implements local change; actual backend/downstream latency remains unmeasured and Low priority. The probe is not a native-mode qualification. |
| Bindgen reuse | **Retain actual aggregate comparison**, with independent capability dispatch. Filename or direct-header mtime cannot cover transitive headers, clang/bindgen/options, target/features and native generation. | P-E4 may add reuse only after a complete selected-header closure and library/tool key exist; otherwise comparison remains the concrete target. |
| Nextest resolution | **Select one prepared invocation inventory**, including actual Cargo unit closure, binary metadata, package metadata, normalized filter/profile/platform arguments and executable associations. Existing native capture already passes retained metadata into child execution; ordinary/validation orchestration should consume that same prepared object rather than reacquire it. No cross-run cache. | P-E6; changed input/environment/config or unknown provenance refuses reuse. Force-validation resolution remains independent and mandatory. |
| Feature opportunities | **Remove unused BLAKE3 rayon opt-in and production pse-py operations/test-support**, after complete consumer migration/checks; no `update_rayon` call or production testing-module consumer was found. **Retain DataFusion default functions/SQL/Parquet/compressed-file support and Arrow/Parquet consumed boundary features.** The actual SQL/invariant and document-reader contracts are not a fixture-only function whitelist. | P-E4/P-E6; an explicit narrower supported input/function contract plus exact selected closure may justify further pruning. Public dead hashing setting is a separate 36d boundary decision. P-D5 explicit float_roundtrip precedes pruning. |
| Provenance scope | **Retain RUST_INPUTS v3 and current full capture.** Scripts, config, absent providers, generated/dirty inputs and package/native roots cross simple filename scopes. No inspected local projection establishes completeness. | P-E6; a consuming responsibility supplies a complete positive/absence dependency declaration, then compare its invalidation against the conservative capture. No new provenance registry is selected. |
| Parity setup | **Retain locked uv sync and incremental maturin develop.** Skip neither from stat/lock equality alone: installed distributions, editable source, interpreter, feature/native provenance and import identity remain consumed premises. Select sharing immutable admitted documents within the already session-owned Runtime where exact source/lifetime permit, not scientific-result reuse. | P-E6; complete installed-environment/extension ownership could permit exact reuse. A demonstrated setup bottleneck can reopen further amortization, while IDAES/Pyomo remains independent and fail-never-skip. |
| Host redesign | **Retain current disjoint functional/timing lanes, quotas and finite partitions.** Idle timing borrowing needs enforceable revocation/drain without perturbing a newly admitted measurement, cross-project authority, and resident-store accounting; weights alone do not provide that. No required blocked operation was identified that warrants this policy change. | P-E5/P-E6; a named constrained operation plus enforceable timing/interference/borrower contract, followed by admitted unchanged-artifact observations. This is an explicit retained policy, not an unperformed mandatory benchmark. |
| Launch widths | **Select explicit recipe classes and operation-derived defaults.** After host admission, derive Cargo/nextest CPU width from the actual allowed physical lanes, logical affinity and finite quota; record those separately. Preserve user override within enclosing authority. Derive effect-group capacity and exclusive weight separately from CPU width. | P-E5; actual supported allocation/profile inventory supplies widths. Do not rewrite native library team sizes or store exclusivity from a CPU heuristic. |
| Prior state-review leads | **Current source distinguishes consumed Runtime services; current recovery and typed readiness controls pass.** Keep those mechanisms. Remaining provider/deadline/cache-owner tests must synchronize on the claimed phase/owner and retain operation outcome; do not infer committed-before-expiry from a short lease. | P-G1 touched scientific/lifecycle acceptance; these controls neither reopen old plan status nor claim all recovery/native paths qualified. |

**Tested**, zero-failure target: `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_test_resources -q` ran 50 controls, all passed. Five selected supervisor recovery/readiness controls also passed. The isolated late-registration control accepted a pinned incomplete resource after finalization; it did not infer Pass. The abstract metadata model passed ten crash/order cases. Exact commands, unit-graph counts, the corrected probe harness limitation and remaining actual-file/native/whole-product checks are recorded in the evidence. No production target or scientific fixture was changed by this investigation.

## Implementation checkpoint, 2026-10-12

The latest enclosing setup run exposed two cancellation-cleanup races despite
passing isolated controls. The selected native workload now retains its non-raising
signal owner through its cleanup callback, joining authenticated physical drain
under the original process/handoff/parent clock. Only populated or active owners
may be observed again; unavailable or mismatched authority refuses immediately,
and ordinary cleanup without a cancellation clock gains no new waiting budget.
The pending-registration fixture injects its signal into the exact masked thread;
the production registration protocol is unchanged. All 65 native-operation controls
pass, including pipe-gated descendant reaping, first and repeated signals, expiry,
unknown ownership and spawn failure. Independent review additionally corrected the
measurement boundary: the child timestamp is captured at successful wait return,
before drain, and published afterward. All 37 measurement controls pass. Enclosing
setup and assembled acceptance remain open; no allowance was widened.

An enclosing retry also exposed recipe-fixture import leakage under a caller's
checkout `PYTHONPATH`: namespace-only temporary scripts could resolve production
command modules instead of the intended harmless argument recorder. Temporary
fixtures now provide a regular package and the complete current ADR recipe wrapper.
All nine recipe transport controls pass with that adversarial inherited path. The
accidental disposable proposed ADR was archived outside the checkout and removed;
accepted records and concurrent work were preserved. The interrupted suite remains
failed; its observed cap exhaustion does not justify widening fixture allowances.
The assessment argument fixture received the same package isolation correction.
The complete setup suite now passes 940 controls with the inherited checkout path,
including actual owner interruption and exact terminal/drain composition.

A final independent readback identified a reaped-launcher numeric process-group fallback.
Cancellation now uses that fallback only while the launcher remains unreaped; afterward
only its authenticated scope may be stopped and joined. The revealing control requires
the real scoped stop/drain for matching ownership and refusal for a changed invocation,
with no numeric fallback in either case. Active-launcher fallback controls still pass.
The refreshed full 940-control setup run includes this repair and completes successfully.

Selected P-E1–P-E6 source changes are integrated: per-record metadata and frozen
invocation membership, direct dispatch with exact terminal/drain ownership, direct
responsibility tests, lower admission and generator roots, Runtime-owned surrogates,
workspace fallback retirement, explicit launch widths and feature-scoped native inputs.
The actual lower-root graph contains 422 units across eight selected roots and excludes
Runtime, mathematics, compiler, Operations, native backend/sys and Python. Its 40
all-target tests pass. The miniature same-unit build-input probe also passes; it is
not a whole-build speed measurement.

The broad light tooling selection passed 454 of 455 controls; the remaining assertion
used obsolete permission wording and its repaired focused control passes. This is
composite evidence, not a rerun of all 455. Actual interruption/terminal controls retain
their scope and authenticated acknowledgment requirements. Failed Runtime campaigns
remain failed; exact registered fixture recovery preserved outcomes, pins and databases.
A test-support lifetime repair now joins tracked automatic protection release at final
lexical fixture drop, with a gated acknowledgment control and three neighbors passing.
Production asynchronous cleanup is unchanged. Process death alone remains insufficient
drain evidence.

Documentation publication, aggregate codegen/bindgen comparison and governance pass.
The affected hygiene recipes, native lint and all 398 feature combinations plus the
separate no-default check now pass; the original failed composite remains failed. Production metadata migration, pooling,
host-lane borrowing and broader provenance reuse remain unselected or excluded.

The assembled ordinary-native attempt exposed a terminal projection clock covering the
whole frozen membership rather than each bounded metadata operation. Frozen test outcomes
remain unchanged; their partial projection is not enclosing completion. The repair's
91 affected resource controls and authentic resumption now pass: all 788 members are
projected, the original 208-member prefix remains unchanged, and frozen failures and
unobserved drain remain unchanged. Full tooling setup passes after the typed metadata
repair as well as the explicit worker/artifact fixtures, supervisor private-file permissions
and report freshness before real signal injection. The subsequent exact-integer clock
regression and all 101 controls in its two consumer modules pass without competing
activation admission. No ResourceWarning was observed in the full setup run; the earlier unattributed observation
has no recovered selector or diagnostic and is not claimed fixed.


A revealing native cancellation retry exposed an incorrect storage-only drain projection:
it retained unrelated execution ACK guards while omitting their named units. The
projection now includes only storage-named guards, preserving malformed-scope refusal
and the parent's complete guards. The regression also requires a pending storage guard
to retain its affiliation. Host/supervisor controls and source review pass, and normal
authenticated reconciliation releases the failed run's drained allocation. Its test
failure remains unchanged; no deletion or terminal-outcome fabrication occurred.

A later assembled run exposed a separate reclaim clock mismatch: metadata settlement
retained its five-second wait across a database removal with an existing ninety-second
allowance. Default reclaim now selects that finite operation clock once and shares it
across claim, effect and settlement; a supplied absolute clock is retained exactly.
Directory deferral and authentic eligibility remain unchanged. The deterministic
six-second-effect and expired-caller-clock controls pass with all 149 affected resource
and assessment controls. This does not preempt synchronous filesystem removal or prove
disposal when the original clock expired. Prior failed cleanup records remain pinned.
The actual producer fixture now uses the Rust/build input closure, preserving relevant
execution and cleanup drift detection while excluding unrelated concurrent review prose.
Independent retirement readback found no selected production second path among the
removed dispatcher, unrelated exit sweep, workspace fallback, relocated surrogate,
bound-context or admitted-seed mechanisms. The inert build-review catalog entry for
the removed workspace-hack crate was deleted; reviews for surviving owners are retained.

## Maintainer stop handoff, 2026-10-12

Execution stopped after the requested immediate wrap-up. The [coordinator’s stop checkpoint](36-efficiency-alignment.md#maintainer-stop-checkpoint-2026-10-12) owns current evidence boundaries, worktree reconciliation and the remaining execution route. The passing full setup and feature checks retain their recorded source scopes. Worktree cleanup is complete; current-source scope-end checks and assembled acceptance remain with P-G1.
