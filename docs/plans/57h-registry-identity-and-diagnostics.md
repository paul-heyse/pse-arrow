---
title: "Plan 57h: Registry, identity and diagnostics"
status: superseded
date: 2026-10-27
adrs: [ADR-0210, ADR-0116, ADR-0178, ADR-0224]
review_sources: [docs/library_leverage/reviews/declarations-registry-and-python-boundary.md, docs/library_leverage/codebase/authoring-and-identity.md, docs/library_leverage/codebase/registry-generation-and-python-boundary.md, docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [ design, preparation, diagnostics, libraries, validation]
---

# Plan 57h: Registry, identity and diagnostics

> **Superseded on 2026-10-28 by [Plan58](58-rust-owned-system.md).**
> This document retains historical implementation, decisions and evidence only. Its open
> packages and future-tense instructions authorize no further execution. No scope transfers
> automatically, and supersession is not completion or a passing qualification result.



## Database-boundary review integration

K01/K03 consume responsibility-owned generated durable contracts through native-independent
Python leaves; H retains registry identity, diagnostics and generator authority. Coordinate
H06/I01 generator changes with K rather than declaring another wire schema or hand-editing
outputs. New actor/control errors retain typed diagnostics and source privacy. F07's attribution
changes do not turn diagnostic observations into durable authority. The conditional embedded
registry product remains governed by H04's complete compatibility/integrity/source-equivalence
assessment; the boundary review selects no shortcut or automatic registry cutover.


## Purpose and authority

This companion develops CBO-037, CBO-038, CBO-039, CBO-040, CBO-042, CBO-045 and the Python
templating half of CBO-046 from the [revised G8 review](../library_leverage/reviews/declarations-registry-and-python-boundary.md).
The [coordinator](57-library-leverage-and-correctness.md) owns their one disposition ledger,
operator decisions and combined completion. The package table below owns execution state here.
Authoring this draft authorizes no product implementation or decision-record amendment.

The inspected planning baseline is `93946f572` (product source `4f52fd239`), consuming the complete
updated review, codebase/library inputs and additional-pattern revision 4, with retained probe
evidence integrated at `1540fd164`. Existing evidence keeps its own source, conditions and limits.
Core 3.4, its efficient-architecture heuristics and test-assessment companion, the pse-arrow binding
and process-simulator 1.5 guide this focused foundation assessment. It does not initiate a broad
audit or establish scientific qualification.

The enduring owners remain blueprint §4.1–§4.2/§4.6 (registry and generation), §5 (identity),
§21.5 (Python contracts), §22 (source authoring) and §23.2 (diagnostics). These plans do not change
their meaning. Proposed ADR-0210 must be reconciled before a different hydration carrier lands;
accepted ADRs are immutable. [57i](57i-python-and-developer-workflows.md) owns trusted document
crossings, row/document model preservation and public Python classes. The SurrealQL half of
CBO-046 remains with the operations/transport companion; this document does not replace its
parser, DDL or transaction decisions.

## Current foundation and resulting design

### Source parsing and diagnostic presentation

`pse-authoring` already uses winnow for lexing. Most declaration parsing expresses contextual
identity, type-variable binding and source-span ownership. Its separate canonical renderer is
an intentional inverse, challenged by round-trip properties. Rewriting the parser into winnow
structural combinators or a PEG does not remove those responsibilities. The walker is mostly
exhaustive, but its predicate mutation and free-path wildcard arms can hide new variants; the
hand proptest strategies can omit a new variant without a compiler failure. H03 closes those
small gaps while retaining byte/depth budgets, exact UTF-8 spans, first-error behavior and
deterministic identity assignment. Multi-error recovery is a separate functional capability,
requiring a functional-extension inquiry if an editor/LSP consumer appears; partial ASTs must
not accidentally acquire authoritative identity.

Diagnostics already carry `DiagnosticLocation { document, start, end }` through the typed facts.
Rendering belongs at a caller-local presentation boundary with the corresponding source bytes,
not on each error enum and not inside `DiagnosticReport` or the wire/Arrow envelope. H02 chooses
miette 7.6.0's feature-free `NarratableReportHandler`, avoiding a dependency-feature decision
without sacrificing source context. Build `MietteDiagnostic` with typed projected code/help and
`LabeledSpan`, attach `NamedSource` with `Report::with_source_code`, and render from the existing
structured diagnostic. Group multiple source locations by document; preserve cause order and
typed classification independently of text.

The loader genuinely owns source: `authoring_driver::document::{Document, DocumentBundle}` retains
document identity/path/content and `Document::text()` distinguishes text from binary documents;
`OwnedDocumentSet` retains the allocation owners. `NativeModelingPackage` instead holds a runtime
`ModelingPackage`, whose `SourceRevision` owns an admitted revision. This is not yet proof that
every retained/reopened package has a directly available id-to-text view. H02 must follow that
specific retained-source chain before offering a package render method. It must borrow the
existing authenticated source owner, never reload a current path as if it were the original
revision or create an unconditional second source copy. Load/edit failures can attach a PEP 678
exception note while the boundary owns the failed input. A source-less report remains useful
without an excerpt. An unavailable source on an explicit render request is explicit absence or
refusal, never a fabricated excerpt.

Presentation guards handle missing documents, reversed/out-of-range spans and offsets inside a
UTF-8 codepoint without panicking or slicing invalid text. Prefer refusing an invalid label while
preserving the original report; any clamping must be presentation-only and visibly bounded.
No rendered source is added to reports, agent/MCP projections, logs or durable artifacts. Read
the actual exception-to-agent consumers when wiring notes: private exception text cannot become
an accidental structured-message fallback. No source-bearing exception serialization is licensed
by this improvement.

### Typed codes and identity bytes

`impl_diagnostic!` projects one typed code into miette; the separate static derive codes are the
drift path. In `CanonicalError`, miette currently renders `runtime::infrastructure` while the
typed `Codec` case projects `validation.invariant`. H01 extends the existing macro with optional
`forward`, `help`, `related`, `source` and `facts` clauses, preserving wrapper overrides and child
forwarding. Migrate all ten non-generated derive-code files, classifying the five without typed
implementations and the out-of-vocabulary `Withheld` spelling. Supply a legitimate typed code
where the failure is public; otherwise document the actual private/non-error scope. Do not
invent a new diagnostic family merely to preserve an unregistered string. `disposition()` remains
its separate exhaustive policy projection. Optional `labels`/`severity` clauses are warranted
only by an actual migrated consumer, not as a second presentation model.

H01 also resolves the low-priority vocabulary observation: share the existing declarative macro
from `pse-diagnostics` into its dependent `pse-vocabulary` with optional description/schema/
conversion arms, provided the different serde routes, `ALL` types and String conversions stay
exact. This is a local consolidation, not replacement of strum or registry vocabulary authority.
If the consolidation would obscure these distinctions, retain the two small projections with
their concrete difference explained; record that bounded judgment at H01 rather than silently
dropping the observation. A new derive/proc-macro crate is unnecessary for either this work or
identity hardening; that alternative would require an ADR and formal review.

Hand `SemanticFrame` remains the identity authority. Its curated tags, domain frames, canonical
expression rendering, order, first-use ordinals and attribution exclusions differ from serde's
framer and cannot be replaced mechanically. H03 classifies all non-generated `fn frame` sites,
then turns direct field reads into exhaustive destructuring without `..`. Nested selected
records must also be exhaustive: destructuring `applicability::Node` while reading `claim.*`
still permits a new `Claim` field to escape. Cover `Claim`, `Permission`, struct-like enum variants
and analogous nested records. Write an intentional exclusion as `field: _` next to its framing
decision. Keep `derived.rs`'s relationship macro and its distinct `FrameValue` u64 layout, primitive
and collection implementations, graph canonicalization and generated frame emitters in their
proper owners. Frozen vectors and published identities remain unchanged. A changed byte requires
the hashing decision route, not an updated golden accepted as routine hardening.

### Embedded registry product, conditional on a complete preservation route

`builder/products.rs::ResolvedRegistryProducts` exposes the complete declaration-derived product
and hydrates fresh maps, Arc owners, synchronization and empty implementation/proof caches.
It carries relations, obligations, enums, identities, structures, logical types/storage,
invariants, migrations, algorithms, documents, artifact profiles, contract graph/schemas/
expectations and semantic descriptions/closures. These are not three duplicate encodings:
the large Rust module constructs model types; `standard-data.jsonl` pools canonical JSON leaves;
fifteen IPC files carry self-description batches. Preserve that distinction in implementation
and benefit claims.

The [existing coupling probe](../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md)
demonstrated that a new `InvariantSpec` field prevents xtask from compiling before regeneration;
the emitted Rust literal is on the generator's own build path. Its dev-profile measurements
show material LLVM work for the 87k-line module under the probe's shared-host conditions.
Its hand-edited scratch stub is evidence, not a permitted production bootstrap. Cargo feature
gating is not a remedy under workspace feature unification.

On **2026-10-27** the operator confirmed the **conditional embedded-product target** (coordinator
RC02), requiring complete compatibility, integrity, determinism and source-equivalence evidence
before cutover and amendment of proposed ADR-0210. This confirms the direction to investigate and
plan; it proves no complete guard and authorizes no product implementation.

The preferred carrier is postcard **1.1.3**, conditionally: derive serde on the actual products
and transitive members where appropriate, generate with `postcard::to_allocvec`, embed with
`include_bytes!`, and hydrate through the existing owner. Decode exactly one product using
`postcard::take_from_bytes`; reject a nonempty remainder. `from_bytes` ignores trailing input at
this release. Borrow strings from the `'static` embedded slice, including explicit borrow
annotations for containers such as `Vec<&str>` and `Option<&str>`; no leak, string interning pool
or unconditional conversion to owned strings is the target.

Postcard cannot deserialize arbitrary `serde_json::Value`/`deserialize_any` shapes. Carry
`FieldContract`, description values and every unsuitable or hash-ordered Arrow leaf through
canonical JSON text or the existing indexed canonical pool. `logical_storage` contains nested
`DataType` fields whose metadata is a `HashMap`; do not serialize that map directly. The same
inspection must cover `contracts.schemas`, nested field metadata, dictionary ordering, nullable
children, references and every other transitive product member. Sorting a top-level map alone
does not make the payload deterministic. IPC batches retain receiving/array validation.

Three independent obligations govern cutover:

| Obligation | Required mechanism and limit |
| --- | --- |
| Compiled layout compatibility | An expected structural description produced from the **current compiled product types**, independently of the committed blob and its generated metadata; include ordered field names/types, enum variants and transitive shapes. Detect swapping two same-typed fields. A generated layout constant beside the payload can be stale together and is insufficient. |
| Payload integrity | Bind exact payload bytes and relevant side tables/IPC identities with the existing digest facilities, verify before hydration and report malformed, truncated, trailing and altered data distinctly. This detects corruption relative to the generated expected digest, not malicious replacement of both payload and digest or stale source values. |
| Source equivalence | Preserve independent `catalog::assemble()` versus actual generated hydration comparisons across the complete product and exact schemas/metadata/closures/fingerprints, plus fresh owner and receiving controls. Correctly encoded stale or wrong values remain observable here even when shape and checksum match. |

H04 is a bounded prerequisite investigation, not a claimed completed serializer design.
`serde-reflection` is a candidate for the first row: tracing current `Deserialize` implementations
can supply field names/order, but constrained deserializers need valid samples, enums need all
variants, naming of containers/generics can collide and samples of the standard catalog alone
do not exercise absent variants. Resolve exact release/source and demonstrate complete coverage
of this actual type closure before selecting it. Do not create a second mandatory hand-maintained
model schema, source-parser mirror, manual version list or registry-wide validation framework
to make the guard appear complete. Source text digests and `size_of`/type-name digests do not
establish serde layout compatibility. A model-owned derive/descriptor route is another candidate
only if its added machinery is justified and follows the crate/decision route if needed.

H04's deliverable is a concrete selected serialization/guard route with counterexamples that
distinguish all three rows, or an explicit remedy-open result identifying the failed member or
guard premise. It must not authorize H05 merely because standard data round-trips. If no credible
complete compiled-type guard exists, keep the current product until a focused alternative is
decided: a crate split breaks bootstrap coupling while retaining named literals, at the price of
a crate ADR/review and a less complete compile-cost remedy. Self-describing JSON/CBOR would
require owned/Cow string semantics and wider migration; rkyv requires archived access or full
deserialization; databake and facet remain unresearched leads, not selected fallbacks. No silent
downgrade from the preservation requirement is allowed.

When H04 succeeds and the decision is authorized, H05 replaces the emitter and generated literal
atomically with the new carrier and actual consumers. The generator must compile with a stale
embedded product and regenerate from independent declarations; runtime hydration may reject stale
layout until regeneration. Tests must exercise added/renamed/reordered fields, including two
same-typed fields, without hand-editing generated artifacts. Runtime does not fall back to source
assembly on corruption, silently repair data, import caches/proofs, or recompute content identities
to hide disagreement. Standard `registry()`/`shared_registry()` semantics and custom builder
authority stay intact. Two fresh source generations must produce identical bytes independently;
one `codegen-check` comparison alone does not establish repeat-run determinism.

Startup decode and retained allocation cost remain unknown. H04/H05 preserve a targeted route to
compare first-use construction/drop and compilation under comparable dev-profile conditions.
Quantitative speedup is an optional benefit claim requiring measurement and its workload; the
demonstrated regeneration deadlock and emitted-work removal already motivate structural change.
Reject material startup/retention regressions under ADR-0210's revisit contract rather than citing
the historic 304 ms/30 ms fixture timing as a new carrier result.

### Generated Python syntax and retained wire contracts

H06 supplies a qualifier context to `types::storage` and structure emitters so they write local
`Name`/`SemanticId` or qualified `s.Name`/`v.SemanticId` at their source. It records typing names as
types are emitted and writes imports once. Delete `local.replace("s.…", …)`, `.replace("v.", "")`
and the document emitter's import-line surgery. Validate that `@SEMANTIC_WIDTH@`,
`@CONTENT_WIDTH@` and every remaining `@[A-Z_]+@` marker are resolved. Preserve deterministic
imports with a sorted used-name set. Tests include identifiers ending in `s` or `v`, nested
annotations, unused/required imports and an intentionally unresolved marker. Full generation
publishes the changed outputs; no generated files are hand-edited.

Retain CBO-042's `Versioned<T,N>`/`BodyAccess` strict flatten, integer header, separate domain
`version`, schema closure and semantic directional `Output` names. Schemars contract modes are
already used; they do not eliminate opposite-contract naming/deduplication. Serde's unsupported
flatten-plus-deny-unknown combination and msgspec tag semantics cannot replace this contract.
The duplicate native pre-scan is removed under 57i I01, not by changing envelope interpretation
here. Counts of catalogue/generated classes describe the baseline and are not completion quotas.
No schema emitter or template engine migration is justified by line count alone. minijinja/askama
address placeholders but not the unsafe code rewrites; genco would require a broad emitter rewrite
with unestablished import-order determinism. If the SurrealQL owner independently adopts a common
template engine, sharing the tiny Python placeholder portion can be reconsidered after H06's
qualifier and import responsibilities remain explicit.

## Decision prerequisites

| Item | Current planning decision | Consequence and route |
| --- | --- | --- |
| CBO-040 / ADR-0210 carrier / RC02 | Direction confirmed 2026-10-27: conditional embedded-product target; feasibility remains open | Demonstrate complete compatibility/integrity/determinism/source-equivalence evidence, then amend the still-proposed ADR's literal outcome and enduring carrier wording through the authorized decision/design route before H05; if its status changes, reassess amendment versus supersession. No ADR acceptance implied here. |
| Identity hardening | Preserve all bytes | No hashing-contract change. Any byte discrepancy stops the migration and goes to the coordinator before dependent publication. |
| Source rendering | Narratable, caller-local only | No boundary change. Graphical `fancy-base` remains optional future feature work; do not move excerpts into reports. |
| Proc macros/crate split | Alternatives only | A crate addition requires ADR plus formal review; H04 may recommend but does not enact it. |

U1 explicitly includes conditional upstream opportunities, without contacting upstream. A miette
expression-valued code derive could project typed codes someday, but is low value beside H01.
Schemars opposite-contract deduplication could share definitions but cannot own semantic `Output`
names; no contribution is needed for H06. These are retained conditional leads, not cutover
prerequisites. U2's replayed recorded points are preserved by the science owners and by the public
view/identity contracts here; this document introduces no seed-only substitute. U3 keeps SurrealDB
canonical and supplies no reason to change the registry's in-process product carrier.

## Packages and current state

Each package includes its actual producers, consumers and immediate deletion. They are coherent
delivery boundaries, not fixed agent assignments. Shared codegen surfaces require coordinated
edits with 57i and the SurrealQL owner. None has been implemented by plan authoring.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [H01](#h01) | pending | Typed diagnostics and affected error owners | One code projection, optional macro clauses, classified derive sites and vocabulary judgment |
| [H02](#h02) | pending | Source-owning presentation boundary | Safe excerpts, private exception notes, retained-source decision and privacy controls |
| [H03](#h03) | pending | Identity and authoring owners | Exhaustive outer/nested frames and variant witnesses, unchanged bytes and parser behavior |
| [H04](#h04) | pending | Registry product and generator owners | Complete serializer/compiled-layout/integrity/canonical-order decision before fixing H05 |
| [H05](#h05) | pending | Standard registry producer and hydration consumer | Conditional on H04 and the decision amendment under confirmed RC02: complete embedded products, regeneration and literal-emitter deletion |
| [H06](#h06) | pending | Python generator owners, coordinated with 57i | Direct names/imports, marker refusal, regenerated consumers and retained strict wire semantics |

### H01

Change `pse-diagnostics` macro/vocabulary owners and every affected derive-code producer together
with Python inspection's text/report bridge. Tests cover the existing `CanonicalError::Codec`
counterexample, each migrated variant's code in miette and its typed report, forwarding plus help
override, ordered related causes, absent optional clauses and private `Withheld` classification.
Use actual registered vocabulary as expected domain values while independently checking rendered
path versus dotted wire forms; comparing two projections through the same faulty helper is
insufficient. Delete static derive code spellings and superseded boilerplate at migrated sites.
The taxonomy governance test still allows either implementation route under §23.2; do not add a
new alignment lint or broaden governance merely because this migration uses the macro.

### H02

Add one presentation helper over `BoundaryDiagnostic` plus borrowed source lookup, then wire
load/edit failure consumers in `pse-py` and the runtime/admission boundary that actually retains
the failed source. Verify ASCII and multibyte spans, EOF/trailing whitespace, several documents,
missing/binary sources, invalid spans and preserved report/code/cause fields. Exercise agent-safe
projection separately from exception notes so source privacy is actually observed. Decide the
optional retained-package method from the real source chain; if text is absent there, ship the
load/edit capability and explicit no-source behavior without a new reload/copy authority. An
explicit package method that changes public API shape joins 57i I02's boundary decision first.

### H03

Classify the source search instead of trusting the review's roughly twenty-site heuristic. Edit
framing owners in `pse-model`, `pse-modeling`, `pse-quantity`, `pse-operations` and discovered
consumers as needed, including nested `Claim`/`Permission`; preserve generated and distinct leaf
framing. Add focused completeness/compile controls where a field or variant omission would
otherwise pass and retain existing independent golden vectors. Harden the two authoring walker
wildcards and proptest variant witness in the same behavior-preserving package; existing parse/
render properties must still cover budgets, EOF and contextual identity. Do not create a new
identity derive or change the test generator's scientific validity strategy.

### H04

Work from `builder/products.rs`, `model/*`, `resolved_contract` and fingerprint products, including
transitive containers and custom serde implementations. Determine exactly which actual model
types can derive serde and where canonical leaf adapters are needed without mandatory mirror
types. Independently derive current compiled layout or demonstrate the limitation; cover enum
variants absent from standard data, constrained deserializers, generic naming and same-type
field reorder. Settle payload framing/version/digest extent, side-table binding, exact-input
refusal and borrowing lifetime. A successful bounded experiment must exercise the complete product,
not a miniature surrogate. The decision records failed candidates and the evidence still needed;
it can select an alternative but cannot waive the three distinct guarantees. No build/probe runs
are initiated by this authoring document. Execution authorizes such experiments separately.

### H05

Update actual serde owners, canonical leaf adapters, `pse-codegen` registry producer, embedded
artifacts and the standard hydration consumer in one coherent change. Regenerate with the normal
full generator; remove per-type token emitters and the replaced generated literal/unused pool
entries when all consumers move. Keep IPC/native receiving and custom assembly paths. Targeted
controls cover independent complete equivalence, new runtime owner identity, empty custom
registry scalars, cached sharing within one process, corrupt/missing/extra IPC, nonempty postcard
remainder, truncated and validly encoded altered values, layout-stale rejection, two fresh byte-
identical generations and stale-artifact generator bootstrapping. No compatibility decoder,
automatic source fallback or forbidden generated stub survives. Preserve startup/retention
acceptability evidence at the declared conditions and distinguish it from build-cost measurement.

### H06

Change `codegen/python/{mod,types}.rs`, structure emitters and `codegen/documents.rs`, their focused
generator controls and all regenerated Python contracts/import consumers. Test representative
local and qualified contexts, nested structures and lexical collisions with explicit expected
annotations/imports; comparison only to previous generated bytes cannot expose the rewrite bug.
Run full `just codegen` after generator changes and inspect the output diff. Preserve actual
input/output schema behavior via existing directional/strict-flatten tests; a schema snapshot is
not runtime validation. Delete text rewrites and import surgery immediately when their direct
emission replacements are proven. No new template dependency is selected.

## Verification and handoff

**Proposed:** packages name future checks, not results. During implementation, compile touched
crates with `just check-package <pkg>` and run scoped `just unit-package <pkg> <filter>` selections
through the pinned checkout environment. Registry/Arrow correctness closures explicitly retain
force-validation. Python presentation/codec controls use the relevant targeted Python selection
with its declared marker and correctness extension when needed. Regeneration is functional work.
All commands report actual failure count against zero. No compilation, provider workload, probe,
generator run or product test was run for this plan authoring.

[57j](57j-qualification-and-publication.md) owns assembled qualification after all functional scope:
relevant integration/native/Python journeys, generation/static/manual checks and any selected
benefit measurements. H01/H03 can proceed independently of the unresolved carrier; H06 and I01
coordinate generated document edits but do not depend on blob selection. H05 cannot begin from
an interface-only promise of a complete guard. Package completion alone does not close an
opportunity with remaining decision, consumer or assembled qualification obligations.

At close, move enduring carrier/rendering/generator explanations to their existing owners through
the authorized route and reconcile the coordinator's dispositions before retiring this plan.
Current outcome is **Proposed** only. No implementation, acceptance, measured benefit or formal
decision status is asserted.
