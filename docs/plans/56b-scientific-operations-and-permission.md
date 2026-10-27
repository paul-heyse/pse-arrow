---
title: "Plan 56b: Scientific operations and permission"
status: in-progress
date: 2026-10-25
adrs: [ADR-0227]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, numerical-execution, validation]
---

# Plan 56b: Scientific operations and permission

## Completion audit — 2026-10-27

**Interface-checked:** source `57cce1756318357a5797f99e83f1ad9115661368` and accepted
B01–B06 scope, including ordinary nested/provider/trajectory consumers. B01–B05 remain complete
at their recorded bounded conditions. B06 is incomplete for [CA01](56-design-principle-alignment.md#ca01),
which reopens DA-039 in the coordinator. The earlier B01–B06 blanket closeout is corrected here;
its historical tests and original failures remain unchanged.

`pse-math/src/implicit_products.rs` creates `MathError::Contract` for poisoned preparation,
provider and product mutexes. The equivalent residual-provider path in `implicit.rs` creates
`MathError::Library`. Actual `ImplicitProvider::prepare_demand` and `demanded_derivatives` callers
reach these constructors. Nested provider errors preserve the original diagnostic: Contract maps
through `MathContract` to `InvalidModel`, while Library maps to infrastructure. Runtime strategy
therefore reports contract failure for one real infrastructure failure and operational failure
for its equivalent peer. Typed host remedies consume that incorrect producer fact. The existing
runtime synthetic `MathRuntimeError::Internal` control does not exercise the poisoned owners.
Related migration residue maps trajectory allocation errors and a pure compiler task join failure
to `WorkflowError::Input`; include those real operational constructors in B06's finishing scope.

**Remaining implementation and acceptance:** preserve typed operational causes and source attribution
through the real constructors/nesting/runtime projection. Poison actual provider/product preparation
owners and exercise residual/product/preparation paths, asserting equivalent classification and
redacted remedies. Cover the allocation/join constructors at their owning boundaries. Do not add
message matching or another validation layer. No permanent lazy refusal cache was established:
these generic errors are outside the deterministic cache whitelist and are returned, not persisted
as `ScientificRefusal` by the inspected study path.

**Evidence limits:** source inspection establishes the missing migration; no new scientific tests,
compilation or provider runs were needed. The recorded Rust/Python results remain composite original
passes plus affected reruns, not a new clean full invocation. Permission, replay, native capability,
service custody and physical-context results remain supported. No measured benefit is added.

## Purpose and assessed foundations

**Implemented:** completion-owned scientific permission governs every aggregate, derived product
and public/recovered consumer; each analysis kind owns its lifecycle facts. This companion
addresses DA-009/017/021/038/039. [Plan 56](56-design-principle-alignment.md) owns scope,
disposition and decisions. The accepted implementation and selected local qualification are recorded
in the Verification and Outcome below; the original audit retains its diagnostic baseline.

Reuse the pure `workflow/numerics.rs` decisions, shared `RunSupervisor`, typed `RunOutcome`,
immutable preparations, original assessment and class-specific native adapters. Keep runtime's
scientific-workflow responsibility under ADR-0175. The present weaknesses are consumers deciding
again and broad internal reach, not proof that all science needs a new crate or policy engine.

## Target: owned facts with distinct scopes

| Fact | Owning meaning |
|---|---|
| Candidate permission | `numerics::{native_use,original_completion,complete}` decides result/seed use from actual evidence and policy |
| Original aggregate permission | Completion owns requested coverage, successful original report and complete nonempty required assessments |
| Product-local mathematical validity | Fitting/local analysis owns rank, approximation, stationarity/statistical prerequisites and applicable PSD evidence |
| Product permission | Governing original permission and the required local validity, retaining reasons and qualifiers |
| Estimate qualification | Existing `estimate_qualified` keeps its separate stationary/identifiable point-estimate meaning |
| Publication/recovery | Durable owners state committed/retained/unresolved effects; they do not grant scientific permission |

Select the conservative existing invocation/`RunResult::usable` rule. A failed original fit may
retain computed covariance/interval bytes for diagnosis, but cannot expose them as qualified fit
inputs or publish a contradictory usable-product claim. Do not relabel the bytes as external
covariance, equate covariance validity with estimate qualification, or invent conditional/partial
scientific reuse. A horizon publication failure retains authentic prefix evidence but does not
promote the whole requested operation through a publication-failure exception.

Completion records carry the governing scope, decisions, reasons and qualifiers, not a new bool
for each consumer to interpret. Version `Completion` and its persisted projections together.
Local results, studies, invocation, Python, result inspection and canonical hydration copy those
facts. Current DA-010 content reconciliation, external provenance, PSD, coordinate compatibility,
allocation and cancellation protections remain necessary independent checks.

### Operation ownership and local reasoning

Keep a closed analysis enum and typed request/report pairing. Each kind supplies request facts
(sources, identity, context/provenance, numerics and tagged replay constructor) and completed
facts/products (permission, native report/trace, start availability, retained extent and projection
inputs). One central exhaustive dispatch invokes the per-kind implementations. Lifecycle readers
do not maintain a new match per question. A plugin hierarchy or universal analysis language is
unselected; ordinary structs, enum dispatch and domain functions are sufficient.

Persist one tagged replay constructor instead of parallel optional constructor fields. A modeling
sequence may own a vector in its variant. Put horizon/modeling-specific payloads with their
outcome variants; eliminate omission wildcards. Preserve host-only shooting versus the persisted
study subset and compact/full horizon distinctions.

Narrow internal workflow visibility by responsibility. Immutable source, physical and prepared
science values cease carrying the whole effectful runtime merely for convenience. Execution
facades supply services when an effect is needed, with explicit lifetime ownership. Remove the
modeling-canonical/fitting declaration cycle by placing common declaration projections with
modeling. This is an incremental responsibility split, not a wholesale runtime rewrite.

### Capability and failure contracts

Publish default-settings capability plus a configured-capability query from the same resolved
adapter admission. Attribute dynamics from the actual method/backend; remove the non-IDAS
wildcard. Recovery-profile availability comes from the adapter seam rather than `Backend::Pounce`
checks. KINSOL retains its accepted fixed-point role. Direct Python settings should remove
always-refused PETSc composed options rather than adding a new workflow just to keep their names.
Settings-sensitive Clarabel/SCIP thread refinements remain explicit.

Construct failures by meaning: request/scientific refusal, unsupported, resource/effort,
cancelled, integrity/infrastructure and internal invariant. Add a math internal representation;
do not mechanically rename all `contract()` calls. Lazy admission may persist deterministic
scientific refusals, but transient resource/time/internal failures must not become permanent
scientific rejection. Effect knowledge controls whether retry is safe; class alone cannot prove
no effect. Diagnostics retain source identities and native class/stage/rule; F consumes them for
redacted remedies and appropriate telemetry severity.

## Packages

### B01 — Governing aggregate and product permission

Extend the existing completion owner and serialize its facts/reasons through C03/C05. Migrate
`RunResult::{usable,invocation_evidence}`, `fitting/results.rs::uncertainty` including own-prediction,
local `parameter_covariance`, persisted `fit_covariance`/reconciliation, study facts, Python
`NativeRunHandle::usable`, horizon completion and result inspection. Both qualified-fit
constructors require product permission; retained diagnostic payload does not itself supply it.
Remove duplicated aggregate conjunctions and `Unknown` used where the producer's actual permission
is available. Missing historical/current permission refuses explicitly; no success is inferred.

### B02 — Route evidence and composite decisions

Make residual LM and NLP routes supply a small common native evidence view to the shared
completion policy; do not fabricate a KKT report for LM. The same incumbent policy applies to the
same evidence, without claiming limited feasible incumbents are stationary or statistically valid.
Extract profile-pin acceptance and horizon move authorization into pure functions. Pins may
strengthen stationarity, but declare their original-check obligations and preserve applicability
qualifiers through interval ends. Keep numerical iteration library-owned.

### B03 — Kind-owned request/completion projections and tagged replay

Migrate `durable.rs::{identities,request_inputs,request_context,request_provenance}`, replay,
attempt documents, start availability, diagnostics, study preparation/binding and Python accessors.
Share declared-procedure dispatch and predicted-start composition where the operation is identical.
Delete superseded projections, optional-constructor conventions and `_ => None` omissions together
with their callers/fixtures. A newly supported kind supplies its owned facts at the dispatch seam.

### B04 — Narrow scientific value/service boundaries

Use B03's consumed interfaces to separate source/prepared state from execution services. Move
fitting declaration projection out of the reverse modeling dependency and narrow visibility of
kind, durability, result and formulation internals. Migrate one coherent owner/consumer slice at a
time; keep existing native, store and cancellation lifetimes explicit. The required value slices
are `ModelingPackage`, `FitProblem` and prepared source/model values: retain immutable model,
physical and source context while passing explicit execution/persistence services at effectful
entry points instead of reaching through `prepared.source.runtime`. Separate project merge/
preparation from modeling semantic internals; kind, durability, project, result and formulation
owners expose narrow projections and restrict visibility to their actual consumers. Formulation
consumes A02 structural facts and explicit observed values; it does not classify targets inside
provider observation. Preserve the public workflow facade where useful without retaining its
whole composition root in scientific values. Completion/composite/formulation decisions must be
exercisable without starting a store or solver. No replacement composition root or crate is added.

### B05 — Resolved capability publication and attribution

Migrate static/configured inventory, backend attribution and recovery-profile consumers. Remove
unreachable direct boundary choices and regenerate their contracts. Keep exhaustive adapter
selection and refusal before effects. No solver replacement or altered numerical algorithm is
part of this package.

### B06 — Typed failures through admission and recovery

Migrate constructor families by owning operation, prioritizing lazy admission, result-inspection
integrity, poisoned providers and missing-input attribution; then cover remaining math/workflow
families within the affected supported operations. Couple native cause construction with C02's
effect knowledge and F03's presentation. Equivalent conditions receive one class across paths;
ordinary observation events do not masquerade as registered diagnostic codes.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| B01 | complete | 56b | Original/product permission agrees across local, durable and public consumers; failed-fit diagnostic bytes retained |
| B02 | complete | 56b | Shared native evidence and pure pin/horizon decisions preserve complete endpoint requirements and qualifiers |
| B03 | complete | 56b | Exhaustive kind-owned request/completed facts and one tagged replay constructor replace parallel projections |
| B04 | complete | 56b | Explicit services replace whole-runtime retention in selected scientific values; binding/issuer and lifetime controls pass |
| B05 | complete | 56b | Adapter-owned default/configured capabilities, recovery starts and concrete dynamics attribution; generated direct settings migrated |
| B06 | in progress — CA01 | 56b | Typed consumers are delivered; actual operational producer constructors and equivalent poisoned-owner acceptance remain |

B01's pure decision can precede the canonical cutover; persisted completion requires C03/C05.
B03 provides early constructor/fact declarations to C before all lifecycle migrations finish.
B02 and B05 can proceed independently after their actual evidence contracts are ready. B04 uses
working projections, not merely an agreed interface. No whole-document barrier is implied.

**56a prerequisite — 2026-10-25:** the pure completion-owned governing conjunction now supplies
`RunResult::usable`; protected canonical result inspection reads the producer's existing Assessment
from the same admitted selection. Missing permission remains explicit; selected-product usability
keeps its separate meaning. This does not version Completion or complete the local/persisted
covariance and downstream permission migration. [56a](56a-model-meaning-and-physical-interpretation.md#verification-and-outcome--2026-10-25)
owns this slice's acceptance. The completed B01/B02 adoption below now supplies DA-038's remaining evidence.

## Revealing acceptance

The acceptance target was to extend `original_fit_failure_withholds_completed_statistical_products_in_invocation`
through result rows, local/persisted covariance and downstream propagation. Assert retained bytes,
agreed refusal and unchanged independent local validity/estimate meanings. Add successful and
external-covariance controls, LM/NLP policy/termination equivalence, missing and partial coverage,
callback errors, applicability-qualified profile pins and horizon publication failure.

Exercise every existing kind through the shared projections; malformed replay combinations refuse,
and host-only shooting never silently becomes a study operation. Compare local and recovered
completion facts. Construct the model/fit/prepared scientific values and exercise their pure
projections/decisions without runtime infrastructure; exercise their effectful execution through
the explicit service boundary as a separate control. Check project/result/formulation dependency
direction and restricted visibility, not a target count of files or `impl` blocks. Pure completion
tests alone cannot close DA-021. Capability controls include
KINSOL fixed-point, Clarabel/SCIP settings-sensitive threading and exact dynamics method identity.
Inject resource/time refusal during lazy admission, internal/integrity failures in inspection,
genuine missing initial conditions and equivalent poisoned-provider errors. Assert typed class,
affected sources, effect/recovery state and redacted remedy, not exception prose.

Targeted tests accompany each integrated change with explicit force-validation for Arrow closures.
G records the selected scientific/native/Python/operator qualification below. No measured speed,
capacity or memory benefit is claimed.

## Verification and Outcome — 2026-10-25

### What was built

**Implemented:** B01–B05 and their required C/F adoption are delivered; B06 remains incomplete as detailed in the completion audit. Completion v2 owns original
coverage/report/assessment permission and separate product permission with required qualifiers.
Covariance, own propagation, invocation, studies, Python and protected stored readers consume it.
A failed original fit retains diagnostic covariance, Wald and profile bytes without granting
qualified reuse. Independent step permission remains separate from whole-horizon permission.
Profile interval permission requires the complete unique two-end inventory, finite bound/threshold
ends and no chain worker failure; a successful collection of stopped chains is insufficient.

**Implemented:** kind-owned request/completed facts replace lifecycle reach-ins; replay v5 has one
closed tagged constructor. Stored completion v3 requires current permission and explicit progress;
historical versions refuse instead of inferring permission. Standalone fit-profile v2 identifies
required chain/point qualifiers and refuses v1 at header admission. Profile points v3 and parameter
intervals v2 retain those facts in relations. Receipt framing and file placement are unchanged.

**Implemented:** preparation/source/publication/execution services supply effects to immutable
scientific values. Selected model, fit and prepared owners retain their data and legitimate leases,
not the whole Runtime. Common binding checks authenticate actual math/registry/native bindings;
durable registration also authenticates the immutable original source issuer before effects.
Independent callers over the same bindings retain valid lifetime behavior after the original
caller closes. Ephemeral execution retains configured canonical reads while suppressing automatic
writes. Common fitting declarations moved to the modeling owner, removing the reverse cycle.
Supported conformance and benchmark callers migrated with their actual fixture resources intact.

**Implemented:** adapters own default/configured capability and recovery profiles; dynamics
attribution is exhaustive over concrete methods. Direct PETSc settings expose executable root
controls; internal scientific composition retains its complete settings and identity. Typed internal,
resource/time, integrity/infrastructure and scientific causes survive wrapping. Admission effects
distinguish no publication, acknowledged source and unresolved publication. Lazy rejection uses an
explicit deterministic scientific whitelist; it does not permanently reject transient/internal
failures. Host remedies consume typed facts and source references.

### Verification and accepted evidence

**Tested — local Linux, baseline zero:** these are distinct, overlapping scopes, not an additive
total or one uninterrupted passing campaign. Rust correctness routes explicitly enable Arrow
force-validation; actual native iteration uses the selected native environment. The retained
`qualification.sh` in each receipt directory contains the exact expanded filters and arguments.
All fixture directories below are under `build/qualification/`, with a sibling `-receipts` directory.

| Command / scope | Actual result and retained conditions |
|---|---|
| Workspace/all-target compile, then supported native graph including `pse-py/native-solvers,pse-runtime/canonical-tests,pse-tests-conformance/native-acceptance,pse-benches/native-process` | Passed after migrating the initially exposed external conformance/benchmark callers; zero errors/warnings |
| `scripts/pse-env --native= -- just unit-package pse-runtime <pure-filter> --execution-effects native-local` | 34 passed, 0 failed with canonical/solver effects disabled; Completion/numerics, strict replay, product completeness, pin/horizon policy, service lifetime, actual poisoned compiler and draft deadline controls. Invocation `ca12ed1b40134cb688dd57d2b4137b57` |
| `scripts/pse-env --native -- just unit-package pse-backend-native <capability-filter> --features native-solvers --execution-effects native-local` | 5 passed, 0 failed: default/configured capabilities, concrete dynamics attribution, PETSc direct document, admitted recovery settings and actual KINSOL dense/SPGMR/Picard solve |
| `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_capability_diagnostics scripts.tests.test_capability_native scripts.tests.test_capability_cli` | 53 passed, 0 failed; schema/remedy/CLI controls, separate from actual receiver evidence |
| Requirement/diagnostic controller unit selection | 22 passed, 0 failed; pure versus canonical/native test requirements and typed diagnostics |
| `PSE_SURREAL_STATE=<owned> just unit-package pse-runtime <selected-filter> --features native-solvers,canonical-tests --profile local` in `56b-canonical-RV7WElox` | 69 executed: 67 passed, 2 failed. Passed scope includes failed-fit local/persisted covariance/reuse, profile/Wald controls, protected permission/hydration, binding/issuer, service/replay/cancellation and typed failures. Failures were stale replay version and an unentered direct-horizon activity fixture; invocation `4604ead4bbac4543b69e9cfa508bd48a` |
| Same route, affected replay and all five direct-horizon controls in `56b-canonical-GlC4fqOB` | 6 executed: 4 passed, 2 failed. The activity correction reached the actual publication-pressure branches; the remaining assertions compared whole and step permission scopes incorrectly |
| Same route, only the two affected horizon controls in `56b-canonical-Rrnk4woG` | 2 passed, 0 failed; every non-governing Completion field compared exactly, exact independent step versus whole coverage/refusal asserted. Invocation `7ce7c58cb80d4e47bbf3465e7dc8c618` |
| `PSE_SURREAL_STATE=<owned> just native-python <output> <selected-local/workflow/boundary/settings/accuracy/canonical-nodes> --basetemp=<owned> -m 'unit or component or integration' -q` in `56b-canonical-RV7WElox` | 80 executed: 71 passed, 9 failed. Failures exposed stale directional document types, attempt phase assertions, missing required qualifiers and a constructor using a different process budget; all corrected without lowering science |
| Same Python route, affected nine in `56b-canonical-GlC4fqOB` | 8 passed, 1 failed; remaining settings assertion expected private exception prose rather than the typed decode diagnostic |
| Same Python route, affected settings plus profile codec and real fixed-fit profile producer in `56b-canonical-Rrnk4woG` | 3 passed, 0 failed; actual producer emits fit-profile v2, v2 qualifiers decode, historical v1 refuses before malformed body decode, invalid direct settings preserve typed location/rule |
| `just py-extension-prepare native dev`; `scripts/pse-env --native -- .venv/bin/python <retained-public-probe> <state> <output>` in `56b-canonical-HY2sh2AV` | Both passed. Installed correctness Python, actual STDIO MCP and derived CLI agree exactly on failed original permission while retained false check bytes remain readable. Native/dev extension SHA256 `aafb653f5db72cd10bbbfe8b01f0541be38657eb58c6e697cb5990e55584a033`; public `receipt.json` records original run/attempt and exact document |

**Tested — scope-end static/manual checks, baseline zero:** `just hygiene` attempted all 24 leaves;
20 passed and 4 failed (types, compiled Python stubs, default/no-default Clippy). Each failed leaf
was repaired and passed on rerun; the original aggregate remains failed. The receipt is
`build/assessment/20261026T013703.178111Z-hygiene-3605190-6cd3a0`.
`just governance` passed all seven checks at
`build/assessment/20261026T011716.593273Z-governance-3537907-0293e6`.
Full `just codegen` published all ten targets; final `just codegen-check` passed all five leaves,
including actual compiled API stubs, at
`build/assessment/20261026T030225.624038Z-codegen-check-3860802-6e6031`.
`just typecheck`, `just clippy-default`, `just clippy-no-default`, `just lint-native-contracts` and
`just lint-native-data` passed. The full supported native workspace/all-target Clippy graph above
also passed with `--locked -- -D warnings`; the final changed runtime/test scope passed native
Clippy with `-p pse-runtime --all-targets --features pse-runtime/native-solvers,pse-runtime/canonical-tests`.
No warnings/errors were waived. `just docs` passed and published 678 chapters;
`scripts/pse-env --docs -- python3 -m scripts.document_lifecycle validate` passed with 1,483 native
scope items and zero errors. `just turn-end` passed both ADR index and formatting at
`build/assessment/20261026T030810.766604Z-turn-end-3879169-42dda8`. Supporting policy/native/host and
static logs are retained in `build/qualification/56b-source-and-static`. Documentation-only closeout
did not repeat product qualification.

**Interface-checked — independent source assessment:** review of `8943cf557` found incompatible
service/issuer admission and stopped-profile product permission gaps. The corrections at
`f164adcd8` were independently checked, then `dfa6f67e1`'s external callers, resources, boxed replay
wire and typed failures were checked with no remaining actionable finding in that scope. A later
focused boundary review identified standalone fit-profile versioning; `07fa43c32` amended the
proposed ADR before `7fc1ad924` migrated catalogue/generated readers and the actual producer tests.
Source review is distinct from functional/scientific acceptance and is not a new broad audit.

**Tested — deployment custody:** all four fixtures used the existing functional profile,
`--no-resident`, genuine canonical server/native worker and random owned ports. Each was initialized
with current producers before admission. Final stop receipts show inactive service, closed writes,
quiesced admission and no owned listener. The first failed horizon fixture retained one creation
borrower: the exact registered owner was authenticated, its original test process was terminal,
and its empty child/reference inventory was drained before successful stop. Databases and original
failed/passing records remain retained. The preexisting selected store
`/home/paul/.local/state/pse-arrow/surreal-plan49-surrealism-20261022` was preserved.

### Mistakes made and corrected

The first integration exposed unmigrated conformance/benchmark consumers and stale test documents;
these were migrated with real resources and typed current facts. Initial policy classification,
replay-version and horizon fixture/oracle failures were repaired at their owners. Horizon oracles
now distinguish independently accepted steps from failed whole operations and settle owned handles
before assertions. The generated profile contract initially retained a v1 header despite requiring
new qualifiers; the wire catalogue now owns v2, including actual-producer and header-first refusal
controls. Public probe setup initially omitted spill storage, then selected too small a pool; both
were corrected. A subsequent real MCP call correctly refused a noncurrent extension because the
selected environment changed `OLDPWD`/`PSE_WORKER_BINARY`; preparing the image in that exact
environment produced the passing public journey. No currency or scientific guard was weakened.

The root mistakenly launched the excluded 505-case `just features-powerset` campaign. It was
interrupted and its authenticated owned allocation drained; it was not restarted. Receipt
`build/assessment/20261026T015814.696354Z-features-powerset-3695775-e2ee5b` remains incomplete,
with combinations interrupted and no-default not run (exit 130). This is not feature-matrix
qualification. Default, no-default and supported native graph lint/compile evidence above retains
its actual narrower meaning.

### Deliberate deviations and limits

Only the necessary C03/C05 completion/replay/fresh-deployment and F01/F03 contract/remedy adoption
was included. Wider canonical declaration/settlement, locator placement, host operation catalogues,
worker/read reuse and observability remain with their Plan56 owners. RC01–RC06 remain pending;
there is no new crate, solver substitution, altered numerical algorithm, payload-inline redesign,
old-data compatibility reader, broad repartition or lowered scientific profile. Existing receipt
framing and physical placement remain. Boxed replay payloads reduce enum size without changing wire
meaning. Legitimate prepared leases retain custody rather than reacquiring a whole runtime.

Acceptance composes original passes and exact affected reruns. No single clean 69/80-case invocation,
full solver fleet/parity, 505-case powerset, benchmark/speed/capacity/memory benefit, desktop-host
attachment, distribution or other-platform qualification is claimed. Public evidence covers an
actual local STDIO MCP session and CLI. ADR-0227 remains proposed; implementation evidence does not
change its formal status. Enduring meaning belongs to blueprint §19.2/§19.4/§19.8/§21.1 and the
selected qualification basis in §24.2. This companion remains in progress and is retained while the active
coordinator and C/F/G packages consume its evidence; it authorizes no further production work.
