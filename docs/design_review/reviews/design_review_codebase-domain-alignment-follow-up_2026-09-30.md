# Follow-on design review: domain contracts, composition and evolution

**Decision: Revise.** The follow-up adds **12 findings** and **10 refinements** to the earlier
recommendations. The largest additions concern checked chemistry versus executed reactions,
unknown composition in conservation, implicit-root equivalence, temporal conservation,
context-dependent document reuse, and completion/allocation ownership. Four findings are
explicit target capability/composition gaps (FU03/FU04/FU07/FU09); they do not allege silently
wrong results. The review also narrows several earlier remedy claims rather than accepting
their proposed implementations unchanged.

## 1. Scope, drivers and coverage

| Field | Assessment |
|---|---|
| Subject | Follow-on to the [whole-codebase domain-alignment review](design_review_codebase-domain-alignment_2026-09-30.md), referred to below as **the first review**. Its F01–F35 identifiers retain their original meaning; FU identifiers belong to this review. |
| Baseline | Production working tree at `31001ee49da1547a5c2257515291385e9f7e4294`. At review start, comparison with `0c55c78a` found no differences in `crates/`, `packages/`, `python/`, `tests/`, `xtask/`, Cargo manifests/lockfile or architecture sections. Concurrent documentation and Plan 24 work were preserved. |
| Standard | Core **3.3**, process-simulator profile **1.3**, [pse-arrow binding](../design_principles/binding/pse-arrow.md). **Design tier; target purpose.** |
| Boundary | Production scientific packages, modeling/admission, mathematical lowering, execution/composition, reuse, durable workflows and their data/Python boundaries. Tests were read where they distinguish a contract or coverage limit. |
| Exclusions | Plan 24 and `thermo-knowledge/`; broad numerical qualification; CI/governance tooling; implementation of recommendations. Reference behavior is considered clean-room. |
| Method | Six focused assignments, in two waves, across three read-only reviewer agents plus a coordinating reviewer. Assignments cover physical/material semantics, process composition, mathematics, authoring/admission, durability/evolution, and reuse/library fit. Two reviewers then challenged findings they had not authored; the coordinator incorporated the corrections. This is agent review, not independent human review. |
| Disposition | No remediation plan has adopted these recommendations. Owners below are **proposed responsibility owners**, not scheduled work. The first review remains intact; neither review is architecture authority. |

The target is physically meaningful model extension and composition across square simulation,
optimization, dynamics, sensitivity, fitting and studies. The main question is whether the
contracts that make a component valid continue to govern it when it is composed, translated,
reused or executed through another mechanism.

This review selectively checks premises that affect its recommendations. It does not repeat
the first review's whole-crate census or independently reaffirm all its positive conclusions.
The [Plan 23 boundary audit](design_review_thermodynamic-boundary-audit_2026-09-30.md) and
Plan 23 Outcome retain their implemented-scope and historical qualification meaning. Their
acceptance does not establish adequacy for every target extension discussed here.

**Evidence convention.** Source observations below are **Implemented**; inspected interfaces
and library capabilities are **Interface-checked**. Counterexample scenarios are reasoned,
unexecuted discriminators unless explicitly stated otherwise. All corrections and their
benefits are **Proposed**. No numerical failure, speedup or qualification result is inferred
from reading source.

## 2. Decomposition, ownership and dependencies

The first review's decomposition is useful context. The additional responsibility boundaries
that matter here are:

| Responsibility | Meaning it must own | Consumer and locality requirement | Local evidence/verification boundary |
|---|---|---|---|
| Chemical knowledge | Complete composition, reaction coefficients and conserved transformations | Kinetic laws and control volumes consume a checked reaction/material projection; they do not redefine its coefficients | Authored admission without a solver or store |
| Parameter selection | A selected parameterization, its subjects, orientation, applicability and provenance | Mixture construction selects existing parameterizations without copying their scientific values | Package selection and completeness checks |
| Process composition | Independent stream state, conserved transport and each unit's storage disposition | Connections, initialization and time realization consume these meanings without deriving them from port count or global route | Specialization and structural assessment; selected dynamic journeys |
| Implicit operation | Residual relation versus selected function, admissible branch and derivative capability | Native/accelerated evaluation and factorable lifting honor the same declared operation or expose their incompatibility | Admission, original residual checks and focused substitution fixtures |
| Temporal conservation | Inventory, independent net flux, allowed event transfer and closure tolerance | Integration and endpoint qualification consume one authored conservation contract | Native balance operation plus authored lowering |
| Admission, reuse and durability | Covered in the additional findings below | Semantic dependencies and requested occurrences must survive representation and lifecycle changes | Incremental/clean comparison and policy tests |

An ordinary extension should change its scientific declaration or contextual binding. A new
core concept can legitimately change several owners. These findings concern omitted or
independently reinterpreted meanings, not the number of affected files.

## 3. Contracts, authority and constraints

### Physical and semantic obligations

| Element/operation | Dimension, basis and convention | Validity or semantic obligation | Proposed authority |
|---|---|---|---|
| Reaction source | Species coefficients multiply a rate/extent on a declared basis | Executed coefficients and complete participant mapping agree with the conserved reaction | Chemical declaration plus reaction/material binding |
| Chemical composition | Element counts and charge; absent evidence differs from zero count | Conservation requires known composition of participating species | Chemical composition/completeness operation |
| Reduced potential | Physical T, density and amount mapped to explicit coordinates | Mapping direction, references, species/composition dependence and derivative chain remain explicit | Physical typing and authored coordinate transformation |
| Stream | Selected state specification plus transported material/energy | Species identity, normalization and unlike property contexts have an explicit compatibility/mapping rule | Authored state/connection contract |
| Dynamic inventory | Amount/energy and signed flux, with physical time | Inventory change equals integrated original flux plus declared event transfers | Authored temporal conservation operation |
| Implicit output | Declared physical result and inputs | Root/branch meaning, domain and available derivative order survive implementation replacement | Admitted implicit contract |
| Empirical parameterization | Subjects, method, coefficients and source | Mathematical domain, empirical applicability knowledge and extrapolation permission remain distinct | Selected parameterization and consuming model |

These are proposed corrections to the identified gaps, not assertions that all are already
represented. In particular, an unknown empirical envelope is not an unlimited envelope, and
a regular root at one point is not proof of a unique branch over the admitted domain.

### Well-posedness

The inspected integrated path admits an algebraic/differential partition and checks its
structure (`workflow/modeling/dynamics.rs:870–935`). The new composition gap is in
package-level refusal of memoryless units in a time-dependent analysis, not an absence of
native DAE support. Grouped connections must preserve independent state specifications;
bundling every scalar equality is not a proof of well-posedness. The first review's F29
factorable-admission issue is not independently closed here.

## 4. Change scenarios and composition

Unchanged scenarios retain the first review's S01–S11 definitions. The following refinements
distinguish the additional alternatives:

| ID | Stimulus and kind | Required architectural response | Evidence/acceptance boundary |
|---|---|---|---|
| <a id="fs01"></a>FS01 | Bind a checked reaction to a material subset or alternate kinetics; contextual binding | All nonzero participants are covered and the executed source is the checked transformation | Admission rejects a missing product or competing coefficients; inert species remain valid |
| <a id="fs02"></a>FS02 | Add unformulated species or an apparent/true-species translation; domain extension | Unknown composition remains legal as knowledge but cannot establish conservation | Missing evidence differs from established imbalance; element and charge controls |
| <a id="fs03"></a>FS03 | Extend a binary mixture to a ternary mixture using different published fits; instance/binding | Select existing pair parameterizations without copied coefficients or a new lookup algorithm | Mixed-source, multiple-fit, symmetry and missing-pair controls |
| <a id="fs04"></a>FS04 | Compose algebraic mixer and separator around a holdup reactor; composition | One time-dependent model with local zero/stored accumulation and explicit initial conditions | Integrated/simultaneous comparison and steady reuse of the same definitions |
| <a id="fs05"></a>FS05 | Reset an inventory or end at an event; policy/composition | Declared transfers close and endpoint obligations are actually evaluated | Undeclared, correct and incorrect impulse; event-relative versus fixed-horizon checks |
| <a id="fs06"></a>FS06 | Substitute implicit evaluator or factorable representation; mechanism | Preserve selected-root meaning or declare relaxation/capability difference | Two roots, restricted branch, boundary/coalescing roots and original reevaluation |
| <a id="fs07"></a>FS07 | Substitute a C1 provider in a first-order implicit consumer; implementation | Demand only the derivatives needed by the inner solver and consumer | C1 value/first-order acceptance, second-order refusal, C2 positive control |
| <a id="fs08"></a>FS08 | Edit a package manifest without editing its modeling files; revision change | Incremental and clean admission agree on the same final sources | Identity-policy/package-identity changes and unaffected-document reuse |
| <a id="fs09"></a>FS09 | Revisit parameter values from different predecessors; study composition | Occurrences remain distinct while compatible preparation is shared | Forward/reverse continuation, independent repeats and per-occurrence idempotency |
| <a id="fs10"></a>FS10 | Reopen an old artifact after enum/codec evolution; representation change | Preserve recorded meaning and distinguish reading, writing and migration | Directional compatibility, old-domain predicates and unchanged stored versions on open |
| <a id="fs11"></a>FS11 | Abandon a staged step or retain/evict multiple rebound products; lifecycle | Admission and allocation owners outlive the work/storage they authorize | Gated drop/completion test, shared provenance and escaped-owner accounting |

Scenarios are proposed acceptance evidence, not completed test runs.

## 5. Mechanisms and execution

| Stage/owner | Formulation and derivative contract | Scaling/domain/capability | Outcome and independent check |
|---|---|---|---|
| Chemical admission → reaction binding | Derive executed coefficients from the checked transformation | Complete participating composition and material support | Typed missing-evidence, incomplete-support or imbalance refusal |
| Physical partial lowering | Retain independent formal arguments and Symbolica differentiation | Physical coordinate transformation and Delta(result)/Delta(argument) types | Check transformed laws and invalid coordinate bindings separately |
| Implicit admission/evaluation | Separate residual requirements, output derivative availability and requested order | Explicit branch restrictions/selection; original intervals and residual tolerance | Shared root acceptance plus derivative-neighborhood conditions |
| Factorable lifting | Residuals and bounds alone may describe a larger relation | Exact equivalence needs selection restrictions/evidence; otherwise relaxation/refusal | Keep original evaluation and honest ExactExport/RelaxedExport labels |
| Dynamic preparation → integration | Lower local accumulation and temporal conservation separately from route | Native index/profile restrictions remain; conserved inventory and flux use physical tolerances | Evaluate reset transfer and endpoint-relative closure independently of solver status |

The remedies reuse existing mathematical and native operations. No new solver iteration,
generic symbolic algebra, global ontology or universal workflow framework is implied.

## 6. Architectural assessment and gates

These judgments concern the inspected contracts and representative changes. They do not
replace the first review's independent observations in areas not reassessed.

| Foundation | Verdict | Evidence and consequence |
|---|---|---|
| AP-01 Separation of concerns | **violated, bounded** | R6/R7: scheduler completion, scientific outcome, diagnostic identity and retry policy require separate owners/projections; a common workflow extraction must preserve those distinctions |
| AP-02 Stable consumed contracts | **violated** | FU03/FU06/FU07: pair selection cannot express ordinary independent choices; implicit realizations do not share selection and demand-aware capability contracts |
| AP-03 Composition | **violated** | FU04/FU09: global time analysis prevents algebraic-unit composition; reusable binding equality forbids distinct study occurrences |
| AP-04 Adequate model and authority | **violated** | FU01/FU02/FU05/FU06: checked chemistry, composition knowledge, temporal conservation and selected-root meaning do not fully govern their consumers |
| AP-05 Explicit constraints | **violated** | FU02/FU08/FU11/FU12: missing evidence, omitted reuse context and incomplete resource ownership leave consequential obligations outside enforcement |
| AP-06 Local reasoning and testability | **violated, bounded** | FU11/FU12: reasoning about native completion and rebound ownership requires knowledge outside the advertised reusable operation; R6 separates pure study policy from necessary storage effects |

| Gate | Judgment | Evidence or limit |
|---|---|---|
| G1 Authority | **fail** | FU01: a checked reaction and the independently supplied executed coefficients can disagree |
| G2 Semantic fidelity | **fail** | FU02: unknown composition is indistinguishable from zero to conservation; FU06 distinguishes a selected function from its residual relation |
| G3 Validity | **fail** | FU01/FU02/FU08/FU10: support/completeness/context/finite-result obligations are not fully enforced at the relevant admission boundary |
| G4 Hidden behavior | **unresolved at whole-system scope** | No new undeclared-effect defect established; effect purity outside the inspected operations was not independently requalified |
| G5 Consistency and resource lifecycle | **fail, bounded** | FU11/FU12: CPU admission can end before native work and new retained allocations are not fully charged; no new failure of catalog atomicity or crash recovery is claimed |
| G6 Transformation and reuse | **fail** | FU06/FU08: selected-root equivalence and context-complete parser reuse are not established |
| G7 Truthful capability claims | **fail, bounded** | FU06's unconditional exact lifting overstates the selected-function equivalence it establishes; no false global optimum has been demonstrated |
| G8 Library leverage | **unresolved as a fresh whole-codebase verdict** | No additional bespoke/library violation established. The inspected numerical/library owners are appropriate; the first review's minor findings were not comprehensively repeated |
| G9 Architectural fitness | **fail** | The individual foundation violations above are not offset by successful fixtures or strengths elsewhere |
| PS-G1 Physical consistency | **fail** | FU01/FU02/FU05: reaction source, conservation evidence and temporal jump obligations can diverge from the intended physical contract |
| PS-G2 Well-posedness | **unresolved beyond inspected admission** | The integrated partition checks are present. Proposed grouped-state connections and the earlier F29 factorable scope still need their own evidence |
| PS-G3 Numerical integrity | **unresolved as a complete result guarantee** | FU06 exposes a representation/selection mismatch, but original evaluation remains protective. No new numerical campaign established invalid published values; the first review's shooting finding is not treated as newly tested here |

An unresolved row is not a pass. Proposed corrections do not improve the current gate verdict
until implemented and supported by their specified evidence.

## 7. Findings

### <a id="fu01"></a>FU01 — Checked reaction meaning does not govern its executed source vector

**Additional cause; high priority.** AP-04/AP-05, DP-01/DP-03, PS-03; G1/G3/G9,
PS-G1; FS01.

**Observed.** `packages/reference/physical/models/chemistry.pse:33–37` checks elemental
and charge conservation over `chemistry.stoichiometry`. `thermodynamics/models/reactions.pse:10–23`
separately accepts a `stoichiometry` function and evaluates it only over the supplied
`components`. CSTR/PFR require their material component sets to equal the reaction package's
set, then consume its callback-derived coefficients (`process/models/reactors.pse:27–38,56–77`).
Equality of two supplied sets does not establish inclusion of every nonzero participant.

**Trigger/consequence.** A balanced reaction identity can accompany an altered coefficient
callback, or matching component sets that omit a product. The resulting material equations
can consistently use a source vector different from the one that passed chemical admission.
Residual closure over those equations does not establish elemental conservation.

The shipped saponification binding forwards to the authoritative table
(`seed-data/models/saponification.pse:27–35,49–55`). This finding concerns the admitted
extension contract, not a demonstrated wrong result in that seed.

**Correction.** Chemical knowledge owns the stoichiometry. A checked reaction/material binding
consumes reaction identities and a species mapping, verifies complete support, and produces the
projected coefficients. Kinetic laws supply rates, not a second definition of the same reaction.
Remove the unrestricted coefficient callback for this role. A genuinely different reaction
basis or lumped translation needs an explicit conservation-preserving mapping.

**Alternative.** Checking callback equality against the table is a smaller repair, but retains
two configurable representations. Prefer deriving the source from the one checked declaration.

**Acceptance.** Reject a missing participating product before solving; admit inert additional
species with zero source; prevent a callback-only edit from changing a checked reaction's source;
share the projection between CSTR/PFR. Exercise admission without a solver/store.
**Owner/route:** authored chemistry and reaction/material binding, within blueprint §9.7 and
ADR-0127; a new generic kernel contract, if needed, takes a short ADR.

### <a id="fu02"></a>FU02 — Conservation admission treats unknown composition as zero

**Additional cause; high priority.** AP-04/AP-05, DP-02/DP-03, PS-03; G2/G3/G9,
PS-G1; FS02.

**Observed.** Species may have no formula and then no derived molar mass; formula cells default
to zero (`physical/models/chemistry.pse:19–26`). Reaction conservation sums those cells without
requiring known participating compositions (`:33–37`). Apparent-species dissociation checks
charge equality only (`:28–31`). Unformulated species are intentionally supported by
`domain-fixtures/models/schema-fixtures.pse:13–21`.

**Trigger/consequence.** A nonzero reaction between neutral unformulated species can satisfy
the declared element/charge equations vacuously. A charge-balanced apparent dissociation can
also fail elemental conservation. Absence from a complete composition and absence of composition
evidence are different inputs to the conservation operation.

**Correction.** Preserve unformulated species for cataloguing and applicable models. Give
composition an explicit completeness judgment; require it for participating conservation
claims. Zero-defaulted cells remain valid inside a known complete composition. Validate
apparent/true-species translations against elements as well as charge. Do not use availability
of atomic weights or molar mass as the universal proxy for knowing the formula.

**Acceptance.** Admit unformulated entities, but reject or explicitly mark unresolved a
conservation claim that needs their missing composition. Distinguish missing evidence from an
established imbalance. Retain sparse known-formula zeros and balanced controls; refuse
charge-balanced but element-unbalanced dissociation.
**Owner/route:** authored chemical schema and admission fixtures; blueprint §9.1/§9.7/§9.9
through the design route where the contract changes. No kernel change is established as necessary.

### <a id="fu03"></a>FU03 — Pair selection identifies a source rather than each parameterization

**Additional target/locality gap; medium priority.** AP-02/AP-03/AP-04, PS-02; G9; FS03.

**Observed.** Pair values are keyed by source/property/species pair, but package selection is
only `(package, property) → source` (`domain/models/interactions.pse:13–17`); absence policy
and dispatch are similarly package/property-wide (`:27–30`). NRTL sets have a source/i/j
key without a fit-variant identity (`methods/models/nrtl.pse:11–13`). Mixing expressions consume
those operations in `thermodynamics/models/peng-robinson.pse:15,65` and `pcsaft.pse:36`.

**Trigger/consequence.** A ternary extension needing A–B from source X and A–C/B–C from source Y,
or two fits from one source, cannot express those choices independently through the stock
selection contract. Mixed sources need copied composite-bank rows or custom selection logic;
same-source variants need source repackaging or custom modeling. A provenance source and a
parameterization are distinct concepts. Sources are extensible, including computations and
release artifacts (`domain/models/provenance.pse:10–38`); this is a stock contract limitation,
not an impossibility of representing variants through any user extension.

**Correction.** Identify the parameterization, including subjects, method/property family and
variant; attach provenance to it. Reuse the pure-parameter subject/property/source/variant
pattern (`domain/models/properties.pse:26–33`) where it fits. Select parameterizations per pair and validate
orientation, completeness and applicability through the common operation. Preserve directional
versus symmetric pairs and distinguish an explicitly predictive zero from a fitted zero.

**Alternative.** Per-pair source selection fixes mixed sources, but variants still need
repackaging as distinct sources rather than an explicit parameterization contract. Custom
callbacks remain appropriate for new policy, not a routine composition of existing fits.

**Acceptance.** Select a ternary mixture's existing records from two sources without copied
coefficients or a new lookup function; independently select same-publication variants; diagnose
missing pairs and preserve directional/symmetric behavior.
**Owner/route:** authored domain/method contracts and seed migration; no Rust scientific branch
or new library is warranted. This complements rather than duplicates F11.

### <a id="fu04"></a>FU04 — Global dynamic analysis is conflated with local storage

**Additional target composition gap; medium priority.** AP-03/AP-04, PS-11; G9; FS04.

**Observed.** `pse-modeling/src/analysis.rs:153` derives `analysis.dynamic` from the global route.
The mixer and separator refuse dynamic analysis because they lack holdup
(`process/models/mixer.pse:13`, `separator.pse:12`); CV0D requires holdup for a dynamic analysis
(`control-volumes.pse:27–34,47`). The integrated preparation nevertheless supports algebraic
variables/equations alongside differential states (`workflow/modeling/dynamics.rs:870–935`).

**Consequence.** Existing memoryless units cannot compose with a storage reactor/vessel in a
dynamic flowsheet. This is an explicit current refusal, not silent wrong execution. Indexed
ports alone, as proposed in F33, do not remove it.

**Correction.** Local storage disposition determines zero accumulation versus an inventory
derivative. Global analysis determines temporal realization. Reuse `has_holdup` if sufficient;
an algebraic unit still contributes its balances at the current time. Preserve legitimate
refusals for unsupported differential index or spatial behavior.

**Acceptance.** A time-varying feed → mixer → holdup CSTR → separator composes in integrated
and simultaneous modes; only declared storage adds differential states, initial conditions are
required where needed, and the same definitions serve the steady case.
**Owner/route:** process packages and dynamic composition; blueprint §10.3/§13 design change,
short ADR only if a generic kernel contract must change. Delete blanket refusals after the
replacement composition is exercised.

### <a id="fu05"></a>FU05 — Authored dynamics does not compose a reusable temporal conservation contract

**Additional cause extending F16; high priority.** AP-04/AP-05, PS-03; G3/G9, PS-G1; FS05.

**Observed.** Authored dynamic preparation emits `native::Contract { balances: vec![], … }`
(`workflow/modeling/dynamics.rs:1403–1416`). Authored events provide guards, resets and successors
without a conserved-transfer declaration (`pse-modeling/src/specialize/fixture.rs:180–201`);
resets become same-layout state expressions (`dynamics_events.rs:252–273`). Native `Balance`
already contains conserved state, independent flux quadrature, tolerance and event impulses
(`pse-backend-native/src/dynamics.rs:523–543`), and its transition check rejects undeclared jumps
(`:1464–1493`). That check has no authored balance entries to consume.

**Trigger/consequence.** Doubling a conserved inventory at an event can satisfy instantaneous
rate balances before and after the jump unless an independent temporal closure is supplied.
This is a reasoned contract escape, not a reproduced numerical failure.

**Important strength.** `process/models/vessels.pse:66–75` already authors independent
inventory-minus-integrated-flux checks. The gap is reusable conservation and jump semantics;
it is incorrect to say that all authored dynamic conservation is absent.

**Correction.** Declare the relationship among conserved inventory, signed original flux,
physical closure tolerance and permitted event transfer. Lower it into the existing native
balance mechanism, extending that contract only as necessary for authored expressions. Its
current contract is a single state index, constant scale and constant event impulses; it does
not already supply arbitrary conserved expressions or state-dependent transfers.
Dimensions alone do not establish that a state is conserved. Keep the flux check independent
of the solved derivative residual.

**Acceptance.** For a zero/constant-flux inventory, an undeclared jump fails qualification,
a declared matching transfer closes, and an incorrect transfer fails. Cover mode switches
and both integrator routes. Migrate the working vessel operation only after equivalence is
established and its independent oracle retained. Exercise a CV-based model without bespoke
terminal integral checks.
**Owner/route:** authored conservation, modeling/compiler lowering and runtime composition;
short ADR for the authored/kernel contract and blueprint §10.1/§13.3/§13.5.

### <a id="fu06"></a>FU06 — Implicit function selection is lost in residual-only exact export

**Additional cause; high priority.** AP-02/AP-04/AP-05, DP-08, PS-06/PS-07;
G6/G7/G9; FS06.

**Observed.** `AdmittedImplicit::factorable_definition` exports a single residual without a
regime assessment using residuals and bounds, with no selection/uniqueness contract
(`pse-compiler/src/workspace/modeling/executable/factorable.rs:15–49`). `ImplicitDefinition`
holds those fields (`pse-math/src/factorable.rs:402–410`); lifting introduces auxiliary
unknowns and initializes `Fidelity::Exact` (`:2146–2155`). KINSOL accepts an original-residual-
verified root reached from the declared start (`pse-backend-native/src/implicit.rs:331–361`),
whereas cubic acceleration refuses unless exactly one verified root survives
(`pse-math/src/implicit_cubic.rs:128–145`).

**Discriminator.** For `y² − p = 0`, `p=1`, `−2≤y≤2`, start `y=+1`, nested evaluation can
return the already exact positive root; cubic acceleration refuses the two roots; residual
lifting admits both. Local nonsingularity (`F_y=±2`) does not establish branch uniqueness.
The existing `implicit_residual_exported_exactly` test uses nonnegative bounds, which remove
this ambiguity (`pse-math/src/factorable_tests.rs:532–591`). It was read, not rerun.

**Consequence/limit.** A selected function's graph can become a larger relation while retaining
an Exact label. Export fidelity affects proposal/bound labels
(`pse-backend-native/src/execution/factorable.rs:869–880,917–920`). This does **not** demonstrate
a false optimum or infeasibility certificate: the larger set can provide a valid relaxation
bound, and `observe`/`assess` re-evaluate original outputs and constraints (`:842,959–1019`).

**Correction.** Distinguish a residual relation from a selected implicit function. Own branch
restrictions or operational selection once, separate from ordinary numerical initial guesses.
Exact lifting requires evidence of equivalence to that meaning; representable restrictions
can suffice, so global uniqueness is not universally required. Otherwise label the lifting
Relaxed or refuse exact export. Native and accelerated candidates share root acceptance and
appropriate derivative-neighborhood checks. Retain original-space verification.

**Alternative.** The smallest safe correction is conservative relaxed classification where
selection equivalence is not established. More capable exact export carries justified branch
restrictions. Symbolica continues to own polynomial root isolation; no new general uniqueness
prover or root solver is implied.

**Acceptance.** Two-root, restricted-branch, coalescing-root and boundary-root cases distinguish
set equivalence from local derivative validity. Candidate reevaluation and bound source labels
remain truthful. Capability differences between realizations are explicit at admission.
**Owner/route:** authored implicit semantics/compiler admission and math adapters; short ADR
reconciling ADR-0100/ADR-0105 and blueprint §7.5/§9.5.

### <a id="fu07"></a>FU07 — Implicit composition requires second derivatives regardless of demand

**Additional target capability/locality gap; medium priority.** AP-02/AP-03, DP-15, PS-07;
G9; FS07.

**Observed.** Generated implicit providers advertise Second smoothness/derivatives and compile
every residual at Second (`pse-compiler/src/workspace/modeling/executable/implicit.rs:255–283,871–883`).
Ordinary `PreparedBody::compile` already accepts demanded order and refuses an order above
supplied smoothness (`pse-math/src/execution.rs:339–355,404–416`). Implicit evaluation
distinguishes Value/First/Second (`pse-math/src/implicit.rs:287–323`); the native root adapter
uses first residual derivatives (`pse-backend-native/src/implicit.rs:186–203`).

**Consequence.** A legitimate C1 provider cannot replace a C2 provider inside an otherwise
first-order nested computation, even when the inner solver and outer consumer need no Hessian.
This is a truthful refusal and avoidable capability coupling, not fabricated derivative values.

**Correction.** Derive available implicit derivative order from residual/provider capabilities;
separate inner-solver requirements from outer demand; compile the order required by both and
refuse specifically when availability is insufficient. Existing order enums and compilation
interfaces suffice as the starting point. Update the generated provider and nested native
`OracleContract` together (`pse-backend-native/src/implicit.rs:282–283`); distinguish derivative
availability from smoothness. Do not weaken regime smoothness or affine-rate proofs.

**Acceptance.** A regular C1 closure supports value/first-order requests; a second-order request
refuses with the unavailable capability; C2 retains second-order behavior. Tests distinguish
solver residual requirements from requested output derivative order.
**Owner/route:** implicit/compiler capability contract, short ADR and blueprint §7.2/§9.4–§9.5.

### <a id="fu08"></a>FU08 — Incremental document reuse omits identity-bearing parsing context

**Additional current admission defect; high priority.** AP-02/AP-04/AP-05, DP-03/DP-09;
G3/G6/G9; FS08.

**Observed.** `OwnedDocumentSet::edit` permits edits to inventoried documents including
`package.toml`, then calls `load_reusing` (`authoring_driver/document/owned.rs:147–197`).
The loader reads the new header and computes new document identities, but chooses reusable
documents using path, bytes and `DocumentSpec` only (`document/load.rs:196–210`). Unchanged
`.pse` content reuses old batches/spans (`:243–253`); the new identity policy is consulted
only in the fresh-parse arm (`:254–263`). Fresh explicit-policy parsing requires `@id`
(`pse-authoring/src/language/parser.rs:1332–1344`). Binary reuse also retains the previous
`DataDocument.id` (`load.rs:211–225`).

**Discriminator/consequence.** Load a named-policy package with a `.pse` file lacking explicit
IDs, then change only the manifest to explicit policy. A clean load refuses the final sources,
whereas incremental reuse skips the required parse. Package-ID edits can similarly combine
new context with old identity-bearing artifacts. Admission can depend on edit history rather
than the final immutable source inventory. This path was inspected, not reproduced by execution.

**Correction.** Key reuse by the parser/binding context actually consumed, including identity
policy, document identity and selected document contract. Recompute bound rows, spans and
data-document wrappers when that context changes. The simplest correction reparses affected
documents; separating context-free syntax from bound identities is optional and must earn its
complexity. Replace the incomplete predicate rather than retaining a privileged incremental
admission path.

**Acceptance.** Clean and incremental loads agree after named↔explicit policy changes.
Package-ID editing either explicitly refuses or produces the same identities and attribution
as clean loading. Unchanged documents still reuse owners when their full context is unchanged;
preserve the behavior covered by `editing_one_document_reuses_other_parser_owners`.
**Owner/route:** runtime document loader/owned edits; repair within blueprint §5.2, no new ADR
inherently required. This is distinct from F15's desired finer reuse granularity.

### <a id="fu09"></a>FU09 — Durable studies reject repeated bindings that are distinct experiments

**Additional target/locality gap extending F05; medium priority.** AP-03/AP-04, PS-11;
G9; FS09.

**Observed.** `BindingContent` includes case, route and overlay but not predecessor
(`workflow/study.rs:138–145`). `start_study` rejects repeated binding hashes while storing
the predecessor separately (`:412–432`). The operational schema has occurrence identity
`(study_id, point_index)` but also unique `(study_id, binding_hash)`
(`pse-schema/src/catalog/operations.rs:590–617`). In-process studies check predecessor
ordering without this binding-uniqueness restriction (`modeling/engines.rs:338–351`).

**Consequence.** A forward/reverse continuation study cannot revisit identical parameter values
from different predecessor states durably. Repeated independent trials are also excluded.
A reusable value description is not the identity of a requested experiment; different starts
are already scientifically relevant dependencies.

**Correction.** Retain point-occurrence identity and permit occurrences to share a binding.
Each occurrence owns its start dependencies, attempts and results. Deduplicate compatible
preparation, not requested solves. Keep submission/retry idempotency per occurrence. Remove
binding uniqueness through the adopted store-evolution route without weakening content equality.

**Acceptance.** Same-binding points with different predecessors and deliberately independent
repeats are representable in both executors; each has its own lineage/outcome. Repeated submission
of the same occurrence remains idempotent while preparation can be shared.
**Owner/route:** shared study semantics and generated operational schema, blueprint §19.3;
the schema transition depends on F13's adopted migration decision.

### <a id="fu10"></a>FU10 — Numeric admission does not check finiteness after canonical conversion

**Additional bounded invariant gap; lower priority.** DP-03, PS-01; G3.

Binary admission checks a source magnitude's finiteness, then stores
`(value * scale + offset).to_bits()` without checking the converted result
(`pse-modeling/src/document.rs:529–549`). Inline conversion likewise constructs a numeric value
without a post-conversion finite check (`specialize/value.rs:1500–1525`). Generic conformance
checks type shape, not this invariant (`:1569–1597`). A finite extreme value multiplied by a
finite unit scale can overflow to infinity. Later evaluation may refuse it; no published
nonfinite result has been demonstrated.

Use one checked canonical-magnitude operation for inline and column admission, preserving
source/row attribution and the existing two-rounding conversion semantics. Do not silently
substitute fused arithmetic, saturation or arbitrary magnitude ceilings. Verify conversion
overflow, affine conversion and lawful near-limit controls. This is a local admission-owner
repair supporting F34, not a new quantity framework.
**Owner/route:** modeling numeric admission; ordinary correction within the existing physical
value contract.

### <a id="fu11"></a>FU11 — A staged step's CPU admission can end before its native work

**Additional bounded lifecycle defect; high priority.** AP-05/AP-06, DP-19/DP-20; G5/G9;
FS11.

**Observed.** `NativeSession::run` documents cancellation by dropping its returned future
(`pse-runtime/src/math/staged.rs:207`). CPU permits remain local to that future (`:235–240`),
while the dispatched `Request` carries work without those permits (`:245–258`). `Stop::drop`
sets a cooperative cancellation flag (`:109–114`). Session supervision retains memory and
the job slot through thread join (`:176–197`), but not the per-step CPU permits.

**Consequence/limit.** Dropping the waiting future after dispatch returns CPU permits even if
native work has not yet reached a cancellation checkpoint. Another session can acquire that
allowance while the earlier work remains active. This is an ownership argument, not measured
oversubscription. Ordinary token cancellation waits for the result (`:260–266`), and public
`SolveHandle` supervision often retains the inner future (`math/solves.rs:1323–1358`). The
finding concerns the reusable primitive's documented drop behavior, not every cancellation.

**Correction.** Transfer per-step permits with dispatched work and release them after its
required completion/cleanup. Preserve release during idle intervals between steps; holding
permits for the whole session would obstruct intervening preparation. The existing
completion-owned job path is a local precedent (`math/jobs.rs:218–273`).

**Acceptance.** Use a deterministic gated worker: dispatch, drop the future, observe that CPU
allowance stays unavailable until the worker exits, then reuse the session. Cover failed send
and scope entry. The existing `abandoned_caller_holds_pool_and_cpu_through_tls_destructor`
test covers the separate job path and is not this scenario's evidence.
**Owner/route:** runtime math lifecycle; ordinary correction within blueprint §18.8.

### <a id="fu12"></a>FU12 — Value rebinding copies allocations that its accounting treats as shared

**Additional ownership defect extending F15; medium priority.** AP-05/AP-06, DP-20;
G5/G9; FS11.

**Observed.** `PreparedCase` derives Clone but owns its occurrence map, coefficient-value vector
and `Derived` value (`pse-compiler/src/workspace.rs:813–839`). Compiler `rebind` begins with
`self.clone()` despite describing occurrences as shared
(`workspace/modeling/executable/solve.rs:331–348`). Runtime's completely unchanged path returns
the existing preparation and is sound (`pse-runtime/src/math/modeling.rs:542–551`). However,
its fast path that calls compiler rebind reserves zero extra bytes (`:553–560`), and the rebuild
branch charges presolve, coefficient products and derived values but omits the new occurrence
map and fixed-value vector (`:564–572`). `own_rebind` retains the new preparation
(`math/products.rs:67–79`). Full-preparation accounting recognizes those extents
(`pse-compiler/src/workspace.rs:844–859`).

**Consequence/limit.** Retaining several value-bound products can retain separately allocated
maps/vectors without their corresponding incremental reservations. This establishes an
ownership/accounting mismatch, not measured RSS, an exhaustion threshold or the total net
undercount; other rebuild charges can overlap existing storage conservatively.

**Correction.** Share immutable semantic/provenance products under explicit owners, and retain
independent binding state where values differ. Charge new owned allocation deltas and retain
the original owners through cache eviction and workspace rotation. Alternatively charge the
actual copies. Repeatedly charging total product size would double-charge shared subgraphs;
changing the LRU container alone does not fix ownership.

**Acceptance.** Retain multiple rebindings, drop the original and evict the cache. Verify
immutable provenance sharing, live reservations and release after the last relevant owner.
Exercise fast and rebuilt paths with nonempty occurrence maps; no timing threshold is proposed.
**Owner/route:** compiler prepared products/runtime allocation ownership, blueprint §14.3–§14.3.2
and §18.8; short ADR only if a new enduring prepared-product contract is introduced.

## 8. Library fit and ownership cost

| Capability | Fit and integration owner | Decision and important limit |
|---|---|---|
| Polynomial root isolation | Symbolica **3.0.0**; current cubic adapter already calls `UnivariatePolynomial::isolate_real_roots`, returning isolated roots and multiplicities | **Retain.** Share semantic root acceptance; do not reimplement isolation or infer general nonlinear uniqueness from it |
| Differentiation and implicit linear response | Symbolica stage differentiation and faer **0.24.4** solve/factorization behind their existing owners | **Retain.** FU06/FU07 concern applicability/meaning, not missing numerical algorithms |
| Fine-grained body admission | Salsa in compiler-local scope; owned products escape the database | **Develop F15 conditionally.** Granularity and complete dependencies matter; database handles do not become durable or cross-workspace identities |
| Byte-bounded prepared retention | DataFusion **55.1.0** `DefaultCache`, already used for native artifacts | **Viable F15 mechanism after ownership separation.** `CacheValue::size` is supplied by the integrator; the cache does not establish semantic identity, escaped-owner accounting or fresh attribution. Pinned contracts reject zero-sized entries |
| Native completion admission | Existing runtime job supervision and Tokio owned semaphore permits | **Reuse existing completion ownership.** Permit drop releases capacity; move ownership with work instead of adding a scheduler |
| Schema migration execution | The earlier review's refinery proposal over the existing PostgreSQL driver | **Unevaluated candidate in this follow-up.** A runner cannot decide directional compatibility or whether opening may mutate |
| Chemistry, state connection and pair selection | Authored contracts over existing generic admission mechanisms | **Domain work.** No library substitution removes the need to declare these scientific meanings |

Exact-release skill contracts and local pinned source were inspected for the new library
claims. Context7 was resolved and queried for Symbolica, DataFusion and Tokio; generic search
results did not override pinned APIs. The [Symbolica polynomial documentation](https://symbolica.io/docs/python_api/Polynomial)
supports the isolation capability; the Rust signature came from the pinned
`symbolica-faer-oximo` contract. DataFusion's size/LRU contracts came from its pinned
`DefaultCache` and `CacheValue` skill records. This is **Interface-checked**, not runtime
qualification or performance evidence. No additional dependency is required by these findings.

## 9. Improvements to the first review's remedies

### R1 — Preserve coordinate meaning, not a blanket prohibition on reduced laws (F01/F03/F08)

Retain the physical-typing diagnosis. A legitimate `τ=Tc/T`, `δ=ρ/ρc` law should be expressible
through an explicit physical-boundary mapping. Reject direct binding under a different coordinate
contract. `Reduced(kind, reference)` alone omits mapping direction and references dependent on
species/composition. Anonymous intermediates need kind algebra, basis, subjects and affine
restrictions, not dimension vectors alone. Ordinary scalar scaling remains valid.

Preserve the existing derivative foundations: independent partial slots
(`pse-compiler/src/typed_math.rs:723–728`), Delta(result)/Delta(argument) typing
(`pse-math/src/typed.rs:542–551`) and Symbolica differentiation over shared bindings
(`typed_partial.rs:14–68`). Test explicit transformed-law equivalence, wrong/swapped/inverted
coordinates, affine shifts, repeated/indexed/coincident-actual partials and datum-changing
reinterpretation. Removal of unit-stripping spellings is migration evidence, not the acceptance
criterion.

For F08, stable references must name explicit identities in a compatible revision/context.
`specialize::member_id` derives from instance/declaration/coordinate identities
(`pse-modeling/src/specialize.rs:2033–2046`), but blueprint §5.1 deliberately makes named-policy
renames new entities. Do not promise arbitrary rename stability. Physical overlays need the
expected complete target contract as well as magnitude/unit conversion; a unit alone cannot
establish basis, datum or subject compatibility.

### R2 — Parameter identity, applicability knowledge and permission are separate (F11/F12)

F11's phase-key diagnosis remains useful, but `methods/models/pcsaft-parameters.pse:6` declares
applicability to **both liquid and vapor**. The selected records/lookups still use the vapor
key. Correct the narrower premise rather than discard the finding.

F12 should distinguish known unrestricted applicability, unavailable empirical evidence and
explicit extrapolation permission. Requiring a field that authors fill with “unbounded” can
turn missing evidence into an unjustified claim. Put applicability on the form/fit/parameterization
that establishes it and compose it with the consuming model. Mathematical guards are not
empirical validation ranges. Universal constants need no fabricated T/P regression interval;
source uncertainty and the consuming EOS's domain have different owners. FU03 adds the
pair-specific selection context needed to preserve these distinctions.

### R3 — Group ports around state/transport semantics and an executable strategy (F02/F33)

The scalar connection checker establishes endpoint type equality
(`pse-modeling/src/specialize.rs:1368–1421`). A bundle must additionally state independent state
coordinates, species keys, normalization ownership, conserved transport versus derived
observations, and compatibility or explicit mapping between unlike property contexts.
Equating every exported member can over-specify the model; equating only FTPz does not itself
establish equal transported energy under unlike enthalpy models. Do not require identical
property-package IDs when different implementations honor the consumed contract.

Topology and tear selection are also insufficient to make an implicit unit a causal evaluator.
The current `CausalUnitRequest`/`UnitWorker` evaluates explicit output expressions and refuses
undeclared free dependencies (`workflow/strategies/conditional.rs:50–61,92–128,351–389`).
Ordinary CV outlet ports refer to free variables constrained by equations. Preserve the valid
explicit-map path; an implicit unit needs an admitted conditional solve or an explicitly selected
simultaneous initialization strategy. Reuse native solvers. RecycleFlash acceptance must include
the selected execution strategy or a capability refusal before iteration, not merely a correct
five-unit graph and tear. FU04 adds local storage semantics to the same composition work.

### R4 — Event success requires endpoint obligations, not just fewer expected samples (F09/F10/F16)

`dynamics_checks.rs:108–122` evaluates integral checks only at `profile.end` with Completed
termination; completeness also requires the original sample list (`:205–212`). Preparation
requires whole-domain integral checks and their upper-endpoint sample (`dynamics.rs:733–741`).
Simply accepting Event and marking future samples not applicable could omit those checks.

Distinguish an event-defined successful endpoint from a fixed-horizon request. Evaluate applicable
endpoint-relative obligations at the event. A fixed-domain integral is not silently a truncated
integral; a fit/shooting objective needing later observations still lacks them. Keep qualified
trajectory prefix, consumer observation coverage and final usability distinct. FU05 supplies
the independent temporal conservation that endpoint policy must consume.

F16 should share a typed structural assessment that preserves analysis mode and roles, rather
than make one unqualified integer the universal DOF meaning. Likewise, “no accumulators in
Mixer/Separator” and “no route literals in pse-py” are weak substitutes for tests of ownership:
legitimate authored contributions and mechanical adapter mappings may remain.

### R5 — Extract a qualified root response, not a weakened optimization KKT analysis (F22)

The fitting primitive checks square equality closure and physical feasibility, admits scaled
rank, solves the **negative** parameter-Jacobian RHS, checks backward error and maps coordinates
back (`workflow/fitting/oracle.rs:1063–1129`). Extract the operation
`F_x · dx/dp = −F_p`, including these conditions and explicit scaling maps. Root regularity
differs from optimization LICQ/strict-complementarity/SOSC. Include FU06's selection/domain
conditions. Require regular-square derivative comparison, rank failure and guard/bound/branch
neighborhood controls.

The inspected covariance contract already scopes `Exact` to the inverse reduced Hessian of
exact KKT analysis (`pse-schema/src/catalog/native_math.rs:1160–1170`), and blueprint §0.5 limits
the result to local first-order validity. Clarify curvature method versus statistical
approximation in consumer documentation; do not infer a new incorrect-statistical-result finding
from that enum's name alone.

### R6 — Share semantic study policy, not scheduler-state interpretation (F05/F07/F17)

The shared policy must consume scientific usability, seed availability/compatibility and
the intended dependency, separately from whether a job finished. Explicit `Start::Seed`
accepts seed-only candidates (`workflow/staged.rs:148–161`), while durable terminal seed
storage requires accepted/usable steps (`durable.rs:1240–1259`); predecessor lookup and fallback
have their own conditions (`study.rs:234–289`). Completed or partial scientific attempts can
map to completed jobs (`pse-operations/src/jobs.rs:675–686`); `attempt_state` remains visible
in study summaries, so partial results are not claimed to be wholly concealed.

A predecessor edge should express whether it requires a usable scientific result, permits
seed-only continuation, or merely orders execution. Both executors must implement the same
selected fallback/refusal rule. A pure policy returns decisions; adapters retain locking,
artifact I/O and lifecycle effects. Compare fake-result matrices covering usable/seed-only/
missing/incompatible seeds, constant evaluation, failed predecessors, cancellation and future
partial multi-result work. Current durable points are case solves; the latter is an extension
acceptance case, not an asserted current multi-result failure. FU09 preserves occurrence identity.

### R7 — Preserve detailed diagnostic identity and separate retry policy (F06/F07)

The first review's proposal to derive codes from classes needs correction. `TypedDiagnostic`
derives coarse `FailureClass` **from** a detailed code (`pse-diagnostics/src/lib.rs:9–20`);
several codes intentionally share a class (`vocabulary.rs:88–100,112–137`). Reversing this
many-to-one relation would erase distinctions such as missing IDs and inconsistent units.
`BoundaryDiagnostic` has an independently purposed disposition, severity and observations
(`pse-model/src/diagnostic.rs:66–87,150–162`).

Keep the typed cause/code and derive class, boundary disposition and severity through explicit
projections with their own meanings. Retry eligibility additionally depends on the operation
and effect state: a deterministic serialization/invariant error is not transient because it
happened during persistence. Persist typed attempt failures even when no valid model/result
table exists; retain retry causal history and attach candidate members only when available.

Acceptance follows a detailed syntax/numerical code and location across Rust, Python and
storage, and distinguishes transient pre-effect failures, deterministic defects, cancellation
and unresolved effects. “No stringify/wildcard” is a useful search, not the behavioral proof.
Changing the boundary mapping goes through blueprint §23.2 and the chosen durable-contract route.

### R8 — Separate reading, writing, projection and migration (F13/F14)

The current equality check is conservative. The remedy must distinguish: interpretation of
the recorded artifact; consumer projection; mutation under a writer's expected declaration;
and explicitly selected migration to new meaning. `DeclaredCheck::verify` admits a declaration
**before mutation** (`pse-catalog/src/delta/contract.rs:311–328`), so weakening the shared
equality function for older readers can accidentally widen writer admission.

Enum superset compatibility is directional: a new consumer may understand an old artifact;
an old exhaustive consumer does not necessarily understand a new member. Deprecation,
changed constraints/member meaning and referenced contracts need their own treatment.
Preserve witness format, canonicality and identity checks (`pse-schema/src/compatibility.rs:39–76`).

For F14, rederive predicates from the **recorded semantic witness**, not the current registry.
Keep that artifact's original allowed domain and consistency among fields, witness and encoding.
“Transforms at open” must mean an explicit read-only projection when supported. ADR-0114
Outcome 4 and blueprint §20.5 make opening read-only; mutation/migration writes a new version
through the publication route with lineage.

Acceptance covers old-domain preservation, genuinely unknown new members, independent writer
refusal, portable reconstruction across a codec change, malformed/unsupported witness refusal,
unchanged stored versions/properties on open, and explicit migration with old/new lineage.
Refinery or another migration runner executes a decided migration; it does not decide semantic
compatibility. Preserve F13's ADR-0114/R-35 route.

### R9 — Generate and reuse admitted boundary meanings (F18/F19/F20/F31/F34)

For F18, `Serialize + JsonSchema` declares a shape but does not establish semantic constructor
invariants, discriminants or defaults. Put request admission in its Rust owner and generate
Python contracts from the actual owned request/results. Test representative success, refusal,
cancellation and partial results, including enum/optional-field evolution. An empty search for
`json!` is not sufficient evidence.

For F19, cache checked expression **occurrences and roles**, not just declaration IDs or
normalized text: one declaration can contain multiple bounds, predicates and expressions.
Keep document-relative spans, imported/bound context and origins of synthetic AST nodes.
Identical syntax at different locations can share syntax without sharing source attribution
or admitted binding meaning. Test quoted logic paths, repeated malformed expressions,
inherited/overridden expressions and generated origins. FU08 shows why full context matters.

For F20/F31/F34, one semantic predicate may have multiple enforcement boundaries; this does not
require a global SQL phase for every local admission. Explicit immutable validation context
should belong to a composition root. Crate-private installation alone does not remove
construction-order coupling unless incompatible installation is observably refused. A facet
projection must state the consumer's equivalence—physical meaning, source identity and display
metadata are not interchangeable reasons to ignore a field.

### R10 — Separate reusable mathematics, revision attribution and live allocation ownership (F15)

The present view key conservatively includes occurrence IDs, definitions and spans
(`pse-compiler/src/workspace/modeling/executable/solve.rs:198–247`). Fresh views obtain current
occurrences (`:251–262,300–301`); a cached view is only rebound to values
(`pse-runtime/src/workflow/modeling/views.rs:79–96`), retaining its old occurrence map.
Simply removing spans and moving the cache to service scope would create stale diagnostics.
This is a risk in the proposed remedy, not a claim that the existing conservative key serves
stale locations.

Separate immutable mathematical products from each revision's source map/document association
and from value/attempt state. A new revision can share arithmetic while its diagnostics acquire
new attribution; old results retain their old attribution. FU12 supplies the allocation-owner
correction needed for that sharing.

Similarly, a package cache key must identify the full admitted closure, physical preconditions
and capability configuration it consumes, not merely root text. Current publication checks
rows, scope, documents and physical context separately
(`pse-compiler/src/workspace/modeling.rs:327–359,426–449`). Keep exact source-publication identity
distinct from semantic reuse identity. Mutable native evaluators, cancellation and budgets
remain attempt-owned even when admitted programs/packages are shared.

Acceptance: a span-only edit shares mathematics but refreshes diagnostics; referenced data,
physical preconditions and relevant capabilities invalidate the right product; old results
retain their original source; eviction does not release escaped allocations early; cancellation
of one attempt does not affect another's workers. Validate body-level invalidation before
claiming speedup. Semantic hashing changes follow the ADR/design route; container replacement
alone does not.

### Treatment of the earlier findings

This is a recommendation comparison, not a second implementation-disposition ledger.

| Earlier findings | Follow-up treatment |
|---|---|
| F01/F03/F08/F11/F12 | Retain the material direction, with R1/R2's coordinate, contextual identity and applicability qualifications; F11's PC-SAFT wording is narrowed. The precise transfer-direction type design remains to be decided against contribution bindings; no new intrinsic per-unit quantity kind is mandated |
| F02/F04/F09/F10/F16/F22/F23/F33 | Develop composition and acceptance through R3–R5 and FU04/FU05/FU07. F04's shared execution owner and F23's explicit incumbent policy remain useful directions, not newly designed complete interfaces here |
| F05/F06/F07/F17 | Strengthen via FU09 and R6/R7. Replace blanket class→code derivation with preservation of detailed cause/code and explicit policy projections |
| F13/F14 | Retain evolution need, replace undirected compatibility and ambiguous migration-at-open with R8's four distinct operations |
| F15 | Refine substantially through FU08/FU12/R10: complete context, source binding and ownership precede shared-cache adoption |
| F18/F19/F20/F31/F34 | R9 replaces lexical deletion proxies with semantic admission/evolution checks; FU08/FU10 add concrete admission obligations. Individual dead mechanisms in F20 were not all re-audited |
| F21/F24/F25/F26/F27/F28/F29/F30/F32/F35 | No independent disposition change established. Prior findings remain attributed to the first review; source details encountered here do not constitute comprehensive reaffirmation or closure |

## 10. Verification and evidence limits

No product tests, builds, numerical probes, benchmarks, formatters or regeneration commands
were run for this review. Historical Plan 23 results remain historical evidence, not new
qualification. The new scenarios above are acceptance proposals. Source inspection cannot
establish numerical error frequency, production performance or complete product conformance.

| Claim | Evidence | What remains to establish |
|---|---|---|
| FU01–FU12 mechanisms and safeguards | **Implemented**, inspected source at the declared baseline; counterexamples reasoned from those contracts | Execute the named discriminating scenarios when implementing the corrections; no new behavioral failure count is claimed |
| Remedy feasibility and semantics | **Proposed**, compared against simpler alternatives, current owners and relevant source | Resolve the named authority changes and implement the operation contracts before claiming correction |
| Library mechanisms in slot 8 | **Interface-checked**, exact-release skill/local source plus scoped Context7 documentation | No new integration, benchmark or library qualification performed |
| Prior scientific conformance | **Tested historically**, Plan 23 Outcome: `just seed-conformance`, 115/115 seed fixtures (4,046 checks), 6/6 domain fixtures (53 checks), zero failures; local Linux, force-validation, one native thread, 600 s/solve and 120G OS cap | Not rerun. It covers exercised models/cases; it does not cover the new admission/extension counterexamples merely because those use the same packages |
| Runtime/performance consequences | **Proposed** risk or source-derived ownership consequence | No measured oversubscription, OOM, speedup, invalid published optimum or error-frequency claim |

**Challenge results.** Cross-author inspection retained all twelve findings, while narrowing
the pair-selection claim to the stock contract; recognizing extensible provenance sources;
limiting the native balance mechanism to its supported expression shape; requiring consistent
implicit provider/OracleContract capabilities; distinguishing token cancellation from future
abandonment; and excluding completely unchanged shared preparations from FU12. The coordinated
review also corrected the sign/validity contract of the proposed root response and preserved
read-only opening and fine diagnostic codes in the earlier remedies.

**Inspected strengths and rejected leads.** These constrain the recommendations:

- The shipped saponification binding reads authoritative coefficients; unformulated species
  have legitimate non-conservation uses. No failure in the exercised seed is inferred.
- Explicit partial formals prevent coincident actual arguments from collapsing distinct
  derivative coordinates. Symbolica remains the derivative owner; implicit linear responses
  use library factorization and backward-error checks.
- Multi-regime implicit selection already checks ties, admissibility boundaries and branch
  smoothness. FU06 concerns the single-residual contract/export; it does not erase that strength.
- Vessel inventory/flux checks are real independent checks. Dynamic sample checks use the
  sample's actual mode, and mode layouts are checked; a suspected initial-mode-check defect
  was rejected.
- Document bundles have private construction and compare retained declarations, not just a
  fingerprint. Hydration checks mapping context/spans and naming cycles. Those safeguards do
  not reparse the cached `.pse` arm in FU08.
- Inline/binary table data converge through shared identity/reference/completeness phases;
  missing-column defaults differ from explicit null. A suspected row-handle identity/cache
  defect was not substantiated because specialization recovers and frames semantic identity.
- Current exact durable-contract checks refuse conservatively. Catalog CAS, lease/retention
  and crash recovery were not comprehensively re-audited; their earlier positive assessment
  is not reproduced as a fresh pass.
- Native job supervision already retains resources through completion; artifact flights are
  cancellation-aware and epoch-fenced. FU11 concerns the distinct session primitive. Allocation
  addresses in `ProductOwner` are explicitly local allocation identities, not semantic IDs.

Read-only evidence gathering used targeted `rg`, numbered source reads and pinned capability
queries. For example, `ast-grep run --lang rust --pattern 'native::Contract { $$$FIELDS }'
crates/pse-runtime/src/workflow/modeling/dynamics.rs` identified the authored constructor with
empty balances. No repository qualification gate was run; against the repository's zero
failure target, a new test result is **not evaluated**, not “zero failures.”

Unexamined breadth includes full electrolyte/reaction numerical qualification, every provider
and solver combination, all generated SQL, remote publication, and whole-workload performance.
The sparse targeted review intentionally leaves those claims open rather than reopening all
of the first review's evidence.

## 11. Authority changes and proposed disposition

The recommendations remain **Proposed**, not adopted implementation work. There are no new
exceptions waiving MUST gaps. The eventual owning plan should reference these findings and
own their disposition; this report does not modify the original review, accepted ADRs, register
or architecture sections.

| Work/decision | Findings and dependencies | Proposed responsibility owner | Authority route and deletion obligation |
|---|---|---|---|
| Repair bounded existing invariants | FU08/FU10/FU11/FU12; independently actionable | Document loader, modeling numeric admission, runtime math and compiler products | Ordinary fixes within §5.2/§14.3/§18.8; replace incomplete predicates/ownership paths rather than keep bypasses |
| Establish conserved chemistry and checked reaction projection | FU02 → FU01 | Authored chemical schema and reaction/material bindings | §9.1/§9.7/§9.9 design change where contract changes; remove competing coefficients and preserve valid unknown-knowledge entities |
| Improve physical and parameter contracts | R1/R2/FU03; coordinate with first-review W1 | Quantity/modeling typing and domain/method packages | ADR/design route for changes to D5/ADR-0124; ordinary package changes for parameter selection unless a generic kernel gap is established |
| Define state connections, local storage and temporal conservation | R3/FU04/FU05/R4; coordinate with first-review W2/W3 | Authored process/thermodynamic contracts, modeling/compiler, runtime dynamics | Short ADR for new connection/conservation/endpoint contracts; §10/§12/§13/§16.6 updates. Retire manual connection and closure copies only after their responsibilities are preserved |
| Admit implicit meaning and derivative demand explicitly | FU06/FU07 → R5 | Implicit semantics/compiler and math/native adapters | Short ADR addressing ADR-0100/ADR-0105 contract, §7.5/§9.5; root sensitivity also follows F22's ADR-0118 route. Remove unconditional exact lifting/demand, preserve independent qualification |
| Decide contract evolution, then persist shared study/diagnostic semantics | R8 enables schema changes for FU09/R6/R7 | Schema/catalog/operations, runtime study and diagnostics | ADR-0114/R-35 and §20.5/§20.6; §19.3/§23.2. Remove duplicate policy/prose-only loss and binding uniqueness, preserve occurrence idempotency |
| Separate reusable semantic products from source/value/attempt owners | FU08/FU12 → R10; coordinate F15 | Compiler and runtime math | §14.3/§14.3.2; ADR if semantic hashing changes. Replace old cache/ownership mechanism after caller migration, retain no compatibility path |
| Consolidate admitted language/Python boundaries | R9, after owned contracts are decided | Authoring/checker, runtime request owners, schema/document generators and Python adapter | Existing generation/contract route; ADR where the Python boundary contract changes. Delete mirrored meaning, not legitimate mechanical adaptation |

This is dependency guidance, not a delivery schedule. Bounded correctness fixes can proceed
without waiting for the larger target concepts. Store evolution must precede durable schema
changes if existing contents are to survive. Broader cache sharing must follow ownership and
attribution separation. No new crate is proposed.

## 12. Decision

**Behavioral and semantic adequacy: not adequate for the inspected contracts and target.**
G1/G2/G3/G5/G6/G7 and PS-G1 have source-grounded gaps; unexamined guarantees remain unresolved.
Existing test success and independent original-space checks remain substantial evidence with
their stated limits. No claim is made that all current scientific outputs are wrong.

**Architectural fitness: G9 fails.** The most consequential additional gaps are in the
meaning consumed across boundaries: checked reaction versus executed coefficients, missing
composition versus zero, residual relation versus selected function, global time route versus
local storage, source bytes versus parser context, and waiting versus completed native work.

**Overall decision: Revise.** The first review's overall direction is reinforced, but several
of its suggested implementations need the qualifications in slot 9. The additions do not
close existing findings or authorize a remediation implementation.

| Priority | Action | Why it precedes dependent work | Acceptance focus |
|---|---|---|---|
| 1 | Repair context-complete admission and native/resource ownership (FU08/FU10/FU11/FU12) | Bounded existing invariants; wider reuse or concurrency would amplify these gaps | Clean/incremental equivalence, finite canonical values, completion-owned permits and unique-allocation accounting |
| 1 | Make conserved chemistry govern reaction execution (FU01/FU02) | Package extension must not confuse admitted chemistry with an unchecked source vector | Unknown-composition and incomplete-support refusals, authoritative source coefficients |
| 1 | Establish implicit selection/export equivalence (FU06) | Exact lifting and root response depend on what the operation means | Two-root/restricted-root cases, honest relaxation labels and independent candidate checks |
| 2 | Develop connection/storage/temporal contracts (R3/FU04/FU05/R4) | Ports alone do not establish state compatibility, causal execution or conservation across events | Mixed algebraic/dynamic flowsheet, executable recycle, checked event transfer and actual endpoint obligations |
| 2 | Separate compatibility operations and shared workflow semantics (R6–R8/FU09) | Durable evolution must preserve fine diagnostics, occurrence identity and scientific status | Directional readers/writers, read-only open, repeated experiments and typed failure/seed policy |
| 2 | Complete demand-aware physical/mathematical extensions (R1/R2/FU03/FU07/R5) | New scientific forms should compose through existing owners with honest applicability | Explicit coordinate mappings, mixed pair fits, C1/C2 demand and qualified root sensitivity |
| 3 | Apply finer reuse and boundary consolidation (R9/R10) | Benefits depend on the corrected contexts and owners | Fresh attribution over shared math; generated admitted contracts; measured performance only after implementation |

The next remediation plan should combine these dependencies with the first review's open
findings, retaining one disposition owner. Its first acceptance experiments should target the
counterexamples above rather than merely count removed literals, helper files or cache entries.
