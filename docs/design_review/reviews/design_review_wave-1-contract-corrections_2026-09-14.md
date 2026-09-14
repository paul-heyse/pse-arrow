# Design review: Wave 1 contract corrections

## 1. Decision and scope

**Proposal:** ADR-0052–ADR-0058, the revision-7/8/9 amendments and the corrected execution sequence in plan 03.
**Status:** Proposed. This review accepts a bounded implementable design, not the unfinished runtime.
**Reviewer / author:** Codex, for the maintainer's authorized completion plan.
**Affected revisions:** baseline `115c08f`; blueprint revision 6 to revision 7.

The observable outcome is a Wave 1 implementation whose admission, publication, physical typing and reuse decisions follow explicit semantic contracts. P0–P3 form the production pipeline. P10 is exercised with a complete, separately declared predecessor fixture. The remaining pipeline inventory is not an executable capability claim.

**Method and coverage:** inspected the relevant blueprint sections, ADR-0039–ADR-0051, plan 03 interfaces, registry builder/model and change-set declarations, identity/canonical contract, manifest validation, quantity rational/index types and math graph insertion. Reviewed pinned Arrow/bytes ownership and DataFusion constraint interfaces for the proposed boundaries. Attacked duplicate keys before P2, a consistently rehashed invalid manifest, scoped node insertion order, per-occurrence literal meanings, and retained Arrow buffers after provider/session drop. This is a document-stage review: the proposed fixes, end-to-end publication recovery, full rule evaluation and memory peaks have not yet been behaviorally qualified. Earlier tests are a scaffold baseline, not evidence that these corrections work.

## 2. Authority and lifecycle map

| Concept | Semantic identity | Authority / owner | Revision boundary | Update path | Derived forms |
|---|---|---|---|---|---|
| Relation and rule contracts | Versioned semantic ID | `pse-schema` declarations | Registry revision | Checked builder | Rust/Python/docs and typed rows |
| Proposed row operation | Change-set ID, ordinal and role | Typed staged sidecar | Exact base revision | Apply atomically after P2 | Candidate relation batches |
| Valid snapshot | Complete semantic membership | Checked publication | Immutable snapshot | New publication and ref CAS | IPC and manifest |
| Physical interpretation | Complete quantity type and selected operation | Reference rows | Input bundle | Explicit conversion or new definition | Typed IR |
| Reservation ownership | Allocation owner and lease | Accounted runtime | Final buffer owner | Reserve before allocation; release on final drop | Views and clones |
| Reuse eligibility | Exact dependency descriptor and validation scope | Pass contract | Declared input bundle | Invalidate on any dependency change | Hash-indexed memo candidate |

Specialized validation, parsing, canonicalization, fixed-point iteration and resource accounting remain ordinary Rust algorithms. Their contracts are declarative; their instruction traces are not another model. Rename preserves explicit semantic identity and rewrites all bound textual references atomically; named-policy identity changes require delete/insert.

## 3. Semantic contracts and invariants

| Contract | Representation | Enforcement boundary | Failure | Evidence |
|---|---|---|---|---|
| Validity precedes identity | Unpublished candidate; no trusted constraints | P2, then canonical publication | Violating keys; no snapshot/ref | Proposed, ADR-0052 |
| Staging preserves pre/post meaning | Typed key reference plus optional row reference | Change-set construction/application | Missing row, wrong schema/key or stale base refused | Proposed, ADR-0053 |
| Rules preserve their full meaning | Typed expressions, exact rule IDs, joins, recursion and defaults | Registry build and lowering | Invalid or unsupported plan refused | Proposed, ADR-0053 |
| Typing preserves physical distinctions | Complete quantity contracts, occurrence specialization | Registry admission and P10 | Ambiguity/incompatibility refused | Proposed, ADR-0054 |
| Buffer lifetime carries accounting | Owned buffer plus reservation | Allocation and public output boundary | Resource-limit error before publication | Interface-checked ownership route; ADR-0055 |
| Capability registry is closed | Complete input/output/producer/invariant graph | Registry build and compiler request | Unavailable stage explicitly refused | Proposed, ADR-0056 |

Absence is explicit: null phase restrictions select the declared default; empty restrictions admit no phases. Missing optional pass inputs remain explicit dependencies. An unresolved residual type is allowed before P10 and rejected at its output. Hash equality is not semantic equivalence. Exact structural/value comparisons establish the test oracle; any numerical tolerance must come from the selected numerical policy.

## 4. Derivation and execution design

| Stage | Inputs | Output | Preconditions | Effects | Invalidation |
|---|---|---|---|---|---|
| Registry assembly | Declarations | Checked closed registry | Unique valid declarations; resolved typed references | No external publication | Any semantic declaration changes |
| P0/P1 | Package bytes and exact base | Typed authored candidate | Strict document and schema admission | Staging only | Package/base/contract changes |
| P2 | Candidate and invariant rules | Findings, then validated candidate | No uniqueness/FD assumptions | No memo, snapshot or ref | Always checks candidate |
| Publish | Validated complete candidate | Snapshot and ref | Keys, references, membership and bounds hold | Write immutable objects; CAS last | New semantic membership |
| P3/P10 | Complete declared stage inputs | Normalized / typed IR | Supported operations and bound names | Immutable stage outputs | Every input, policy, rule and contract dependency |
| Reuse | Candidate indexed by digest | Previously validated result | Full descriptor and validation-scope match | No semantic change | Any changed dependency, even if output would be equal |

The ownership graph is distinct from the pass dependency graph. A buffer clone extends allocation lifetime, not model revision lifetime. P2 providers expose no unverified key constraints or functional dependencies. Query results use an explicitly reserved copy where mixed shared/new DataFusion ownership cannot be transferred safely. Ref publication is the visibility point; staged and immutable orphan objects are not committed snapshots.

## 5. Representative journeys

**Ordinary extension:** a new relation adds one schema declaration, typed invariants and (when executable) complete producer/consumer ports. Generation emits projections; registry closure rejects an unresolved FK or invariant. No hand-maintained duplicate key list is introduced.

**Meaningful change:** a numerical policy, rule expression, migration default or optional input changes. Its full declaration changes the dependency descriptor. A previous digest lookup may find a candidate, but exact dependency validation rejects reuse. Compare the rebuilt values and ordered IR against uncached execution.

**Boundary:** a manifest with a consistent recomputed checksum contains an unsupported version or invalid relation member. Envelope validation and row/registry validation independently reject it. A checksum only detects encoded corruption; it does not authorize content.

**Failure:** an update stages both old and new rows with the same key in separate typed ports. P2 reports duplicate keys in the resulting candidate before canonicalization. Cancellation or budget failure drops temporary reservations and leaves the old ref. A returned Arrow clone retains its own reservation after the snapshot and session are dropped.

## 6. Acceptance gates

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | Pass, Proposed | One registry key declaration; typed rule/default content; staged rows are sidecars | Implement declaration closure and generated projections |
| G2 — Semantic fidelity | Pass, Proposed | Explicit conversion coefficients, complete physical types, payload dependencies and occurrence typing | Exercise ordered IR and quantity negative fixtures |
| G3 — Validity | Pass, Proposed | P2 candidates have no trusted constraints; validation precedes identity; hashes prove no invariant | Exercise duplicate-key and consistently rehashed invalid inputs |
| G4 — Hidden behavior | Pass, Proposed | Explicit dependency descriptors, lexical recursion and bounded iteration; unsupported operations fail | Check rule binding, scope and bound exhaustion |
| G5 — Consistency and recovery | Pass, Proposed | Exact-base changes; typed pre/post rows; atomic rename and CAS; owned reservations | Fault matrix and retained-owner tests |
| G6 — Transformation and reuse | Pass, Proposed | Uncached path first, exact dependency checks, per-occurrence typing and full payload references | Differential values and dependency mutation matrix |
| G7 — Truthful capability claims | Pass, Proposed | Closed P0–P3 production scope; P10 fixture qualification; later passes and optional packets explicitly deferred | Registry availability tests; no whole-wave acceptance until terminal gates |

## 7. Principle findings

All verdicts below describe the corrected proposal. The concrete baseline gaps motivate the correction and remain implementation work until tests demonstrate closure.

| Finding / verdict | Principles | Evidence or gap | Consequence | Correction | Verification |
| Validity and identity were conflated; Satisfied in proposal | DM-07, DM-15, DM-31, DM-33 | Plan 03 candidate publication precedes P2; blueprint §5.4 describes provider key information | Duplicate keys can be rejected before diagnostics or hidden by optimizer assumptions | ADR-0052: unpublished candidate, no constraints, semantic validation before identity | Duplicate-key P2 returns findings; no ref; rehashed-invalid restore fails |
| Change rows and rule semantics were incomplete; Satisfied in proposal | DM-02, DM-06, DM-14 | Revision-6 §22.2 has only row_key/index tuple and one row pointer; §6.11 lacks lossless expression/recursion fields | Update pre/post rows collide; different rules/defaults appear identical | ADR-0053: typed role ports and complete rule declarations | Update/delete/key-change tests; rule/default mutation tests |
| Physical and graph meaning was underspecified; Satisfied in proposal | DM-06, DM-08, DM-24 | §6.2 omitted conversion coefficients; §6.9 omitted indexed residual type; Piecewise input absent | Datum conversion or literal sharing changes physical meaning | ADR-0054: explicit coefficients, occurrence typing, one operator table | Affine/difference/point and ordered payload tests |
| Reservation did not follow retained buffers; Satisfied in proposal | DM-14, DM-29, DM-35 | Existing resource interface returned reservations separately; Arrow clones may outlive snapshot owner | Reported usage drops while live returned arrays retain memory | ADR-0055: safe owned bytes retain lease, bounded copy for result transfer | Drop-order, nested/dictionary/slice, cancellation and peak tests |
| Executable inventory overstated available producers; Satisfied in proposal | DM-43, DM-44, DM-59 | Plan A-4 requested all 17 rows before later producer schemas were specified | Registered pass has unresolved input/output authority | ADR-0056: closed executable subset; complete P10 predecessor fixture | Registry rejects incomplete registration and requests for unavailable passes |

**Applicability:** authority, semantic contracts, validity, effects, recovery, reuse and capability groups apply because these are the Wave 1 boundaries. Solver convergence, backend scientific equivalence and production throughput are outside this change's supported scope and are not rated. No performance guarantee is inferred from the ownership interface.

## 8. Alternatives and architectural leverage

| Alternative | Duplication / locality | Risks | Cost | Performance evidence | Decision |
|---|---|---|---|---|---|
| Retain scaffold contracts | Duplicate keys and incomplete serialized rule meaning | Invalid reuse, pre-P2 rejection, detached accounting | Low immediate; repeated repair later | None | Rejected |
| Complete existing contracts | Single declarations projected to consumers | Requires comprehensive boundary tests | Bounded Wave 1 changes | No end-to-end measurement yet | Selected |
| Uncached P0–P3, full semantic validation, reserved copies; add memo only after correctness | Same contracts with fewer execution mechanisms initially | Extra recomputation/copy cost; preserves meaning | Lowest sound implementation sequence | Must measure before finer optimization | Selected as first execution milestone within the proposal |

The existing registry and generator have demonstrated multiple consumers. A second validation framework, hash-based validity certificate, pointer-allocation ledger or automatic fine-grained dependency engine is not justified by Wave 1. Specialized algorithms remain direct code behind checked contracts.

## 9. Verification and measurement plan

| Claim / risk | Label | Check | Conditions | Current gap |
|---|---|---|---|---|
| Semantic admission | Proposed | P2 duplicate/null/FK/enum/quantity/metadata negatives | Force-validate enabled, baseline zero | Runtime implementation pending |
| Representation preservation | Proposed | Canonical metamorphic, strict document and generated contract round trips | Compare decoded values and ordered IR independently of digest | Full conformance pending |
| Reuse/invalidation | Proposed | Incremental versus uncached; mutate each dependency | Equal-output mutation still invalidates; changed validity scope refuses hit | Compiler/memo pending |
| Recovery/accounting | Proposed | Publication fault matrix and allocation-owner drop order | Single-writer local store; small deterministic budgets | Store/runtime pending |
| Wave closure | Proposed | `just ci-pr`, `just features-powerset`, `just test-release`, golden checks, pinned enum parity | Each reports failures against zero; Python follows `just py-sync` | No terminal claim yet |

Account for coexistence of input, normalized, encoded, decoded and returned buffers. Record construction/admission/query peaks and elapsed work before choosing an optimization; no throughput threshold is introduced without measurements.

### Additional inspected design completion: ADR-0057

**Proposed:** blueprint §7.5 supplies the existing family and role spellings; its unavailable external citation is replaced by a self-contained contract. Exact dictionary admission belongs to Wave 1. UNCLASSIFIED prevents a pre-P14 row from claiming established structure, and GENERAL_NONLINEAR is distinct from failed/absent analysis. Bilinear, smoothness and convexity claims require semantic conditions on the complete expression. Engineering role remains a producer declaration with provenance. G1/G2/G3/G6/G7 remain Pass for this bounded proposed contract; no P14 behavior is claimed or tested. P14 counterexample and changed-case gates remain deferred under ADR-0056.

### Additional inspected physical boundary: ADR-0058

**Proposed:** blueprint §8.2 previously put the gauge datum in both the psig unit offset and the physical reference conversion. Composing these transforms could add101325 Pa twice (DM-06/DM-08/DM-24). The correction restricts datum-bearing units to explicit quantity context and keeps representation normalization within that reference. Bare context-free conversion fails; the named quantity conversion changes reference once. G2/G3/G6 pass for this bounded proposed contract with the composed numeric/type regression required. Existing Celsius/Fahrenheit point/difference tests remain applicable. No runtime acceptance is inferred from this design amendment.

## 10. Exceptions and unresolved decisions

No SHOULD exception is introduced. Supported scope is narrowed by ADR-0056: later executable pass registrations wait for complete producer schemas; B-evidence and R-3 remain deferred with observable register triggers. P10 acceptance is the complete fixture route, not a fabricated authored-to-P10 pipeline. Native solve and later compiler passes remain later phases. The maintainer owns review of those triggers. Formal ADR acceptance and PR lifecycle remain pending; this review is not runtime certification.

## 11. Decision and implementation changes

**Decision:** Accept the bounded proposed design. All seven gates are decidable with named enforcement boundaries. This does not accept the existing scaffold as an implementation of the corrected design.

| Priority | Change | Principles | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 | Semantic admission before publication/reuse; complete rule/change-set/quantity contracts | DM-02, DM-07, DM-14, DM-15, DM-33 | P2, rule and physical-type negatives | Adversarial inputs with self-consistent hashes |
| 2 | Reservation ownership and fault-safe store | DM-29, DM-35 | Retained-buffer and publication fault matrix | Cancellation/budget/drop-order fixtures |
| 3 | Closed registry, generation, uncached compiler then guarded memo | DM-31, DM-43, DM-59 | Regeneration and actual-value differential checks | Dependency mutation and unavailable-pass tests |
| 4 | Terminal evidence and truthful plan status | DM-60 | Named full commands with zero failures | Outcome ledger separates focused and terminal acceptance |


### Named physical conversion edges (revision 10)

**Proposed — Accept within ADR-0054 scope.** A conversion selection is a reproducible claim, checked against actual child/result complete types, coefficients and parameter facts. Representation-only edges preserve the physical type. The additional persisted selection uses the existing relation shape and makes the dependency explicit. G2/G4 runtime closure requires valid datum-once and forged/wrong-source/wrong-parameter fixtures; this design finding does not replace those tests.

### Typed Affine constants and kernel conversion bindings (revision 12)

**Proposed — Accept within ADR-0054 scope.** An untyped nonzero Affine constant cannot establish physical meaning from the result it claims. Explicit paired type/unit fields and canonical-unit admission make its operand contract checkable. Parameterized physical transforms use the existing complete kernel binding rather than implicit scalar parameters. G1–G7 remain Pass for this bounded design; closure requires wrong/missing constant type and unit, noncanonical constant, signed-zero/order, missing or mismatched binding, and parameter value/unit regression fixtures. Neither the binding locator nor matching identities establishes admission.

### Candidate admission and target-key closure (revision 13)

**Proposed — Accept within ADR-0052/0053/0056 scope.** Composite port keys and equation declaration identities preserve the actual target relation contract (DM-06/DM-09). Kind-specific shape and ownership tests replace fabricated identifiers. Null subject_snapshot marks findings before publication. A required semantic validator runs before store admission; an unavailable implementation refuses admission. Candidate rule sessions advertise no validated constraints. P1 reference-package sections and P3 committed inputs are explicit. G1–G7 remain Pass for this bounded design; complete target, malformed-context, untrusted invariant and publication tests remain required.

### Generic rule-key and prepublication provenance correction (revision 16)

**Proposed — Accept within ADR-0052/0053 scope.** Rule keys can contain text and ordinal values; an IndexTuple cannot preserve them. The shared tagged Cell codec and exact expected head-key schema supply reversible diagnostic references. No key component is dropped or replaced by a digest. Unpublished derivations carry no snapshot identity. G1–G7 remain Pass for the bounded design, with codec round trips, wrong key name/order/type negatives and prepublication-null fixtures required before Tested status.

## Template domain admission addendum (revision 17)

**Verdict: Accept-scoped. Evidence: Proposed.** Blueprint §6.6 gives template domains a composite key while compiled math uses an instantiated domain identity. Preserve that distinction in normalized graphs with an exclusive tagged alternative. Require actual composite membership at source binding and actual domain membership at P10. The change is bounded to representation of an already declared meaning; it does not claim later instantiation passes are available. Verification must exercise a template-local reduction, exact emit/load preservation, invalid mixed alternatives, and refusal of unresolved template coordinates by P10 (DM-02, DM-06, G2, G4).

## Revision publication addendum (revision 18)

**Verdict: Accept-scoped. Evidence: Proposed.** An unchanged logical model may have multiple authoring revisions; snapshot identity cannot select their source/history authority. An exact typed revision receipt in the same conditional ref write makes the revision and snapshot observable atomically. The catalog checks the actual sidecar row and snapshot binding before returning a commit base. Prepublication records use absent snapshot fields explicitly. Required tests cover unchanged-content distinct revisions, stale CAS, mismatched revision rows and interrupted immutable writes (DM-02, DM-15, G2, G4).


## Standalone derivation attribution addendum (revision 19)

**Verdict: Accept-scoped. Evidence: Proposed.** Inspected blueprint §6.13 and §14.2, the required provenance fingerprint column, and the candidate-session API. A standalone candidate check has no published snapshot or stage key; requiring a fingerprint would fabricate execution attribution. The bounded correction makes that field optional and accepts a real caller-supplied execution fingerprint solely for diagnostics. Rule execution still establishes every result from actual rows. No new hash context or execution platform is introduced.

| Gate | Result | Evidence within this correction |
| --- | --- | --- |
| G1 | Pass | Nullable declared field has one registry source. |
| G2 | Pass | No invented execution identity supplies authority. |
| G3 | Pass | Rule predicates still execute over actual candidate bindings. |
| G4 | Pass | Recording supplied attribution introduces no mutation or changed predicate behavior. |
| G5 | Pass | Candidate provenance has absent publication identity and cannot appear committed. |
| G6 | Pass | Attribution remains diagnostic and cannot authorize reuse or a transformation. |
| G7 | Pass | Null and real-attribution fixtures, plus invalid candidate negatives, define observable acceptance. |

| Finding | Principles | Evidence or gap | Consequence | Correction | Verification |
| --- | --- | --- | --- | --- | --- |
| Required execution fingerprint has no source for standalone P2 | DM-02, DM-14, DM-59 | blueprint §6.13 previously required a hash despite unpublished candidate execution | Implementers fabricate unrelated identity or refuse useful validation | Nullable fingerprint and optional actual RunEvidence | P2 prepublication provenance remains null; supplied real evidence survives; violations still refuse publication |

Implementation and full-wave acceptance remain open. The full review's other scope and exclusions remain unchanged.

## Durable change staging addendum (revision 21)

**Verdict: Accept-scoped. Evidence: Proposed.** Typed change operations refer to named pre/post staging ports; an in-memory map is insufficient for replay. The immutable receipt preserves exact typed artifact references and source bytes under the atomic revision pointer. Staged rows never acquire snapshot authority. Tests must reject missing/extra ports, wrong target relation/ordinal, mismatched source preimages and a receipt bound to another revision; C2 replay remains necessary to establish operation semantics (DM-02, DM-15, G2, G4).

### Revision 22: explicit commit revision binding

**Verdict: Accept-scoped. Evidence: Proposed.** The optional preassigned revision IDs allow case rows to declare their exact model revision before validation. The compiler must verify actual predecessor rows and traverse explicit receipt links with a visited set; it must not infer acyclicity from immutable content identities. Required evidence: nonempty case commits, wrong model binding, stale CAS, broken/cyclic history and unchanged-content new revisions under the Wave 1 force-validation gates.

### P10 fixture context

**Verdict: Accept-scoped. Evidence: Proposed.** ADR-0056's explicit fixture relation/port makes neutral and Boolean selections and root inventory real dependencies. Positive complete fixtures and absent/changed/wrong-type context cases must prove admission and invalidation; constructors must not hide semantic settings from the driver. This changes only the expressly isolated P10 predecessor fixture registry.

### Revision 24 scoped addendum — pending conversions and guard ownership

**Verdict: Accept-scoped. Evidence: Proposed.** The exclusive normalized request representation preserves authored conversion intent without manufacturing coefficients. Exact physical typing and actual unit definitions remain the admission basis (DM-02, DM-07, DM-24). Composite guard references preserve source ownership for demand seeds and Integral filters; consumers must validate their declared graph and Boolean semantics. No new opcode, dependency family or hash framing is introduced. Acceptance requires the named negative and positive controls in ADR-0054 and the complete Wave 1 gates; this design verdict does not certify implementation.

### Durable reuse context

**Verdict: Accept-scoped. Evidence: Proposed.** ADR-0052's complete typed context provides the actual inputs required to distinguish a lookup hit from reusable computation. The existing lossless Cell codec preserves floating bits and composite values. Test forced lookup collisions with changed source, declarations, policy, engine profile and optional-input presence; admit actual output data before a hit and retain exact lineage. Bound control sizes and allocation lifetime. Missing context must recompute.

### Fault wrapper dependency

**Verdict: Accept-scoped. Evidence: Interface-checked.** The pinned ObjectStore streaming methods require the already-resolved futures-core Stream trait. A lifecycle-only direct dependency exposes that exact signature without adding a runtime edge or changing resolution. Behavioral evidence remains the full fault-injection gate; this is not a publication correctness claim.

**Revision 24 follow-through, Accept-scoped / Proposed:** Integral binder ownership and seed null/empty/concrete index alternatives preserve independent coordinate meanings. Expression demand ownership is the actual symbol declaration, not its name or a structural hash. Positive/negative controls must exercise wrong-axis binding, duplicate members, budgeted finite expansion and deferred-index refusal before P6.

### Local conditional publication addendum

**Verdict: Accept-scoped. Evidence: Interface-checked design.** Inspected the pinned
object_store 0.13.2 local update rejection and overwrite staging/rename implementation,
Rust 1.98.1 `File::try_lock`, `File::sync_all` and Unix rename routes, and the Linux
directory synchronization contract. This is a design-route assessment before implementation;
actual concurrent and interrupted writes and hardware power loss have not been tested.
Applicability is authority, observable effects, consistency/recovery and truthful capability
claims (DM-02, DM-28–30, DM-35, DM-43, DM-54, DM-59); math, physical units, rule inference
and Python boundary behavior are unchanged.

| Gate | Result | Scoped design evidence |
| --- | --- | --- |
| G1 | Pass | One mutable ref value; every supported local writer uses the stable OS lock. |
| G2 | Pass | Exact retained control bytes, including revision receipt, define the conditional predicate. |
| G3 | Pass | Only previously admitted snapshot and sidecar targets reach publication. |
| G4 | Pass | `open_local` explicitly selects local I/O; reads and queries do not publish. |
| G5 | Pass | Finished synchronized temporary file precedes atomic rename; post-rename sync failure reports uncertain durability explicitly. |
| G6 | Pass | Local serialization does not authorize semantic reuse or replace row/declaration admission. |
| G7 | Pass within scope | Supported local Unix filesystems and cooperating writers are explicit; unsupported locking/platform behavior refuses. |

| Finding | Principles | Evidence or gap | Consequence | Correction | Verification |
| --- | --- | --- | --- | --- | --- |
| Native local update is unsupported | DM-30, DM-43 | Pinned LocalFileSystem rejects PutMode::Update | Second commit cannot move a local ref | Exact-byte predicate under a stable cooperative lock and atomic replacement | Independent catalogs race/stale fixtures; successful second local commit |
| Rename alone does not establish durable reachability | DM-30, DM-59 | Pinned overwrite has no file/directory synchronization | A surviving ref could name unsynchronized objects after system failure | Synchronize immutable objects before ref; temporary file and containing directory; report post-rename uncertainty | Interrupted staging fixtures and inspected sync order; power-loss testing remains outside this evidence |

The simpler alternatives were keeping the backend refusal (fails the required second-local-
commit use case) and unconditional overwrite (loses the conditional predicate). A general
distributed lock service adds no value to this local single-writer boundary. No new dependency
or hash protocol is needed. Required runtime fixtures remain acceptance gates; this verdict
does not certify a completed implementation.

### P10 golden predecessor carrier

**Verdict: Accept-scoped. Evidence: Proposed.** The explicitly named fixture importer
implements the already approved complete predecessor fixture boundary (ADR-0056), with
actual Model inputs and complete derived outputs. Every parent and port is admitted by the
ordinary catalog and the P10 execution uses the ordinary Driver. The fixture remains absent
from production registration. Positive publication/reopen/query controls and changed or
missing actual input values must establish behavior; a digest comparison or direct leaf
facade alone cannot establish this path (DM-02, DM-07, DM-31, G2, G3, G6).

### Revision 28: pending indexed selection

**Verdict: Accept-scoped. Evidence: Proposed.** The exclusive normalized payload retains
actual index syntax and its ordered graph references while giving no unproved member or
physical-type claim. Its source group is a declared template symbol, not a guessed semantic
identity. Actual declaration arity and graph/lexical closure are admission obligations;
later instantiation must resolve real members and P10 refuses unresolved state. Positive
and negative tests named by ADR-0054 qualify this bounded extension (DM-02, DM-07, DM-24,
G2, G3, G6). No new opcode, dependency family or canonical hash framing is introduced.


### Revision 27 addendum: continuous domain applicability and ordering

**Verdict: Accept-scoped. Evidence: Interface-checked design, before implementation.**
The existing §6.3 declarations leave `domains.unit_id` nullable while continuous detail
requires a unit, and no rule states whether every continuous domain has a detail. The
review inspected those exact declarations, the actual index adapter's ordered-member
requirements, and the C3 anti-join/count/comparison interfaces. This clarification makes
the shared existing contract enforceable without introducing a second domain model.

| Gate | Result | Scoped evidence |
| --- | --- | --- |
| G1 | Pass | Actual domain/detail rows and the registered invariant are the sole authority. |
| G2 | Pass | Continuous/discrete applicability and nullable-unit meaning are explicit. |
| G3 | Pass | Required detail, equal units and member ordinal uniqueness are checked before publication. |
| G4 | Pass | Candidate validation performs no external writes; no new capability surface. |
| G5 | Pass | Violating candidate keys remain diagnostics; failed validation prevents publication. |
| G6 | Pass | Joins/counts compare actual IDs, units and ordinals; hashes establish no validity. |
| G7 | Pass within scope | Exact valid/violating row fixtures are required for all added rules. |

| Finding | Principles | Consequence | Correction | Verification |
| --- | --- | --- | --- | --- |
| Nullable unit and optional detail leave continuous coordinates incomplete | DM-07, DM-09, DM-24 | Later index inference can see a domain lacking its actual unit or bounds | Continuous Boolean selects exact detail cardinality and equal explicit units; discrete domains exclude that detail | Missing/extra detail, missing/conflicting unit and matching continuous/discrete cases |
| Repeated ordinals make domain-member order ambiguous | DM-09, DM-24, DM-31 | Ordered index expansion may choose implementation-dependent order | Count actual `(domain_id, ordinal)` groups and return every offending member key | Distinct member IDs with same ordinal in one domain; same ordinal in different domains remains valid |

Inferring a unit from dimension/kind or merely rejecting late in P10 leaves primitive
commit validity incomplete. A new domain platform or hash-based certificate is unnecessary.
Math, kernels, storage framing and Python representations are unchanged. Runtime behavior
and all new fixtures remain implementation acceptance gates; this is a scoped design verdict.


### Revision 29 addendum: unit-bearing smoothing tolerances

**Scope and method:** Interface-checked `Payload::SmoothOp`, `quantity::infer::smooth`, P3 literal lowering and the normalized mathematical projections against blueprint §6.9, §7.2, §8.2 and charter DM-02/07/09/24/31. The existing f64 payload cannot retain a declared epsilon unit, despite the Pa/K examples. This is a bounded design review; runtime correctness and performance are not established here. Other operation families, numerical backends and external side effects are outside this amendment.

| Gate | Result | Scoped evidence |
| --- | --- | --- |
| G1 | Pass | Actual unit and quantity declarations supply all tolerance facts; no new physical registry. |
| G2 | Pass | Coordinate and pending-unit alternatives are explicit; the scalar tolerance's complete key is defined. |
| G3 | Pass | P10 must check full type, dimensions, datum context, positivity and finiteness before resolved emission. |
| G4 | Pass | No new side effects or external capability. |
| G5 | Pass | Failed admission emits no resolved mathematical artifact. |
| G6 | Pass | Ordered operation remains intact; existing checked conversion applies the declared difference semantics and full dependencies. |
| G7 | Pass within scope | Named positive/negative tolerance, round-trip and changed-definition regressions are required before implementation acceptance. |

| Finding | Principles | Evidence or gap | Consequence | Correction | Verification |
| --- | --- | --- | --- | --- | --- |
| Epsilon's declared physical representation disappears | DM-02, DM-07, DM-09, DM-24, DM-31 | §7.2 needs compatible epsilon dimensions but existing smooth payload stores f64 alone; P3 currently refuses a supplied unit | Erasing kPa or Fahrenheit can alter the numeric tolerance or apply an origin incorrectly | Explicit pending unit/value, exact registered scalar tolerance type and difference-aware conversion before resolved coordinate storage | Correct scaling/affine-difference values; wrong dimension/kind/basis/reference and missing-type refusals; changed definitions must recompute |

| Alternative | Decision | Reason |
| --- | --- | --- |
| Keep refusing all unit-bearing epsilon | Reject as completion | Preserves safety but leaves the stated Pa/K authored scope unavailable. |
| Expand smooth operators into generic arithmetic | Reject | Requires additional physical compositions and changes ordered numerical structure. |
| Extend the existing pending payload pattern | Select | Preserves declared facts until the already required physical boundary can validate them. |

| Decision | Evidence level | Remaining acceptance |
| --- | --- | --- |
| Accept scoped design | Proposed; relevant interfaces inspected | Implement and run exact positive/negative physical, projection, round-trip and replay tests. No performance or numerical-backend claim. |


### Revision 30 addendum: enum-valued guard operands

**Scope and method:** Interface-checked the shared source binder, actual template feature/parameter declarations, normalized predicate projection and P3 semantic admission against blueprint §7.7/§22.1 and DM-02/07/09/24/31. The parser preserves `feature != member`, but current binding lacks a declared context for a literal member. This review covers that bounded correction only.

| Gate | Result | Scoped evidence |
| --- | --- | --- |
| G1 | Pass | Actual operand `enum_id` and the registry's exact member declaration provide the meaning once. |
| G2 | Pass | Tagged expression and enum-literal alternatives preserve identity and member text with no numeric surrogate. |
| G3 | Pass | Missing/wrong context, ambiguity, invalid members, ordered comparison and inappropriate membership are refused. |
| G4 | Pass | No new external effects or dependencies. |
| G5 | Pass | Failed binding cannot publish normalized output. |
| G6 | Pass | P3 and rename use the same binder; actual source re-execution validates stored output. |
| G7 | Pass within scope | Positive feature/parameter and reversed-operand fixtures plus each named refusal are required. |

| Finding | Principles | Consequence | Correction | Verification |
| --- | --- | --- | --- | --- |
| Enum guard members have no typed source binding | DM-02, DM-07, DM-09, DM-24, DM-31 | Valid declared guards cannot normalize, while guessing by global spelling could select another enum | Read actual compared declaration enum_id and retain tagged enum_id/member operands | Correct and reversed comparisons; wrong/absent/ambiguous context and arithmetic/membership leakage controls |

Global member-name lookup and numeric stand-ins are rejected because neither establishes the operand's enum. Extending mathematical opcodes or treating an enum scalar as a domain is unnecessary. **Accept scoped design — Proposed**; runtime implementation and the named regressions remain acceptance gates.
