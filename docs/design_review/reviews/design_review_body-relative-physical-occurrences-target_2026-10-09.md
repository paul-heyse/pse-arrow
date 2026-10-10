# Body-relative physical occurrences target

**Date:** 2026-10-09  
**Tier / purpose:** DESIGN / TARGET  
**Boundary:** Immutable physical-occurrence ownership, coordinate enumeration, structural
association and the portable interpretation cutover proposed by
[ADR-0171](../../adr/0171-body-relative-physical-occurrences.md).  
**Standard:** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and
the [pse-arrow binding](../design_principles/binding/pse-arrow.md).  
**Baseline:** Product baseline `4c24721e691187e1a5b28398b29722fbde671da8`, proposed ADR-0171,
and inspected working-tree consumers on 2026-10-09. The concurrent EFF04 changes in
`pse-modeling` selection and extent accounting are preserved; they do not implement EFF03.  
**Reviewer:** Delegated design reviewer, independently assessing the coordinator-authored
ADR. The reviewer previously supplied focused occurrence-design advice, but did not author
the ADR or implement the mechanism.  
**Disposition owner:** [Plan 33 EFF00/EFF03](../../plans/33-efficiency-principles-remediation.md#eff03).

**Architectural fitness: Accept at Proposed target-design strength. Behavioral/semantic
adequacy: Accept for the specified occurrence and replay contracts. Overall decision: Accept.**

The target makes an occurrence a position in an explicitly owned syntax body. That is the
distinction physical admission needs: identical text may denote operations under different
binders, and rewritten nodes may have identical empty source ranges. Retained ASTs provide
syntax and attribution; compact coordinates provide occurrence identity; checked physical
products provide mathematical authority. Neither a digest nor diagnostic text supplies
that authority.

The proposal removes amplified subtree-text preparation without requiring global interning.
It also addresses the less visible binding path, rather than merely moving rendering out of
one constructor. Complete owner inventories, exact owner paths, structural comparison and
explicit wire evolution make the cross-owner change credible. This judgment accepts those
contracts as a target; it does not accept current code, close F05, qualify performance or
establish numerical correctness.

## 1. Scope, drivers and coverage

The affected preparation is shared by scientific functions used in simulation, optimization,
dynamics and estimation. This review concerns their checked physical admissions and retained
syntax, not the algorithms, validity envelopes or numerical outcome checks of those modes.
Relevant workloads are larger/deeper expressions, repeated syntax, generated functions,
selected package projections and strict persistent reconstruction. Repeated per-subtree text
can grow much faster than the actual AST; a shallow fixture does not establish execution fit.

The source inspection covered `pse-authoring/src/dsl/{ast,walk,render}.rs`, modeling admission,
expression checking, retained occurrences, specialization of functions, selected-package
construction, portable FunctionRecord and extent accounting; compiler typed lowering and
portable recipes; identity/replay authority; and runtime portable product request, framing,
qualification and reconstruction. Relevant contract owners are blueprint §5.3 and §14.2–§14.3.
The original [F05 and RC04](design_review_efficiency-principles-codebase_2026-10-09.md#f05)
remain the source-review diagnosis and interpretation prerequisite.

No build, test, benchmark, source mutation or runtime probe was performed. Native solver,
provider and store qualification are outside this decision review. No missing execution
receipt is treated as a passing implementation check.

## 2. Ownership and composition

| Owner | Hidden decision and exposed invariant | Local context required |
|---|---|---|
| Retained checked field/function | Complete source-bearing syntax and deterministic expression-root inventory | Actual AST grammar and explicit field/function owner paths |
| Shared inventory enumeration | Body slots and existing node traversal order | Pure syntax and owner metadata; no store or solver |
| Admission recorder | Checked operation contracts at exact coordinates; conflicting contexts refuse | Physical type context and the owning inventory |
| Structural association index | Accelerate candidate lookup without deciding equality | Bottom-up structural prehash, complete structural comparison and exact owner paths |
| Specialization | New syntax and corresponding new occurrence products | Actual substitutions and existing checked physical authorities |
| Lowering and portable restoration | Consume exact admissions; refuse incomplete or invalid coordinates | Qualified records, retained owner syntax and existing physical receipts |
| Package selection | Retain reached syntax, inventories, dependencies and admissions together | Existing selected declaration closure and outer occurrence keys |

The inventory is derived structure, not another scientific declaration registry. Its single
enumeration contract is intentionally shared across consumers that must agree about positions.
Scientific checking continues to own inference; the compiler cannot repair a missing admission
using an expected result type. The design makes ordinary source-grammar extensions local to
the actual AST and enumeration contract, with affected consumers exercising that contract.

## 3. Contracts and exact inventory coverage

ADR-0171's requirement to cover **every consumed grammar variant** governs its abbreviated
inventory list. The following source-backed cases are included in that requirement; an
implementation cannot copy the existing narrower predicate collector and claim completeness.

| Syntax/owner | Required inventory coverage and ordering | Current source establishing the obligation |
|---|---|---|
| Expression body | One root; node positions follow `Expr::walk`, including path indices, named-call/partial paths, derivative coordinates, binder domains/filters and all expression children | `pse-authoring/src/dsl/walk.rs` |
| Let and reduction/fold | Preserve the existing structural visitation order: let body before binding values; reduction/fold domain indices and filter before their expression bodies | `dsl/walk.rs`; checking evaluation order differs and must not redefine positions |
| Predicate | Comparison lhs/rhs; atom; membership expression **and domain-path index expressions**; recursive Boolean branches; no expressions in Boolean/null leaves | `expression::predicate` calls `path_type(domain)` for membership; `dsl::walk_predicate` visits domain indices |
| Equation | Relation sides; conditional guard, then equation and otherwise equation recursively | `occurrences::bind` and `dsl::EquationKind` |
| Logic/cardinality | Atom expressions; recursive not/and/or/xor/implies; exact-count expression before its child propositions | `dsl::Proposition::expressions` |
| Static syntax | Expression; set/tuple elements; named-application arguments; comprehension domains, filter and body; text has no expression root | `occurrences::static_parts` and `expression::index_domain_type` |
| Function | Body; validity; every envelope predicate; authored applicability expressions; specialized applicability-use predicates and inputs; external output selector and any other actually checked expression field | `check::Function`, `expression::check_declarations`, `specialize/functions.rs`, `portable::FunctionRecord` |
| Finite reduction | Distinct tagged occurrence under a function with the corresponding reduction contract; not a synthetic AST node or an untyped sentinel slot | `specialize::admit_function_occurrences`; `typed_math::finite_reduction_admission` |

Membership-domain path indices are particularly easy to omit: current compiler
`predicate_occurrences` collects the membership expression but does not collect the domain
indices. The target's complete shared helper supersedes that independent collector. Node
ordering is structural ordering, not checker execution order. No coordinate is allocated
according to which conditional branch happened to run.

Slots are local to their owning inventory. Aggregating declaration admissions or associating
function syntax with a checked field must retain that owner namespace or perform an explicit
validated remapping. Equal numeric slots in different fields are not equal occurrences.
Identical structure in separate fields stays separate through exact owner paths; structural
equality cannot collapse contextual owners. These requirements follow the ADR's field/function
ownership and exact-owner-path contracts, rather than adding global AST identity.

Full structural equality remains independent of digest collision. A structural prehash bucket
is not a membership proof. Exact numeric values, unit products, binder structure and ordered
edges survive comparison. Existing `Expr::structural_eq` establishes the required distinction
from span equality, but its clone-and-strip implementation is not a requirement to repeat
whole-subtree copies for every indexed node. The proposed prepared index must not recreate
the removed amplification through equality preparation.

Source ranges are read from retained nodes. Diagnostics may render a selected subtree when
needed, retaining declaration/field attribution and synthetic spans. The compiler's separate
`source` occurrence sequence and `MathLocalOccurrenceV3` interpretation remain unchanged.

| Physical element | Units, basis and convention | Validity and authority |
|---|---|---|
| Checked arithmetic operation | Complete operand/result contracts, including existing scales, bases and reference distinctions | Existing physical admission and generic obligation; coordinates select the product, not its physics |
| Bound reduction | Actual binder kind/domain and prototype physical contract | Existing binder-instantiation checks survive specialization and lowering |
| Guards and applicability evidence | Existing authored predicates and typed input meaning | Retained syntax and receipts survive reconstruction without fresh readmission |

Well-posedness analysis, variable roles, degrees of freedom and structural rejection remain
upstream/downstream under their existing owners. This representation neither establishes nor
weakens them. Diagnostics continue to identify model/source elements.

## 4. Representative scenarios

| Scenario | Stimulus and expected boundary | Evidence/acceptance strength |
|---|---|---|
| S01 — Binder/synthetic fidelity | Repeated `(x/x)` under Length and Scalar binders after clearing all spans retains distinct admissions | Existing `kernel_types` test supplies a relevant control; replacement expectations must be independently specified |
| S02 — Complete grammar | A membership domain contains indices, or a generated function adds guards/applicability inputs | Shared inventory covers all consumed expressions; omission or invalid coordinates refuses, without an independent consumer switch |
| S03 — Rewrite and replay | Specialization rewrites syntax, then a portable product reconstructs it | Rebuild inventory/admissions together; restore strict receipts, preserve guard/reduction meaning and refuse stale coordinates |
| S04 — Selection and edit | Sparse selected closure or whitespace-only source edit | Selected owners retain their complete local inventory; mathematical meaning and current source attribution remain separate |
| S05 — Growth and collisions | Deep ASTs, many repeated roots, or forced prehash collisions | No retained subtree text or repeated subtree rendering; collision resolution preserves structure and owner distinctions |
| S06 — Interpretation replacement | Historical function/recipe/product bytes meet the new reader | Versioned refusal and direct derived-state rebuilding; no old decoder or migration path |

These are mechanism and representation changes. They reuse current scientific definitions.
A new property-model family and a native solver replacement are not needed to settle this
boundary; the relevant simulator journeys are local admission, edit/re-solve preparation and
the scientific boundary round trip.

## 5. Execution, evolution and failure

Prepare the owner inventory/index from immutable syntax, record admissions against it, and
carry the complete product through selection or capture. A rewrite invalidates affected
coordinates and rebuilds them. Reconstruction resolves coordinates against the retained
inventory, validates records and consumes existing receipts. Missing admissions remain typed
refusals at the operation that requires them. No readmission fallback can conceal an omission.

Necessary syntax traversal is legitimate. The improvement is removal of rendering/copying
each subtree and rediscovering membership independently at each boundary. Compact metadata
and on-demand diagnostic rendering have a credible route as AST size grows. Equal-body matching
must reuse prepared structure rather than clone/strip every subtree anew. Numerical speedup,
capacity and exact allocation reductions remain unmeasured.

FunctionRecord v2, `pse.admitted-body.v2` and runtime product/request v4 deliberately separate
the new interpretation. Runtime v3 request construction currently depends on the runtime
interpretation, semantic body identity and physical context; changing the request interpretation
therefore separates historical candidates before recipe replay. Historical frames retain their
meaning. Disposable products rebuild; authored/external inputs and unrelated live state are
protected. This is a coordinated representation cutover, not a new persistence protocol.

No new formulation, evaluation or solve stage is introduced. Formulation policy, derivative
source/order, scaling, problem class, solver capability, status mapping and tolerances remain
under the existing mathematical/analysis owners. The relevant fidelity obligation is carrying
their checked inputs and receipts unchanged.

## 6. Foundations and gates

All verdicts below concern the Proposed contracts, not a completed implementation.

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 | Satisfied | Syntax, occurrence addressing, physical inference and replay authority have distinct owners |
| AP-02 | Satisfied | Explicit inventory/coordinate and version contracts allow internal representation replacement |
| AP-03 | Satisfied | Shared enumeration composes checking, specialization and lowering without copied scientific policy |
| AP-04 | Satisfied | Body, contextual owner, node position and admission capture the consequential distinctions; digests do not govern membership |
| AP-05 | Satisfied | Owner namespaces, traversal, malformed-coordinate refusal and interpretation changes are explicit |
| AP-06 | Satisfied | Pure enumeration/association and targeted admission tests require actual semantic inputs, without unrelated service startup |
| AP-07 | Satisfied | Prepared compact structure replaces amplified subtree rendering/storage; no global interning or new cache framework |

| Gate | Verdict | Reason/scope |
|---|---|---|
| G1 Authority | Pass | Checked syntax/physical products retain authority; structural prehashes are not membership authority |
| G2 Semantic fidelity | Pass | Owner and binder distinctions, full grammar, exact structure and source attribution are retained |
| G3 Validity | Pass | Invalid/missing/conflicting coordinates and admissions refuse; finite reduction retains its actual contract |
| G4 Hidden behavior | Pass | Enumeration/association are pure; portable replay consumes qualified records without readmission |
| G5 Consistency/recovery | Pass | Owner syntax/inventory/admissions travel together; direct rebuilding has an explicit interpretation boundary |
| G6 Transformation/reuse | Pass | Specialization rebuilds affected inventories; selection preserves local owners and complete dependencies |
| G7 Truthful claims | Pass | ADR evidence is Proposed; implementation, performance and scientific qualification remain separate |
| G8 Library use | Pass | This is application syntax addressing; no numerical solver, differentiation or algebra is reimplemented |
| G9 Architectural fitness | Pass | All seven foundations are satisfied within the stated target boundary |
| PS-G1 Physical semantics | Pass | Target preserves complete physical products and binder distinctions; no new units/bases are inferred from coordinates |
| PS-G2 Well-posedness | Not applicable | No change to roles, structural analysis, specification or solve formulation |
| PS-G3 Numerical correctness | Pass for preservation contract only | Strict receipts, guards and missing-admission refusal survive; no numerical accuracy, derivative-order or solve-outcome claim is qualified |

Relevant refinements are DP-04/06/08–10/19/21–24 and PS-01/06/09/11. Heuristics H10,
H12, H15–16, H19–20 and H26–28 support the execution and assurance argument. They do not
introduce a cost model, benchmark prerequisite or new proof machinery.

## 7. Findings

No blocking target-design finding or SHOULD exception is identified. The inventory cases in
§3 and controls in §10 are concrete implementation obligations under the ADR's complete-grammar,
owner-path and strict-replay contracts. They are not evidence that the current code satisfies
those contracts. Original source-review F05 remains dispositioned by Plan 33.

## 8. Library fit and alternatives

The source AST and physical inference libraries already own their respective meanings. No
third-party collection, interning or generic syntax framework supplies this repository's
field/function owner paths and physical-admission boundary. Ordinary standard containers and
the existing framed-hash mechanism suffice for the small derived index. Numerical libraries
retain mathematical construction and execution; this decision introduces no competing solver.

| Alternative | Contract and execution tradeoff | Judgment |
|---|---|---|
| Keep current keys, render once | Useful narrow correction but retains subtree-text storage and separate binding discovery | Insufficient for the chosen compact target |
| Drop strings, keep hash-only membership | Compact-looking keys lose collision-independent structural/contextual authority | Reject |
| Global AST interning | Can share syntax but adds unnecessary global identity/lifetime obligations | Not justified by this target |
| Owner-local inventory | Compact coordinates, explicit ownership, prepared exact association and versioned replay | Selected; simplest complete design for the stated contract |

## 9. Change locality and tradeoffs

The coordinated edits span modeling checking/admission/occurrences/specialization/portable
records/extent accounting, compiler operation scopes/portable recipes, and identity/runtime
interpretations. That coordination is justified by the new core addressing contract, rather
than evidence of entanglement. Subsequent grammar changes use the shared enumeration owner.
No compatibility shim, global interning service or second inference path earns its cost here.

EFF04's outer `(declaration, role, repeated-field position)` ordering and declaration-range
`occurrences_of` helper remain intact. Selected-only construction carries reached inventories
with their owners. Syntax-based dependency discovery, including negative/membership/provenance
consequences, is preserved; admission coordinates are not a dependency-graph replacement.

## 10. Verification and evidence limits

| Claim/risk | Evidence | Required distinguishing control |
|---|---|---|
| Current amplification and ownership boundaries | Source-inspected current implementation | Admission construction, binding and lowering were read; not a runtime measurement |
| Complete inventory and exact contextual addressing | Proposed | Fresh nested binders/cleared spans, identical separate fields, membership-domain indices, full function families and every logic/static variant |
| Collision independence | Proposed | Force equal prehashes for structurally different bodies and identical structures under different owners |
| Specialization and reduction fidelity | Proposed | Rewrite roots/positions; empty and filtered reductions; actual binder/prototype checks; missing admission refusal |
| Strict portable reconstruction | Proposed | v2 capture/replay, complete receipts; duplicate/absent/out-of-range/conflicting coordinates; no readmission fallback |
| Attribution and selection | Proposed | Whitespace edit preserves mathematical identity and changes attribution; sparse selection retains exact reached inventories/dependencies |
| Work/storage removed | Proposed | Deep/repeated AST preparation and retained-extent controls; show removed subtree strings/renders and no relocated clone-per-subtree preparation |
| Interpretation boundary | Proposed | Old FunctionRecord/recipe/runtime product rejection; fresh rebuilding; no legacy decoder |

Existing binder, compound-guard, finite-reduction, portable-receipt and source-identity tests
are useful controls, not executed receipts for this review. New expectations must include
independently specified positions/owners; comparing two paths through the same enumeration
helper alone would not establish completeness. Tests and builds are **not_run**; no performance
measurement or product/scientific acceptance is claimed.

## 11. Rule impacts and disposition

| Impact | Existing owner | Required route and disposition owner |
|---|---|---|
| RC01 — Structural occurrence addressing and portable interpretation | Blueprint §5.3 and §14.2–§14.3; existing frame/recipe/function-record meanings | ADR-0171 and corresponding narrow architecture amendments; Plan 33 EFF00/EFF03 |
| RC02 — Runtime request/product interpretation | Blueprint §5.3 runtime body-envelope statement and runtime portable owner | v4 direct cutover through ADR-0171; Plan 33 EFF03 |

No SHOULD deviation is requested. The maintainer-confirmed source-review RC04 is implemented
through this decision route; acceptance of this target does not mark that implementation
complete. Plan 33 owns current F05/RC04 status and follow-up evidence. This review does not
create a second mutable backlog.

## 12. Decision

**Behavioral/semantic adequacy: Accept at Proposed contract strength. Architectural fitness:
Accept at Proposed target-design strength. Overall: Accept ADR-0171 within this boundary.**

The strongest evidence is the inspected syntax/checking/portable composition and the explicit
target separation of contextual coordinates, structural equality and scientific authority.
All applicable target gates pass; no unresolved premise changes the selected direction.
Implementation must still establish the complete enumeration, ownership and replay contracts
with the controls above. Adoption neither closes EFF03 nor qualifies unexamined simulator
behavior, performance, persistent deployment or mathematical accuracy.
