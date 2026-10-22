---
title: Python and scientific boundary remediation
status: in-progress
date: 2026-10-21
adrs: []
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [design, libraries, validation]
---

# Python and scientific boundary remediation

Execution state and pause handoff: [the coordinator](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21).

This companion develops R17, R18 and R20/F10 from the
[integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md)
and [recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
The [coordinator](49-library-native-remediation.md) owns rule decisions, live package status,
finding disposition and combined qualification. This **Proposed** implementation is governed
by the recorded target decisions, including RC14 and RC18. It consumes the
runtime request owner from [49e](49e-runtime-and-native-execution.md) and the schema catalogue
and tooling contracts developed by [49g](49g-contracts-maintenance-and-tooling.md); it does not
duplicate their status. [49h](49h-qualification-and-design-publication.md) owns assembled evidence
and durable design publication.

The intended boundary preserves semantic identities, physical coordinates, truthful statistical
validity and bulk Arrow transport. Python authors inputs and consumes results; Rust owns
admission and science. This work applies to every affected side-route consumer, not only the
main solve method. Strict decoding, version-first refusal, original-space qualification, missing
result partiality and caller-independent durable execution remain in force.

## Inspected foundations and evidence limits

Static inspection used HEAD `e08b1f1dd` on 2026-10-21 and the current audited review. No product
tests, builds or probes were run while authoring. Relevant enduring contracts are blueprint §8,
§19.8 and §21 in the [physical](../authoritative_design/sections/physical-semantics.md) and
[workflow](../authoritative_design/sections/workflows-and-results.md) owners. The applied standard
is Core 3.4 with Efficient Architecture 1.0 and Process Simulator 1.5.

[EnumValue](../../crates/pse-py/src/enums.rs) currently projects generated vocabulary outputs
but has no corresponding strict input conversion. Python workflow methods still parse strings
or select native policy. The registry and generator can own vocabulary/identity-role declarations;
the boundary wrapper should consume those declarations rather than become a second vocabulary.
The [boundary inventory](../design_review/evidence/in-use-library-remediation-2026-10-21/inventory-boundary.md)
and [codec evidence](../design_review/evidence/in-use-library-remediation-2026-10-21/recommendation-audit/README.md)
bound the already researched library mechanisms.

The selective msgspec 0.21.1 audit probe exercised direct/nested hooks, hex round trips,
width refusal and NewType erasure for the existing bytes-subclass `SemanticId`. That evidence
removes the need to replace the value class merely to obtain hook dispatch. It does not
establish generated-document/extension integration or runtime distinction among NewType roles.

[ParameterCovariance](../../crates/pse-runtime/src/math/settings.rs) admits extent, distinct
parameters, finite values, approximate symmetry and nonnegative diagonal. Its run id is copied
into [propagation](../../crates/pse-runtime/src/workflow/uncertainty.rs) without reconciling actual
fit content. The source-derived counterexample `Σ=[[1,2],[2,1]]`, `J=[1,-1]` produces variance
`-2`: finite and symmetric does not mean covariance-valid. This counterexample was not executed
through Rust during plan authoring. The fitting covariance owner already withholds unavailable
or unqualified products, so strengthening input admission must preserve those states rather
than manufacturing certainty from a successful multiplication.

## Selected boundary design

### Registry vocabulary and identity transport

Each registry vocabulary gets strict `EnumValue<T>: FromPyObject`, declaring
`INPUT_TYPE = type_hint_identifier!("pse.contracts.enums", T::NAME)` alongside the existing
output hint. Require the actual generated `StrEnum` instance before applying strum `FromStr`;
an arbitrary string or another enum with the same spelling is not an admitted typed input.
This enables generated stubs to show the real input contract. Live event `kind` and `termination`
use the same vocabulary projection or a deliberate PyO3 complex enum, rather than raw strings.
Scientific artifact kind, owner kind and terminal class use the registry declarations also
consumed by the store companion; boundary/store code must not invent parallel enumerations.

[PyO3 0.29.2 FromPyObject](https://docs.rs/pyo3/0.29.2/pyo3/conversion/trait.FromPyObject.html)
supplies `INPUT_TYPE`, and `PyOnceLock::import` supplies cached type lookup. Emit a distinct
type cell for every `RegistryEnum` implementation from the existing `vocabularies!` macro.
A static in a generic implementation is shared across type parameters and would cross-contaminate
enum lookups. Test two different vocabularies in both import orders and multiple interpreters
within the supported PyO3 lifecycle, rather than validating one warm type cell only.

Identity schemas carry the role as `x-pse-identity`, using the existing custom-keyword pattern.
The schema/document generator projects that role to `identities.<Role>` NewTypes. The codec
registers msgspec `enc_hook`/`dec_hook` for `SemanticId` hex transport, while `Identity<R>`
extracts the existing bytes-subclass value and checks exact width. A 128-bit value establishes
identity shape; NewType aliases establish static roles only. Where an operation needs actual
membership or role validity, Rust resolves it against the selected source/context. It must not
treat NewType erasure as proof or add fake runtime role checks that inspect an erased alias.
The [msgspec extension contract](https://jcristharif.com/msgspec/extending.html) supplies the
hook mechanism; the pinned selective receipt and generated integration tests decide adoption.

### Physical bulk ingress and immutable custody

Accept `pyo3_arrow::AnyRecordBatch` for keyed starts, initialization stages, diagnosis samples
and per-state tolerances. A thin local extractor supplies the Python stub hint that pyo3-arrow
does not supply itself. Each operation selects its registry relation contract, including
`pse.semantic_id` keys and `pse.quantity_value` value metadata. Admission checks actual field
shape, key uniqueness, expected membership, required coverage, finite/range rules and physical
unit compatibility. Row order is transport order; state identity controls numerical placement.
Preserve operation-specific sparse or complete coverage semantics instead of globally requiring
every possible state in every table.

Arrow metadata alone cannot prove that an external producer owns immutable storage. Before
asynchronous/native use, admission establishes an owned immutable numeric snapshot with resolved
identity-to-coordinate mapping and physical conversions. It may retain buffers only when custody
proves the needed immutability; otherwise copy once at this trust boundary. The snapshot's lease
lasts through native drain, including Python producer destruction or mutation after submission.
Consumers reuse the admitted snapshot and do not independently rescan metadata or reinterpret units.
This deliberate crossing removes repeated scalar conversion while retaining protection against
uncontrolled external mutation; it is not a zero-copy performance promise.

Return native attempt primals as `PyRecordBatch` with registry-governed identity and quantity
columns, replacing `list[tuple[str,float]]`. The same field metadata must survive C Data/C Stream
ingress and egress through supported Arrow producers. Check round-trip metadata and physical
meaning, rather than assuming the C protocol's presence establishes extension metadata behavior.
The [Arrow PyCapsule protocol](https://arrow.apache.org/docs/format/CDataInterface/PyCapsuleInterface.html)
and [pyo3-arrow](https://docs.rs/pyo3-arrow/) explain transport; the registry remains semantic authority.

Time needs two contracts. Physical origins, steps, sample grids and horizons retain signed,
precision-bearing quantity/time values, including negative origins and sub-microsecond increments
where admitted. Positive wall-clock budgets may use `Duration`/`datetime.timedelta` only where
range, sign and precision fit the budget contract. Pinned PyO3's `std/time.rs` rejects negative
timedeltas and exports microseconds; blanket Float64-to-timedelta conversion would narrow supported
physical science. Any budget conversion states rounding/overflow/refusal behavior explicitly.

### Document catalogue input and consumer contract

The separate R19 owner supplies one `(type, slug, direction)` catalogue with a versioned root
for every exported document, including Completion. It uses schemars 1.2.2 serialization and
deserialization generator contexts chosen through `SchemaSettings` before construction.
Direction participates in canonical schema-set identity. The extension fingerprint reflects
the actual compiled catalogue, and generated Python carries the corresponding expected value;
`pse_ids` owns framing and hashing. Feature-gated non-schema host enums alone do not prove a
schema difference, so that owner establishes real compiled roots and their feature variation.

TOOL01 owns catalogue generation, root registration, the `DocumentValue<T>` return migration,
decoder registration and deletion of duplicated document membership/pairing. This companion
consumes that catalogue through `DocumentValue<T>`, generated codecs and stubs.
Bytes-returning methods stop choosing a Python decoder type by hand. Inputs and recorded documents
retain version-first refusal, strict unknown-field handling and version-specific interpretation.
An import mismatch fails before normal calls. The catalogue's implementation/generator ownership
stays with its companion, while BND01/BND02 supply identity keywords, vocabulary and table-wrapper
types that it needs. Coordinate those shared editing surfaces before the generator cutover.

Operator tooling consumes the same Rust conformance/admission/diagnostics owners after the R21/R22
dependency-direction change. Boundary wrappers expose them without moving scientific policy into
Python. Privileged canonical maintenance, historical restore/bootstrap and generated guarded
mutation work remains with the store/tooling owners; an ordinary writer guard is not a substitute
for their different lifetime and authentication contracts.

### Covariance origin, content and mathematical admission

Separate origin from mathematical validity. A certified fit covariance is hydrated from the
actual available qualified in-process or canonical result, including ordered parameter identities,
matrix, unit/interpretation dependencies and local validity. If accepting a caller's complete
payload for this route, reconcile every one of those fields against the authoritative record.
Resolving a genuine run id alone cannot certify altered values. Missing, withheld, stale or
incompatible fit evidence yields truthful refusal/withholding before propagation.

External covariance remains an explicit supported origin with uncertified statistical provenance.
Its caller-supplied label cannot impersonate a fit result. Prefer a discriminated input contract
that makes `qualified fit reference` versus `external covariance + assumptions` explicit; exact
type names are local implementation choices. Existing versioned settings/documents require the
coordinator's RC18 route and explicit interpretation evolution where serialized shape changes.
Do not silently reinterpret an old caller run id as newly verified provenance.

Both routes require mathematical admission: checked square extent, distinct parameters, finite
values, declared approximate symmetry, and positive semidefiniteness under a declared scale-aware
tolerance. Covariance entries carry products of parameter units. Select a deterministic scaling
into a dimensionless self-adjoint matrix using the admitted parameter scales, and define tolerance
in that normalized space; congruent positive diagonal scaling preserves PSD. This prevents a
single absolute threshold mixing kelvin-squared, pressure-squared and cross units. Scaling and
tolerance become explicit numerical policy and result assumptions, not hidden decoder defaults.

The proposed initial numerical policy uses the recorded fit parameter scales for hydrated
fit covariance, or explicit admitted positive parameter scales for external covariance.
When scales are omitted on the external route, the Rust owner supplies one canonical unit
per parameter and records that choice. For `D = diag(1/s_i)`, form `A = D Σ D`; reject
non-finite/overflowed normalization. Use the infinity norm of this dimensionless matrix and
`tau = 64 * f64::EPSILON * max(n, 1) * max(1, norm_inf(A))` for the symmetry and negative
eigenvalue tolerance. Checked extent conversion and finite norm/tolerance arithmetic are
required: overflowed norm accumulation or a non-finite tolerance refuses/withholds, never
turns the comparison into an unlimited acceptance band. After admitted symmetry, pass the lower triangle to faer and accept
only finite eigenvalues with minimum at least `-tau`. This is a declared proposed numerical
tolerance, to be checked against independent scaled/singular/indefinite controls; it is not
an established error bound for every problem. Record scales, triangle and tolerance policy
with validity. Do not silently repair the covariance. A propagated negative variance is
withheld as numerically unresolved/invalid rather than published as certified or clamped;
this also covers a tiny negative direction admitted only within the eigenvalue tolerance.

Pinned faer 0.24.4 offers `self_adjoint_eigenvalues(Side)` returning eigenvalues or `EvdError`
without eigenvectors ([solver source](https://docs.rs/faer/0.24.4/src/faer/linalg/solvers.rs.html)).
After square/symmetry admission, choose and declare the triangle passed to this routine. Accept
zero eigenvalues and numerically admissible values within the selected tolerance; reject/withhold
genuinely negative eigenvalues. A positive-definite-only Cholesky gate would reject legitimate
singular covariance and is excluded. Do not silently project an invalid matrix to the PSD cone
or clamp a negative propagated variance into an apparently certified value.

The high-level faer wrapper uses global parallelism and allocates its own scratch. The selected
integration must control those actual effects: use the underlying `self_adjoint_evd_scratch`
and `self_adjoint_evd` with `ComputeEigenvectors::No`, an admitted `MemStack` and explicit admitted
parallelism where the wrapper cannot satisfy that contract. This remains faer's eigensolver,
not a project factorization. Its workspace requirement plus normalized matrix and eigenvalue
storage are admitted before the call. No unrelated eager global-pool initialization is required.
Eigensolver scratch, working storage and parallelism use existing resource admission. Extent
overflow, non-finite scaling/eigenvalues, decomposition failure, cancellation or exhausted capacity
produce a typed unavailable/refusal outcome. If factorization cannot stop immediately, its native
lifetime still drains before release. The resulting `JΣJᵀ` retains identity-based column alignment
and existing Jacobian validity. Derived validity is the conjunction of mathematical admission,
upstream statistical/local assumptions and derivative validity; successful propagation does not
promote external origin to certified fit provenance. Keep reported provenance and availability
separate from the matrix's mathematical check.

## Packages, dependencies and adoption

RC14 carries the keyed physical-input/time-contract change and RC18 carries covariance
origin/content/validity. The coordinator records each explicit operator decision and the
necessary boundary/architecture/version routes. The recommendation here preserves signed and
precise physical time, separately scoped budgets, qualified content hydration and explicit
external PSD covariance. An accepted route must precede its dependent input/serialized-contract
change; a rejected item requires adaptation rather than treating review adoption as authorization.

Package status is maintained only in the coordinator. The following definitions identify complete
behavioral units and required consumers, leaving local names and implementation layout open.

| Package | Required product | Delivered contract and retirement |
|---|---|---|
| BND01 — vocabularies and identities | Registry-owned enum/identity declarations shared with store; catalogue keyword support | Strict enum inputs/outputs, per-type cached lookup, retained SemanticId hooks and generated static role hints; delete hand vocabulary/hex parsing at migrated boundaries. |
| BND02 — physical Arrow boundary | Registry relation contracts; runtime admitted-input interface from RUN03 | Owned keyed starts/settings/diagnostic inputs and typed primal output; migrate every affected side route and retire positional/float/list entry paths they replace. |
| BND03 — covariance content and origin | Coordinator RC18 decision; fit result hydration/availability contract; catalogue version evolution | Qualified content lookup/reconciliation and explicit external origin, carried through settings, propagation, results and Python codecs; remove id-only provenance assumptions. |
| BND04 — mathematical covariance validity | BND03 origin/validity types; explicit scale and numerical tolerance policy | faer PSD admission preserving singular matrices, resource/failure handling and unchanged identity-aligned propagation; remove diagonal-only acceptance as a sufficient covariance guarantee. |
| BND05 — scientific boundary adoption | TOOL01's working catalogue/fingerprint/typed returns plus BND01–BND04 products and runtime preparation entry | All affected scientific document/table consumers expose truthful availability/provenance and physical custody through Rust and Python; remove remaining local scientific-input interpretations. |

BND01 can establish strict enum conversion before the full catalogue arrives; identity-role
generation needs its actual keyword contract. BND02 needs a working runtime custody/admission
interface, but does not require the entire native capability rewrite. BND03 and BND04 are one
scientific cutover for externally publishable results: an intermediate origin type cannot be
called corrected covariance admission until the mathematical check is reached. BND04's pure
matrix admission tests can be developed independently of store startup. BND05's end-to-end
document cutover waits for the actual compiled catalogue, not only an interface sketch.

The complete migration footprint is enum-bearing settings, live simulation events, document
identity fields, starts (`with_primal_start` and stored-start ingress), block-initialization stage
inputs, diagnostic sample inputs, per-state absolute tolerances, other positional state-setting
inputs discovered at these owners, native primal outputs, and covariance settings/fit/propagation
results. Trace their Rust producers, binding extractors, Python codecs/stubs and downstream table
consumers before retiring each old path. A representative method passing is not migration closure.
Unit/quantity resolution remains a project-domain decision: do not implement a free-form resolver
without establishing how units map into registry identity domains. This does not delay keyed
inputs whose physical context and conversions are already owned.

## Verification and qualification limits

The following checks are **Proposed** acceptance, with zero failures as the baseline. The
historical msgspec hook probe is **Tested** only on its recorded pinned selective conditions;
the faer method is **Interface-checked**, and the PSD tolerance/integration remains **Proposed**.
No runtime or scientific qualification is claimed by writing these documents.

| Scope | Targeted independent controls |
|---|---|
| Vocabulary and identity | Reject raw strings, wrong enum classes and malformed width; accept the generated enum; exercise two enum imports in both orders; nested/version-first documents round-trip retained SemanticId hex hooks; erased NewType roles cannot bypass source/context membership. |
| Arrow custody and physical meaning | Reorder keyed states and preserve numerical placement; reject duplicate/unknown/missing-required keys and incompatible units; accept valid conversion; mutate/drop the external producer after submission and observe unchanged admitted values through native drain. |
| Every side route | Starts, initialization, diagnosis and per-state atol consume the keyed relation; operation-specific partial coverage stays correct; output primals preserve field/extension metadata across supported Arrow C producers. |
| Time | Negative physical origin and sub-microsecond step retain meaning; wall-budget conversion tests exact allowed precision/range and typed overflow/refusal; no generic timedelta conversion reaches physical sample grids. |
| Catalogue adoption | Generated stubs and runtime extraction agree; nested identity documents decode correctly; stale extension/Python schema-set pair refuses at import; input/output direction and actual feature-dependent roots participate without collisions. |
| Content and origin | Genuine fit id plus altered matrix, parameter order, units or validity cannot obtain certified provenance; hydration matches the complete qualified result; missing/withheld fit stays unavailable; explicit external covariance remains external after successful propagation. |
| Mathematical covariance | Reject `[[1,2],[2,1]]`; show the independently specified `J=[1,-1]` oracle yields `-2` without admission; accept zero and rank-deficient PSD matrices; test scale-sensitive mixed-unit cases and values on both sides of the declared eigenvalue tolerance. |
| Numerical failure and publication | Refuse extent overflow/non-finite/decomposition failure and respect resource limits; retain singular PSD support; withheld Jacobian/covariance withholds derived results; negative variance is never silently clamped into a certified output. |

Implementation uses focused Rust/Python unit checks and package compile recipes while each
behavior is developed, with force-validation where Arrow is consumed. The coordinator selects
final component, linked solver, Python and multi-process/durable journeys, schema generation,
static/manual checks and any required measurements after all functional scope is implemented.
The assembled controls must exercise a real fit-to-solve covariance journey and a Python keyed
input through asynchronous/native ownership, not only codec or matrix helpers. Latency, copying
and capacity benefits remain unmeasured until a named representative workload is compared.
