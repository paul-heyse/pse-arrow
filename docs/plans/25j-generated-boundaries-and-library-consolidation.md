---
title: "25j: Generated boundaries and library consolidation"
status: draft
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s09]
---

# 25j: Generated boundaries and library consolidation

## Context and target

The [series coordinator](25-design-remediation.md) owns dispositions. This plan owns F18,
the generator/cardinality parts of F26, and the generated consumers of F24/F32. R9 governs
the remedy: a generated shape alone does not establish semantic admission.

Today hand-written Python mirrors, JSON documents, string enum getters and adapter-local
request rules can diverge from Rust. The target is one owned request/result meaning, one
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

These are **Proposed** decisions. The Python-boundary change uses an ADR plus design review
under D1/blueprint §21; generation continues through the existing schema/document route.
No accepted ADR is edited in place and no new crate is required.

## Packets and dependencies

| Packet | Prerequisites | Responsibility and target | Deletion | Status |
|---|---|---|---|---|
| <a id="j1"></a>J1 Owned document and admission boundary | F1/F2/F3; E2/E4; C1 | Move adapter-owned request meanings into the Rust owner; register actual request/results with generation; use one typed document bridge | Ad hoc JSON shape construction, adapter policy and duplicate schema definitions for migrated operations | planned |
| <a id="j2"></a>J2 Python consumer cutover | J1; F2/F3/F4, E2/E3/E4 and C1 interfaces as applicable; G2 for changed persisted representations | Generate models, enums and native stubs; migrate all Python/public Rust boundary consumers and defaults | Hand-written mirrors, string enum getters, parallel wire types and copied defaults | planned |
| <a id="j3"></a>J3 Generator/library consolidation | F1 and I1 utility contracts | Generate common vocabulary surfaces; use adopted graph/cardinality mechanisms; review regenerated outputs | Bespoke emitter DFS, repeated enum glue and quadratic cardinality/uniqueness mechanisms | planned |

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

## Proposed acceptance and handoff

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

## Outcome (recorded after implementation)

### What was built

Not implemented; this document records Proposed changes.

### A mistake made and corrected

Record an actual implementation correction at closure.

### Deviations from the plan, deliberate

None recorded; decision changes follow their owning ADR/design route.
