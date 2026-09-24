# Contract foundations scoped design review

## 1. Decision and scope

**Proposed; Accept-scoped for implementation design only.** Plan 10 N01/N02 and their
N00 setup. Reviewed by Codex against the design charter. Method: inspect Registry,
compiled_contract, generated relation admission, FieldCheckedBatch, P4 scalar arithmetic,
Python transfer classification and pinned Arrow 59.3.0 interfaces. No runtime campaign
or performance qualification. Sections 1–11 cover N01/N02; §12 extends the scoped
design review through N00–N05. N06–N18 remain outside its implementation verdict.

## 2. Authority and lifecycle map

| Fact | Authority | Derived representation / lifetime |
|---|---|---|
| Semantic declaration | Immutable registry | Native resolved graph; retained by sealed handles |
| Compiled expectation | Generated registry projection | One immutable graph per generated output |
| Successful binding | Exact graph comparison | Proof retained by actual registry arena |
| Physical metadata | Actual Arrow schema | Full transfer observations, independent of equivalence |
| Row validity | Existing local admission | Checked batch; not publication evidence |

## 3. Semantic contracts and invariants

| Contract | Boundary | Refusal / oracle |
|---|---|---|
| Complete reachable meaning | Foreign/generated admission | Changed domain/check/reference rejected despite copied IDs |
| No infinite graph expansion | Registry arena and visited pairs | Self/mutual cycles terminate; missing references fail |
| Prose is not execution identity | Explicit metadata role projection | Docs-only equivalence; unknown metadata still rejects |
| Native storage is not validity | Raw batch admission | Existing array/local checks remain required |

## 4. Derivation and execution design

Resolve declarations once; generate independent native expectations; compare exactly;
retain handles through schema-preserving operations. Changed projections re-admit.
No publication, storage mutation or session creation occurs in contract comparison.

## 5. Representative journeys

An enum change with copied IDs is detected through resolved domain values. A documentation
change can bind semantically while transfer reports still observe differing bytes. An
unrelated registry node does not affect comparison of another root. Failed comparisons
publish no successful proof. Dictionary/union/map child changes produce transfer failures.

## 6. Acceptance gates

| Gate | Design verdict | Evidence / implementation obligation |
|---|---|---|
| G1 Authority | Pass (Proposed) | Registry authority and independent generated expectations reconcile exactly |
| G2 Semantic fidelity | Pass (Proposed) | Complete graph, explicit metadata roles; preserve full physical observations |
| G3 Validity | Pass (Proposed) | Sealed handles separate from raw-array and relational validity |
| G4 Hidden behavior | Pass (Proposed) | Comparison is pure except bounded admission-proof bookkeeping |
| G5 Recovery | Not applicable | No durable effect or publication protocol changes |
| G6 Reuse | Pass (Proposed) | Exact proof before local handle reuse; projection does not copy authority |
| G7 Capability claims | Pass (Interface-checked/Proposed) | Arrow exact equality inspected; implementation claims require units |

## 7. Principle findings

Applicability: authority, types, transformation, boundaries and architectural consolidation
apply. Numerical solvers, distributed publication and execution scheduling are unchanged.

| Finding | Principles | Evidence / gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| Satisfied by proposed boundary | DM-01, DM-02, DM-07 | Resolved graph plus private handles | No digest-only authority | Implement complete admission | A01 unit matrix |
| Satisfied by proposed metadata split | DM-24, DM-59 | Current strings include prose; transfer checks actual fields | Equivalence must not claim byte retention | Explicit roles; full transfer observations | Docs/unknown metadata tests |
| Satisfied by proposed bounded reuse | DM-56 | Current generated borrow compares complete strings | Repeated allocation/comparison | Arena-owned proofs | Count and lifetime tests |

## 8. Alternatives

| Option | Assessment |
|---|---|
| Existing complete strings | Correctness intent useful, but duplicated and incomplete for transitive relation graphs |
| Hash/native compatibility shortcut | Simpler but insufficient semantic admission |
| Native fields plus shared resolved graph | Selected; native equality with explicit domain references and dictionary ordering |

## 9. Verification

**Proposed:** N01/N02 consolidation units, Python transfer matrix, generated source checks.
**Interface-checked:** Arrow Field equality omits dictionary ordering; equals_datatype omits
nested names/metadata. PyArrow native child enumeration covers maps/unions/list views/run-end
encoding, with dictionary value traversal handled separately. No timing benefit established.

## 10. Open work

Implementation and its tests; broader acceptance remains N18. No unresolved design choice
blocks this slice. The proposed ADR and named blueprint follow-up remain explicit.

## 11. Decision

| Decision | Conditions | Priority |
|---|---|---|
| Accept-scoped | Implement N01/N02 with exact admission, deletions and isolated units; no claim of runtime acceptance | First Plan 10 slice |

## 12. Native foundations scope extension (N00–N05)

**Accept-scoped (Proposed design); Interface-checked.** The authorized extension implements
ADR-0072's native validation, values and diagnostics contracts. ADR-0073 allocates the
later engine/resource decisions; this extension does not accept those later implementations.
The prior N01/N02 verdict and evidence remain limited to their recorded source cut.

| Authority | Single declaration / derived implementation | Removal |
|---|---|---|
| Values and self-description | Exact Arrow fields, arrays and field-directed literals in schema; bootstrap uses production declarations without recursively constructing a registry | Cell, generated Cell codecs, authoring row decoding and schema Cell framing |
| Local semantics | Registry/session-owned prepared predicates; bounded visible occurrences before evaluating partial expressions | Scalar Cell validation and independent capture walkers |
| Relational obligations | Relation-ID-to-LogicalPlan inputs; native joins, grouping, unnest, union extraction and casts | Whole-bundle row validators and catalog-owned generic obligations |
| Identity | IDs native field descriptors, shared semantic payload/frame encoding; named metadata projections | Mirrored Layout/DictKey and RowConverter as durable value identity |
| Generation | One native field traversal with Rust/Python/JSON Schema/Markdown policies | Independent logical/physical structural walkers and handwritten JSON string escaping |
| Failures | Leaf detailed-code/coarse-class vocabulary and borrowed native cause traversal | Catalog classifier and error-copy helpers |

### Contracts and tradeoffs

A literal preserves the exact floating-point bits. Semantic hashes normalize NaN payloads
but retain signed zero. Field metadata has explicit physical, execution and value identity
projections; unknown metadata remains significant. Registry fingerprints and row tokens
move to their v2 formats once. Named IDs and the canonical IPC outer frame are unchanged.
No old-format reader or compatibility row API is provided.

Prepared validation captures the actual SessionState and Registry owners. A changed session
requires a new implementation. Logical local-validation predicates reuse that implementation.
Validity covers all root rows even when retained findings are truncated; cancellation is
an error. Findings use a declared relation, actual root keys, JSON Pointer locations and
lossless field/value observations. Delta owns storage conversion and public operation builders;
its private DataValidationExec is evidence, not an application import.

Native arrays remain the production bulk boundary. Test-only literal fixtures call the same
field-directed codec; generated typed fixtures append directly to native builders. Bootstrap
self-description constructs columns directly and shares buffers when cloned. The bootstrap
must not acquire a dependency on generated relations or call full registry assembly recursively.

### Gates and implementation oracles

| Gate | Scoped design judgment | Required evidence |
|---|---|---|
| G1 | Accept-scoped | Sole declaration, native report schema and vocabulary projections; no Cell callers |
| G2 | Accept-scoped | Literal bit/metadata/null-parent round trips; union/run/dictionary occurrence tests |
| G3 | Accept-scoped | Prepared field matrix, exact owners, absent/empty targets, duplicate keys and storage-lowering agreement |
| G4 | Accept-scoped | Physical expression preparation is outside evaluation; no hidden durable effects |
| G5 | Not applicable to this slice | Delta publication/recovery remain N14/N18 obligations |
| G6 | Accept-scoped | Owner-local preparation/proofs and shared immutable arrays; no digest-only authority |
| G7 | Interface-checked | Exact pinned native APIs and focused units; benchmark execution and integrated acceptance deferred |

The simpler alternative—retaining the row enum and wrapping existing validators—retains
multiple semantic authorities. Native built-ins plus the bounded occurrence adapter remove
that duplication. The adapter is justified for value visibility/paths; native relational
operators handle joins and multiplicity. A universal execution facade or extra codec crate
is unnecessary for this slice (DM-02, DM-07, DM-15, DM-24, DM-47, DM-54, DM-56, DM-60).

Verification belongs to the execution inventory. This verdict authorizes the design scope;
it is neither runtime certification nor an accepted ADR status. N17 still blocks integration
and performance execution until all Plan 10 packages and deletions close.
