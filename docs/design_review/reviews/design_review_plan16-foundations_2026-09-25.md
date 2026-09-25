# Plan 16 foundation contracts: target design review

## 1. Scope, purpose and coverage

| Item | Scope |
|---|---|
| Subject | Plan 16 P00–P04 execution packet and proposed ADR-0088–0092 |
| Standard | Core 2.0; process-simulator 1.0; pse-arrow binding |
| Tier / purpose | Design / target |
| Reviewer / date | Codex, implementation author's design review / 2026-09-25 |
| Decision | Accept the specified target; implementation and final independent review remain pending |

This is a design review, not independent implementation certification. It covers
selected admission, physical provider/reaction contracts, identity and vocabulary
foundations for steady simulation, optimization, dynamics and fitting. Solver
selection/acceptance, dynamics changes, publication implementation and whole-system
qualification belong to later packets. Evidence is Proposed, or Interface-checked
for the pinned APIs named below; no new runtime result is claimed.

## 2. Authority and identity map

| Meaning | Owner / update | Revision and projection |
|---|---|---|
| Durable vocabulary | pse-schema declarations | Generated pse-model, Arrow and Python |
| Model, case, analysis | Authored relations / workflow admission | Immutable revision, separate case and analysis projections |
| Arithmetic | pse-math contract / Symbolica | Body content and prepared environment keys |
| Material/property | Authored binding / pse-kernels registration | Species-to-record mapping, semantic data and raw provenance |
| Errors | pse-diagnostics | Stable class, stage, source IDs and observations |

| Physical quantity | Units / basis | Convention and authority | Validity |
|---|---|---|---|
| T / density | K / mol m^-3 | Complete quantity type | Positive declared envelope |
| Composition | mol/mol | Ordered species, explicit dependent species | Interior for multicomponent derivatives |
| H | J/mol | Separate reference; ideal zero 298.15 K for bundled data | No implicit formation data |
| S | J/(mol K) | Separate reference; pure ideal zero 298.15 K, 1 bar | Mixing and residual retained |
| Heat/work | Declared energy rate | Distinct directional roles | Independent closure |

Labels and source spans do not define body identity. Reordering preserves identities
where order is semantically irrelevant; coordinate and port order remain explicit.
Provider code remains opaque behind the registration contract, not authoritative data.

## 3. Contracts and invariants

| Invariant | Enforcement / refusal | Evidence |
|---|---|---|
| No selected semantic loss | Shared admission classifies selected rows; unsupported source-attributed error | Proposed P03 |
| Typed physical interchange | Quantity-kind/basis/reference/phase validation before registration | Proposed P04 |
| Exact powers retain domain | Compiler rational facts; typed inference and guarded runtime | Interface-checked existing guards |
| Complete reuse dependencies | Versioned projection and canonical floating equality | Proposed P01/P02 |
| Coherent revision | Publish and prepare under one compiler lock | Existing solve path; P03 extends other operations |

Missing/not-computed/unsupported/failed evidence is typed. Variable roles stay explicit;
existing structural analysis rejects ill-posed requests before solving. This packet
makes no new solver well-posedness claim. Equivalence is semantic identity plus numerical
agreement, not whole-program bitwise determinism or symbolic algebraic equivalence.

## 4. Derivation and execution

| Stage | Inputs/output and reuse | Mechanism | Effects, limits and numerical contract |
|---|---|---|---|
| Selected admission | Revision + selected dependencies → checked Inputs/report | Existing authoring/registry and typed workflow | Finite domains; source refusal; no solve |
| Typed math | Expression/formals/physical facts → guarded body | Symbolica/Numerica; Salsa observes initialized environment | Explicit initialization; exact literals; original guards; library derivatives |
| Provider | Bound data/coordinates → values/Jacobian/Hessian | FeOS State and num-dual gradient/partial_hessian | Worker-owned; fixed seed lanes; bounded output; declared envelope/reference |
| Flow projection | Typed ports/transfers → multigraph | Existing petgraph-backed FlowGraph | Keeps isolates/multiplicity; topology does not select solver |

Scaling and solver selection remain P05/P07. Trial/domain errors do not become success.
No new tolerance or convergence rule is introduced here. Source mappings survive
lowering; mutable library states never become semantic inputs or shared query state.

## 5. Journeys

A new heater variant changes declarations and bindings, using existing balance and
provider mechanisms. Value edits alter case identity; structural edits rebuild affected
projections. Builder/document inputs converge at the same admission boundary. A failed
edit leaves the old revision intact. Out-of-envelope property trials return a typed
refusal. Recycle algorithms, dynamic events and end-to-end studies are deferred to their
named packets; this design does not claim they are qualified by foundation tests.

## 6. Gates

| Gate | Verdict for target design | Evidence / required implementation |
|---|---|---|
| G1 | pass | Existing owners; scoped projections and immutable revision |
| G2 | pass | Shared tags and complete physical/provider binding |
| G3 | pass | Named selected admission/registration boundaries and negative tests |
| G4 | pass | Explicit initialized symbolic environment; no tracked registration |
| G5 | pass | Atomic revision preparation and worker-owned state; publication only specified for P12 |
| G6 | pass | Exact power facts/guards, complete dependency keys and signed zero |
| G7 | pass | Explicit refusals and evidence labels; no current runtime assurance inferred |
| G8 | pass | Pinned library APIs below own generic algorithms |
| PS-G1 | pass | Quantity/reference/element validation; separate heat/work |
| PS-G2 | pass | Existing structural analysis retained; topology kept distinct |
| PS-G3 | pass | Original guards, derivative contract and explicit phase policy; no new solver outcome claims |

## 7. Findings

| ID | Finding | Principles / gate | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| F01 | Original P01 licensing work exceeds the selected personal-use requirement | DP-16 / G8 | Plan 16 P01 restricted-mode proposal; user explicitly excludes extra licensing code | Unneeded lifecycle complexity | Use existing licensed configuration; explicit initialization only | Source review |
| F02 | Fixed ternary arity and one caloric reference cannot express the selected target | PS-01/02 / PS-G1 | Current FeosPorts arrays and shared caloric_reference | Species mapping or entropy convention may be wrong | Explicit records, N-dependent coordinates, separate H/S datum | P04 permutation/reference tests |
| F03 | Whole-object/IEEE equality conflicts with semantic reuse scope | DP-04/09 / G6 | Current Inputs PartialEq and diagnostic definition identity | Signed zero can be stale; renamed source recompiles body | Scoped framing and matching equality | P02 mutation tests |

The execution packet incorporates these corrections. All core principles apply to
the design except DP-12 iterative convergence, which changes only in later packets;
PS-08/09/10 solver/initialization changes are likewise outside this implementation
scope. The remaining applicable principles are satisfied at specification level by
the owners, boundaries, mechanisms and verification above, not claimed implemented.

## 8. Library-leverage ledger

| Capability | Existing bespoke risk | Built-in fit and gaps | Selected |
|---|---|---|---|
| Exact powers | f64/rational structural comparison | Symbolica exact rational Atom; physical guards stay project-owned | Exact authored ratio |
| Property records | Labels attached to bundled order | FeOS Parameters::from_records is fallible; duplicate binary maps overwrite | Prevalidate then construct; avoid subset data loss/panic |
| Variable-width AD | Handwritten derivative rules or fixed arity | num-dual gradient/partial_hessian map fallible vector outputs; FeOS requires Copy dual | Fixed lane blocks over dynamic species |
| Topology | Reconstruct own traversal | Existing FlowGraph and petgraph preserve topology | Bind declared ports/decisions |
| Relational validation | Assume metadata enforces relations | DataFusion set operations fit batch checks; FK metadata alone does not enforce | Keep explicit selected semantic admission |
| Incrementality | Hidden global symbol registration | Salsa inputs carry environment and immutable values | Initialize before tracked work |

## 9. Alternatives

| Alternative | Meaning/code cost | Risk | Decision |
|---|---|---|---|
| Baseline implicit conventions | Multiple independent assumptions | Silent omissions and mismatches | Replace |
| Universal new model/compiler | Duplicates existing owners | Large second authority | Reject |
| Library-owned thin transformations | Domain mapping remains local; generic algorithms reused | Must qualify pinned call contracts | Select |
| Simplest viable | Same as preceding, homogeneous only, existing license | Explicitly smaller phase scope | Select |

## 10. Verification plan

| Claim | Evidence now | Targeted check / expected result |
|---|---|---|
| Exact powers and guards | Proposed | Analytic value/jet tests and rejected invalid domains |
| Equivalent identities | Proposed | Reorder/source rename vs consumed value/profile changes |
| Selected admission | Proposed | Builder/document equivalence; unsupported selected rows fail; unrelated rows allowed |
| Provider semantics | Interface-checked APIs | Species permutation, H/U swap refusal, independent entropy reference and finite-difference derivatives |
| Reaction conservation | Proposed | Balanced constructed reaction; unbalanced selected reaction rejected |

Only targeted functional checks run while implementing. Integrated/nonfunctional
checks wait until functional scope completion, with complete Plan 16 acceptance P18.

## 11. Authority changes and exceptions

Proposed ADR-0088–0092 precede implementation. Blueprint §0.5 and active §1/D12
claims must reflect the target while preserving Plan 14 historical qualification.
Use the explicit design edit route and a revision row. Decision PR acceptance remains
pending; no accepted ADR is rewritten and no SHOULD exception is introduced.

## 12. Decision

**Accept the target design.** Its boundaries and refusal behavior are specified;
this is not acceptance of the unimplemented scope or an independent final review.

| Priority | Change | Findings | Acceptance evidence |
|---|---|---|---|
| Correctness | Exact admission, physical bindings and identity projections | F02/F03 | Named targeted tests, then final qualification |
| Library fit/locality | Explicit init, library-owned derivatives and declarations | F01/F02 | Pinned APIs plus exercised replacement paths |
| Cost | Preserve caches, block AD and prepare once | — | No performance claim until measured |
