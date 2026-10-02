---
title: "25j: Generated boundaries and library consolidation"
status: done
date: 2026-09-30
adrs: [ADR-0151]
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s09]
---

# 25j: Generated boundaries and library consolidation

## Context and target

The [series coordinator](25-design-remediation.md) owns dispositions. This plan owns F18,
the generator/cardinality parts of F26, and the generated consumers of F24/F32. R9 governs
the remedy: a generated shape alone does not establish semantic admission.

The reviewed baseline had hand-written Python mirrors, JSON documents, string enum getters
and adapter-local request rules that could diverge from Rust. The target is one owned request/result meaning, one
admission operation, and mechanically derived Rust/Python transport and documentation.
Python stays a thin typed adapter; it does not become a second scientific or workflow authority.

The boundary migration covers workspace/publication/export documents, study status and
cancellation, publication settlement, native attempts and diagnostics, flow selection and
closed candidate reasons. It also covers the newly settled overlay, execution, sensitivity,
endpoint, incumbent and failure contracts from the other plans. Do not limit the migration to
the examples that happened to be counted in the review.

## Decisions and consumed contracts

- Request/result structures belong to the Rust operation that consumes or produces them.
  Move flow-selection ownership and uniqueness/admission rules out of the Python adapter.
  The generator consumes those actual owned types, including enum discriminants, defaults,
  optional fields and semantic constructor boundaries.
- Preserve the repository's chosen generated Python model conventions. The appropriate
  generation source is the owned Rust document or registry declaration; writing a parallel
  schema merely to generate a mirror would retain the defect.
- Route structured documents through the common typed serialization/decoding boundary.
  Return generated enum types from getters, including termination, backend, qualification,
  diagnostic code, class, severity and settlement. Human-readable explanations remain separate
  fields, not the wire representation of a closed reason.
- Generate CandidateReason from the canonical registry vocabulary selected by 25e/25f.
  This plan consumes the reason and projection semantics; it does not infer a result from
  a convenient enum name or message.
- Defaults come from Rust-owned request admission. Generated Python constructors may expose
  them mechanically, but omitted and explicit values must reach the same owner. In particular,
  initialization/homotopy defaults must not be copied back into Python.
- Use the reviewed library replacements for bounded glue: petgraph topological ordering in
  the Python emitter, attrs cardinality validators, and linear-time uniqueness using the
  contract's admitted equality/key. Preserve deterministic output order. A uniqueness
  implementation must support the admitted element domain: hashable values can use a set;
  structured values use a type-directed equality-consistent key rather than becoming newly invalid.
  Defining that key is implementation work, not an existing facility assumed here. Do not reuse
  I1 content identity: it distinguishes signed zero, while collection equality can identify
  -0.0 and +0.0. Preserve map and nested-value equality as well.
- Hex conversion is owned by I1, closed-vocabulary semantics/derive policy by F1. J3 migrates
  their generated consumers. This avoids a second utility owner here.

The target was authorized under ADR-0151 and its Accept review at **Proposed** evidence.
The implementation and focused verification below complete this plan's functional scope;
generation continues through the existing schema/document route under D1/blueprint §21.
No accepted ADR is edited in place and no new crate is required.

## Packets and dependencies

| Packet | Prerequisites | Responsibility and target | Deletion | Status |
|---|---|---|---|---|
| <a id="j1"></a>J1 Owned document and admission boundary | F1/F2/F3; E2/E4; C1 | Move adapter-owned request meanings into the Rust owner; register actual request/results with generation; use one typed document bridge | Ad hoc JSON shape construction, adapter policy and duplicate schema definitions for migrated operations | done |
| <a id="j2"></a>J2 Python consumer cutover | J1; F2/F3/F4, E2/E3/E4 and C1 interfaces as applicable; G2 for changed persisted representations | Generate models, enums and native stubs; migrate all Python/public Rust boundary consumers and defaults | Hand-written mirrors, string enum getters, parallel wire types and copied defaults | done |
| <a id="j3"></a>J3 Generator/library consolidation | F1 and I1 utility contracts | Generate common vocabulary surfaces; use adopted graph/cardinality mechanisms; review regenerated outputs | Bespoke emitter DFS, repeated enum glue and quadratic cardinality/uniqueness mechanisms | done |

J1/J2 proceed by complete operation slices after their owner is ready; they need not wait for
every operation before starting. No slice retains an old and new public API in parallel.
The coordinator records these packet dependencies; compilation and focused codec/admission
checks accompany the migration. Generate whenever a declaration/generator changes, using the
recipe-owned bootstrap path when generation itself depends on newly changed contracts.

### J1 — Shape and semantic admission

**Implementation vision.** The operation has a decoded transport structure and, where context matters, a distinct admitted
product. Generation exposes fields, discriminants, defaults, enum membership and syntactic
constraints. The native owner resolves identities and validates cross-field/contextual conditions,
returning a request bound to its selected revision. A Python flow-selection constructor can check
shape; only runtime admission establishes that its connections belong to the model and its
strategy is supported. The boundary inventory maps every exported operation to this owner,
admission operation and generated request/result/diagnostic products. Source-authoring omissions
and aliases remain governed by their source schema, not a hydrated-document substitute.

Separate decoded transport from admitted requests where the operation has contextual invariants.
For example, a flow selector is decoded once, then admitted against the chosen model; an overlay
is resolved and physically checked by 25f/25a. Python must not supply an independently implemented
version of either validation.

Inventory the actual public exports and callers, using F18's examples as leads. Every moved
document retains its typed cause, source attribution and operation identity. Errors before a
result exists must still decode as the intended diagnostic/failure contract. Do not require a
valid scientific result just to serialize a refusal.

### J2 — Consumer behavior

**Implementation vision.** The final pipeline is owned Rust declaration → schema/document registration → generated Python
models/enums/stubs → typed codec bridge → native admission/execution → generated result or
diagnostic decoding. Python performs mechanical conversion, not another quantity conversion,
route decision or default choice. Unknown discriminants/malformed fields fail decoding; physically
or contextually incompatible values fail native admission with F1's envelope. Omitted and explicit
defaults have the same native behavior. Partial runs carry a typed decision and available output
roles; cancellation and pre-result refusal remain distinct products. Builders, getters,
exceptions and readers migrate as one operation slice.

Migrate request builders, handle/result readers, exception conversion, getters and stub
signatures together. A success path alone is insufficient: partial execution, cancellation,
pre-solve refusal, failed attempts, qualified incumbents and event-ended trajectories all
exercise different portions of the interface. Persisted representation changes consume G's
explicit migration rules; public API cleanup does not silently upgrade stored artifacts.

Named-policy renames remain entity changes. Any convenience path input resolves against the
selected revision before persistence; generated Python must not promise general rename
stability. Preserve source/display data as attribution without making it semantic identity.

### J3 — Small libraries with unchanged contracts

**Implementation vision.** Vocabulary declarations provide member spellings/descriptions; common strum-based mechanics
provide enumeration, parsing and display. Semantic projections still come from their operation
owners. The emitter's dependency declarations form a graph yielding a deterministic order or an
attributable cycle, not hidden recursive traversal state. Collection declarations supply element
type, cardinality and equality. Emit length checks directly and derive an equality-consistent
uniqueness key for each supported element family. Signed zeros and nested/reordered maps must
follow collection equality, not content-hash identity. Regenerated consumers then preserve both
accepted values and refusals while obsolete utility implementations disappear.

The original review contains pinned fit evidence for strum, hex, petgraph and attrs. Check
their selected-release capability records when implementing; use current documentation where
API details need confirmation. Library adoption must preserve unknown-member errors, const
surfaces where consumed, lowercase canonical literals and deterministic generated ordering.
A cycle in declaration dependencies remains an attributable generator error.

Preserve the distinction between source-authoring schemas and hydrated Rust document schemas;
schemars output for an admitted document cannot automatically replace a source grammar schema.

For collection uniqueness, test duplicates of each admitted element family, including scalar and
nested signed zeros and maps with reordered entries. A faster
implementation that refuses a formerly valid structured element is not equivalent. Remove the
old mechanisms and their mechanism-specific tests; retain semantic contract controls using the
new owner. Regeneration is functional work, not deferred polish.

## Consumed 25d prerequisite slice

**Implemented/Tested, 2026-10-01:** J2's required mathematical-response slice is integrated.
Registry-owned selector documents, Root sensitivity admission and local-validity/withheld-reason
fields are regenerated into Rust/Python/schema/document consumers. Public Rust tests exercise
response publication and withholding; linked Python compilation checks the generated boundary.
This is not Python workflow execution qualification. Wider J1/J2 request/default migration and
J3 generator/library consolidation remain open. [25d Verification](25d-mathematical-realization-and-response.md#verification)
owns commands, conditions, composite results and limits; this does not close J2.

## Consumed 25e prerequisite slice

**Implemented/Tested, 2026-10-01:** Required J1/J2 execution consumers use Rust-owned declared admission and optional initialization overrides, with the registry-generated route/procedure/endpoint/qualification vocabulary and actual PyO3 stubs. Python inspection/direct initialization/study and conformance admission exports pass focused controls. Wider request-document/enum/default generation and J3 consolidation remain open.
[25e Verification](25e-declared-analyses-and-qualification.md#verification) owns commands,
conditions, composite results and limits; this does not close the companion plan.

## Consumed 25f prerequisite slice

**Implemented/Tested, 2026-10-01:** J1/J2 expose Rust-owned StudyRequest, StudyDefinition,
operation/binding documents, status/outcomes and full diagnostic envelopes through generated
Python/schema/native boundaries. The replaced case-array study API and handwritten mirrors are
deleted. Closed records/tagged variants, source-owned semantic vocabulary, fixed-array lengths
and heterogeneous tuple positions retain their constraints in generation; diagnostic observations
preserve explicit nonfinite evidence. Focused codecs, generator controls and Python constructor/
decoder checks are owned by
[25f Verification](25f-studies-diagnostics-and-continuation.md#verification). Wider J1/J2
request/default migration and J3 graph/cardinality/strum mechanics remain open. The consumed
semantic vocabulary slice does not close F24 or the full enum-mechanism consolidation;
cross-language workflow qualification remains 25k.

## Consumed 25g prerequisite slice

**Implemented/Tested, 2026-10-01:** Registry declarations generate migration lineage,
versioned descriptor root inventory, reset/scan identities and orphan/reset relations with
closed dispositions. Generated operational statements cover the shared protection fence and
clock-based lease expiry. Rust/native consumers compile against the regenerated boundary;
[25g Verification](25g-durable-contract-evolution.md#verification) owns commands and limits.
This completes the required durable-evolution boundary slice, not J1/J2's full request/default
migration or J3's remaining generator/library consolidation. Cross-language qualification remains 25k.

## Focused acceptance and handoff

Focused checks use isolated Rust request/codec units and Python constructor/decoder units,
with no unrelated database or solver. Check:

- Success, refusal, cancellation and partial-result documents preserve fields and typed reasons.
- Every enum getter/stub exposes its generated enum; unknown input is refused by the owner.
- Omitted initialization/default options equal explicitly supplied Rust defaults.
- Flow-selection duplicates and incompatible overlays are refused identically through each API.
- Enum/optional-field changes regenerate all consumers; unsupported persisted versions take G's
  explicit refusal/migration path.
- Generator dependency cycles are diagnosed; stable input yields stable output; cardinality and
  uniqueness preserve semantics for scalar and structured elements.

Use current targeted recipes and explicit force-validation where required; no full Python,
integration, lint or publication campaign runs per packet. The complete cross-language journeys
are executed once in [25k](25k-integrated-qualification-and-closure.md).

The handoff is a single generated public boundary over the settled target. An empty search for
JSON macros or mirrors can support deletion accounting but cannot establish that semantics
survived. The coordinator resolves F18/F26 only after all their component obligations are met.

## Verification

**Implemented/Tested, 2026-10-02.** The complete exported operation inventory now uses its
actual Rust owner and the common typed native codec. The historical prerequisite receipts
above retain their original dates, scope and exclusions; this receipt completes the remaining
J1/J2/J3 scope. The failure baseline is zero. Rust unit recipes explicitly enabled
`pse-relations/force-validate`; licensed compiler controls used `direnv exec .`, the memory-cap
wrapper and one test thread. The Python controls used the rebuilt locked editable dev
extension with `force-validate,native-solvers`, and `py-unit-native` supplied the native
environment and memory cap.

| Command and mode | Result against zero | Scope established |
|---|---|---|
| `just unit-package pse-codegen 'test(codegen::documents::tests)'` | 7/7 passed | Closed document/tag/tuple shapes, variable cardinality, declared nested equality keys and uniqueness emission |
| `just unit-package pse-codegen 'test(codegen::python::tests)'` | 3/3 passed | Deterministic graph dependency order; attributable cycles/missing declarations; nested declaration order |
| `just unit-package pse-vocabulary 'test(vocabularies_have_one_spelling_per_member) \| test(spellings_unchanged)'` | 2/2 passed | Common vocabulary mechanics preserve canonical spellings |
| `just unit-package pse-schema 'test(binary_literal_requires_canonical_lowercase_pairs)'` | 1/1 passed | Canonical lowercase binary literals and malformed-input refusal |
| `just unit-package pse-compiler 'test(reuse_tests) or test(modeling::conditional) or test(checked_compound_guards_preserve_original_arithmetic_occurrences) or test(workspace::modeling::flow::boundary_unit::) or test(kernel_flow_projection)' --test-threads 1` | 13/13 passed: J flow 4, I retained-body/occurrence controls 9 | Actual flow projection plus duplicate/context/cost admission; the additional I controls retain their owning [25i](25i-identity-reuse-and-resource-ownership.md#verification) scope |
| `just unit-package pse-runtime 'test(workflow::controls::boundary_unit::) \| test(workflow::modeling::documents::boundary_unit::) \| test(workflow::fitting::documents::boundary_unit::) \| test(workflow::diagnostic_documents::boundary_unit::) \| test(workflow::progress_documents::boundary_unit::) \| test(workflow::strategies::requests::boundary_unit::)' --test-threads 1` | 10/10 passed | Rust omission defaults, selected-source penalties and fixture admission, grouped native diagnostic evidence, exact/nonfinite progress, fit worker/scientific distinction, duplicate selected-set refusal |
| `just unit-package pse-runtime 'test(initialization_and_diagnostic_durations_keep_closed_standard_encoding)' --test-threads 1`; `just unit-package pse-backend-native 'test(dynamics_profile_duration_schema_matches_closed_serde_representation)' --test-threads 1` | 1/1 + 1/1 passed | The shared duration schema matches actual owners' standard serde fields, bounds, overflow and unknown-field refusal |
| `just codegen` | Passed all 6 targets and Python candidate annotation checks; Ipopt/hakari refreshed | Registry consumers, actual owned schemas/documents and Rust-serialized boundary fixtures regenerated from source |
| `just check` | Final all-target compile: 0 source errors, 0 source warnings | Connected H/I/J production, tests and benchmark consumers compile together |
| `just py-sync-native` | Passed editable dev rebuild and actual compiled-stub generation | Current optional native consumers, installed extension and generated stubs agree |
| `direnv exec . just py-unit-native python/pse/tests/test_native_boundary_contracts.py python/pse/tests/test_generated_contracts.py python/pse/tests/test_versions_agree.py python/pse/tests/test_native_stub_surface.py -q` | 49/49 passed, 0 failures | Actual diagnostic enums, generated getter annotations and exact extension/stub surface, native defaults, scalar/nested/map equality and duplicate refusal, duration decoding, exact build/lockfile provenance, and the eight owner-serialized outcome branches |

The connected result is a **composite receipt**, not an initially clean run. The first complete
generation compile had 26 errors: 21 cascaded from a missing direct fixture-owner dependency
and five came from old context/cancellation calls. Repairs then exposed a physical-loader
registry-owner refusal, the missing closure in the standard-duration schema, its production
dependency reference and a missing Debug warning. The final generator passed; the final
duration builds and workspace compile contain no source warnings. The initial workspace
all-target check had 36 fixture/benchmark compile errors and three distinct source warnings;
the allocated owner repairs produced the zero-source-error/warning rerun above. The earlier
flow selection run passed three new controls but failed the existing actual graph projection;
H repaired contextual resolution of retained child-constructor declarations, and the combined
13-test rerun passed. The initial runtime boundary build was blocked by three source-loader
header/limit joins; its ten-test rerun passed after the owning repair. These repairs did not
weaken a contract or retain a parallel public path.

The toolchain still prints an imported `proc-macro-error2 v2.0.1` future-compatibility notice;
it is recorded separately from the zero current source finding result. No broad hygiene,
integration, scientific solver execution, storage/publication journey, parity or performance
campaign was run for this packet. The typed partial/cancelled/refused/failed/incumbent/event
payload fixtures test Rust serialization followed by generated Python decoding; they do not
establish that a solver computed the fixture's qualification. Full product and cross-language
journey qualification remains [25k](25k-integrated-qualification-and-closure.md).

## Outcome (recorded after implementation)

### What was built

**Implemented/Tested, 2026-10-02:** J1/J2/J3 functional scope is complete. Workspace, published
and exported artifacts, settlement, completion/lineage, study and attempt products, strategy
and flow requests, declaration and inspection documents, fitting/profile evidence, progress,
diagnostics, warm-start snapshots and build provenance derive from their actual Rust owners.
One typed native document codec and one mechanical enum projection serve all migrated
consumers. Python builders/readers/exceptions use generated documents and enum types;
genuine native handles and typed settings retain their native capabilities.

Flow selection now belongs to the compiler/runtime flow owner. Decoding exposes occurrence
lists; admission checks duplicates, cost and the selected model's actual graph with its immutable
validation context. Rust owns omission/default resolution, including initialization and selected
run/start controls. The handwritten requirement-planner bridge is deleted while effect,
settings and budget policy remain at their actual owners.

The deleted surfaces include handwritten Python wire mirrors, native progress/incumbent/
resource/cache/route/diagnostic-wire mirror classes, the separate workflow document codec,
adapter-local flow meaning, copied defaults and string-valued enum projections. Their callers
use the generated owner products; no old/new public API is retained in parallel. Standard
duration serialization still belongs to serde; its shared schema projection supplies the closure
that schemars omits. Registered selected sets reject duplicate decoded occurrences rather than
silently collapsing them. Generated attrs and msgspec collection keys preserve equality for
signed zero, records and reordered nested mappings, with declared length constraints.

J3 uses common strum vocabulary mechanics, the shared hex owner and deterministic petgraph
ordering. Regenerated models, schemas, docs and actual PyO3 stubs complete the cutover.
**Interface-checked/Tested within the named conditions:** the final connected compile and
49 isolated Python tests establish the exported boundary and codec behavior, not execution of
unselected scientific or storage workflows. The historical prerequisite slices retain their own
qualification limits; 25k owns the remaining comprehensive qualification.

### A mistake made and corrected

**Implemented/Tested:** moving documents to their real Rust owners exposed native import and
typed-hash joins that had relied on adapter-local representations. Completion/publication now
lower typed revision, request and profile identities with `as_id()` only at generated persisted
row boundaries; the native import gateway installs identity helpers before loading generated
documents. Connected native compilation and the checkout metadata/codec tests pass. The
initial duration schema also understated serde's refusal contract; the fix reused one shared
projection of the actual standard-duration representation rather than making the emitter accept
open records. The final owner and Python controls verify that correction.

### Deviations from the plan, deliberate

None. ADR-0151 and its target review govern the boundary change. Rust-serialized isolated
fixtures were added to cover every named outcome branch without invoking a solver or store;
this implements the planned bounded acceptance scope rather than expanding qualification.
