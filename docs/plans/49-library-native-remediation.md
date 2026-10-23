---
title: "Plan 49: Library-native remediation and SurrealMX integration"
status: in-progress
date: 2026-10-21
adrs: [ADR-0214, ADR-0215]
review_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md, docs/design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md, docs/functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md]
scenario_sources: [docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md]
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [current-work, design, libraries, storage, preparation, numerical-execution, validation]
---

# Plan 49: Library-native remediation and SurrealMX integration

## Purpose and authoring basis

Use the libraries' composed capabilities to remove repeated interpretation, canonical
row-location machinery, mathematical reconstruction and copied orchestration, while
preserving physically typed scientific execution and truthful durable work. The target
is designed as a whole; the review's illustrative remedies do not fix its implementation
shape. SurrealMX **1.0.0 is selected**, including the integration required to use it with
SurrealDB 3.3.4. It is not an optional experiment or contingent on upstream bundling it first.

The [integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md),
[recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md)
and [source coverage](49-source-coverage.md) supply the findings, recommendation corrections,
scenarios, historical evidence and candidate routes. Both new library overviews—
[SurrealDB](../libraries/surrealdb.md) and [Symbolica](../libraries/symbolica.md)—were read
in full before target authoring. Feature follow-ups used their web links, tagged upstream
source and installed pinned APIs, as the operator requested; Context7 is not the source
for these follow-ups.

**Authoring baseline, 2026-10-21. Interface-checked:** source at
`e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`, with the revised review, audit companion and
supporting audit evidence retained as explicit dirty inputs. The review's original product
baseline is `a791c533eca0f65b195895f071e60cd1e38e121c`, diagnostic publication `0530dcaa1`.
**Proposed:** the integrated target and implementation packages below. Existing probes
retain their original Tested/Measured scope; no product tests or benefit measurements ran
as part of plan authoring. Document validation is recorded separately at the checkpoint.

The standard is Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and
the pse-arrow binding. This is focused foundation assessment integral to authoring, not a
new broad diagnostic audit. Current architecture sections remain authority for implemented
contracts; this plan owns proposed work and its adopted dispositions. Dependency updates
are ordinary technical choices; architecture/interface changes take their existing decision
route. No production implementation is authorized by creating these documents.

### Operator selections and limits

- The operator authorized a hard design pivot using static library assessment as the design
  premise, including newly considered integrations. Probes address concrete risks rather
  than gate every useful feature. Design selection is distinct from implementation acceptance.
- On 2026-10-21 the operator explicitly selected SurrealMX 1.0: “Whatever integration it takes
  for 1.0, we should pursue that.” The version requirement in SurrealDB's manifest is not
  evidence of API incompatibility. Observed adapter differences are implementation work.
- Preserve all 16 durable lanes, independent participants, caller-independent work and no
  new execution daemon. SurrealDB remains canonical; payload/cache formats do not become
  another commit/lifecycle authority. The existing shared managed **database server** remains.
- Preserve physical types, guard creation before algebra, validity/envelope meaning,
  library-owned derivatives, conservation, original-space outcome qualification, native drain
  and value rebinding. Legitimate vendored integration hooks are retained or migrated.
- RC01–RC21 are explicitly accepted target boundaries, recorded below. Acceptance fixes
  these design choices; it does not establish implementation, complete any scientific
  qualification or bypass required ADR/design publication before dependent changes.

## Remaining-scope reassessment — 2026-10-22

The maintainer authorized the combined remaining-plan and library opportunity review and
coordinated **proposed** revisions. The
[2026-10-22 review](../functional_reviews/plan49-remaining-scope-and-library-integration-2026-10-22.md)
is the current design-improvement input alongside the original review/audit. It inspected
clean source `fa7868336`, complete SurrealDB/Salsa/Z3 overviews and actual consumers, with
independent library research and architecture challenge. Its O01–O08 are opportunities,
not renamed DA/F findings. Source findings and historical receipts retain their identities.

**Implemented baseline:** the pause handoff records typed resolved production/lowering,
regional math, result statistics/pruning and locator retirement on substrate v8, shared
runtime requests/supervision, strict physical Python documents, covariance admission,
compiled catalogue and generated maintenance. The earlier authoring-baseline paragraphs
below and in companions describe their original date, not work presumed still unwritten.
Remaining consumer retirement, acceptance and enduring publication remain open.

**Reassessment proposals at the review baseline:** selected 49a–49c work is now implemented;
the [execution Outcome](#outcome--remaining-49a49c-2026-10-22) records its acceptance. Other
companions keep their existing package states.

- STO09 composes bounded study scope acquisition and revision-fenced repair pages inside
  SurrealDB; Rust retains scientific policy. Bound total encoded premises, preserve per-item
  progress and notification/cursor fences, and reconcile unknown effects.
- RES07 joins protected block/batch metadata and retains one selection-local DataFusion
  physical expression/pruning predicate across pages. Physical file authentication and full
  decoder admission remain; a metadata join cannot erase their premises.
- PRE01 first updates exact Salsa 0.28.5 and migrates memory inspection to exclusive lane
  accounting with an owned observation. PRE08 makes typed entity fragments and graph facts
  incrementally derived, retaining complete original root obligations and body reuse.
- PRE09 is a bounded configuration/explanation decision, comparing Z3 with finite enumeration;
  TOOL08 adds bounded typed blocker/source/consumer graph explanations using existing graphs.
  Neither installs a new runtime policy engine, graph service or execution daemon.
- QUAL01–QUAL05 consume these changes through representative contract tests. The canceled
  native sweep stays incomplete; no product campaign or benchmark resumed for this review.

At that review checkpoint the new package rows remained **proposed**. STO01–STO08 evidence
retains its original scope. Reopening 49a is limited to STO09 operation composition, not a rerun of the
completed engine/transport integration. Regeneration remains the selected data transition.

### Rule decisions from the reassessment

RC01–RC21 remain accepted as recorded. No reversal is required for bounded batching, native
selection-plan reuse, the Salsa patch or pure typed fragments. RC22 was explicitly selected
on 2026-10-22; RC23 remains unselected following its no-adoption recommendation:

| Rule | Proposed consequence | Decision / route | If not selected |
| --- | --- | --- | --- |
| RC22 | Execute the same Rust study policy in a pinned Surrealism module within a database transaction; change execution/deployment/capability contracts at blueprint §19.3/§20 | accepted by maintainer, 2026-10-22; proposed ADR-0214 records the I26/GOV02 module, failure, resource and activation contract before implementation | STO09 uses batched scope retrieval, pure Rust decisions and guarded application |
| RC23 | Add authored finite configuration exploration and attributed conflict/Unknown results, potentially with a Z3 binding and new public/replay contract | pending; PRE09/I27/GOV02 presents finite vocabulary, alternatives, native identity and evidence before target adoption | Existing concrete requests/routing remain; no configuration feature or SMT dependency is added |

RC22 is selected implementation scope as of 2026-10-22. RC23 remains an unselected proposal;
its inquiry does not authorize a new configuration policy or public request shape. A third-party patch
update alone needs no ADR. A selected new binding/operation or architecture change follows
its existing ADR/design route before dependent implementation.

### Reassessment sequencing before 49a–49c execution

This records the sequencing chosen at reassessment. Completed 49a–49c work and selected
acceptance are reconciled in the Outcome below; the package table and 49d–49h companions
own the actual remaining series work.

1. Reconcile retained-state disposition and representative fixture changes at their existing
   owners; preserve unknown effects. PRE01's Salsa patch/accounting can proceed independently
   of store state and PRE08. Do not repeat the canceled native sweep.
2. Define STO09 complete scope-page/apply contracts and RES07 joined metadata/native-plan
   contracts against actual consumers. Registry/generator changes and client cutover form
   coherent units with targeted controls and immediate retirement of replaced per-item paths.
3. Complete PRE04's residual producer/caller retirement while PRE08 establishes entity-local
   premises, immutable fragments and root import/assembly. MAT keeps consuming immutable
   region products; existing body reuse and original scientific checks remain.
4. Finish RUN/BND/TOOL consumer migration and existing required acceptance. TOOL08 can use
   the established native graph contracts without waiting for PRE08. PRE09/I26 are bounded
   optional decisions and do not block settled work or create a second production path.
5. Reconcile QUAL01 to actual changes, select the remaining representative assembled journeys,
   then scope-end checks/publication. Reuse applicable evidence only at its original scope.

The review/documentation engagement left production execution paused. Its handoff was
this section together with the exact earlier [pause evidence](#pause-handoff--requested-closeout-2026-10-21).
No service, dependency, product code or accepted architectural record changed in the review.

**Authoring checkpoint:** the integrated review and coordinated companion revisions are complete
for this engagement. Independent challenge corrections to bounded progress, descriptor coverage,
fragment/root acceptance and historical wording are incorporated. **Tested, documentation only:**
`just turn-end`, documentation publication with `just docs`, and lifecycle validation completed
against the zero-error baseline; these establish document integration, not product behavior.
The earlier unsupported-profile cleanup notices remain at their existing retained-state owner.
At that authoring checkpoint, conditional RC22/RC23 decisions and all new implementation/qualification remained open.

## Execution resumption — 49a–49c, 2026-10-22

The maintainer authorized implementation of the detailed remaining 49a–49c execution
approach. STO09, RES07 and PRE08 are now execution scope alongside the remaining acceptance,
consumer migration and retirement in those companions. Other 49d–49h obligations retain their
existing owners; necessary consumer edits do not close those whole companions.

The maintainer selected investigation followed by recommendations for I26/RC22 and PRE09/RC23,
with no optional feature implementation before a subsequent explicit selection. Their bounded
investigations are complete at the linked inquiry owners. I26 recommended a same-source Surrealism policy pivot for the concrete STO09 readiness
consumer. The maintainer explicitly selected RC22 on 2026-10-22. Its activation and acceptance
contract is now execution scope under proposed ADR-0214; batched external policy is the
intermediate implementation to retire after the in-transaction replacement qualifies. PRE09 recommends retaining existing concrete-request
routing without a configuration feature or Z3 binding for its bounded supplied-profile
question. These conclusions do not block the selected production work.

**Current checkpoint:** the selected Surrealism readiness cutover, joined protected result
pages/selection-local native plans, direct resolved preparation and entity-local Salsa fragments
are integrated. External scientific-snapshot scope/apply, public legacy block paging, scalar
row locators and rendered-body reconstruction are retired. Finite retention is 131,072 entries
and 2 GiB of known retained allocations, with explicit shared-owner accounting. The
[execution Outcome](#outcome--remaining-49a49c-2026-10-22) owns selected functional/native/Python
acceptance, measured conditions, corrected failures and final qualification limits. PRE09 is
complete as an inquiry recommending no new feature or Z3 binding; RC23 remains unselected.

The earlier `surreal-plan49-closeout-20261022` service was quiesced and then stopped through
its authenticated immutable supervisor's abrupt-stop route after graceful stop correctly
refused unresolved fixture creations. Actual server process/cgroup drain and storage allocation
release completed. Its data and all eleven incomplete creation records remain preserved,
pinned and unresolved; no absence or successful outcome was inferred. The replacement uses
normal setup in the distinct private state `surreal-plan49-surrealism-20261022`, on the
released functional-store slot. Its final fresh instance is active, authenticated and open,
with only the production module selected. A failed pre-bind launch after supervisor readmission is settled through
authenticated archived-source/configuration evidence and actual kernel drain; changed or
unknown histories still refuse. Its lifecycle controls passed 63 targeted tests. No third
concurrent server or borrower-record rewrite is
selected. The older poisoned deployment remains separately retained.

The preparation, readiness and server/module contributions are integrated in main. No
whole-product qualification or speed/memory improvement is claimed. Remaining 49d–49h work
retains its owners and is not closed by the selected 49a–49c acceptance.

## Outcome — remaining 49a–49c, 2026-10-22

### What was built

**Implemented:** STO09 and the remaining RES/PRE packages implement the selected target.
The sole production `study::readiness` export runs the existing Rust policy inside bounded
SurrealDB transactions. Complete output admission precedes writes; immutable original intent/
epoch receipts distinguish accepted repair, committed policy refusal and unknown effect.
Claims still check fresh authoritative facts. External scope/apply and scientific-snapshot
transport are removed. The pinned source-built SurrealDB 3.3.4/MX 1.0.0 server adds actual global
controller/single-memory ownership, acquisition/cold-initialization deadlines and cancellation
checks; these guarantees require source integration beyond the dependency change.

**Implemented:** joined protected result pages supply the exact block/descriptor population.
Each selection retains one native physical predicate/pruning plan across pages. Typed,
snapshot and SQL consumers preserve complete physical/decoded admission, original coordinates,
conservative null/IEEE pruning and residual-before-limit. Scalar locators, the superseded
public block-page route and the unused native file-cache adapter are removed. Actual analysis/
preparation caches and permission-bound IPC readers remain.

**Implemented:** direct resolved physical bodies and declaration-generated handles/roles feed
library lowering without rendered-name reconstruction. Query-owned entity fragments cover
supported producers; complete root connection, contribution, conservation and closure checks
remain. Exact Salsa 0.28.5 supplies exclusive inspection with an owned nullable observation.
Canonical shared-allocation census, structural/observation demand separation and finite
131,072-entry/2GiB known-retention defaults preserve the complete reference population.
Mutable/historical lanes remain detached; no additional compiler persistence or flight adapter
is adopted. PRE09 closes as an inquiry recommending no new configuration feature or Z3 binding;
RC23 remains unselected.

**Tested:** these selected Linux checks use the pinned toolchain, explicit force-validation
for Arrow-consuming correctness closures and a **zero failure/warning baseline**. Selections
overlap; counts are not a grand total. Earlier STO01–08 evidence retains its original scope.

| Command / mode / selected scope | Actual result and conditions |
| --- | --- |
| `just unit-package pse-relations 'test(result_batch::tests::)' --test-threads 1` | 6 passed, zero failed; 0.985s, invocation `8c6387817e6d4ce8a1c3da4195eb14de`. All four key shapes, admitted sorting/IEEE values, empty/null/nonfinite facts, schema/digest/interpretation binding, uniqueness and refusal/cancellation allowances. |
| `just unit-package pse-operations 'test(canonical::metadata::tests::) \| test(canonical_results_blocks_narrow_ranges_and_payload_receipts) \| test(multipage_result_read_hands_protection_to_analysis_before_retirement)' --features pse-operations/canonical-tests,pse-operations/canonical-maintenance --test-threads 1`, current production store | 4 passed, zero failed; 36.917s, invocation `95843f8606c94e6c9636b7e03ef34cb9`, complete owned drain. Missing/duplicate/extra/foreign descriptors, identity/ordinal/digest/interpretation/cursor/short-page/exhaustion controls, bounded 64+64-member/4MiB wire layout and multipage protection-to-analysis/retirement. |
| `just unit-package pse-compiler 'all()' -p pse-modeling -p pse-model -p pse-quantity -p pse-study-module` | 738 passed, zero skipped/failed; 54.796s. Resolved identity, fragments, physical premises, dependency completeness, pure policy and retention/backdating. |
| `just unit-native-capability-package pse-runtime 'native-solvers,canonical-tests' 'solver,klu,isolation,uno,petsc' <PRE07 selection> -p pse-compiler -p pse-backend-native` | 51 passed, zero failed; 13.697s test execution. Exact frozen selector/XML/terminal: invocation `29e663bfe34648499906e195b1dfbf77`; native provenance/selection `build/native-tests/1792670299036298025/`. Independent physical/derivative/conservation checks, incremental/fresh preparation, native provider callbacks, Ipopt/POUNCE/KINSOL/Diffsol/IDAS and actual private-child A/B/A reconstruction/reap. |
| `just unit-package pse-operations` with canonical-tests, selected native readiness/receipt filters and one test thread | 15 passed, zero failed; 139.578s, run `4190b705`. Full-u64 policy equivalence, complete-output rollback, fresh facts, immutable receipt/prefix/restart/epoch fences, notification/recovery and cancellation before/after commit. |
| Actual short-module fault and queued-acquisition controls | Fault rollback/recovery and pool 4 saturation/original100ms queue expiry passed: trap, strict 1s busy timeout and memory refusal leave no late canary commit. These are the passing two of the initial four-control run; capacity/cancellation fixture corrections are recorded below. |
| `just unit-package pse-operations 'test(canonical_studies_surrealism_qualification_global_capacity_) \| test(study_module_receipt_qualification_variants_) \| test(canonical_studies_surrealism_open_refuses_)' --features pse-operations/canonical-tests --test-threads 1`, ordinary canonical observer | 3 passed, zero failed; 64.743s. Invocation `acf986f487b4470d857856edf597f195`. Sixteen actual 5s guest intervals cover the entire extra-request submission/refusal interval through cache eviction; typed capacity refusal and post-drain reuse pass. Public open refuses removed module/tampered read-only backend and succeeds after exact restoration. Exact short/long receipt variants reject mixed inputs. |
| `just unit-native-package pse-runtime '<three result_pruning_tests plus five connected/durable/study controls>' --features native-solvers,canonical-tests --test-threads 1` | 8 passed, zero failed; 108.152s. Exact selector/XML/terminal: invocation `7f0dbec85640431ab34b2a7da79c39fc`; native provenance `build/native-tests/1792672053562403792/`. One physical plan over pages/unknown facts, residual-before-limit, protected schema/decode skipping, original IEEE coordinates, escaped Arrow retirement, durable reopen, retired seed refusal and single physical study admission. Bounded page/three-row/one-occurrence fixtures. |
| `just unit-package pse-codegen 'test(codegen::documents::tests::)'` | 14 passed, zero failed. Generated standalone documents use shared mechanical identity conversion without importing the public orchestration codec. |
| `python -m unittest scripts.tests.test_surrealism_module scripts.tests.test_surreal_server scripts.tests.test_host_admission` | 312 passed, zero failed; 32.2s. Packaging, source/profile binding, activation/deadlines/capacity and authenticated historical-launch settlement. |
| `just py-sync-native-correctness`; `just py-unit` on document hooks, standalone wire, generated contracts/edit codec, `-n 0` and task-private basetemp | Extension/stub validation passed; 52 Python controls passed, zero failed, 1.04s. Initial global pytest-temp ownership error was environmental; private basetemp rerun and `just ready` passed without changing global temp ownership. |
| `just native-python build/plan49-native-python-final-20261022 -m 'unit or component or integration' -n 0`, seven selected result/lifetime/review/reload/Arrow node IDs, private basetemp | 6 passed, 1 failed, 86.96s. The failed canonical-result fixture omitted required standalone document headers. After repair its exact integration node passed in `build/plan49-native-python-result-repair-20261022`, 74.32s. Composite acceptance is six unchanged passes plus one repaired pass, not a clean seven-test rerun. |
| Force-validated `pse-canonical-recovery <selected-state> --seed-signal-results --signal-results-receipt <receipt>` | Passed. Only two one-row files are created; exact joined batch identity/statistics, consecutive ordinals, distinct file instances and page exhaustion are checked. Protection release and connection disconnect complete before return. Receipt `/tmp/plan49-seed-signal-results-repaired-20261022.json`; no recovery/crash/backup campaign runs. |
| `python -m pse.tests.canonical_signal_control --state <selected-state> --fixture-receipt <two-file receipt> --exclusive-owned-instance 0a7d4aa4-fee0-429a-a95f-fa224a749dc7 --spill-directory build/plan49-final-signal-pull-spill`, actual installed force-validated native extension | Passed; `/tmp/plan49-final-signal-pull.json`. Actual stopped-server owned-response boundary, original KeyboardInterrupt/cause, reader discard, retained grant until resume, joined pull, exact ticket release/result collection and escaped one-row Arrow ownership. The owned server is resumed; no unrelated service is signaled. |

Selected clients/native children drained through their actual owners. The three ordinary
operations controls record runner-owned drain as unobserved; the later authenticated server
stop/drain and restart supply separate physical evidence. The final eight-test native run
records complete owned drain. Required XML, terminal records and native provenance remain
outside worker worktrees.

**Tested, deployment handoff:** final fresh instance `0a7d4aa4-fee0-429a-a95f-fa224a749dc7`
in `surreal-plan49-surrealism-20261022` is authenticated/open with production-only module
SHA256 `f13446a6bf52aff8835b160d82367e67dd9445cec35997582dfbc701951b8282`.
Final initialization/start and authenticated quiesce/stop/drain/same-instance restart passed.
The actual process selects 16 global controllers and 1000ms execution ceiling. Production
retains 64MiB/pool 4/strict 1s/KV0, compilation cache 4; qualification exports are absent.
Server/MX envelopes are 8/4GiB with one 32GiB native worker in 48GiB total; this does not qualify
a new sixteen-native-worker deployment. The admitted executable SHA256 is
`cdc3d974dfe69303161201b73d1b8a68b5e9f59c1bb98f57621c3efbeafcb749`.

**Measured, current-target cost point:**
`just case-measure build/plan49-results-publication-measurement-20261022 --case p44-results-publish-chain-80-5 --cargo-profile dev`
passed with no source/environment changes during sampling. Eighty states, five trajectory
samples, one lane, three selections and no padding exercise admission/source publication,
simulation/result publication and dense/selected/graph reads with existing analytic, identity,
row-count and digest assertions. Ten flat Criterion samples report 33.558s mean (reported
interval 33.418–33.737s); runtime/fixture setup and explicit teardown are untimed. Compilation
is untimed, but cooperating/external compiler load was observed; filesystem cache/isolation
are uncontrolled. This is not a before/after speedup, isolated latency bound or broad campaign.
The report separates scientific pool reservations from process RSS; retained-runtime teardown
returns owned accounting to zero.

**Tested, retention accounting:** current ThreeUnits retains 126 canonical bodies covering 845
actual instances and 856 original rows. First/warm known retained allocation counts are
1,426,036,749 bytes, rising to 1,453,825,251 after unrelated publication, with 56,663 entries,
generation zero and all 126 body Arcs/backdating results retained. These are owned accounting
observations, not RSS or memory savings. Historical 143 rendered bodies are not a required
current canonical partition or proof of a particular 17-body improvement.

**Scope-end static/manual verification:** initial full `just hygiene` exposed seven failing
leaves, repaired structurally: ADR evidence metadata, SPDX/REUSE, Ruff, Pyrefly, Python import
direction, default Clippy and no-default Clippy. Full `just clippy` subsequently passed both
default/no-default all-target modes; `just lint-solver-contracts` passed the full linked-native
all-target graph. Global Pyrefly reports zero errors (three existing suppressions), Ruff passes,
four import contracts have zero broken contracts, REUSE covers 11,775 files and Rust documentation
passes. Generator publication and final `just governance` pass include complete codegen
comparison/family checks. Focused `just check-package pse-runtime --features pse-runtime/canonical-tests`
also passes with zero warnings without native features, after matching two helper cfgs to
their solver callers. The repaired recovery fixture's exact `canonical-tools,canonical-tests`
force-validated bin compile/build and all-target Clippy also pass with zero warnings/errors.
Final global Ruff passes after the Python fixture correction. Documentation
publication passes with 590 chapters; lifecycle validation passes with zero errors over 1,011
scope items and five declared exclusions. `just turn-end` passes ADR index/formatting.
`just features-powerset` passes all **420 depth-two feature combinations** and **34 no-default
package checks**, with zero rendered warnings/errors. Its assessment is
`build/assessment/20261022T123431.111710Z-features-powerset-1223236-34e320/`; this includes fresh
post-repair coverage of every tooling combination. Independent parallel cache-warmup subsets
overlap that matrix and are not additional coverage. Two optional singleton-filter warmups
were refused by cargo-hack before any checks; the supported combined filter then passed.
Repaired leaf passes do not imply a fresh monolithic hygiene pass. Old unsupported-profile
cleanup deferrals retain their original owners and unresolved effects; a successful check
does not erase them. Selected 49a–49c implementation, acceptance and publication are complete;
remaining series work stays at 49d–49h.

### A mistake made and corrected

Broader preparation tests exposed nine exchange/temporal premise regressions. Repairs preserve
actual PairedExchange endpoints, distinguish logical temporal parameter owners from actual
state replicas, and retain shared-parameter agreement and synthetic policy-axis arity. The
final 738-control pass includes added cached regressions; matching two representations was
not accepted as an independent scientific oracle.

The initial 500ms capacity hold did not certify overlap because cold initialization could
reach acquisition after that window. A separate exact-input-bound qualification artifact uses
a 5s cooperative hold and 6s manifest ceiling. Its observed intervals certify all 16 occupied
controllers throughout the typed refusal; production keeps its 1s manifest. Local cancellation
also cannot establish absence of dispatched SDK work: revised controls prove original receipt
settlement on both sides of commit. Duplicate fixture identities and missing Python document
headers were repaired without weakening production validation. Earlier failures remain failed
attempts, including initial enclosing-clock initialization failures before normal initialization
of the exact final target passed.

The independent tooling feature matrix exposed a remaining signal-control fixture caller
missed by the earlier source census, which omitted `xtask`. Four feature combinations failed
with the same absent statistics field/retired API root cause. The fixture now admits actual
batch facts and consumes the joined protected page, preserving its original distinct-file/
ordinal assertions and adding exact identity/statistics/exhaustion checks. The expanded census
includes `xtask` and finds no old public page callers. The exact compiled/Clippy graph, bounded
CLI and current entered-pull SIGINT control pass; the full matrix records post-repair coverage
separately. Earlier matrix failures are not relabeled successful.

### Deviations from the plan, deliberate

Selected RC22 executes the same Rust policy through Surrealism rather than retaining the
review's intermediate external scope/apply design. No SQL policy copy survives. Source
integration supplies strict controller ownership/deadline behavior. Cold JIT and synchronous
host sections are finite but not instantly preemptible; cancellation is observed at bounded
return/epoch/cooperative yield and dispatched effects are reconciled.

The complete reference live/backdating population exceeded provisional 1GiB accounting even
after eliminating over-broad premises. The selected default is finite 2GiB, with entry/LRU
limits retained and actual shared-owner accounting. No process-wide saving is claimed.
Generated data is regenerated without a compatibility importer or fallback representation.

Scope-owned findings are reconciled below; shared runtime, mathematical, boundary and tooling
findings stay open at remaining owners. The canceled production-configuration native sweep
stays canceled. No full scientific-model library, IDAES parity, hardware power-loss, new-platform,
release/distribution or controlled performance-improvement qualification is claimed. Unknown
creation records/older poisoned state remain preserved: neither process absence nor the new
deployment settles them. ADR-0214/0215 remain formally proposed. Downstream active readers
still consume 49a–49c design, so these completed companions remain retained under ADR-0096.

Preparation, readiness and tooling branches were verified patch-integrated, their ancestry
recorded in merge `1829e2fe3`, and all three clean worktrees/merged branches removed. Required
receipts and deployed immutable artifacts remain outside those worktrees. The main checkout
is the continuing owner. Remaining 49d–49h packages retain their current scope and status;
this selected execution does not close or resume their broader campaigns.

## Execution resumption — remaining 49d–49e, 2026-10-22

The maintainer authorized the conversational execution approach for all remaining MAT01–MAT06
and RUN01–RUN10 scope at clean HEAD `8dcaf5911`. Completed 49a–49c Outcomes supply the current
preparation, protected-result and same-source Surrealism readiness prerequisites. Historical
external scope/apply wording is not a new implementation obligation. The principal mathematical,
request, policy and native seams are already implemented; this continuation owns concrete repairs,
consumer/retirement acceptance, composed scientific and lifecycle qualification, and publication.

Three disjoint execution assignments cover mathematical products/compiler consumers,
runtime/host contracts, and native adapters/retained-worker custody. The coordinator owns
cross-boundary integration, live-store and actual sixteen-native-owner acceptance, package/finding
dispositions, documentation and discrete commits. Existing Direct/Reverse/Regional product routes
retain their deterministic eligibility; settled optional inquiries are not reopened. Build commands
retain the configured sixteen jobs and unbounded ordinary build mode; scientific deployment
envelopes and ownership remain explicit. The canceled broad native sweep stays canceled.

The Outcome below records completed MAT01–MAT06 and RUN01–RUN10 acceptance. Independent
49f–49h work remains open; this continuation establishes no comparative performance gain.

### Current continuation checkpoint

All remaining 49d–49e packages are complete for their selected contracts and acceptance. The
[Outcome](#outcome--remaining-49d49e-2026-10-22) records constructor/projection repairs, complete
explicit fitting policies, original-clock read retry, charged terminal phases and actual
Rust/native/Python consumers. The final reference selection passes all six controls with normal
physical drain; its publication test includes retirement's first returned cleanup page while
retaining exact two-attempt retirement, independent graph and escaped Arrow assertions.

Earlier failed runs retain their actual results. Only the maintainer-selected obsolete generated
invocation/store lifetimes were discarded; unknown logical effects were not promoted to success.
The normal selected Surrealism deployment remains the handoff target. The package/finding tables
below retain independent 49f–49h scope; no canceled campaign is resumed by this closeout.

## Reading route and ownership

| Document | Local design and packages |
| --- | --- |
| [49a: SurrealMX canonical store](49a-surrealmx-canonical-store.md) | STO01–STO09: server/engine/transport integration, typed lifecycle, graph access, schema lifecycle and coordinated cutover |
| [49b: Protected results and selection](49b-protected-results-and-selection.md) | RES01–RES07: exact batch summaries, predicate lowering, protected access and locator retirement |
| [49c: Resolved model and incremental preparation](49c-resolved-model-and-incremental-preparation.md) | PRE01–PRE09: bounded retention, generated handles, resolved identities, indexes and scientific policy products |
| [49d: Symbolica mathematical execution](49d-symbolica-mathematical-execution.md) | MAT01–MAT06: joint demanded arithmetic, derivative composition, library execution and bounded mathematical inquiries |
| [49e: Runtime and native execution](49e-runtime-and-native-execution.md) | RUN01–RUN10: cancellation, shared requests, parameter studies, seam capability, numerical statuses and native lifetime |
| [49f: Python and scientific boundaries](49f-python-and-scientific-boundaries.md) | BND01–BND05: generated vocabulary, physical Arrow custody and covariance provenance/PSD |
| [49g: Contracts, maintenance and tooling](49g-contracts-maintenance-and-tooling.md) | TOOL01–TOOL08: catalogue, conformance/launch consumers, generated operator functions and dependency split |
| [49h: Qualification and publication](49h-qualification-and-design-publication.md) | QUAL01–QUAL05: S01–S11, selected MX/gRPC faults, scientific/boundary integration and scope-end qualification |
| [Source coverage appendix](49-source-coverage.md) | Navigation from recommendations, candidates and all retained artifact families; no duplicate live ledger |

This coordinator alone owns package state, findings, inquiry outcomes and rule decisions.
Companions explain design and acceptance; their package tables have no competing status.
An implementation package can migrate several semantic consumers without taking ownership
of their contracts. Root integration owns shared registry declarations, manifests, codegen,
activation and combined acceptance. Parallel authors/executors coordinate editing surfaces.

Plan 28 retains its E3/E4/E5 and other actual scope; Plan 32 owns document lifecycle conventions;
Plan 40 retains diagnostics qualification. Completed Plan 37/48 supply current context and
decision rationale, not a backlog. This series does not silently close or restart them.

## Combined target and foundation assessment

### Canonical execution: shared server, SurrealMX 1.0 and gRPC

Build the managed server from pinned SurrealDB 3.3.4 source with a narrow MX 1.0 integration,
and move the application SDK/parser family to 3.3.4. The [tagged upstream manifest](https://raw.githubusercontent.com/surrealdb/surrealdb/v3.3.4/Cargo.toml)
declares `surrealmx = "0.27.0"`: a Cargo compatibility range, not an exact `=` pin or an
unbounded minimum. Selecting 1.0 requires a declaration/resolution change. It does not prove
the adapter is incompatible. The inspected [tag comparison](https://github.com/surrealdb/surrealmx/compare/v0.27.0...v1.0.0)
shows read/cursor values changing from `Bytes` to `ByteSlice`; preserve their lifetime and
convert only at the KVS-owned value boundary. Inspected constructor, persistence and commit
interfaces retain their shape. The first implementation step changes the requirement and
builds with only conversions actually needed, preserving the adapter's explicit SI and
existing defaults/error fallback. A second step deliberately adopts SSI, finite controls,
new typed errors and useful engine statistics. gRPC migration is separate from both steps.
The lockfile is updated only for the selected family. See the official
[1.0 upgrade instructions](https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0).

Use memory-resident canonical data with synchronous append-only logging, fsync on every
append and periodic snapshots, under finite deployment-owned capacity. The
[3.3.4 memory adapter](https://raw.githubusercontent.com/surrealdb/surrealdb/v3.3.4/surrealdb/kvs-mem/src/lib.rs)
already wires persistent storage configuration. MX 1.0's memory/transaction-age options
must have explicit refusal and owner semantics. Default ephemeral `memory` is not the
production configuration. Historical `SELECT ... VERSION` is not promised; canonical
domain history, manifests and receipts still are. Server engine capacity includes pinned
versions; server overhead and client streams require their own finite owners.

The final proposed adapter explicitly requests serializable snapshot isolation. Standalone
SSI defaults already exist in both inspected releases; SurrealDB explicitly selects SI,
so a version bump alone does not change isolation. Retain lifecycle guards and qualify
the actual SurrealQL read/index/range paths and 16-lane retry behavior. Existing generated data is disposable under the maintainer's fresh-start decision.
No old MX log is imported or used as rollback input; newly created data retains recovery
and immutable receipt guarantees.

Select gRPC as the single target canonical request transport. Stock gRPC is not merely a
drop-in replacement for the patched WS client: capacity, per-operation context/deadline,
response charging and physical disconnect/drain need lawful integration hooks. Preserve
application/control admission separation, absolute deadlines, dispatch uncertainty and
immutable-receipt reconciliation. The SDK exposes `stream_items()`, but the current payload-drain-owned request path
explicitly refuses it; canonical `bounded_query` still buffers bounded results. Any adoption
must qualify aggregate ownership, late failure and physical drain first;
rows are provisional until statement and transaction success. Frame bounds are not
aggregate response bounds. Transport replacement must not change effect/replay permission.

Registry-generated tagged physical lifecycle/enum types, immutable-field constraints,
transition assertions and typed refusal codes remove parallel interpretation. Preserve
closed-awaiting-manifest and the complete u64 generation range; physical ingestion and
observational AttemptState remain deliberate projections. `FOR UPDATE` does not protect
predicate phantoms, so guard/sentinel rows remain. Pure endpoint associations can use
LIGHTWEIGHT relations; attributed/event-bearing edges use classic ENFORCED relations.
Bounded native inline adjacency replaces extra application caching where its actual edge
shape fits. Necessary commit-order sequencing remains; broad local pacing is investigated.

Select SurrealKit beta.6 for registry-generated schema artifact embedding, frozen rollouts
and schema-source parity tests. It supplies schema lifecycle machinery; generated guarded
functions still own domain maintenance. One fenced fresh initialization and activation replaces the
old store and regenerates data from current sources. There is one production
authority at each point, with no final RocksDB or WS alternate path. The old catalog/payload state is disposable; no compatibility importer or backfill is required.
The existing beta.6 configuration, rollout producer and exact maintenance settlement/
activation owners are foundations to adapt. Their actual clients join the gRPC migration;
the plan does not assume they are absent or already transport-compatible.

### Result execution: canonical blocks, library pruning and protected payloads

Append exact statistics over admitted batches on existing canonical block records, bound
to payload digest, schema and interpretation. The four scalar relation families have
declared keys. DataFusion `PruningPredicateBuilder` skips impossible batches; the canonical
predicate becomes a residual filter before limit, preserving null/NaN/infinity and original
row coordinates. Full protected decoder admission stays. All scalar row-locator writers,
readers, maintenance and tests retire together when the replacement controls pass.

Joined protected metadata pages improve operation shape; incremental SurrealDB streaming
is an unselected integration inquiry, not a property of current canonical reads. Bounded paging
does not move numerical rows into the canonical store. Parquet, projected decoding and membership summaries are bounded
follow-ups with distinct soundness/format questions. Native DataFusion file caches are
bound only at an actually reached source generation or their unused adapters are removed;
protected IPC is not a consumer of those caches.

### Semantic preparation and library mathematics

Use a project-owned resolved, identity-typed physical representation before library algebra.
It retains binder, instance, guard, premise, provenance and unit meaning. Generated relation
handles/membership remove string-policy inventories. Canonical local slots frame reusable
bodies; actual instance binding is separate. Changed body/derived identities have explicit
version/domain frames; rendered text and registration addresses are not identities.

Retain Salsa's granular queries and count LRU, with a finite allowance for admitted live
roots and actual metadata/input/interned/backdating retention. Correct defaults and the
catalogue firewall independently of the representation migration. No nonexistent public
value-holding counter or byte-weighted LRU is assumed. Dependency→consumer indexes include
non-expression rate nodes; current-identity lookups can precede resolved lowering.

Lower admitted arithmetic to Symbolica definitions/aliases and joint demanded output
regions, choosing symbolic local derivative products with correct library chain composition
across intermediate/provider/branch barriers. Deterministic preflight chooses the eligible
route before expensive differentiation; capacity and cancellation refuse rather than change
the artifact opportunistically. C++ compiled execution retains shared code with private
worker scratch. The interpreted evaluator has no public immutable-program/private-scratch
split. `InliningPolicy::Auto` currently equals `Always`; retained subevaluators cannot be
vectorized in 3.0.1. Alias expansion during construction does not supply earlier derivatives.

The broader Symbolica/Numerica ecosystem supplies exact algebra, transformers, coefficient
domains, precision and streaming opportunities. Selected uses and bounded inquiries are
specified in 49d, with no invented full interval certification, universal exact nonlinear
solver, custom arithmetic interpreter or mandatory device backend.

### Runtime, native seam and scientific boundary

One supervisor owns run lifecycle and inherited cancellation/native drain. Shared runtime
requests and preparation products replace host interpretation; host-only operations project
explicitly into the unchanged Version 7 study domain. Parameter-valued Simulation/Fit points
rebind prepared meaning with fresh IC/history rules. Explicit strategy operations retain a
documented lifecycle exception; no new durable strategy or Shooting study feature is added.

Compiler-produced annotation/view decisions and pure start policy feed all relevant consumers.
Settings-dependent native capability owns fixed-point/Picard maps and recovery suitability;
fresh native sessions and independent original residual qualification remain. Uno small-step
code 5 becomes Numerical, its four numerical hunks stay declared, and SQP-LBFGS remains.
FERAL estimates support pre-admission alongside post-factor enforcement; partial allocator
hooks do not erase foreign-envelope coverage. Actual retained rayon workers join before
their charge ends.

Generated enums, identity projection and physical keyed Arrow inputs replace hand-paired
vocabularies and positional side routes. Immutable input custody precedes asynchronous work.
Physical times retain signed/range/sub-microsecond semantics; timedelta is budget-specific.
Qualified covariance is hydrated or fully reconciled with actual fit content/validity.
External origin remains explicit. Independent faer PSD admission accepts singular matrices
and refuses indefinite inputs; a fit id alone certifies neither content nor statistics.

One direction-aware document catalogue feeds schemas, Python types and extension identity.
Conformance/launch/operator tools consume Rust meanings. Product observation/result-selection
contracts stay below tooling; internal module separation is selected before an optional
crate. Enduring architectural publication follows implemented results through confirmed
rule routes.

## Rule decisions

The operator explicitly accepted RC01–RC18, RC20 and RC21 on 2026-10-21 by selecting
“Accept all listed rule changes as written.” RC19 had already been explicitly selected.
GOV01 is complete for these 21 items. The retained-rule alternatives explain the choices;
they are not pending decisions. Acceptance does not select unknown inquiry winners or
authorize production execution. Accepted ADRs remain immutable; GOV02 supplies the
appropriate new/superseding ADR or design revision before implementation changes its contract.

| Rule | Accepted target and consequence | Decision / date | Route before dependent implementation | Retained-rule alternative (not selected) |
| --- | --- | --- | --- | --- |
| RC01 | Permit physically resolved identity-typed specialization, local-slot body framing and versioned derived identities | accepted, 2026-10-21 | GOV02; blueprint §§5.3/7.1, identity ADR route; PRE04/MAT | Keep current no-IR rule; redesign identity remedy before adoption |
| RC02 | Define demanded guarded arithmetic regions and correct symbolic/chain products; deterministic preflight, no whole-group Dualizer | accepted, 2026-10-21 | GOV02; blueprint §§7.1/14; MAT01–MAT05 | Retain wording, document a conforming regional interpretation before adopting it |
| RC03 | Finite live-root/backdating allowance plus actual bounded metadata/value retention and revised defaults | accepted, 2026-10-21 | GOV02; blueprint §§14.3.2/14.4; PRE01 | Current defaults still rotate the reference workload; no claim F09 is fixed |
| RC04 | Exact per-batch statistics and DataFusion residual/pruning replace per-row locators; full protected decoder stays | accepted, 2026-10-21 | GOV02; blueprint §§19.2/20 append contract; RES | Keep locators; predicate lowering alone does not deliver write-volume target |
| RC05 | Registry-generated lifecycle/enum constraints; transition-controlled mutable fences; record-id assertion/index retirement after readers migrate | accepted, 2026-10-21 | GOV02; registry/blueprint §20; STO lifecycle/graph | Keep text/flag physical schema; R2 remains independently useful |
| RC06 | Generated typed refusal codes replace prose conventions; error taxonomy itself retained | accepted, 2026-10-21 | GOV02; generator convention and §23.2 alignment; STO | Prose-dependent callers remain; cannot claim DA-008 resolved |
| RC07 | Settings-dependent capabilities, map representations/recovery and accurate Uno/PETSc matrix | accepted, 2026-10-21 | GOV02; blueprint §§18.1/18.7/18.9; RUN06 | Keep conservative seam and host route; no false extensibility claim |
| RC08 | Declare Uno integration versus four numerical hunks and dense SQP scope, preserving methods | accepted, 2026-10-21 | GOV02; proposed ADR-0157 route; RUN07 | Correct F02 mapping independently; numerical binding declaration remains open |
| RC09 | Keep product detail/selection contracts below tooling; select internal module split, optional crate only after separate evidence/ADR | accepted, 2026-10-21 | GOV02; blueprint §23.1; TOOL05/TOOL06 | Keep placement; do not introduce runtime→tooling cycle |
| RC10 | Describe the actual Simulation/Fit value-binding route, not nonexistent PreparedSimulation::rebind | accepted, 2026-10-21 | GOV02; blueprint §13.5; RUN04 | Correct current-state wording but do not claim new binding support |
| RC11 | Correct statrs hold explanation for egobox-moe feature unification | accepted, 2026-10-21 | TOOL07; ordinary manifest-note correction | Leave stale rationale explicitly unresolved; no family upgrade implied |
| RC12 | Shared request/preparation structs, host superset and typed study projection; Version 7 study domain preserved; admit parameter Fit/Simulation points | accepted, 2026-10-21 | GOV02; blueprint §19.3 and affected binding contracts; RUN03/RUN04 | Retain host duplication and current study restrictions; no silent Shooting variant |
| RC13 | State explicit conic/recycle/block-initialization strategy lifecycle exception; do not add durable strategy execution | accepted, 2026-10-21 | GOV02; blueprint §§17/19.2; RUN02 | Exception remains unstated; no accidental run/replay claim |
| RC14 | Physical keyed Arrow side inputs, identity-keyed tolerances and precision-preserving time; budget timedelta only where suitable | accepted, 2026-10-21 | GOV02; Python-boundary ADR/blueprint §21; BND02 | Bare/positional routes remain and DA-005 is not closed |
| RC15 | Direct KKT FERAL a-priori estimate with retained complete envelope/post-factor enforcement | accepted, 2026-10-21 | GOV02; blueprint §18.8/native binding route; RUN08 | Retain after-factor accounting and full envelope |
| RC16 | One compiler annotation/view product per mode and explicit pure start policy | accepted, 2026-10-21 | GOV02; blueprint §16.2 and view contracts; PRE05/RUN05 | Host/compiler policy copies remain; no single-owner claim |
| RC17 | Generated guarded operator domain mutations, explicitly separate privileged install/restore/activation | accepted, 2026-10-21 | GOV02; blueprint §20.6/ADR-0164 maintenance contract; STO/TOOL04 | Script remains domain writer; no single-owner claim |
| RC18 | Covariance content/validity reconciliation, explicit external origin and tolerance-governed PSD accepting singular inputs | accepted, 2026-10-21 | GOV02; blueprint §19.8/scientific input contract; BND03/BND04 | Do not label id-only or indefinite propagation certified |
| RC19 | Select SurrealMX 1.0 integration with 3.3.4 shared server and synchronous persistence, preserving lanes and caller independence | accepted, 2026-10-21 | GOV02 records selected storage binding/activation; STO01–STO03 and cutover controls | Not pending: operator explicitly selected integration, not optional comparison |
| RC20 | Replace patched WS with one gRPC transport carrying equivalent bounded admission/deadline/effect/response/drain semantics | accepted, 2026-10-21 | GOV02; supersede/reconcile retained WS target from Plan 37 and transport owner; STO | Retain lawful WS transport; MX 1.0 storage selection still stands |
| RC21 | After the minimal MX 1.0 version integration, explicitly select SSI in the adapter and preserve actual conflict/retry, guard and visibility contracts | accepted, 2026-10-21 | GOV02; blueprint §20 transaction contract and selected native storage binding; STO02 | Keep explicit SI with current guards; MX 1.0 upgrade, crash-safe persistence, bounds and statistics still proceed |

## Execution sequence and shared prerequisites

This is the original sequence and contract rationale. The dependency-ordered next work in
the 2026-10-22 reassessment above accounts for implemented prerequisites and proposed additions.

1. GOV01 has recorded the accepted rule decisions; GOV02 schedules concrete decision/design routes.
   RUN01 horizon lineage, RUN07's truthful code mapping and non-contract local investigation
   can proceed independently once implementation is separately authorized. PRE01 defaults/
   bounded-retention work does not depend on PRE04's resolved representation.
2. STO01 first establishes the smallest MX 1.0 upgrade with existing SI. STO02 deliberately
   adds SSI, finite controls, typed failures and statistics; it does not infer them from the
   dependency bump. STO storage integration and gRPC lifecycle design share capacity/effect contracts, not a
   requirement to deploy together before either can be tested. Typed refusals can precede
   lifecycle migration. Shared registry vocabulary supplies BND01; PRE02 handles supply
   RES declared-field access. TOOL01 catalogue has an early membership/direction contract
   and later complete boundary consumer adoption; do not make that distinction a cycle.
3. PRE03 current-identity indexes can precede PRE04. PRE04 supplies final body/derived
   identity and instance-binding contracts consumed by MAT and the resolved-index migration.
   MAT may develop over existing stage programs, retaining current chain propagation until
   the correct replacement is qualified. PRE05 policy products precede RUN05 consumer removal.
4. RUN02's paired finish interface precedes RUN03; RUN03 working request/preparation entry
   precedes RUN04 parameter studies. RUN06's working map seam precedes removing Python's
   KINSOL choice from the host migration. BND02 consumes the runtime's admitted-input custody
   boundary, not a new Python numerical authority.
5. STO schema/freeze/maintenance contracts precede TOOL04 script mutation cutover; RES block
   summaries and migrated payload readers precede locator deletion. Product selection/detail
   contracts precede TOOL05 extraction. All changed producers, consumers, fixtures and old
   paths move together as targeted replacement tests pass—no compatibility production path.
6. Test the actual MX/gRPC migration mechanism before activating the replacement. Coordinate
   freeze, drain, fresh initialization and one activation; regenerate current fixtures. Native
   and boundary migrations retain existing support until their actual replacements pass.
7. Once all selected functional packages and inquiry outcomes are integrated, run QUAL02–05
   assembled journeys and scope-end hygiene/manual checks. Measurements substantiate only
   quantitative claims. Publication does not convert unexamined scientific scope into a pass.

## Package state

Remaining rows distinguish implementation progress from package acceptance. Package
acceptance is explained at its companion owner. GOV01 records completed operator decisions;
GOV02 is the remaining coordinator decision/publication route.

| Package | Status | Owner | Required capability / completion |
| --- | --- | --- | --- |
| GOV01 | done | coordinator | Operator explicitly accepted RC01–RC21, 2026-10-21; no product implementation claimed |
| GOV02 | in-progress | coordinator | Proposed ADR-0214/0215 and blueprint239–246 publish selected targets and completed 49a–49e acceptance. Remaining 49f–49h publication stays open; formal ADR acceptance is separate. |
| STO01 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: pinned source-built3.3.4/MX1.0, owned adapter conversions and actual source units; store Outcome. |
| STO02 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: SSI, synchronous persistence/finite controls, authenticated native statistics, reader/fault controls; store Outcome. |
| STO03 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: generated v7 phases/constraints/refusals, migrated consumers, terminal manifest event and exact witnesses; store Outcome. |
| STO04 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: bounded gRPC/physical generations, original clocks/effects, control progress, reconnect and removed custom WS; store Outcome. |
| STO05 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: native records/ranges/graphs, key assertions before index retirement, nodes-first ENFORCED edges and bounded fallback; store Outcome. |
| STO06 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: commit-order paging and sixteen-lane progress. I01 retains necessary source/sequence pacing with explicit rationale; store Outcome. |
| STO07 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: fresh functional/reference profiles, frozen schema activation, interrupted initialization, new-data recovery/backup/restore; store Outcome. |
| STO08 | done | [49a](49a-surrealmx-canonical-store.md) | Implemented and Tested: selected assembled durability/native/Python/interruption journeys, repaired hygiene, full Clippy/manual/feature checks and installed handoff; store Outcome and limits. |
| STO09 | done | [49a](49a-surrealmx-canonical-store.md#sto09--bounded-study-operations) | Implemented and Tested: selected RC22 same-source read-only policy inside bounded native transactions, exact receipt settlement, immutable activation, live fault/capacity/catalog controls and external scope/apply retirement. Remaining-scope Outcome owns results and limits. |
| RES01 | done | [49b](49b-protected-results-and-selection.md) | Implemented and Tested: admitted batch statistics, exact schema/digest/interpretation binding and generated selection vocabulary. Remaining-scope Outcome. |
| RES02 | done | [49b](49b-protected-results-and-selection.md) | Implemented and Tested: DataFusion pruning/residual before limit; conservative unknown/IEEE semantics, protected admission and native-decode skipping. Remaining-scope Outcome. |
| RES03 | done | [49b](49b-protected-results-and-selection.md) | Implemented and Tested: scalar locators/consumers retired on v8; assembled Rust/Python SQL, protected result, review/export, reopen and retirement acceptance. Remaining-scope Outcome. |
| RES04 | superseded | [49b](49b-protected-results-and-selection.md) | Maintainer waived existing-data preservation; no historical backfill is required |
| RES05 | done | [49b](49b-protected-results-and-selection.md) | Implemented and Tested: unused native file-cache adapter removed; actual analysis/preparation and protected reader owners retained and exercised. I02. |
| RES06 | done | [49b](49b-protected-results-and-selection.md) | Tested: selected assembled Rust/Python/export/retirement journeys; bounded complete-operation measurement recorded. I03–I05/I16 retain explicit no-additional-adoption triggers; no comparative speed claim. |
| RES07 | done | [49b](49b-protected-results-and-selection.md#res07--joined-metadata-and-retained-native-selection) | Implemented and Tested: joined protected metadata with exact descriptor coverage and one selection-local native plan across pages; superseded public block paging retired. |
| PRE01 | done | [49c](49c-resolved-model-and-incremental-preparation.md) | Implemented and Tested: exact Salsa 0.28.5, exclusive nullable owned accounting, shared-owner census and finite production defaults; complete reference live/backdating population remains generation zero. |
| PRE02 | done | [49c](49c-resolved-model-and-incremental-preparation.md) | Implemented and Tested: declaration-generated handles/roles and semantic framing replace spelling-derived meaning. |
| PRE03 | done | [49c](49c-resolved-model-and-incremental-preparation.md) | Implemented and Tested: current-identity indexes and complete dependency/consumer graph, including rates and membership. |
| PRE04 | done | [49c](49c-resolved-model-and-incremental-preparation.md) | Implemented and Tested: direct resolved physical bodies/derived identities and all supported producer consumers; rendered reconstruction retired. |
| PRE05 | done | [49c](49c-resolved-model-and-incremental-preparation.md) | Implemented and Tested: tracked annotation/view/start products and structural versus observation demand; downstream RUN05 acceptance remains independently owned. |
| PRE06 | done | [49c](49c-resolved-model-and-incremental-preparation.md) | Interface-checked and Tested: key/pointer lifetime review, retained completion-owned Flights, detached mutable/historical lanes and shared census; no additional cache adapter or persistence. |
| PRE07 | done | [49c](49c-resolved-model-and-incremental-preparation.md) | Tested: 738 pure library controls plus 51 selected native/scientific/portable controls and public reload/ownership journeys; wider MAT/QUAL campaigns remain separate. |
| PRE08 | done | [49c](49c-resolved-model-and-incremental-preparation.md#pre08--typed-incremental-fragments) | Implemented and Tested: entity-local tracked premises and immutable fragments across supported producers; original root closure/contribution obligations and body backdating retained. |
| PRE09 | done — investigation only | [49c](49c-resolved-model-and-incremental-preparation.md#pre09--finite-configuration-exploration-decision) | Bounded operation, alternatives, unknown/limit semantics and no-adoption recommendation recorded at the linked owner; RC23 remains unselected. |
| MAT01 | done | [49d](49d-symbolica-mathematical-execution.md) | Implemented and Tested: Deterministic demanded-region/product preflight, complete identity and cancellation/refusal controls; 49d–49e Outcome. |
| MAT02 | done | [49d](49d-symbolica-mathematical-execution.md) | Implemented and Tested: Library definition/alias DAG and joint regions across source/host barriers; shared-let and actual consumer controls; 49d–49e Outcome. |
| MAT03 | done | [49d](49d-symbolica-mathematical-execution.md) | Implemented and Tested: All five local derivative products compose across provider/branch/intermediate boundaries; independent analytical/action checks; 49d–49e Outcome. |
| MAT04 | done | [49d](49d-symbolica-mathematical-execution.md) | Implemented and Tested: Interpreted/C++ consumers, actual AST/literal/exported census, worker clones and compilation/drain controls; 49d–49e Outcome. |
| MAT05 | done | [49d](49d-symbolica-mathematical-execution.md) | Implemented and Tested: Compiled guard/product/curvature controls and selected physical/native/public scientific acceptance; no whole-library/performance claim; 49d–49e Outcome. |
| MAT06 | done | [49d](49d-symbolica-mathematical-execution.md) | Interface-checked: I11/I12/I13/I25 bounded no-additional-adoption decisions and concrete revisit conditions; 49d Outcome. |
| RUN01 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: one supervisor preserves caller/durable cancellation, original deadlines, partial horizons and actual native drain; runtime and final reference controls at the 49d–49e Outcome. |
| RUN02 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: five sixteen-owner controls plus isolated primary death preserve caller independence, fenced cancel, actual stop/join, publication and protected retirement; explicit nondurable strategy exception remains. Outcome. |
| RUN03 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: shared typed requests/preparation, paired outcomes, semantic admission/deployment and host-only Shooting refusal; complete explicit-input LM/covariance policies with six Runtime-free controls and retained actual consumers. Outcome. |
| RUN04 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: parameter-valued Simulation/Fit points preserve physical conversions, recorded starts and fresh consistent ICs; runtime and selected public scientific acceptance. Other overlay/history additions are not adopted (I22). |
| RUN05 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: runtime consumes compiler-owned annotation/view/start products; deliberate mode differences and pure predicted-start/fallback controls remain. Outcome. |
| RUN06 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: settings-dependent capabilities, fixed-point/Picard and separately admitted recovery preserve scaling, authorized starts and original residual; actual linked/unlinked controls. Fresh sessions remain (I17). |
| RUN07 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: named Uno outcomes map code 5 to Numerical; declared SQP-LBFGS integration and four numerical hunks retain actual FFI, curvature, cancellation and allocation-refusal controls. Outcome. |
| RUN08 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: FERAL pre-factor estimate plus post-factor retained enforcement; complete opaque foreign envelopes remain. Partial hooks and redundant pool/sizing adapters are not adopted (I15/I16). |
| RUN09 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: independent LM criteria/profile identity and effective work caps retain original stationarity/covariance gates; native policy/scientific and current installed Python acceptance. I18 and Outcome. |
| RUN10 | done | [49e](49e-runtime-and-native-execution.md) | Implemented and Tested: actual retained worker handles join through stop and partial construction; fresh/preinitialized-host POUNCE/Symbolica controls preserve scoped ownership without resizing host global pools. Outcome. |
| BND01 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND02 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND03 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND04 | in-progress | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| BND05 | proposed | [49f](49f-python-and-scientific-boundaries.md) | Local package contract and acceptance at the linked owner |
| TOOL01 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL02 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL03 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL04 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL05 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL06 | done — retain hub and internal tooling module | [49g](49g-contracts-maintenance-and-tooling.md) | Two actual steady/fit and bounded-lifetime inquiry controls passed; no replacement substrate or separate crate selected. |
| TOOL07 | in-progress | [49g](49g-contracts-maintenance-and-tooling.md) | Local package contract and acceptance at the linked owner |
| TOOL08 | proposed | [49g](49g-contracts-maintenance-and-tooling.md#tool08--bounded-graph-explanations) | O07: typed immediate-blocker and bounded source/consumer explanations over current canonical graph identities, with explicit partiality. |
| QUAL01 | in-progress | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL02 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL03 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL04 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |
| QUAL05 | proposed | [49h](49h-qualification-and-design-publication.md) | Local package contract and acceptance at the linked owner |

## Finding dispositions

The following records current correction dispositions. Review “Addressed” and audit
“Retain” are not completion. Partial/challenged diagnoses retain their limits; the store
Outcome supplies the selected STO evidence without closing combined downstream obligations.

| Finding | Review scenarios | Disposition | Work owner | Required evidence or question |
| --- | --- | --- | --- | --- |
| DA-001 | S11 | resolved — Implemented and Tested | PRE04 | Direct resolved physical identity before algebra; 738 pure and 51 selected scientific/native controls at remaining-scope Outcome. |
| DA-002 | S11 | resolved — Implemented and Tested | PRE04 | Versioned derivation frames, declared relationships and old-frame refusal; remaining-scope Outcome. |
| DA-003 | S11 | resolved — Implemented and Tested | PRE02 / PRE04 | Generated grammar/role handles and direct typed producer; rendered/text reconstruction retired. Remaining-scope Outcome. |
| DA-004 | S04 | open — store portion implemented/tested | STO / BND01 | Store native phases/projections pass; complete shared boundary vocabulary remains BND01. See store Outcome. |
| DA-005 | S08 | open — proposed route | BND02 / I20 | Physical keyed side inputs; unit resolver is a separate project decision |
| DA-006 | S07 | resolved — Implemented and Tested | PRE05 / RUN05 | Compiler-owned annotation/view products and deliberate mode distinctions consumed by runtime; compiler/runtime and selected assembled acceptance at 49d–49e Outcome. |
| DA-007 | S04 | open — store portion implemented/tested | STO lifecycle / TOOL04 | Native transition/holder/release controls pass with race guards; broader generated operator migration remains TOOL04. |
| DA-008 | S05 | resolved — Implemented and Tested | STO typed refusals | Registry-native codes, typed exhaustion and distinct dispatch/cancel/replay/effect controls; unknown/forged errors fail closed. See store Outcome. |
| DA-009 | S10 | resolved — Implemented and Tested | RUN02 / RUN03 | Discriminant-paired requests/outcomes and typed host/study projection; request/refusal and actual consumer controls at 49d–49e Outcome. |
| DA-010 | S08 | open — proposed route | BND03 / BND04 | Hydrate/reconcile complete covariance; independent PSD admission |
| DA-011 | S04 | open — proposed route | TOOL01 | Direction-aware actual compiled catalogue and import identity |
| DA-012 | S03 | resolved — Implemented and Tested | RUN01 / RUN02 / RUN03 | Shared supervisor preserves actual run lifecycle and native drain; strategies remain explicitly nondurable. Final reference and runtime controls at 49d–49e Outcome. |
| DA-013 | S09 | resolved — Implemented and Tested | PRE05 / RUN05 | Pure predicted-start/fallback owner and complete runtime adoption; explicit-fact policy and actual consumer controls at 49d–49e Outcome. |
| DA-014 | S09 | resolved — Implemented and Tested | RUN04 / I22 | Selected parameter-valued Fit/Simulation points preserve conversions, starts and fresh ICs; other overlay/session-history additions remain unselected per I22. Outcome. |
| DA-015 | S10 | resolved — Implemented and Tested | RUN03 | Runtime-owned semantic admission, typed host projection and shared deployment sequence; ingress revalidation and per-host assertions remain. Outcome. |
| DA-016 | S07 | resolved — Implemented and Tested | RUN06 | Settings-dependent map/recovery seam with authorized starts, scaling and original residual; linked/unlinked actual routing controls. Outcome. |
| DA-017 | S07 | open — runtime portion Implemented and Tested | RUN06 / TOOL07 | Settings-dependent advertised capability equals actual admitted execution under linked/unlinked controls; broader documentation/tooling reconciliation remains TOOL07. 49d–49e Outcome. |
| DA-018 | S07 | resolved — Implemented and Tested | RUN07 / I23 | Declare and retain legitimate Uno integration plus four numerical hunks; named outcomes, SQP-LBFGS and actual FFI/refusal controls pass. I23 and Outcome; no forced method loss. |
| DA-019 | S04 | open — store portion implemented/tested | STO lifecycle / TOOL04 | Fresh installation/activation and exact event witnesses pass; broader generated guarded operator mutations remain TOOL04. Historical data preservation superseded. |
| DA-020 | S10 | open — proposed route | TOOL05 / I21 | Product contracts before tooling; optional tracing/crate outcome |
| DA-021 | S10 | resolved — Implemented and Tested | RUN03 | Complete LM eligibility and staged covariance withholding consume explicit actual inputs; six Runtime-free tables plus thirteen retained adapter/scientific controls pass. 49d–49e Outcome. |
| DA-022 | S01 | resolved — Implemented and Tested | PRE01 / RUN08 / I06 / I16 | Exclusive owned Salsa accounting/shared census and native FERAL pre/post enforcement exercised; complete opaque envelopes retained. Known allocation accounting is not RSS or complete allocator-hook coverage. Outcome. |
| DA-023 | S04 | open — proposed route | TOOL07 / QUAL05 | Documentation follows selected implemented changes through decision route |
| DA-024 | S10 | open — proposed route | TOOL07 / I24 | Purpose decision on native child and pse.canon.v2; no automatic deletion |
| DA-025 | S05 / S06 | open — store/result portions Implemented and Tested | STO / RES / TOOL04 | Necessary participants retained; scalar locator writes/collection removed, protected result retirement exercised. Broader operator migration remains TOOL04. |
| DA-026 | S06 | resolved — Implemented and Tested | RES01–RES06 | Exact batch summaries and conservative native pruning/residual replace scalar locator writes; selected assembled acceptance and current complete-operation measurement at remaining-scope Outcome, without comparative speed claim. |
| DA-027 | S11 | resolved — Implemented and Tested | PRE04 / MAT01–MAT05 | Direct resolved bodies and demanded library regions remove text re-expansion; five derivative products, interpreted/C++ and selected physical/public consumers pass. Outcome. |
| DA-028 | S11 | resolved — Implemented and Tested | MAT01–MAT05 / I11 | Guarded joint regional/alias realization and independent product/action checks pass; no universal expansion magnitude, speed or RSS claim. MAT Outcome and I11–I13/I25. |
| DA-029 | S01 | resolved — Implemented and Tested | PRE03 / PRE04 | Complete current dependency/consumer graph, including rates, guards and memberships; incremental versus fresh preparation controls pass. |
| DA-030 | S01 / S02 | resolved — Implemented and Tested | PRE01 / PRE06 | Production defaults preserve complete current ThreeUnits live/backdating population; pointer/key uses retain complete equality and live owners. No broad RSS or speed claim. |
| DA-031 | S07 | resolved — Implemented and Tested | PRE05 / RUN05 | Runtime consumes the compiler-owned view choice/product; compiler and actual consumer qualification at 49d–49e Outcome. |
| DA-032 | S06 / S11 | resolved — Implemented and Tested | PRE02 / RES | Declaration-generated handles/roles and complete membership replace spelling-derived interpretation; result and preparation controls at remaining-scope Outcome. |
| F01 | S03 | resolved — Implemented and Tested | RUN01 / RUN02 | Caller/durable cancellation, inherited typed TimeLimit, partial horizon and actual native join pass runtime and final reference controls. Outcome. |
| F02 | S07 | resolved — Implemented and Tested | RUN07 | All named Uno outcomes and code 5 Numerical mapping pass actual bridge/adapter controls; callback and cancellation overrides remain. Outcome. |
| F03 | S05 | resolved — Implemented and Tested | STO refusals / I01 | Typed retry exhaustion/fresh decision under original clock; commit-order cursor and sixteen-lane controls pass. Necessary sequence/source guards retained. See store Outcome. |
| F04 | S03 | resolved — Implemented and Tested | RUN10 / I14 | Actual retained worker JoinHandles survive destruction acknowledgement and join before release; partial-construction/stop and reached scoped-pool controls pass. I14 and Outcome. |
| F05 | S09 | resolved — Implemented and Tested | RUN09 / I18 | Independent LM cosine/cost/step criteria retain physical stationarity and covariance gates; revealing disagreement and actual public retained-result controls pass. I18 and Outcome. |
| F06 | S06 | resolved — Implemented and Tested | RES cache package / I02 | Unused native file-cache adapter removed; reached analysis/preparation and protected IPC reader ownership retained and exercised. No hit-rate claim. |
| F07 | S04 | resolved — Implemented and Tested | STO graph/access | Consumers use native records/ranges/graphs; direct CRUD key/id/u64 and access-plan controls precede redundant key-index removal. Engine scans remain distinct from page bounds. |
| F08 | S04 | open — proposed route | TOOL07 | Correct stale statrs note without pretending type-crossing defect |
| F09 | S01 / S02 | resolved — Implemented and Tested | PRE01 | Finite production-default live-set/backdating retention, churn/oversize and complete reference body coverage; remaining-scope Outcome. |
| F10 | S08 | open — proposed route | BND03 / BND04 | Content-sensitive fit provenance; indefinite refused and singular PSD admitted |

## Inquiry outcomes

Each question has a route and a revealing outcome. Pending adoption is not a second
implementation path; either the selected mechanism replaces its predecessor or it is not
adopted. A no-adoption conclusion can complete an inquiry without resolving an unrelated
finding. Technical library capability is judged on merit; licenses are not rejection reasons.

| Inquiry | State | Owner | Question and evidence | Adoption / no-adoption consequence |
| --- | --- | --- | --- | --- |
| I01 | resolved — retain necessary pacing | STO / QUAL03 | Actual sixteen-lane mixed work attributes queue/hold/conflicts and proves control progress; source creator/reclaimer and sequence commit dependencies remain. See store Outcome. | Keep ProblemTurns and shared source-creator/sequence guards. No safe excess-scope replacement established; no throughput improvement claimed. |
| I02 | resolved — remove unused adapter | RES / F06 | The reached protected result reader owns an instance-keyed IPC reader; no production DataFusion file source reaches the removed native file-cache adapter. The Parquet authoring importer uses a separate direct reader. | Retain active analysis/preparation caches. Revisit when a named DataFusion file-source consumer supplies immutable version/etag or owned generation and eviction; qualify replacement, cold/hit and cancellation there. |
| I03 | resolved — retain min/max | RES | Existing facts own min/max, null and row counts, not exact bounded multi-value membership sets. Native contained means all/none of the queried literals; unknown must remain unknown. | No parallel membership index. Revisit an already-owned exact bounded dictionary/set with proved complete coverage, null/NaN semantics and same-file binding. |
| I04 | resolved — retain full decoder | RES | A projected IPC reader omits fields and changes its schema; full footer schema alone does not admit omitted batch buffers or their semantics. | Keep full protected validation and batch admission, then engine projection/residuals. Revisit a measured wide payload with immutable prior full-batch admission, predicate/key closure and omitted-column corruption controls. |
| I05 | resolved — retain protected IPC | RES | Native Parquet page/row selection exists, but current protected result descriptors, checksums, footer validation and decoder consume IPC. Parquet authoring import is a separate boundary. | Revisit a reached workload with measured page/range benefit under full-file integrity, explicit format/interpretation decision, IEEE bit preservation, full admission, conservative residuals and finite decoded memory. No comparative speed claim. |
| I06 | resolved — retain shared census | PRE01 | Salsa 0.28.5 exclusive memory inspection supplies an owned observation but cannot deduplicate shared input/generation allocations by itself. | Keep the shared-Arc census and nullable observation; no false zero when inspection is unavailable. |
| I07 | resolved — typed fragments adopted | PRE08 / PRE06 | O05: query-owned local fragments cover supported producers; incremental versus clean preparation, complete contribution/closure and unchanged body reuse pass selected controls. | Retain complete root assembly and original scientific obligations. No affected-only global validation or measured speedup claim. |
| I08 | resolved — retain independent publication lanes | PRE06 | Salsa 0.28.5 storage-clone mutation cancels and waits; existing historical/current workspaces retain independent publication ownership. | Immutable snapshots may share reads; writable/historical lanes remain detached. No new public read-handle seam. |
| I09 | resolved — retain reached pure Flights | PRE06 | Existing exact-key flights retain producer clocks, independent waiter stop and actual exit; quick_cache0.6.24 directly awaits a supplied loader. | Retain the reached flight contract rather than add an adapter to recreate it. Native drain is never cache-flight completion. |
| I10 | resolved — no additional compiler persistence | PRE06 | Existing qualified artifact recipes restore reconstructible products with version/receipt readmission; no distinct reached cold-work need is established. | Keep those owners and in-process correction; add no persistent compiler cache or lifecycle authority. |
| I11 | resolved — retain scalar formals and aliases | MAT06 | Which function-shaped formal/retained definition path preserves cross-process identities, derivative expansion and compiled export? | Adopt the exact supported post-admission path; otherwise canonical scalar slots plus selected alias expansion remain. |
| I12 | resolved — retain scalar profiles | MAT06 | Does JIT/SIMD/CUDA improve an eligible repeated/batched workload with declared float, cancellation and scratch semantics? | Select and qualify a specific realization; otherwise retain current scalar interpreted/C++ eligibility, no custom interpreter. |
| I13 | resolved — retain scoped library algebra | MAT06 | Do exact solving, polynomial/coefficient domains, transformers or term streaming remove a concrete preparation burden? | Adopt scoped library work preserving coverage/domain/remainder and cleanup; otherwise keep current lowering and record conditions for reconsideration. |
| I14 | resolved — scoped teams and retained upstream setup | RUN10 | Actual fresh/preinitialized-host POUNCE/Symbolica controls preserve scoped teams and the embedding host pool; exact conditions and limits at the 49d–49e Outcome. | Control reached work under admitted shared owner or report incompatibility; do not eagerly create/pin a global pool without a consumer. |
| I15 | resolved — retain envelope | RUN08 | Can PETSc/KLU hooks retain original allocation owner/size across free/realloc and process initialization, with bypasses identified? | Use domain-specific accounting with remaining envelope; otherwise retain the complete foreign envelope. |
| I16 | resolved — retain admitted ownership and sizing | RES / RUN08 | Arrow claim tracks existing backing with an infallible reservation and replaces a shared per-buffer owner; it cannot replace fallible preflight or imported lease custody. Project HeapUsage and generated rows already account their actual structures; shared result facts retain one owner. | No additional pool or sizing crate. Revisit a concrete unsupported allocation/graph lifecycle with fallible admission, no double charge, last-alias custody and measured overhead; DataFusion structure sizing alone is not an admission proof. |
| I17 | resolved — fresh sessions | RUN06 | Does KINSOL neutral reset match independently fresh sessions including history and errors? | Adopt only qualified reset or record warm history in semantics; otherwise fresh sessions stay. |
| I18 | resolved — fixed independent criteria | RUN09 | Use the pinned non-minpack library defaults explicitly: separate relative cost, relative step and cosine criteria, each 30 times f64 epsilon, independently of physical stationarity and covariance qualification. Five actual LM controls passed, including the original-policy comparison and cosine/stationarity disagreement; retained public result reopening also passed. | Keep exact profile/replay identity and effective work caps. Selected native/scientific and public Python acceptance passes at the 49d–49e Outcome; broader QUAL scope remains separate. No public tuning knobs or speed claim. |
| I19 | resolved — identical selected catalogues | TOOL01 | Does the actual direction-qualified exported catalogue vary across supported feature builds? | Actual default and linked-native force-validated extensions share the generated catalogue identity; future exported-shape variation must generate its actual identity. No all-feature-powerset claim. |
| I20 | resolved — reuse existing physical owner | BND02 / PRE02 | Actual admitted Ports own coordinate identity/quantity/unit; BindingQuantity and CanonicalConversionPlan already resolve complete registered physical meaning. Tolerances/characteristic scales require Delta conversion, not point offsets. | Use those owners with the new keyed relation contracts and immutable snapshots; no independent unit table/resolver. BND02 integration and qualification remain open. |
| I21 | resolved — retain hub and internal module | TOOL06 | Actual no-layer/current-hub/candidate-layer steady and fit results/covariance agree bitwise. Attributed spans reach the candidate, but the actual native.worker phase has no corresponding candidate span; bounded state drains on completion, future drop and panic. A separate tooling crate would currently require a runtime/catalogue dependency seam; no independent build benefit is established. | No Layer substrate or new crate adoption. Keep the current hub and corrected internal dependency boundary; no speed or overhead improvement is claimed. |
| I22 | resolved — no additional adoption | RUN04 / MAT06 | Reached Simulation/Fit overlays are parameter-valued; typed point replacement does not define IC constraints, schedule topology or observation provenance. Existing IDAReInit/IDASensReInit and bounded neutral storage reuse are already consumed. Current structural scopes already serve their reached consumers. | Retain those paths. Revisit a named new input meaning or measured compatible setup burden, with separate identity/history decisions and fresh-contract reset controls. No universal bit equivalence or performance benefit is established. |
| I23 | resolved — retain declared binding | RUN07 | Is there a technically equivalent upstream Uno path preserving SQP-LBFGS and the four numerical-hunk meanings? | Upstream when equivalent; otherwise declare retained numeric integration. Different IPM/PETSc methods are not substitute remediation by name. |
| I24 | decided — implementation open | TOOL07 | Does native child lane or pse.canon.v2 have an explicitly named bounded exploration purpose? | Operator selected retain native-child for bounded provider-free process-isolation experiments and retire unused relation canonicalizer/API/tests; preserve independent shared allocation and identity contracts. Execution remains TOOL07. |
| I25 | resolved — no hot-start adoption | MAT06 | Can hot_start reuse an identity-bearing previous scheme on a non-direct path with a concrete preparation benefit? | Adopt only with scheme in identity and correct derivative semantics; direct-translation alias target remains and cancelled truncated optimization is never retained. |
| I26 | resolved — selected module Implemented and Tested | STO09 / GOV02 | [49a I26](49a-surrealmx-canonical-store.md#i26--surrealism-decision-investigation-2026-10-22) supplies source basis; remaining-scope Outcome records actual rollback/recovery, exact settlement, controller saturation, catalog refusal and production-only restoration. | Same-source read-only readiness replaces external scope/apply. Production manifest remains strict 1s/64MiB/pool 4; long-hold capacity artifact is qualification-only and removed from final deployment. |
| I27 | resolved — no new feature/binding adopted | PRE09 / GOV02 | [49c PRE09](49c-resolved-model-and-incremental-preparation.md#pre09i27-decision-investigation--2026-10-22) compares finite supplied-profile enumeration with Z3 assumptions/cores and defines independent acceptance. | Retain concrete requests/routing; RC23 remains unselected. Revisit a named jointly constrained multi-stage consumer or demonstrated finite-enumeration limit. |
| I28 | resolved — retain bounded canonical pages | STO09 / RES07 | Joined protected metadata/readiness pages are implemented and tested; bounded_query materializes responses and stream_items refuses payload-drain-owned requests. | Reopen streaming for a named consumer only with original deadlines, aggregate bytes, late failure and physical drop/drain. |
| I29 | resolved — no notification authority added | STO09 / TOOL08 | LIVE/changefeeds/async events provide observations with retention/delivery/import limits; current durable cursors/recovery remain necessary. | Reopen an actual wakeup/operator consumer with bounded catch-up; notifications never replace exact receipts, canonical claim or native drain. |

## Verification, checkpoint and completion

**Proposed at authoring:** companions' targeted tests and QUAL assembled qualification.
Product implementation and qualification were not_run during authoring; the store Outcome
below records later executed scope. The review's original memory-engine and guard-free measurements are historical
only. The selective msgspec identity-hook test settles its narrow codec question, not the
complete boundary. G4/G6 and scientific/profile coverage retain their actual limits.

Authoring completion requires complete source routing, consistent shared contracts,
bounded unresolved inquiries, coherent sequencing, recorded rule decisions, lifecycle
bindings/navigation and a successful documentation build or an exact external blocker.
Implementation completion additionally requires every selected package, all producer/
consumer migration and deletion, required assembled qualification, confirmed decision
publication and evidence-backed finding dispositions. No plan closes another plan by analogy.

**Authoring checkpoint, 2026-10-21 (historical):** ten proposed documents were authored, including the coordinator,
eight companions and source coverage. MX 1.0 selection is explicit; its minimal dependency
integration is separate from deliberate SSI/control/statistics adoption and gRPC migration.
Focused independent design advice found no outstanding material gap after the covariance
tolerance and existing SurrealKit ownership clarifications. All 21 rule decisions are accepted
and GOV01 is complete; GOV02 and implementation/qualification packages remain proposed.
Product source and the user's library documents were unchanged during authoring.

**Tested, documentation only:** publication builds the series with scoped search; lifecycle
validation and source routing cover 56 packages, 42 findings and 25 inquiries as 123 native
items at one owner: GOV01 complete, 122 items open. Relative link targets are present. Initial metadata vocabulary/table
formatting mistakes were corrected. The normal turn-end/ready entry was blocked before
checks by the existing shared-registry fenced-recovery requirement; adr-index/formatting
passed with the previously documented isolated test registry. Shared-state recovery remains
unchanged. These checks establish documentation integration, not product, migration,
native, numerical or performance qualification. All product acceptance remains Proposed.

### Execution checkpoint — fresh Plan49a target, 2026-10-21

The maintainer authorized all STO01–STO08 and selected fresh regeneration of existing
data. All eight store packages are done; the store Outcome below owns actual acceptance.
Selected functional journeys, final feature/manual checks and installed handoff pass.
The coordinator remains in progress for its other companions. RES04 is superseded by the
fresh-data decision; no historical preservation/backfill is required. RES01–RES03, full
PRE02 and broader TOOL04 are not prerequisites by association.

The final MX1.0/gRPC/v7 schema is installed in fresh functional and reference profiles with
their original numeric allocations. Schema publication, native metrics/faults, new-data
recovery, persistent maintenance, sixteen actual native owners, independent caller/worker
continuation, Python consumers and entered pull/analysis SIGINT are qualified at their
selected boundaries. Typed unknown effects, fenced absence proofs, physical response drain
and protected history remain explicit. Necessary ProblemTurns/source/sequence guards and
attributed ENFORCED relations are deliberate retained choices, not deferred failed remedies.

ADR-0214 and blueprint239 preceded dependent implementation; blueprint240 publishes selected
acceptance. Formal ADR status remains proposed. The shared fixture registry's preserving
recovery retained its existing records. Failed/unknown fixture outcomes retain pins and
incomplete SQL dispositions even after authenticated physical drain; they are not rewritten
as successful. The functional profile is authenticated/open; reference and private
qualification profiles are stopped/drained. The consumed companion remains retained for
the active coordinator and downstream boundary/maintenance work.

## Outcome — Plan49a canonical store

**Implemented and Tested, 2026-10-21:** STO01–STO08 are complete, including selected
assembled qualification, scope-end static/manual acceptance and installed handoff.
The Plan49 series remains in progress for its other companions.
ADR-0214 remains proposed under the separate decision-PR rule. Other companions retain
their packages and findings; selected store acceptance does not accept their implementation.

One source-built SurrealDB 3.3.4 server uses exact SurrealMX 1.0.0, explicit SSI, synchronous
AOL/EveryAppend and periodic snapshots. Finite engine memory, transaction age, queues and
outer process allocations remain distinct. Authenticated existing metrics expose engine
statistics and reader pins. The pinned server source, patch inputs, resolved lock and binary
are retained in its installation receipt. The final installed server is
`v3.3.4-mx1.0.0-baedb00f7f55498e9879`, binary SHA-256
`4da6ffce33f240eb5dee5fe24e46de7aa225b7cddfe074d7f0e09fb0a273a500`.

The sole production request path is bounded gRPC over an owned physical HTTP/2 connection.
Original operation clocks include admission, setup, reconnect, decoding and cleanup;
reserved control admission survives application saturation. Server sessions belong to the
physical connection. A replacement generation restores acknowledged setup and never replays
application mutations. Native typed engine errors and registry-generated domain refusals
retain separate meanings; unknown outcomes remain unknown until receipt settlement.

Registry-generated v7 storage uses five physical phases, native literal unions, immutable
identity, record-key assertions, full-u64 decimal bounds and irreversible compaction.
One synchronous event freezes terminal seed manifests; exact catalog witnesses include it.
Redundant key indexes and obsolete flag/prose/WS consumers are removed. Nodes precede
ENFORCED edges in bounded study ingestion. Native record ranges, graph readiness and
bounded inline dependencies replace applicable access machinery. The fresh publication is
`database/canonical-schema/rollouts/20261021131104__canonical_28f894ce3c844206`;
the replaced publication tree is retired. Existing-data import/backfill and old-AOL rollback
are deliberately absent. Fresh identities fence retired callers; new data retains recovery,
protected-file access and closed backup/restore.

### Verification and evidence limits

**Tested:** the following are local Linux checks against a **zero failure baseline**, using
the pinned nightly, exact source-built server and explicit Arrow force-validation wherever
Arrow is consumed. Native runs use linked solver capabilities and finite supervised
allocations; the reference selection actually enters sixteen native owners. Counts below
are distinct selections, not a sum of overlapping controls. Initial failures and affected
reruns are retained; composite acceptance is not a fresh monolithic suite pass.

| Command / mode and selected scope | Actual result and retained evidence |
| --- | --- |
| Source server locked release build, `storage-mem,cli,mcp`, explicit unwind | Passed; final release 17m07s, zero compiler warnings/errors. Source unary structured-error units: 2 passed. |
| `cargo test --manifest-path <selected-server-source>/Cargo.toml -p surrealdb-kvs-mem --lib --locked`, compile allocation, two jobs, pinned nightly/default debug/unwind | 10/10 passed (seven adapter and three configuration controls), zero compiler warnings; `build/qualification/plan49a-adapter-final/adapter.log`. Owned values across release, getm order/missing entries, conditionals/savepoint/reverse cursor, SSI absent/range conflicts, finite age/reader release, memory refusal and durability/configuration guards. Earlier admission refusal was not_run; no test process started. |
| `just unit-package pse-operations 'test(canonical) and not test(canonical_transport::)' --features pse-operations/canonical-tests,pse-operations/canonical-maintenance`, four nextest threads | 283 run: 278 passed, 5 failed; the exact affected five passed after repairs. Runs `c5cd0719-322f-41c9-9d5a-a05898027146` and `e5e942be-c497-437b-aef3-eca869c551af`. Covers native lifecycle/refusals, holders, compound guards, event tamper, index retirement, SSI point/absent/range conflicts, publication order and sixteen-lane mixed pacing. |
| `just unit-package pse-operations 'test(canonical_transport::)' --features pse-operations/canonical-tests,pse-operations/canonical-maintenance`, controlled peers | 29/29 passed, run `039d4b89-cafe-4725-b6df-99c8d03856e7`. Codec/allocation, provisional rows, late errors, clocks, admission, reconnect and physical drain. Engine API isolated units: 18 passed. |
| Actual retained-client server SIGKILL/restart control | Passed, run `1cc30814-5986-4c66-80a1-254c304cf8e0`; `build/plan49a-restart-55e550e673ae42f6a90da7963449fdb6/complete.json`. Same client restores acknowledged setup, reconciles original receipts and advances publication sequence without mutation replay. |
| `.venv/bin/python scripts/tests/surreal_mx_check.py --binary <selected-server> --strace /usr/bin/strace`, bounded disposable native server | 14/14 passed; final lint-refactored probe rerun also 14/14 passed at `build/qualification/plan49a-mx-hygiene-final-h31hrb9y/probe.log`. Authenticated metrics/public filter, held-snapshot pins, cancel/socket reader drain, connection/session admission, snapshots, synchronous reopen, atomic refusal, acknowledged SIGKILL+torn tail, fsync EIO/unknown/poison settlement, interrupted snapshot replacement, finite memory exhaustion and corrupt framed-log refusal. |
| Actual SurrealKit beta.6 patched gRPC sync/replay parity and frozen forward qualification | Parity 15/15 passed plus forward/cold-catch-up/tamper/missing/fork/gap/owner-death controls; `build/plan49a-tooling-{parity,forward}-20261021-codes/qualification.json`. Published tools/producers remain immutable. |
| `python -m scripts.tests.surrealkit_persistent_fixture_check --directory build/plan49a-persistent-drained --binary <selected-tool> --canonical-binary target/debug/xtask`, exclusive linked mode | 24 passed, cleanup empty. Exact pending nonce/catalog/event settlement, lost activation acknowledgement, forward replay, fresh authority and payload identity, child/listener drain and closed unknown-SQL/barrier controls. |
| Same persistent fixture command with `--initialization-only --directory build/plan49a-interrupted-initialization` | 3 passed, cleanup empty. Lost initialization acknowledgement stays closed; retry resumes the same fresh identities after authentic physical drain. |
| `python -m scripts.tests.canonical_recovery_check target/debug/pse-canonical-recovery --directory build/plan49a-recovery-wrapper --profile-state <functional-profile>`, exclusive linked mode | Passed. Actual source/lineage/product/payload admission, large staging, SIGKILL/reopen, exact protected history, lower/original kernel caps, closed backup/restore validation, stale authority/creation refusal and fresh creation after restore. |
| `python -m scripts.tests.surreal_fixture_check --evidence build/plan49a-supervisor-thaw`, exclusive linked mode; separate `--mcp-only --evidence build/plan49a-mcp-current` | Both passed; `passed.json` records retirement and drained contexts/scopes. Lifecycle includes authenticated native readiness, unknown readback, finite managed worker, launcher death, qualified restart and offline restore. MCP preserves VIEWER write denial and exact context/authentication. |
| `just native-test -p pse-runtime --lib -E 'test(workflow::durable_tests::) \| test(workflow::caller_lifetime_tests::) \| test(workflow::study_tests::canonical_) \| test(periodic_recovery_preserves_local_cancelled_owner_until_turn_release)'`, exclusive linked mode | 21 run: 18 passed, 3 failed; the exact three plus two typed-failure controls passed in a five-test single-thread rerun. Runs `fb4df346-1aec-496f-9c68-9f35c0d82d76` and `dcfa2fe9-f366-4582-bf2d-af6c84025aa6`. Worker loss stays unknown unless refreshed exact fenced proof establishes absence. |
| `just native-test --managed-primary-route -p pse-runtime --lib -E 'test(managed_primary_study_enters_sixteen_) \| test(managed_primary_study_cancellation_) \| test(managed_primary_publication_serves_exact_partial_history_)'`, reference profile, one nextest thread | 3/3 passed, run `3659a6ed-34ab-41a2-a388-c152f4087d93`. All sixteen native owners, cancellation/tail refusal, exact partial history and escaped Arrow retirement; original 160 GiB reference partition retained. |
| `just worker-test -E 'test(worker_runs_authored_case_end_to_end) \| test(study_parallel_workers_admit_one_summary_and_exact_point_attempts) \| test(durable_submission_survives_submitting_process_exit)'`, exclusive linked mode, one nextest thread | 3/3 passed, run `a60d25f3-0d9b-496b-a460-57a9ac322122`. Two independent workers, exactly one summary/point attempt and durable continuation after submitting process exits before worker start. |
| `just py-sync-native-correctness`; `just native-python build/plan49a-python-private python/pse/tests/test_canonical_results.py python/pse/tests/test_studies.py -k 'canonical or durable_study' -n 2 --basetemp build/plan49a-python-private-tmp` | Extension build passed. Python 11 selected: 7 passed, 4 failed; exact affected four passed with `-k 'constructor_preserves or idempotent_cache_clear or admitted_solve or selection_and_progress_reopen'` in `build/plan49a-python-phase`. Original native solution, retained scalar, protected selection/reader and durable study assertions preserved. |
| `python -m pse.tests.canonical_signal_control` result-pull and `--analysis`, exclusive linked current extension/private server | Both passed; `build/plan49a-signals/pull.json` and `build/plan49a-signal-analysis/passed.json`. Actual paused owned response, original KeyboardInterrupt/cause, grant/response drain and subsequent analysis recovery. |
| `python -m unittest scripts.tests.test_surreal_server scripts.tests.test_canonical_recovery_check scripts.tests.test_surrealkit_maintenance scripts.tests.test_test_resources`, light tooling units | 349 passed. Later lint-only server fixture edits: affected two passed. Selector preparing-runner controls: 4 passed. Source/tool/maintenance/type fixture repairs: 55 passed. These counts overlap other tooling selections and are not a grand total. |

`just unit-package pse-operations 'test(typed_refusal_unit) | test(canonical_codec_unit)'
--success-output immediate`, compile24GiB/two jobs, passed **8/8 with zero warnings**;
run `354d849c-eb3c-4af0-8165-2d8ea5550819`. An added assertion initially failed (7/8)
because the first boxed representation hid the native concrete error behind its box in
standard source traversal. The corrected representation retains the original native
error type, Internal kind and exact message for all six MX classes, plus all registry
refusal/codec controls. No warning suppression or policy relaxation was introduced.

The continuing `just hygiene --live` bundle ran under the wide allocation with linked
native capabilities and two compiler jobs; its actual result was **22 passed, 6 failed**
leaves against zero, at
`build/assessment/20261021T153304.715241Z-hygiene-2075353-3327ad/summary.md`.
Generated equivalence (all targets plus Python annotations/actual installed stubs/Ipopt),
family pins, Rust documentation, boundaries and the other leaves passed. The six failures
were typos, patch-file license metadata, Ruff, Pyrefly and both Clippy modes. The first
four passed their affected recipe reruns with zero findings; unrelated historical resources
remain retained when the preserving cleanup owner cannot operate their retired profile.
Clippy's large-error cascade (574 library/587 library-test diagnostics per mode) is repaired
by boxing the original native store error, without suppressing warnings or losing its cause;
redundant fixture defaults, cfgs and item order are corrected. A second two-mode Clippy run exposed one remaining
test-only Option borrow expression (one error per mode); its correction and both final
default/no-default modes passed with zero warnings/errors at
`build/assessment/20261021T154703.561727Z-clippy-2168316-d9773e/summary.md`.
Full linked-native/canonical/maintenance workspace/all-target Clippy also passed with
`--features pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-operations/canonical-maintenance`
and explicit force-validation, `-D warnings`, wide allocation/two jobs;
`build/qualification/plan49a-final-static/native-clippy.log` has zero warnings/errors.
`just lint-native-contracts lint-native-data governance features-powerset docs-rust`
then passed under the same wide/two-job linked environment; raw output is
`build/qualification/plan49a-final-static/manual.log`. Governance's 25/25 tests, all
generated equivalence/family checks, **419/419 depth-two feature combinations**, **33/33
no-default package checks** and final-source Rust documentation pass with zero
warnings/errors. The continuing governance and powerset summaries are
`build/assessment/20261021T155109.894585Z-governance-2186097-89c10e/summary.md` and
`build/assessment/20261021T155258.867217Z-features-powerset-2195251-c3393b/summary.md`.
The initial hygiene bundle plus affected successful reruns is the acceptance account;
there was no second monolithic hygiene run after documentation edits.

`just docs` passed for the 586-chapter book and scoped search. Metadata/lifecycle validation
also passed with zero errors. `just ready` passed skills/doctor against the final selected
MX1.0 profile; both selected supervisors were explicitly readmitted while stopped/drained.
`just turn-end` passed ADR indexing/formatting; it is not product qualification.

**Tested, final installed handoff:** `just py-sync-native-correctness` rebuilt and installed
the final linked force-validated extension and passed its actual compiled API stub check.
`python -m scripts.surrealkit install` published the final SDK input artifact
`034f5326d548f218fc16a44c25049ed98ef2456ba2b361c4a47b351f5c5c7dcb`
without modifying earlier producer artifacts; `build/qualification/plan49a-final-static/install.log`.
Both supervisors were explicitly readmitted from final source while stopped/drained.
The first ad hoc handoff assertion mistakenly compared the observation's state path with
the readiness enum; its log already reports native READY. The corrected read-only check
uses `ServiceObservation.phase`, verifies authenticated listener ownership/open admission,
and checks reference/private drain without replaying startup or SQL mutations.
That check passed at `build/qualification/plan49a-final-static/handoff-verified.log`.
The functional instance `45ede5c2-0142-49fc-8b20-3e0a750c1106` is open and authenticated;
the reference instance `9a91dcbb-51e3-4722-b80b-603e55edb81d` and private SIGINT profile
are stopped/drained. Final `just ready` passed skills/doctor, including the current linked
extension and open MX1.0 profile, at
`build/assessment/20261021T172503.076540Z-ready-2531419-afa349/summary.md`.
Final publication after completion/navigation changes also passed: `just docs` published
586 chapters with scoped search (`build/qualification/plan49a-final-static/docs.log`), and
lifecycle validation reports zero errors across 1002 native scope items. Retirement
preflight confirms that the completed companion is still consumed by the active
coordinator, ADR and boundary/maintenance documents; owner release is not established,
so it remains retained. Other Plan49 packages and unresolved combined findings stay open.

### Mistakes corrected

Actual controls exposed optional field assertions skipping NONE, lost typed errors across
gRPC status, session ownership gaps, protobuf over-accounting, a terminal seed-manifest
mutation seam and stale payload-freeze state after acknowledged reopen. Their implementing
repairs preserve native rollback, exact identity and uncertainty; affected controls passed.
Native recovery initially treated all expired point claims alike. It now proves never-started
absence only from an exact post-fence projection; a native-started lost worker remains
unknown and cannot gain a retry. The proof controls are sequential cases, not a forced
concurrent start/recover interleaving.

The old worker recipe bypassed the preparing runner and its declared 16 MiB libtest stack;
three selected tests aborted. The recipe now uses the existing runner and explicit store
resume. The repaired selection passed. Authenticated physical drain of the two aborted
fixtures retained their incomplete SQL dispositions and pins; no failed record was rewritten
as successful. Initial Python tests used removed physical flags and obsolete WS prose;
they now consume the generated phase enum and typed failure classification. The analysis
SIGINT controller watched a retired reservation name and never sent its signal; the fixture
now watches the actual scientific-selection owner and reports controller failure first.
Initial failed fixtures and unknown maintenance outcomes remain retained, closed and pinned.
Earlier placement/loader refusals started no product test; their corrected runs use the
declared native environment and allocations. The first Python launch also failed before
collection because the shared temporary root belonged to another user. Private `--basetemp`
directories corrected that environment issue without changing shared filesystem ownership.

### Deliberate deviations and supported limits

**Implemented:** `ProblemTurns` is retained. Source staging/close and begin-run share the
`source-creators:<problem>` write guard with reclamation; sequence assignment retains its
commit guard. Removing these without a replacement would break source lifetime or paging.
Renewal/cancellation and distinct problems retain their separate scope. Actual sixteen-lane
controls demonstrate progress under these conditions; no contention or throughput win is
claimed. All assessed graph relations carry attributes/receipts, so classic ENFORCED edges
remain; bounded inline dependencies use a high-degree fallback. Native plan observations
separate output limits from engine scans: e.g. 17 emitted after 53 scanned, and fallback
64 emitted after 129 scanned. There is no bounded-scan or speedup claim from those limits.

**Tested limits:** fsync EIO, process death, torn bytes and corrupt headers are named fault
models, not a hardware power-loss experiment. Current synchronous settings are selected;
asynchronous queue controls do not authorize asynchronous durability. No existing-data
preservation is claimed. The engine's 4 GiB encoded-transaction refusal is source-derived;
the narrower application/frame/aggregate limits are exercised without allocating a 4 GiB
test transaction. Asynchronous queue saturation is not tested in the selected synchronous mode.
No all-product/native/scientific-model campaign, new-platform, release/distribution,
exhaustive feature powerset or statistical performance qualification is claimed. Existing
Plan28/32/40 and the remaining Plan49 packages keep their owners. No benchmark is required
to support an unmade quantitative benefit claim. Formal ADR acceptance remains separate.

## Execution checkpoint — remaining series authorized, 2026-10-21

The following development observations retain their original stage and evidence scope. The
[pause handoff](#pause-handoff--requested-closeout-2026-10-21) is the current state; references
to pending checks below describe those earlier stages.

The maintainer authorized the conversational execution approach for all remaining 49b–49h
packages. Clean starting source is `4065690aa`. STO01–STO08 remain done and RES04 remains
superseded; existing generated data can be regenerated. Proposed ADR-0215 and blueprint241
record remaining target contracts before dependent implementation. I24 explicitly retains
the opt-in native-child experiment and retires only the unconsumed relation canonicalizer.

Implemented work awaiting targeted acceptance includes RUN01/RUN02 supervision, RUN07 status
constants/mapping and declared numerical binding, RUN10 retained-worker join, PRE01 retention/
firewalls and PRE03 current-identity indexes. BND01 strict vocabulary projection and TOOL07
canonicalizer retirement are integrated at source; generation and survivor checks remain open.
PRE05 pure policy producers, RUN03 paired outcomes/shared preparation and RUN06/RUN08 native
seam/memory work are in progress. Root owns registry, generated
contracts, decision publication and integration. TOOL03 now invokes the authored Rust
cooperating-source executor through the existing authenticated command-group owner. The Python
protocol/state machine is removed; targeted actual launch, log-failure and drain controls and
public command journeys remain pending.
No historical data backfill or broad qualification is scheduled mid-plan. Remaining package
acceptance, inquiry outcomes and assembled QUAL scope are still open.

The PRE01/PRE03/PRE05 producer checkpoint is **Tested** at the selected pure compiler/model
boundary: `just check-package pse-compiler` (all targets) completed with zero warnings/errors;
`just unit-package pse-compiler 'test(workspace::modeling::reuse_tests) | test(workspace::modeling::granular_tests) | test(executable::projection_tests) | test(executable::policy::tests)' --test-threads 1`
passed 45 tests, zero failures (249 filtered), and the selected `pse-model` prediction-policy
unit tests passed two, zero failures (64 filtered). Baseline is zero. The actual 143-body B2
fixture retains no-edit/unrelated reuse without ordinary rotation; bounded churn/oversize
and dependency/policy controls are included. Runtime RUN05 consumers and assembled qualification
remain open. Subsequent PRE02 declaration/generator work is outside that tested checkpoint.

TOOL01 now has one actual compiled document catalogue, independent serialization/deserialization
schema contexts, direction-dependent output names and generated native registrations. Its
fingerprint includes canonical schemas and owning interpretation versions rather than unrelated
feature bits. Completion now carries Version1. Full `just codegen` publication has passed under the selected functional 64GiB,
single-job allocation, including actual directional document registrations and candidate
Python annotation controls. Wrapper migration, standalone version roots and feature
comparison remain open; publication does not qualify the native/Python boundary.

The PRE02 checkpoint is **Tested** by the source-syntax schema control, 15 expression
traversal/checking units, generated grammar metadata control and 45 compiler reuse/policy
units: 62 passed, zero failures. The producer-only PRE04 checkpoint is **Tested** by
`just check-package pse-modeling` and eight selected `resolved::`/`derived::` units,
including canonical-layout correspondence; zero warnings/errors or test failures.
Finite-fact collection and complete function/provider/indexed/reduction support, direct
lowering and all production consumer replacement remain open.

The consolidated mathematical checkpoint is **Tested** by the selected `pse-math`
`unit-package` run with `compiled-evaluator`: 22 passed, zero failures. It exercises
product preflight/abort, shared value definitions, symbolic mixed-partial composition,
actual C++ alias regions/private worker scratch and existing product guard/contracts.
SelectedPartials First/Second dependent regions consume symbolic local partials; standalone
and reverse/action routes retain their existing realizations. Subsequent per-region profile
and action integration is outside that checkpoint. No assembled product or speed claim follows.

RES01 now declares the four distinct scalar key/sort shapes and dense value selection on
owning registry fields. The new source-catalogue schema and generator controls each passed
one test with zero failures; two initial generator control runs inspected the prior
hydrated registry before publication and failed. The corrected control inspects source assembly.
Runtime field eligibility consumes this generated inventory; actual-session predicate
execution is authored but awaits its focused runtime controls. Exact admitted batch
statistics, pruning, sorting and locator retirement remain open. RUN04 Fit/Simulation point
preparation and bound replay, and RUN05 annotation/view consumers are saved at source;
their focused controls and assembled acceptance remain pending. The subsequent full
`just codegen` run also passed all five targets with these source changes. The current
`just check-package pse-py` completed all targets with explicit force-validation and zero
warnings/errors after correcting the semantic-identity borrow and retaining the original
stored-completion metadata lease through document construction. This is compile evidence;
no installed extension/Python journey is qualified by it. Baseline remains zero.

The latest consolidated mathematical retry again passed 22 tests with zero failures, now
including consumed per-region construction profiles and the independently specified
Jacobian action. Weighted/adjoint eligibility and provider/branch region controls remain open.
The same latest source passed 16 selected `pse-backend-native` controls using explicit
kinsol/pounce/idas/uno features and solver/klu/uno capabilities, three selected linked
`pse-uno-sys` curvature/storage controls, and the `pse-diagnostics` effective-text-budget
control. All 42 tests in that combined queue passed with zero failures. This includes the
actual native freshness/loader, bounded owned UTF8 diagnostics, provider-link parser,
preinitialized-host Pounce serial path, fresh KINSOL maps, FERAL symbolic-peak refusal and
Uno status/stop/storage boundaries. The subsequent runtime libtest compilation completed
with zero warnings/errors under the same 64GiB, single-job allocation. Its selected
22-test queue ran: 19 passed and three failed against the zero baseline. The failures
were two parameter-point fixtures registering a composed neutral unit as atomic and
the inherited-stop horizon fixture omitting its required estimator/controller. Fixture
repairs and their focused retry remain pending; these results do not establish complete
RUN04/RUN05 or assembled runtime acceptance.
The maintainer subsequently requested 16 compiler jobs, concurrent independent builds and
no operating-system memory budget for ordinary builds/tests. That persistent local opt-in
is implemented, retaining process/drain ownership and the separate scientific pool, timing
and reference contracts. Nine focused mode controls and twelve existing allocation controls
passed; two actual kernel build owners overlapped with sixteen compiler jobs, sixteen
physical cores and `memory.max=max`. This establishes resource configuration, not a measured
build-speed improvement. The earlier 64GiB/single-job results above
retain their original conditions. Readiness found host Python patch drift: `.venv` had
followed `/usr/bin/python3.14` to3.14.9. A locked
sync recreated it with managed3.14.8, preserving the repository pin. The editable extension
must still be built before Python execution. The subsequent `just ready` passed with that
missing extension reported as a warning; four unsupported historical service records were
preserved and their cleanup warnings remain. Neither readiness nor resource controls qualify
the uninstalled Python boundary.

The expanded normal `just unit-package pse-math` queue with `compiled-evaluator` and one
test thread passed 191 tests, zero failures (266 filtered), including all five regional
product kinds across actual provider/branch barriers, shared maximum-demand local partials,
reverse construction plans, deep derivative exports, private compiled worker scratch,
compiler cancellation/drain, curvature, implicit products and assembly. The current PRE04
finite-fact/shared-symbol producer passed ten selected modeling units, zero failures; the
earlier lineage-path fixture assertion was corrected before that retry. These are focused
component boundaries; final resolved execution and whole-series qualification remain open.

BND02 now has authored registry contracts for keyed physical starts, initialization stages,
diagnostic samples and tolerances, with one pending-to-admitted integration-controls owner
under construction. BND03/BND04 source work adds explicit external versus qualified-fit
origin, private process-local verification, exact protected fit hydration/reconciliation,
required parameter quantities/units/scales, and prepaid sequential faer PSD admission.
SolveSettings advances to version5; covariance result relations advance to version2.
Full zero-argument `just codegen` subsequently published all five schema targets, Ipopt
bindings and candidate Python annotations. Nested generic documents now retain distinct
owner names/defaults, and identity-keyed maps retain their actual key role. Two targeted
identity-map schema controls passed, zero failures. The native-feature all-target
`just check-package pse-py --features solver-kinsol,solver-diffsol,solver-ipopt,solver-highs,compiled-evaluator`
then passed with explicit force-validation and zero warnings/errors. This establishes
generation and compilation, not native/Python behavior. Full canonical, Python,
resource/cancellation and scientific acceptance for these boundaries remains open.

The first controlled result-file units passed three columnar controls and six relation
controls, zero failures; the latter compilation reported three test-only trivial-cast
warnings, whose repair remains in progress. Two operations allowance/proof controls also
passed, zero failures. The initial columnar test compilation refused an ambiguous empty
schema fixture, repaired before its passing retry. Actual protected-provider pruning,
locator retirement and assembled result qualification remain open.

The current integration checkpoint carries an explicit standalone wire selection for all
144 exported owners: 58 native versions, 85 flat-v1 roots and one value-v1 root. Existing
fixed ProcessCheckpoint4 and InvocationEvidence2 remain their native version owners.
Nested values keep their original domain meaning; explicit body/root joins replace
implicit caller reinterpretation. Submitted starts now use the existing complete native
receipt declaration, with borrowed numeric/IEEE-bit serialization under actual report
custody. The public source-reclamation continuation is part of the same compiled catalogue.
Full generation and the complete native Python compile passed. Twenty-one targeted
native-local controls passed against the zero baseline, including document headers,
parameter-valued simulation IC/trajectory/replay/schedule behavior, LM policy, submitted
receipts and tear coordinate receipts. Actual default and linked-native force-validated
extensions now build and generate their stubs, and their direction-qualified catalogue
fingerprints agree. The installed linked-native extension passes 43 standalone-wire,
private-edit, solve-settings and native-boundary units plus two physical-input units,
against the zero-failure baseline. Initial failures exposed missing identity hooks in
private edit conversion and stale root/body, input/output and optional-name fixture
assumptions; those are corrected. These checks do not qualify every exported root,
every feature combination or the assembled product.

RUN09 implements fixed independent library-relative cost, step and cosine criteria,
effective work caps, FitProfileV6 and ReplayDocument3. Five LM controls cover the actual
original-policy comparison, cosine/stationarity disagreement, qualification/covariance
withholding, limits/cancellation and replay/profile chains. The canonical retained-result
control also passed. This resolves I18's bounded policy inquiry while the package retains
its wider Python/scientific qualification obligations. Proposed ADR-0215 and blueprint242
retain the physical study request8/stored definition12 and standalone wire target.

TOOL04 implements one canonical source cursor and shared physical source deletion, one
bounded foreground page plus explicit continuation, without caller cursor authority.
Cleanup follows the completed mutation timeout owner under the same original deadline;
a deferred page cannot replace its immutable receipt. Actual canonical controls passed
for protected-first-page progress, peer reopening, replay, deadline behavior and exact
root/pin races. The combined nineteen-control selection initially had seventeen passes
and two fixture failures; the corrected pair then passed. One fixture incorrectly reused
a unique guard incarnation; another expected withdrawal to restart the cursor instead
of resuming the six-candidate tail after a protected 64-candidate page. Their corrected
assertions preserve uniqueness and exact continuation. The actual protected-pruning pair
released scalar locator retirement. Its relation, indexes, append/replay/collection
machinery and callers are now deleted with complete generated substrate-v8 outputs.
Fresh v8 state is initialized and authenticated startup passes. The v8 canonical
selection initially passed 21 of 23 controls; both corrected lifetime controls then
passed separately. One now includes the foreground retirement receipt in its exact
one-batch total. The other checks actual charged buffer custody and unchanged scalar
bits through packing, while allowing obsolete source-container metadata to retire.
Selected scope includes provider pruning, historical manifests, retained arrays,
protected reads/exports and append ACK ownership. Native protected-file maintenance
phases and assembled qualification remain open.

Supported initialization exposed ordinary managed command parents that could not hand
work to native children. The command parent now authenticates its exact registered
cgroup, invocation and inode before establishing foreground custody; it gains no native
reuse authority. Seventeen targeted controls passed, including actual-kernel normal and
original-deadline descendant drain. All owned test scopes and allocations were released.
PRE04's direct typed producer, compiler/portable consumers and per-body receiving
source bindings are now integrated in the checkout. Specialization emits typed roots,
original-source correspondence and physical obligations during its one construction rewrite;
compiler projection and numerical construction consume those products directly. The old
dispatch-body expansion and its final consumer are removed. Ordered composition retains
original guards, nominal member boundaries, ranges, actual input contracts and independent
source/member tokens. Provider conversion/partial contracts are prepared before numerical
construction and restored only inside qualified recipe sessions. Broader PRE04 acceptance,
remaining retirement and assembled qualification remain open.

### Pause handoff — requested closeout, 2026-10-21

Verified 2026-10-22. The maintainer requested integration, correction, qualification,
documentation and local commits before pausing. On 2026-10-22 the maintainer canceled the
broad native sweep and selected representative functional assessment without a benchmark
or production-scale campaign. Remaining 49b–49h execution is paused; nothing here completes
those packages or authorizes automatic continuation. There is one checkout on `main`, no
worktree merge is needed, and all workers and one-shot closeout jobs have drained.

**Implemented:** direct typed model/body production and consumers, receiving source maps,
demanded original-equation preparation, protected result/maintenance consumers and strict
directional Python documents are integrated. Original supplier projection now retains real
independent coordinates and only substitutes selected suppliers and their demanded dependency
closure. Private inner KINSOL execution honors explicitly tighter normalized precision while
retaining public defaults, native failure qualification and the original accuracy assertions;
its solver identity advances to v2. Physical body premises remain distinct from the complete
receiving deployment envelope. Generated scientific artifact completeness is qualified by the
admitted modeling/simulation/fit family. Positive native fixtures admit their complete finite
construction envelopes, and escaped registrations prove actual program ownership after cache
clear. No compatibility path or scientific tolerance relaxation was introduced for these fixes.

**Tested, baseline zero:** the final completed checks below passed. Rust correctness selections
used explicit Arrow force-validation; Python used the rebuilt linked, validated extension,
Python 3.14.8 and 16 workers. Ordinary build placement retains the maintainer's unbounded-memory,
16-compiler-job selection; scientific children retain their own finite contracts.

| Command / scope | Result and boundary |
| --- | --- |
| `just clippy` | Full workspace/all-target default and no-default graphs; zero errors/warnings, keep-going, warnings denied |
| Native `just clippy-default` | Full workspace/all-target selected native/canonical/maintenance graph; zero errors/warnings; feature selection below |
| `just typecheck`; `just lint-py` | Pyrefly zero diagnostics (three existing suppressions); Ruff zero findings |
| Full zero-argument `just codegen` | All five publication targets passed, no final generated drift |
| `just test -E '<functional-filter>'` | 3,491 passed, zero failures, nine skipped; default graph on the new functional store, run `e4710ccd-8c7b-42ba-8138-4d00a5801840` |
| `just doctest` | 48 passed, zero failures or ignored doctests |
| `python -m scripts.python_tests --maxfail=0 --continue-on-collection-errors --basetemp <owned-directory>` | All 270 unit/component tests passed in 147.44 s; integration/performance/parity excluded |
| `python -m scripts.setup_report <owned-output>` | All 1,241 tooling tests passed, zero failures/errors/skips |
| Selected native-local repair controls | 17 backend precision/policy, 50 runtime, 19 child/target, 13 IBEX, six dynamics and four Pounce controls passed; these selections are not a unique combined test count or full native qualification |
| `just ready` | Skill synchronization and doctor passed against the new 3.3.4/MX 1.0 deployment |

The functional filter is `not binary(=worker) & not (package(pse-tests-conformance) & test(/^acceptance::/))`.
The final native Clippy feature selection was
`pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-operations/canonical-tests,pse-operations/canonical-maintenance,pse-tests-conformance/native-acceptance,pse-benches/native-process,xtask/canonical-tests`.
The linked extension and its actual compiled API stubs were rebuilt before Python execution.
The child/target selection passed 19/19 with a frozen terminal, completed projection and released
host allocation; run `e3aa395b-6a23-4f2a-a753-28a51183a406`.

**Canceled, not full native qualification:**
`just native-test --profile local -E '<functional-filter>' --features pse-operations/canonical-maintenance`
selected 4,779 tests across 80 binaries. The maintainer interrupted it after 862.203 s of test
execution: **2,759 passed, 11 SIGINT, 2,009 not_run**, with 30 tests and one binary skipped
(including six managed-primary tests excluded by the default filter). No failure was reported
before interruption. Nextest labels the interrupted cases failures; they are not 11 diagnosed
regressions. Run `16c32128-60a0-4317-955e-c3b37af45bd2` and
`build/native-tests/1792650816881675357/native.json` retain the selected scope. The frozen
invocation is `f8d0dc8ea0e2480f849e420e4517e61b`; its XML preserves individual outcomes and
durations. Its assessment is incomplete, while terminal acknowledgment, metadata projection,
physical process/scope drain and host allocation release completed. The identity-mismatch
footer reports interrupted coverage, not additional scientific failures.

Individual completed results remain bounded evidence. In that interrupted run, the repaired
exact-protection settlement, bounded interval reclamation and blocked-product controls passed
in 11.789 s, 14.260 s and 24.136 s respectively. These durations include concurrent execution and
fixture work, not isolated mechanism measurements. Full integrated worker/acceptance/managed
journeys, parity, performance, feature powersets and platform qualification were not_run.
No build-speed, scientific-speed or memory-capacity improvement is claimed.

**Functional assessment:** the broad sweep mixes pure/native-local controls with expensive
live-store qualification. Each initialized UUID fixture installs 251,529 bytes containing 777
table/field/index/function/event declarations in one atomic transaction, with two physical
clients, authorization/context selection and durable registry registration/drain. This is
per-fixture work, not 777 external RPCs or per-query Python mediation. Sampled concurrency
varied from one exclusive control to 16 simultaneous store tests; low occupancy was not a
compiler-job limit. More importantly, two four-byte artifact cases explicitly wait for the
production 60-second original creation window. Whole-test timings do not isolate schema,
transport, registry or cleanup cost. The [representative functional route](49h-qualification-and-design-publication.md#representative-functional-checks)
owns the revised qualification approach and proposed fixture corrections. No replacement
campaign, benchmark or scale/setup rewrite was run after cancellation.

**Store and retained-state handoff:** earlier ENOSPC poisoned the old functional engine.
Free space did not clear its fail-stop. Normal restart correctly refused three unobserved
fixture-creation borrowers from earlier invocation `d63c4731f12e4cc7b2a0d7eef19f7fac`.
That old service, data and evidence were preserved. Its dead-job owner
`bb40005600a842fab3327793beb5dd15` still retains one build slot through its unsatisfied
terminal guard; no supported same-boot recovery acknowledgment was fabricated.

A separate generated deployment was initialized through the normal canonical-tools route and
opened with the current `pse.substrate.v8` contract. The ignored local selector now names
`~/.local/state/pse-arrow/surreal-plan49-closeout-20261022`, gRPC loopback18349,
instance `6e982d05-8e9a-452f-a4a6-c39af9104387`. Host policy explicitly admits two functional
stores, each retaining its own 8 GiB charge; all 57 host-admission controls passed, including
third-store refusal, timing isolation and retention of an unknown owner. Ordinary build
memory policy is unchanged. Both persistent services remain intentional resident state.

The newly canceled campaign retains 217 pinned resources: 206 recorded physically drained,
11 with unobserved creation effects despite dead borrower PIDs. Its current finalizer marks
all resources incomplete when overall selected coverage is interrupted. None was deleted
or relabeled successful. These retained effects differ from the fully drained job processes.
TOOL04/RUN01 retain the recovery/disposition obligation, including truthful per-test outcomes;
unknown external effects cannot be released merely because a process disappeared. Unsupported
historical profile records were also preserved rather than rewritten to silence cleanup output.

**Next work, only after explicit resumption:** implement the representative functional fixture
and selection changes in 49h, retaining actual contract semantics and all refusal/source/clock
assertions. Address retained-state disposition through its authenticated owners. Reconcile
remaining producer/caller retirement and package acceptance, then select the still-required
assembled QUAL journeys and enduring publication. GOV02, the remaining 49b–49h packages and
whole-series qualification remain open. Proposed ADR-0215 does not by itself establish them.

### Bounded native inquiry dispositions

I14 adopts the one-thread Pounce caller path and retains its admitted local pool for
larger batches. The actual native control passed with an already initialized host global
pool. Symbolica3.0.1's reached unlicensed `LicenseManager::new` still attempts a one-thread
Rayon global initialization and ignores failure; a preexisting host pool stays unchanged.
Valid license/unlock paths bypass that fallback. This is distinct from evaluator construction:
`LicenseManager::max_threads` bounds the explicit requested cores, and the optimizer uses
scoped threads with the caller executing the final lane. The current production default is
one requested core; scalar interpreted/C++ evaluation is sequential. Retain complete foreign/
process envelopes and add no eager project global initialization. No blanket elimination,
resizing or ownership of an embedding host's global pool is claimed. The 22 math controls
exercise the configured environment; fresh-process unlicensed/embedding initialization and
constructor-team/cleanup qualification remain required before claiming those combinations.

I15 retains the complete foreign allocation envelope. Tagged [PETSc3.24.0 allocator source](https://github.com/petsc/petsc/blob/v3.24.0/src/sys/memory/mal.c)
requires installation before initialization and does not supply original allocation ownership
on later free/realloc. The deployed SuiteSparse_config7.7.0/KLU allocator setters are global
function pointers; they neither establish lane/owner custody nor cover SUNDIALS/BLAS/MKL
allocations. No complete original-owner/size protocol and bypass coverage is established,
so hooks are not adopted. Existing PETSc process initialization ownership remains intact.

I17 selects independently fresh KINSOL map sessions. Tagged [KINSOL7.1.1 source](https://github.com/LLNL/sundials/blob/v7.1.1/src/kinsol/kinsol.c)
shows vector allocation, callback resets, counter initialization and linear setup, but does
not establish complete fresh-equivalent Anderson/map/KLU history under a neutral retained
reset. The authored map execution therefore creates a fresh session per application. The selected
native fixed-point/Picard controls have passed; runtime/public assembled integration remains open.

I23 retains the declared Uno HiGHS SQP-LBFGS integration documented in
[the numerical binding note](../../docker/solvers/uno-binding.md). The four numerical hunks
remain explicit and distinct from integration/determinism work. Source comparison with
upstream2.9.1 finds a BQPD operator-Hessian candidate, but equivalent controls, accuracy,
acquisition, storage allowance and readback are not qualified. IPM or unconstrained-only
algebra is not a substitute for the selected bounded SQP method. No upstream/dependency
change is claimed. The candidate becomes actionable when those exact obligations can be met.

TOOL02's versioned Rust campaign/settings/run documents now own the defaults, strict decoding,
name inventory and package-relative source/diagnostic checks. Python uses generated msgspec
TOML types and the common Rust context admission; the Rust reference derivative controls use
the same manifest owner. The reference manifest now explicitly supplies interpretation1.
Generation has passed; selected Rust/Python conformance controls remain pending.

RES05 selects removal of the unreached file-cache adapter after current producer/caller
inspection: `bind_state*`, source-generation namespaces and metadata/statistics/listing
activation have no production caller. Their policy knobs, detailed reports and dedicated
mechanism tests are removed with the adapter. Actual syntax/model/preparation caches and
load/query/output admission remain; DataFusion file-cache capacities remain explicitly zero.
The reached physical-plan predicate allowance remains independently owned. Survivor tests
and boundary generation are pending; no hit-rate or latency improvement is claimed.

MAT01/MAT02's selected compiled-evaluator controls are **Tested**: the focused `pse-math`
unit command selecting product preflight, constructor abort, scalar definition-map admission,
deep shared lets and sibling regions passed ten tests with zero failures against baseline zero
at 64 GiB/jobs1. Final resolved producer identity, higher derivative cutover and full math/
scientific acceptance remain open; these tests do not close either package as a whole.

TOOL05 has moved the exact protected result-selection/predicate/usability contracts to the
workflow result owner. Offline pack tooling consumes those types; full/selected result
readers no longer obtain their scientific selection contract from an export module. The
internal tooling extraction is now authored: live detail collection and shared collection
budget/error contracts belong to `observation_detail`; capture, inspection, comparison,
review analysis, packing and publication composition belong to `tooling`. The workflow
owns protected selection contracts, and tooling calls its bounded reader interfaces.
Rust/Python callers have moved; the old root module exports are removed. Compilation and
the affected caller controls remain open.

TOOL04's guarded domain-mutation source is authored. The registry declares maintenance
generation/page-effect records, and generated functions own authority rotation, placement
rehoming, validation-read admission and the existing exact schema-activation transaction.
Supervisor code retains authentication, physical copy/drain and phase orchestration. Page
effects retain original inputs and acknowledgements; a generation change fences older
callers. The script checks persisted phase/original-operation identity before requesting
the database generation. Generated-function behavior, interruption/replay, stale-generation
and protected-payload controls remain open; source publication alone establishes none of them.

The next shared generation checkpoint includes Study operation/request8 and stored
definition12, physical Arrow boundary tables and role vocabularies, covariance relation2,
and optional controlled result statistics. Generic fit/integration schema names now
distinguish concrete instantiations; catalogue-derived names apply consistently to roots,
nested definitions and references. Identity-keyed map schemas retain declared key roles
rather than inferring identity from hexadecimal patterns. The first attempted generation
stopped on the generic-name collision; the repair and subsequent source joins are being
regenerated before their runtime/Python controls. No full package closure is claimed.

### Observation inquiry disposition

The TOOL06 inquiry is **Tested** in the selected native runtime queue: both
`observation_layer_inquiry_actual_steady_and_fit_preserve_science_but_expose_phase_gap`
and `observation_layer_inquiry_bounds_state_and_retires_on_future_drop_and_panic` passed.
Their original science agrees across all three observation modes, while the missing native
phase prevents replacement of the actual observation owner. The finite candidate holds at
most 64 active spans and 256 closed samples and retires them on joined completion, future
drop and unwind. This is a semantic/ownership no-adoption outcome, without a measured CPU
or memory benefit. The optional crate remains unselected: current boundary-catalogue and
private runtime consumers need a deliberate dependency seam before a separate crate could
provide independent build value. The corrected internal tooling module remains selected.
These two passes do not close the separate TOOL05 consumer or whole-series qualification.

### Symbolica optional inquiry dispositions

I11 retains canonical scalar formals and selected argument-free `FunctionMap` aliases.
Pinned Symbolica3.0.1 supports function-shaped indeterminates; capability absence is not the
reason. Reached dependency, guard, class and factorable consumers currently key roles by
`Symbol`, so a common `slot(k)` function would collapse distinct slots until those consumers
admit exact indeterminate keys. Revisit when that role/check migration removes a measured
registration or retained-symbol burden and fresh-process reversed-registration, derivative,
selected-pair and C++ export controls preserve identities and domains. Current alias sharing
remains selected. This is a source-based no-adoption decision, not a failed benchmark.

I12 retains the interpreted/C++ scalar profiles. The [official evaluation guide](https://symbolica.io/docs/numerical_evaluation.html)
documents JIT, vectorized evaluation and generated SIMD/CUDA libraries. Exact pinned
`evaluate/backend.rs` and `evaluate/export.rs`, plus SymJIT2.26.0 sources, determine the
remaining integration. A JIT candidate must explicitly disable fast math and library threads,
bound stack/code/private scratch, and qualify primitive floating semantics and construction
cancellation; lack of contraction controls is not asserted. SIMD needs admitted pack/tail
buffers, fixed ISA and lane/error ordering. CUDA needs host/device/compiler owners and finite
chunk cancellation through synchronization/drain. Current region consumers do not supply
those owners. The existing test-only `product_realization_probe.rs` already invokes a
thread-disabled, fast-math-disabled JIT candidate on guard-free polynomial rows and reports
ISA conditions; this turn has not executed that probe or measured its benefit. Its existence
does not establish a production profile. Revisit an actually grouped repeated region when a total preparation/evaluation
comparison includes these costs. No device or speed benefit is claimed.

I13 retains the reached library algebra: `coefficients.rs`, `implicit_cubic.rs` and
`implicit_selection.rs` already use Symbolica polynomial/coefficient/root operations with
consumed degree, finite-value, domain and original-residual controls. A broader rational
reformulation must preserve excluded denominator points and parameter incidence. Generic
[solution sets](https://symbolica.io/docs/solving.html) require explicit domain, coverage,
coverage guards and branch predicates; unsupported/incomplete cannot become empty or a
selected numerical root. Revisit an actual selected-function certificate or repeated
factorable transformation burden with those obligations. A transformer wrapper alone does
not remove an existing project rewrite engine. [Term streaming](https://symbolica.io/docs/streaming.html)
is not selected for dependency-sharing work: pinned `streaming.rs` would introduce its own
pool, disk/tempfile and teardown/cancellation owners. Revisit only a reached finite term-local
spill whose complete resource and cleanup contract is qualified.

I25 does not add hot-start history. Current `exact_evaluator` uses direct translation;
pinned `evaluate/tree.rs` refuses a supplied hot-start scheme with positive Horner iterations,
while zero iterations only linearizes. The accepted input is an explicit prior expression
scheme, not a cached evaluator. Equal expressions/settings/definition maps already reuse
`ScalarProducts`. Revisit a reached non-direct changed-body caller only with the completed
scheme in versioned identity/cache/serialization premises, cancelled-construction refusal,
derivative/guard equivalence and a total preparation comparison. No extra `optimize_stack`
pass is added: the selected library construction already invokes it; an actual temporary
census must justify another pass.


## Outcome — remaining 49d–49e, 2026-10-22

### What was built

**Implemented:** demanded guarded library regions and all five derivative products, one typed
run supervisor and native cancellation bridge, shared request/preparation/admission, parameter
Simulation/Fit points, compiler-owned start/annotation/view policies and adapter-owned numerical
capability are integrated with their actual Rust/Python consumers. The deliberate nondurable
strategy exception remains; Shooting is still host-only. Study request/operation version 8 and stored
definition version 12 have one current representation. Replaced per-kind lifecycle forwarding, host
policy copies and per-output reconstruction are retired in the selected domain. Deterministic
Direct/Reverse/Regional eligibility is retained rather than converted into transient fallback.

This continuation repairs recycle tear-port physical custody, sparse stage-description charging,
Symbolica AST/literal/exported census and repeated static-package projection. A tracked query
keys projection by complete immutable parent plus normalized selection/read recipe and produces
the original complete-content interned owner; parent/selector do not replace semantic equality.
Cross-parent equal owners, A/B/A reuse, canonical physical owners, attribution and finite LRUs
remain. Construction keeps the original 8 GiB allowance and complete opaque foreign coverage.
Encoded source size is not a numeric population or an RSS measurement.

Durable terminal and cancelled-completion phases use the existing admitted out-of-line action
owner. The exact inner future is charged before construction and held until the future is
dropped; construction and polling stay in the same task. No worker-stack increase, task spawn,
effect replay or renewed deadline is introduced. Refusal follows the existing failed-export
drain. Pure layout controls pass with cancellation continuation/core 3,104/373,440 bytes and
terminal continuation/core 3,064/377,208 bytes. Those are future-storage observations, not an
exact reconstruction of the earlier aborting instruction or process-memory measurements.

**Implemented and Tested:** native map execution preserves shared scaling, permitted starts,
authorized restarts and original residual checks using fresh KINSOL sessions. Uno uses named
status constants (code 5 is Numerical), retaining SQP-LBFGS and its four declared numerical hunks.
FERAL preflight supplements actual post-factor retained-storage enforcement. Surrogate workers
retain actual handles through join, including partial construction. Reached POUNCE/Symbolica
fresh-process and embedding-host controls preserve scoped ownership without resizing a host's
global pool. Partial PETSc/KLU hooks are not adopted as a complete allocation protocol.
Residual LM records three independent library criteria, each 30*f64::EPSILON, actual work caps
and exact replay/profile identity; physical stationarity and covariance remain independent gates.
Complete LM eligibility and staged covariance withholding now consume explicit shape, settings,
resource, qualification, bound and numerical-evidence facts. Thin prepared-problem adapters supply
those facts; policies need no Runtime construction. Existing matrix assembly, physical bound
activity, first-refusal order and typed withholding remain unchanged.

**Tested, selected scope; failure baseline zero:** every Arrow-consuming correctness closure
below explicitly activates force-validation. Ordinary builds retain sixteen jobs and uncapped
OS build placement; native scientific pools and reference roles keep their finite declarations.

| Command / mode | Result and boundary |
|---|---|
| `just unit-package pse-math 'not test(realization_probe)' -p pse-compiler -p pse-runtime -p pse-backend-native` | Latest broad default run: 1963 passed/1 failed/9 filtered, 81.208s. The sole diagnostic-fixture failure was corrected; the exact adjacent selection below passes. This is composite acceptance, not a fresh 1,964-test clean run. |
| `just unit-package pse-runtime 'test(/^workflow::local_caller_tests::observation_/)'` | 4 passed/0 failed/904 filtered after isolated-observation repair; completed/interrupted outcomes, original attribution and inline-future-size controls. |
| `just unit-package pse-math '(test(library::admission) \| test(product) \| test(regional) \| test(symbolic_definition) \| test(deep_shared_let) \| test(abort_binding) \| test(native_scalar) \| test(assembly_reuses_exact) \| test(curvature) \| test(construction::)) & not test(realization_probe)' --features pse-math/compiled-evaluator` | 95 passed/0 failed/383 filtered, 2.434s; interpreted/C++ products, original guards, clones, cancellation and actual construction census. |
| `just unit-package pse-math 'test(symbolic_initialization)' --features pse-math/compiled-evaluator --success-output final` | Fresh children exercise licensed/unlicensed × fresh/preinitialized host. Actual licensed requested width 2, unlicensed width 1, preexisting host width 2 preserved. Initial child-marker fixture failure was repaired and rerun. |
| `just unit-native-package pse-tests-conformance pse-tests-conformance/native-acceptance 'test(authored_physical_nlp_)' --success-output final` | Final 2/2 passed, 16 filtered, 392.616s; heater/root/optimization, four further physical cases and both Ipopt/POUNCE flash assertions; fully drained exit0. |
| `just native-python build/plan49de-python-final-20261022 @build/plan49de-python-final-selection.txt -n4 --dist loadgroup --basetemp=build/plan49de-py-final-temp-20261022` | Assembled selection: 51/51 passed, 601.53s, fully drained exit0 after linked correctness-extension rebuild. Contracts, physical/versioned inputs, modeling/recycle, dynamic/transient fitting, LM publication, live results, reload, CLI/export and custody. This predates the final read/terminal/policy repairs; the following current-extension selection supplements it. |
| `just native-python build/plan49de-python-policy-final-20261022 @build/plan49de-python-terminal-selection.txt -n4 --dist loadgroup --basetemp=build/plan49de-py-policy-temp-20261022` | Post-repair installed extension: 4/4 passed, 44.09s, fully drained exit0. Public LM criteria/original qualification, fixed-fit roundtrip and shared results, typed bound solve/admission and exact lazy-study source retention. This overlaps the earlier 51, rather than adding four distinct cases. |
| `just py-unit -n4` (retained selected run arguments in `build/plan49de-python-units-20261022.log`) | 235 passed/0 failed, 15.63s; earlier unchanged Python unit scope. |
| Managed reference six-test selection below, declared 16-lane reference profile | 6/6 passed, 0 failures/1,488 filtered, 210.792s; normal terminal and drained shutdown. Five sixteen-owner controls plus isolated single-owner primary death. |

The first managed reference selection passed 3/6 in 235.981s. Publication and successful
sixteen-owner execution exposed native SSI read conflicts escaping the worker loop; mutations
already used bounded retry. The second selection passed 2/6 in 255.589s, exposing four remaining
readiness/settlement reads and a Tokio-worker stack overflow during durable cancellation.
The twenty reached complete read queries now use that same typed
32-attempt/original-clock owner. Uncertain, malformed, cancelled and other errors are not
replayed. The disconnect fixture now clears its physical-program cache and joins protected
release before closing its original transport; native work remains independent. Both failed
reference engines were stopped only after authentic process/context drain, leaving their logical
unknown effects and failed terminals unchanged.
After the terminal repair, the third selection passes 5/6 in 200.806s (run
`f3ea848f-0bbe-4171-8d5c-571edc613e07`), including four sixteen-owner lifecycle controls
and the isolated single-owner primary-death control.
Its publication test reaches result retirement, then incorrectly excludes the first cleanup page
returned by `forget_run_results` from its exact two-batch total. The corrected fixture includes
that returned page, retaining bounded completion, inaccessible producer results, independent
graph and escaped Arrow assertions. The third engine stops through normal drained shutdown.

**Tested, final reference acceptance:** run `930b30db-4b20-49ee-a542-7047893e79b3`
passes all 6 controls in 210.792s, 0 failures/1,488 filtered, followed by normal quiesce and
drained stop (instance `991e17bf-26b1-4e44-a36e-625070c17ce0`). The original production
Surrealism artifact retains its strict 1s timeout, 64MiB memory and pool 4 limits. The reference
supplies sixteen native lanes within its declared finite pool; constructor admission retains its
8GiB allowance. Initialization, start, tests and stop share one reference owner. The exact selection is:

```bash
just native-test --managed-primary-route -p pse-runtime --lib -E 'test(/^workflow::study_tests::(managed_primary_sixteen_survive_caller_disconnect_until_explicit_cancel|managed_primary_worker_stop_joins_sixteen_native_owners_without_cancelling_study|managed_primary_study_enters_sixteen_native_owners_and_retains_original_results|managed_primary_study_cancellation_drains_admitted_sixteen_and_refuses_tail|managed_primary_publication_serves_exact_partial_history_and_escaped_arrow_retirement|managed_primary_death_returns_incomplete_study_without_fabricated_terminal)$/)'
```

The other focused selections cover 22 final compiler projection/consumer controls plus 4 retention/
generation controls, 20 runtime lifecycle/parameter/horizon/LM controls, 20 native backend controls,
5 Uno FFI controls, 2 retained-worker controls and 3 default-unlinked refusals. Vessel fitting across
four kinds and the unidentifiable-fit control passed in the earlier composed scientific run;
its original physical aggregate failed on false constructor charging and is replaced only by the
subsequent successful final physical selection above. Guarded-product census is implementation
characterization, with no comparative preparation/evaluation speed or memory-capacity claim.
Raw selections, native provenance, terminal/drain facts and logs are retained under `build/` and
its assessment/native-test owners; incomplete earlier runs are not relabeled successful.

**Tested:** after the terminal-phase repair, the following linked-native selection passes 6/6,
0 failures/1,482 filtered in 21.864s, with normal terminal/drain (run
`a303bece-84c4-481d-a367-e3caff24db6e`). It preserves original completion through cancellation
at seal, exact file-reservation settlement, pre-entry refusal and late-heartbeat/partial-prefix
reopen, alongside the allocation-before-construction/drop and future-layout controls.

```bash
just native-test -p pse-runtime --lib -E 'test(physical_publication_action_tests) | test(canonical_complete_export_cancelled_at_seal_preserves_original_completion) | test(canonical_cancelled_file_reservation_settles_the_exact_study_attempt) | test(canonical_preentry_refusal_settles_only_acknowledged_cancellation) | test(canonical_cancelled_prefix_and_late_heartbeat_reopen_acknowledged_unavailable_science)' --success-output immediate
```

**Tested:** the explicit-policy selection below passes 19/19, 0 failures/1,475 filtered in
0.786s, with normal terminal/drain (run `30d1f33b-a893-47f7-a4e8-ec43a662c20f`). Six direct
controls construct no Runtime; thirteen retained controls exercise actual eligibility adapters,
independent LM stopping, typed limits/cancellation and qualified or withheld native covariance.
Initial test compilation rejected a qualified `scalar!` identifier; the imported identifier fixes
it. One redundant test qualification was then removed; final all-target Clippy owns zero-warning
confirmation. Neither correction changes the policy or numerical assertions.

```bash
just native-test -p pse-runtime --lib -E 'test(eligibility_policy_tests) | test(covariance_policy_) | test(fitting_residual_eligibility_preserves_required_native_contracts_before_callbacks) | test(fitting_residual_dense_workspace_eligibility_precedes_numerical_preparation) | test(fitting_lm_) | test(linear_regression_covariance_analytic) | test(covariance_withheld_)'
```

**Tested, static/manual scope:** `just clippy` passes default and no-default workspace/all-target
modes with `-D warnings`, including the final observation and explicit-policy fixtures; `just lint-solver-contracts`
passes linked-native workspace/all-target Clippy, 0 warnings/errors. `just typecheck` reports
0 diagnostics (3 existing suppressions); Ruff passes. The initial hygiene's 3 E501 findings and
one test-only unreachable macro were repaired and the failed leaves rerun. Other hygiene leaves
passed, including generated output, imports, contracts, Rust documentation and family pins.
`just governance` passes. Depth-two `just features-powerset` passes 420 combinations plus 34
no-default checks, 0 warnings/errors; it predates the final constructor/projection/read/terminal/
policy repairs, whose affected modes have subsequent default/native/C++ coverage. No feature or
dependency changed after that matrix.

The focused native command receipts are: backend 20, run `c90ba1b4-0d74-4095-a866-cbaffd88cb27`;
Uno 5, run `49d1bf80-00a7-4949-af37-d424c67dbe75`; retained-worker 2, run
`28c93e2f-6dea-4dcf-afdb-572ebb17c398`; default-unlinked 3, run
`5e12924f-a4c8-46e0-bb47-9a64dc1c047f`. Each reports 0 failures/errors against baseline zero.
These selections overlap broader controls and are not an additive grand total.

```bash
just unit-native-capability-package pse-backend-native native-solvers solver,klu,isolation,uno,petsc 'test(/single_thread_batch_stays_on_caller|selected_capabilities_match|map_seam_preserves|maps_keep_physical|picard_executes_constant|feral_symbolic_peak|retained_kkt_envelope|library_rungs_act|authorized_recycle_restarts/) | test(uno::tests::)'
just unit-native-capability-package pse-uno-sys link solver,klu,isolation,uno,petsc 'test(/recoverable_trial_uses_native_trust_region_retry|bounded_sqp_solves_and_reports_effective_curvature|bounded_slp_solves_and_reports_effective_curvature|operator_storage_limit_is_terminal_before_matrix_allocation|scope_stop_precedes_native_start/)'
just unit-native-capability-package pse-runtime native-solvers,canonical-tests solver,klu,isolation,uno,petsc 'test(/retained_pool_joins_actual_workers|retained_pool_constructor_failures/)'
just unit-package pse-backend-native 'test(/unlinked_native_return_has_explicit_observation_coverage|published_capabilities_equal_routing_rules|selected_capabilities_match_native_methods_and_refuse_foreign_settings/)'
```

**Tested:** the final pure transport regression command
`just unit-package pse-operations 'test(/^canonical::transport::tests::/)'` passes 23/23,
0 failures/78 filtered, 10.042s (run `57b1fcb9-81db-43e9-bdd2-dee6377b9cb1`). It includes the
new conflict/original-clock/exhaustion/non-replay controls. A stale joined-page fixture expected
4,608 nodes after the earlier selector retirement; its actual declared grant is 8,704. Correcting
that expectation retains all 64-row decode/ordinal/minmax checks and the 2,048-node refusal.

The configured native setup suite initially passed 1,259/1,260 in 176.148s. Its one remaining real
termination-control fixture omitted the required module/server identity; after correction the
exact control passed 1/1 in 1.325s with normal native drain. This is composite 1,260-control evidence,
not a rerun of the entire suite. Earlier unconfigured/inotify-cascade failures retain their scopes.
Compile-heavy aggregate entry defaults now preserve the configured sixteen-core owner; the exact
recipe precedence/borrowed-owner control passes. The boundary launcher now starts with checkout
Python 3.14 instead of system Python 3.12 before archive inspection; `just ready`, exact shellcheck and real
fresh reference initialization pass. `just docs` publishes 590 chapters with scoped search.

**Tested, deployment handoff:** the selected ordinary instance
`0a7d4aa4-fee0-429a-a95f-fa224a749dc7` is restored, authenticated, open and accepting writes,
with no parked state. Its production module remains
`f13446a6bf52aff8835b160d82367e67dd9445cec35997582dfbc701951b8282`
under strict 1s/64MiB/pool 4 declarations. The final reference instance is stopped and quiesced.
The checkout still selects the ordinary Surrealism state, sixteen compiler jobs and unbounded
ordinary builds. No owned worktree remains to merge; selected test owners completed their drains.

### A mistake made and corrected

The first static-package repair used mutable tracked owners and broke existing A/B/A counters;
the final tracked producer returns immutable complete-content owners, retaining cross-parent
sharing assertions. Initial constructor estimates conflated encoded bytes with numeric storage
and formal inventory with reached occurrences. Real science exposed both false refusals; actual
AST/literal/Taylor/exported accounting fixes them without widening scientific allowances.
Recycle export incorrectly looked up tear coordinates among original variable ports; it now
resolves the admitted flow declaration and preserves report order, quantity/unit and role.

The public fixtures contained stale declaration spellings, missing standalone versions and
standalone versus per-experiment physical member identities. They now use actual compiled
registry/reflection and authored integration controls; analytic values, sensitivity, fit and
original-space assertions remain. A later broad run exposed nondeterministic global diagnostic
admission in an outcome test. Its isolated unregistered diagnostics fixture preserves the exact
Completed/Interrupted assertions and production best-effort/loss behavior.

The first policy extraction stopped at request/workspace helpers: complete LM eligibility still
read Runtime, and covariance withholding still required prepared-problem fixtures. Reconciliation
against DA-021 caught that unfinished original obligation before closure. Complete input-based
gates and direct policy controls now complement the retained actual fitting/covariance tests.
The publication fixture also discarded retirement's first returned cleanup page; accounting for
that page preserves its exact two-batch assertion rather than weakening the expected drain.

### Deviations from the plan, deliberate

Flash is independently selectable with all original assertions. The production-sized canceled
native sweep stays canceled; selected functional cases supply this scope's acceptance. MAT06
and I14/I15/I17/I22/I23/I25 retain their explicit bounded adoption/no-adoption decisions, rather
than adding speculative JIT/device/streaming/hooks/history work. No extra execution daemon,
global pool reset, historical importer or weakened scientific clock is introduced.

The maintainer restored inotify 524,288 watches/1,024 instances after authenticated user-manager
watch exhaustion. No teardown deadline was widened. Under the maintainer's explicit disposal
selection, only obsolete invocation `d63c4731f12e4cc7b2a0d7eef19f7fac`'s authenticated dead scopes,
unfinished guard and allocation were retired; the poisoned old store was stopped after all 103
registered worker inventories were observed absent/drained. Private operator receipts remain in
`build/plan49de-closeout-private/`. This supersedes the earlier preservation instruction for that obsolete invocation. No terminal
outcome, acknowledgment or scientific success was invented; historical unknown creation effects
remain incomplete and unrelated owners remain.

Independent 49f–49h, broader model-library/parity/performance campaigns, new platforms and
release/distribution qualification remain open. Proposed ADR-0215 remains formally proposed.
Completed companions remain retained for active readers, with enduring meaning at blueprint
§14.3/§14.3.2/§18/§19 and qualification at §24.2.

## Plan51 overlap reconciliation — 2026-10-23

Plan51's delivered project/research/evidence, lifecycle and logical-exchange consumers compose
this series' existing canonical transactions, protected result readers, source/physical admission,
Salsa preparation and native mathematical execution. Enduring workbench contracts live at
blueprint §19.4/§20.6/§24.1 and the substrate/diagnostic guides; Plan51 owns its selected acceptance.
This overlap does not close 49f–49h's independent obligations or TOOL08's unresolved native
traversal/work bound. No all-path service, canceled native sweep, speculative new Z3 selection
or withdrawn environment/provenance campaign is reopened by the workbench delivery.
