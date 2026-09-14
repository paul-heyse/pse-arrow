# Design review: blueprint revision 4 and library contracts

> Historical assessment of revision 4, identified by the digest below. The
> [revision-5 follow-up](design_review_blueprint-rev5-contracts_2026-09-13.md) reviews
> the resulting amendment; the findings and probe receipts here remain unchanged.

## 1. Decision and scope

**Decision: Revise.** Retain the Arrow relation authority, separate mathematical IR,
explicit case overlays, and native numerical execution. The largest immediate
improvement is to make existing library mechanisms enforce the proposed contracts.
Several mechanisms currently credited with enforcement provide only an interface
or a declaration. Adding more infrastructure before correcting that distinction
would increase the number of places that can disagree.

**Proposal:** [blueprint revision 4](../../authoritative_design/blueprint.md).
**Standard:** [Data Model–Based Design Charter, version 1.0](../design_principles/DATA_MODEL_DESIGN_CHARTER.md),
including DM-01–DM-60 and independent gates G1–G7.
**Reviewer:** Codex. **Date:** 2026-09-13.
**Evidence status:** the platform design is **Proposed**. Selected library routes
are **Interface-checked**; six standalone characterization groups are **Tested**
under the conditions in §9. This is not an implementation acceptance review.

**Observable outcome:** a compiler whose model meaning survives validation,
inference, numerical preparation, storage, and backend transfer; ordinary process
model extensions add declarations and any genuinely new algorithm, with mechanical
adapters derived from those declarations.

**Baseline and constraints:** the repository is skeletal. Existing foundation
checks do not establish a functioning modeling or solving lifecycle. The review
covers the proposed local snapshot/compiler/native/NL/Pyomo architecture and its
stated delivery phases. It does not require a distributed runtime or a new query
engine. Scientific parity with IDAES 2.12.0 remains a future behavioral gate.

### Method and coverage

The analysis reconstructed authority, pass dependencies, invariants, and failure
paths from blueprint §§2, 4–9, 12–24, then compared them with the charter. The heater
and lumped control-volume examples, property-demand algorithm, and phase exits
provided concrete consumers for the extension and proportionality tests.
Blueprint §§10–11 and Appendix A were sampled for architectural integration;
individual thermodynamic correlations, all unit templates, and the entire IDAES
coverage matrix were not independently certified.

The [Arrow](../../capability-maps/arrow-rust.md),
[DataFusion](../../capability-maps/datafusion-rust.md), and
[supporting Rust libraries](../../capability-maps/supporting-rust-libraries.md)
maps supplied discovery leads. Their relevant claims were compared with the
current blueprint, existing probe source, and pinned upstream source. Earlier
reviews were consulted for context; their acceptance statements were not reused
as current proof. The Python contract in blueprint §21 was inspected as a design;
the Python capability map and its behavioral probes were not re-audited.

Context7 was used for DataFusion provider contracts, Arrow extension validation,
and UDF predicate inversion. Versioned upstream documentation supplemented it.
Retrieved examples were treated as leads: Context7 returned some historical or
generic snippets that cannot establish a 55.1.0 contract. Final library findings
rest on the pinned source and executable probes, not snippet ranking.

Local evidence included `just metadata`, Cargo resolution of the scratch probe,
Arrow 59.3.0 and DataFusion 55.1.0 source checkouts, and cached rustdoc JSON whose
`crate_version` and format version 61 were checked. The cached JSON was not
regenerated; no completeness claim is made for that extraction. The source
checkouts matched tags `59.3.0` and `55.1.0`. The probe resolved 292 packages, with
zero third-party versions absent from the repository lockfile.

The inspected blueprint has SHA-256
`fa3e959a0005f97c284818eaeae4af94720fa6f001d4b9f10f98c4a44f5b4cca`.
Review began at `4becdebe856cd52cac944c5114dd48f1e0e5048e`; unrelated concurrent
commits advanced HEAD to `fb0717d9151ada4884bd7ccb5efc27422bb92f7a` during probing.
The blueprint, charter, Cargo manifest, and lockfile were confirmed unchanged
across that movement. Probe receipts bind the actual execution inputs.

Adversarial checks attacked extension admission, conditional evaluation, casting,
canonical bytes, protobuf determinism, floating-point rewrites, pass scheduling,
cache dependencies, and exact-filter verification. Crash recovery, solver FFI,
numerical derivatives, actual provider scans, and end-to-end performance remain
**Proposed** guarantees, not behavior verified by this review.

## 2. Authority and lifecycle map

All enforcement descriptions in this table are proposed platform mechanisms.

| Concept or fact | Semantic type and identity | Authority / owner | Revision or snapshot boundary | Permitted update path | Derived representations |
|---|---|---|---|---|---|
| Relation and operation contracts | Relation IDs, schema versions, operator and kernel IDs | Registry and versioned operation declarations; blueprint §§4, 7.3, 6.11 | Registry/package revision | Registered declaration change; generated artifacts follow | Schemas, views, validators, adapters, documentation |
| Model definition | Authored semantic IDs; explicit versus named policy | Authored/reference relations; §§5.1, 6.1 | Model revision, P2-valid commit | Change set with base revision and conflict checks | Normalized facts, inference, indexed mathematics |
| Case bindings and policy | Case revision; identity-based target rows | Case specifications, discretization policies, solver profiles; §§6.10–6.11 | Case/overlay revision | Change set; child overlay and priority resolution | Bound variables, substitutions, scaling and solve plans |
| Observations | Observation/dataset IDs and source references | Authored observations; §6.10 | Dataset/case snapshot | Explicit import and target resolution | Estimation objectives and diagnostics |
| Selected providers | Method/kernel identities and artifact digests | Persisted resolution from declarations; §§9.6, 18.3 | Compiled artifact and run | Explicit selection policy; runtime host probe | Kernel bindings, resolved solver options |
| Mutable execution state | Run ID, attempt, stage and parent run | Run controller and owned numerical workspace; §§17–18 | Individual execution attempt | Declared stage actions and cancellation | Last iterate, residuals, events, terminal status |
| Results and provenance | Run-keyed rows; derivation IDs | Recorded observations of execution; §§6.13, 20.3 | Published result bundle | Append/publish through the artifact protocol | Reports, stream tables, semantic diffs |
| Stored representation | Canonical content hash versus encoded bytes | Canonicalizer and artifact writer; §§5.3, 20.1 | **Inconsistent as written**: stream identity versus file/Parquet byte verification | Manifest publication after validation | Storage objects and pinned catalog tables; R4-03/R4-04 |

**Deliberately opaque behavior:** constitutive kernels, numerical solvers,
structural algorithms, and external-function libraries can remain ordinary code.
Their selection, units, domains, derivatives, effects, failure policy, and source
digests must be explicit. `KernelSpec` does not yet contain every policy that its
generated adapters are instructed to derive (R4-10).

**Identity behavior:** creation-time IDs and explicit rename semantics are a real
strength: an ordinary rename cannot retarget an old case through a reused name.
Artifact ordinals remain local. Hashes, however, still require corrections for
physical encodings and ignored null payloads. Stable entity identity should not
be confused with identical whole-relation hashes when labels or provenance change.

## 3. Semantic contracts and invariants

| Contract or invariant | Representation | Enforcement boundary | Failure behavior | Verification evidence |
|---|---|---|---|---|
| A relation has its declared structure and valid relationships | `RelationSpec`, extension fields, invariant plans | Generated readers/builders and P2 | Typed invariant errors | **Proposed**; registry registration alone does not validate ordinary queries, E1 |
| Computed values retain their physical meaning | Quantity types, conversion rules, rule-head fields | P10 and rule-output admission | Reject incompatible interpretation or apply a named conversion | **Unresolved** for composite quantity typing; numeric casts demonstrably lose information, E3 |
| A guard suppresses an excluded computation | Conditional expression and kernel contract | Physical execution, including native tape | No evaluation error from an unselected branch | **Tested** library counterexample E2; built-in `CASE` is an available route |
| Content identity ignores irrelevant representation differences | `pse.canon.v1` | Canonicalizer, before reuse/publication | Reject malformed content; normalize irrelevant bytes | **Tested** counterexample E4 for hidden primitive values under nulls |
| Every required property is resolved before realization | Requirements and persisted method resolutions | P6 followed by P7–P9 | Explicit unresolved/ambiguous diagnostics | **Unresolved** scheduling contract: P6 also reads later outputs, R4-06 |
| Reused artifacts have complete dependencies | Pass inputs and memo keys | Memo lookup and per-instance expansion | Miss/recompute when any semantic input changes | **Violated as specified** by the explicit P7 key, R4-07 |
| Readers observe a complete revision/result | Immutable artifacts, manifest, conditional ref update | Publication and run terminalization | Conflict/failure leaves old ref valid | **Proposed** protocol is sound in outline; digest roles require a decision, R4-03 |
| Advertised backend capabilities are complete | Operator/kernel contracts and backend bindings | P16 plus runtime capability resolution | Typed unsupported capability | **Unresolved** descriptor/table disagreements, R4-10 |

**Absence and uncertainty:** free unknowns, absent initial guesses, and unbounded
values are deliberately distinct. Four-valued inference also has an explicit
undecided relation. Preserve those choices. Two remaining ambiguities need closure:
blueprint §18.5 permits invalid kernel evaluations to become null while diagnostics
are optional; §7.6 permits missing observations as null although §6.10 declares
`observations.value` without its nullable marker. A consumer must not infer
invalidity, missing input, or failed evaluation from the same unqualified null.

**Equivalence:** maintain separate contracts for entity identity, canonical logical
content, physical artifact integrity, symbolic mathematical equivalence, and
floating-point agreement. Blueprint §24.1's `1e-10` relative backend comparison
and §24.2's `1e-6` relative solution comparison do not authorize arbitrary
overflow-eliminating rewrites, precision-losing casts, or an unstated fused
multiply-add policy. Near-zero and non-finite cases also need explicit comparison
rules rather than an unspecified relative-only check.

## 4. Derivation and execution design

| Stage or operation | Input revisions and dependencies | Output contract | Preconditions / assumptions | Effects and mutable ownership | Provenance / invalidation |
|---|---|---|---|---|---|
| Author and commit | Documents, registry, base revision | P2-valid authored snapshot | Targets resolve; structural and relational invariants hold | Atomic change-set publication | Source spans and revision parent |
| Normalize and resolve | Authored facts, package contracts, feature and domain declarations | Normalized facts, decided requirements, selected methods | Demand seeds exist independently of their realization | Pure computation over snapshot inputs | Rule/supporting-row evidence; stage inputs must be complete |
| Instantiate and expand | Resolutions, template bindings, domains, laws | Indexed then scalar mathematical artifacts | Guard and domain decisions complete | Local staging; no authored mutation | Semantic IDs and explicit source-to-output derivations |
| Bind and prepare | Graph, case, parameter/fixed values, numerical policy | Ordered problem, analysis, tape, backend bundle | Valid units, capabilities and selected approximation | Owned preparation workspace | Value/assumption dependencies; R4-07/R4-09 |
| Execute | Prepared artifacts, resolved options, provider binaries | Typed run outcome and result bundle | Domains guarded; derivatives available as selected | Mutable numerical workspace, solver/subprocess effects | Attempt IDs and source-linked events |
| Publish and inspect | Complete artifacts and manifest | Pinned immutable snapshot; read-only projections | Verified logical content and physical integrity | Conditional ref update; inspection changes no model meaning | Reproduction inputs and report lineage |

There is a concrete cycle in the documented stage dependencies:

```text
P6 property-demand closure -> P7 template equations -> P8 law equations
          ^                       |                        |
          +---- equation/contribution demand per §9.6 -----+
          ^
          +---- initialization-plan requirements from P15
```

P15 itself consumes the problem produced downstream of P6. This is different from
the specified fixed point *inside* a rule stratum. A bounded correction is to
derive demand seeds from normalized templates, laws, ports, displays, and declared
initializer requirements, then realize their closure. Alternatively, explicitly
contract a mutually recursive resolution stage with monotonicity and termination
conditions. The generic DAG driver cannot infer either decision from §14.1.

The shared `compiled.math_*` names also need stage-qualified artifact inputs and
outputs. P7–P10 produce or replace overlapping relations, while `RelationSpec`
offers one `producing_pass_id`. A pass-local artifact bundle or versioned stage
handle is enough; this does not require a new database or a new physical layout.

**Relationship structures:** containment, process connectivity, expression
dependencies, structural incidence, scheduling, and provenance remain distinct.
The use of petgraph for specific derived graphs is appropriate. A process recycle
must not be rejected because its scheduling projection needs a tear or SCC.

**Provider and boundary contract:** the catalog correctly pins a manifest; the
numerical problem remains whole; unsupported backend bindings fail before solve.
Add active semantic validation before DataFusion optimization and at output
admission. Preserve the complete input row set when pushing filters: physical
pruning may exclude only rows proved irrelevant to the selected query.

**Coherent publication:** retain immutable object creation, a complete manifest,
and one conditional ref update. Specify separately the checksum of each stored
object and the canonical relation hash it represents. An existing content-named
object can be reused only under a stated integrity/trust contract; its name alone
is not an integrity check after corruption or incomplete external restoration.

## 5. Representative journeys

### Ordinary extension: add a property method

A new equation-based heat-capacity correlation should add its method declaration,
parameter quantities and natural units, expression template, and independent
value/derivative fixtures. P6 selects it; P9 instantiates it; generated boundary
code follows the same contract. No new provider framework is needed.

For a genuinely new opaque method, one Rust algorithm is legitimate additional
work. Its descriptor must also declare the null/domain policy, purity, branch
semantics, derivative conditions and actual backend routes. Merely setting
`Immutable`, `is_strict`, monotonicity, or Hessian availability in an adapter would
introduce an independent semantic decision. A method claiming a property but
omitting its realization must fail conformance even if demand closure terminates.

### Meaningful change: add a species to a bound package

Keep the instance ID, template contents and Boolean features unchanged; extend
the material package with a third species. Its state symbols and balance domains
must expand. The explicit per-instance P7 memo tuple in §14.3 is unchanged,
although the required output differs. A fresh compile and an incremental compile
therefore have no specified reason to agree. Include the resolved domain/package
and binding artifacts in the expansion input contract, or use a conservative
complete-stage key until finer invalidation is justified.

A related value journey must distinguish a fixed value from its initial guess.
Blueprint §18.1 binds fixed and parameter symbols outside the solver vector, but
§14.4 describes a fixed-value change as changing only `variable_order.initial`.
The bound-value segment must be refreshed explicitly even when its tape and
sparsity can be reused. Conservatively rebuilding that segment is inexpensive
compared with silently solving the prior case.

### Boundary: rule output and stored artifact

An engine expression emits `Float64(1.9)` for an integer target. Both
`can_cast_types` and `safe: false` accept the conversion; the result is `1` (E3).
Reattaching the target field metadata cannot restore the lost fraction. The same
storage type is even less informative for quantities: attaching a Pa field to a
bar-valued result performs no factor-of-100000 conversion.

For storage, the same two-row relation has canonical-option IPC stream, IPC file,
and Parquet encodings of 648, 954, and 773 bytes in E4. The file and Parquet digests
both differ from the canonical stream digest. Blueprint §§5.3 and 20.1 currently
ask that one target hash mean both things. These are compatible only after their
roles are separated or storage verification explicitly decodes and canonicalizes.

### Interruption and guarded failure

The cancellation design is useful: a failed native solve retains its last iterate
under a terminal failure status, and a killed NL process contributes no partial
SOL result. That is **Proposed**, not exercised here.

An earlier failure occurs without cancellation: a conditional kernel selects a
valid branch, yet the generic DataFusion UDF evaluator evaluates the other branch
first. E2 reproduces this with a failing argument and both conditional flags set.
Built-in `CASE` succeeds on the same input. The native tape needs the corresponding
control-flow contract; an unconditional topological sweep cannot implement lazy
guards simply because the opcode is named `Conditional`.

## 6. Acceptance gates

These are document-stage verdicts. A pass means the inspected design has an
identified authority/mechanism, not that a skeleton implementation has passed it.

| Gate | Pass / fail / unresolved / not applicable | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Pass, Proposed** | D1/D2/D13, §§5.1, 6.10, 22.2 identify one write path, stable entity identity, case ownership, and non-authoritative projections. No independently editable competing model was identified in the inspected scope. | Preserve these boundaries; separate the two artifact identities in R4-03. |
| G2 — Semantic fidelity | **Fail** | Lossy casts, invalid/null collapse, incomplete composite quantity rules, and noncanonical null bytes; R4-04/05/08/09/10. | Specify and enforce semantic conversions, state distinctions, canonicalization, and numerical equivalence. |
| G3 — Validity | **Fail** | The claimed automatic extension validation route does not execute on an ordinary query; E1, R4-01. Generated views remain useful but do not establish the broader claimed plan boundary. | Invoke generated validators at every supported admission path and recheck outputs. |
| G4 — Hidden behavior | **Fail** | Conditional flags are optimizer declarations; generic UDF argument execution remains eager. Native control flow is unspecified; E2, R4-02. | Use actual guarded execution and declare kernel effects/failures. |
| G5 — Consistency and recovery | **Unresolved** | Manifest/CAS and terminal-run separation are specified, but physical-byte verification conflicts with logical content naming; R4-03. Actual interruption paths were not run. | Decide the identity/integrity protocol and exercise interrupted publication. |
| G6 — Transformation and reuse | **Fail** | P7's declared key omits semantic inputs; rewrites lack an adequate numeric contract; retyping loses information; R4-05/07/09. | Complete dependency keys and transformation conditions; compare reuse with clean recomputation. |
| G7 — Truthful capability claims | **Fail** | Registration, conditional flags, and sorted HashMap insertion are credited with guarantees they do not provide; E1/E2/E5. Backend tables also disagree; R4-10. | Correct the claims and derive executable capability checks from one contract. |

An unresolved gate is not a pass. No maturity total is computed.

## 7. Principle findings

Rows are grouped by structural cause. P1 denotes correctness work required before
implementing the affected contract; P2 denotes verification or operational work
that must precede the associated acceptance claim.

| Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| **R4-01 — P1: extension registration is mistaken for active validation** | DM-07, DM-43, DM-59 | Blueprint §§4.3–4.4 claim validation of storage and metadata “for every plan.” Existing map probe X1 directly invokes `create_extension_type_for_field`; it does not establish query admission. E1 registers the factory, then successfully queries both a malformed known extension and an unknown name; factory calls during query: zero. | An implementer relying on the stated session guarantee admits a semantically invalid field into an operation that assumes its extension contract. | Generate one recursive field validator; invoke it at provider/bundle admission, relevant plan boundaries and result admission. Install a platform `AnalyzerRule` using the existing registry to reject invalid plan fields; retain generated Arrow view validation outside the engine. Small generator and catalog/rule integration surface. | An ordinary supported query/import path must reject wrong storage, unknown name/version, malformed metadata, and nested extension errors before execution. Include a direct-helper positive control; do not count that control as admission coverage. |
| **R4-02 — P1: conditional declarations do not implement conditional execution** | DM-24, DM-28, DM-43 | Blueprint §18.5 says `conditional_arguments`/`short_circuits` ensure an excluded branch is never evaluated. E2 and the pinned `ScalarFunctionExpr::evaluate` show eager argument evaluation. §18.2 describes one topological tape with no branch/mask execution contract. | A valid conditional expression fails on a domain error in an unselected branch; hoisted common subexpressions can reproduce the same error. | Lower relational conditionals to built-in `Expr::Case`/`CaseExpr`; make native branch/mask control explicit. Keep UDF flags as truthful optimizer restrictions. Custom physical expressions are needed only if built-in control flow cannot express an actual kernel requirement. | Mixed batches selecting different branches; excluded division/log failure; nested conditionals; common-subexpression optimization on/off; native and DataFusion outcomes agree on values and errors. E2 supplies the minimal counterexample. |
| **R4-03 — P1: logical content identity and stored-byte integrity conflict** | DM-12, DM-15, DM-14, DM-42 | Blueprint §5.3 hashes canonical IPC streams and excludes Parquet bytes. §20.1 names IPC files and Parquet objects with that content hash but requires each write to hash “the bytes actually serialized” against its target name. E4 confirms the three encodings have different digests. | A conforming physical writer rejects valid artifacts, or substitutes a physical hash and loses the specified logical identity and reuse behavior. Readers cannot implement the two checks consistently. | Distinguish canonical relation hash from encoding checksum/format in the manifest, or explicitly decode and canonicalize for logical verification while checking physical completeness separately. Define existing-object validation and the snapshot's relation membership, excluding self-referential bookkeeping. Local manifest/writer/reader contract change. | Persist one logical relation as stream, file and Parquet: common logical hash, distinct valid encoding checksums, identical decoded values. Corrupt/truncate each representation and interrupt before manifest/ref publication; no invalid snapshot becomes visible. |
| **R4-04 — P1: canonicalization leaves physically hidden values significant** | DM-15, DM-32, DM-48 | Blueprint §5.3 specifies sort, concatenate, dictionary expansion and NaN normalization, but no normalization of values beneath null validity bits. E4 gives equal nullable Int64 columns different null payloads, takes rows in PK order, concatenates, and writes with the stated IPC options: bytes differ. | Equivalent relations get different hashes after construction or an encoding round trip. Cache reuse and deterministic replay depend on buffer history. | Define recursive canonical representation of null slots, nested masked children, offsets, dictionary metadata and other ignored physical content. Normalize the hashing copy, preserving runtime data separately. Treat canonicalization as a typed algorithm, not an IPC writer option. | Metamorphic null-payload, all-valid bitmap, sliced/nested array, empty-domain, dictionary and Parquet-round-trip fixtures. Equal declared content must hash identically; changed valid values must differ. E4 is one counterexample, not full coverage of these types. |
| **R4-05 — P1: castability is not lossless or semantically compatible retyping** | DM-06, DM-24, DM-41, DM-42 | Blueprint §14.2 rule 6 uses `can_cast_types`, `CastOptions { safe: false }`, then reattaches target metadata. E3 silently converts 1.9 to 1 and rounds 9007199254740993 through Float64. Neither API checks quantity kind, basis, reference state or units. | A bad rule can produce numerically altered values or relabel incompatible quantities as a valid head relation. Overflow rejection does not prevent either case. | Derive an explicit admissibility policy from the head contract: exact type/semantic agreement by default; named conversions for units/bases; narrowly allowed numeric conversions with representability checks. Check original semantic evidence before attaching the destination schema. | Reject fractional-to-integer, large-integer-to-float precision loss, same-storage wrong units/reference state, nested metadata loss and nullable-to-required violations. Admit only explicitly declared exact/converting cases. |
| **R4-06 — P1: the pass graph and property-demand seeds are not decidable as written** | DM-18, DM-21, DM-22, DM-31 | Blueprint §9.6 takes instantiated P7 equations, P8 contributions and initializer requirements as P6 inputs; §14.1 requires P6 before those consumers and derives a DAG. P7–P10 also write overlapping `math_*` relations despite one producing-pass field in §4.1. | Two implementers must invent different sequencing, staging or hidden repeat loops; a requirement introduced by a realized method or initializer can be missed or discovered too late. | Specify normalized demand-seed extraction or one explicitly contracted joint fixed point. Name stage artifact bundles and their producer/version boundaries. Retain the external fixed-point executor for provenance where needed. | Validate the stage dependency graph mechanically; exercise a method requiring another property, a law-only property and an initializer-only bubble-point requirement. Reordering independent stages must preserve complete resolved output. |
| **R4-07 — P1: expansion reuse omits bound semantic inputs** | DM-31, DM-32, DM-33 | Blueprint §14.3 names P7's key as `(instance_id, template hash, resolved features)`, while §6.6 and §11.2 bind domains and child models from property packages and parameters. §14.4 also describes fixed-value changes as only changing initial values although §18.1 binds fixed values separately. | Adding a species, changing a bound unit set or a non-feature template binding can reuse old symbols/equations. A fixed-value change can retain stale bound values under an overly literal implementation. | Define a complete instantiation input bundle, including binding/domain/package contracts and relevant reference data; key by its content. Specify bound-value refresh independently of tape reuse. Start with conservative whole-stage invalidation. | For each dependency change, compare incremental output with a clean compile, including added/removed species, domain members, package units, child bindings, fixed values and parameter treatment. Identical hashes for untouched outputs alone are insufficient. |
| **R4-08 — P1: composite physical typing is not closed over the declared distinctions** | DM-01, DM-06, DM-07, DM-24 | Blueprint §8.3 makes every `Mul/Div` result a point and mostly combines dimensions, while §8.1 requires quantity kind, basis, reference state, scale kind and shape. The result rules for those other components are absent. `weighted_mean` is required by §§7.4/8.3 but has no explicit operator/payload discriminator in §6.9/§7.2. | With symbolic dimensionless α, `α·ΔT` becomes a point, making `T + α·ΔT` an illegal point-plus-point sum. Multiplying a referenced enthalpy into an energy-flow term leaves implementers to invent how its reference convention survives. | Complete the quantity-operation algebra, including multiplication by dimensionless factors, reference/basis propagation and explicit weighted-mean recognition. Keep dimension checks as one component. One operator/quantity contract addition should drive all adapters. | Positive/negative physical-type fixtures for scaled differences, weighted means, gauge/absolute pressure, molar/mass flows, and enthalpy transport across incompatible reference states. Test through composed expressions, not only direct additions. |
| **R4-09 — P1: symbolic rewrites lack a consistent floating-point policy** | DM-24, DM-40, DM-48 | Blueprint §7.3 calls `log(exp(x)) → x` unconditional; §7.4 merges/reorders affine terms; §18.2 emits fused multiply-add sequences. §7.3 simultaneously requires opt-in reassociation, and repository Rust rules prohibit incidental FMA changes. E6 produces infinity versus 1000 and 1 versus 0 under these algebraic changes. | A rewrite changes domain/error behavior or finite results far beyond the stated tolerance, with no persisted selection explaining the difference. | Decide whether each stage preserves guarded real semantics or a specified floating-point evaluation. Guard range-sensitive rewrites; make reassociation/FMA and approximation policy explicit, versioned, and part of preparation/reuse inputs. Distinguish an affine combination of arbitrary subexpressions from a problem affine in decision variables before §21.2's `LinearExpression` assertion. | Overflow/underflow, cancellation, signed zero, boundary derivatives and affine combinations containing nonlinear children; compare policies and backends with stated absolute/relative/non-finite rules. E6 is a numerical counterexample, not a benchmark. |
| **R4-10 — P1: the kernel/result contract cannot supply all promised adapter semantics** | DM-08, DM-28, DM-43, DM-44, DM-47 | Blueprint §18.5 derives strictness and validity-to-null behavior from `KernelSpec`, but §6.11 lacks a strictness/null-output policy; invalid outputs have diagnostics only “when requested.” §8.2 requires kernel natural units not present explicitly in that record. `ImplicitRef` is unavailable in DataFusion in §7.2 and supported in §18.9. Missing observations are nullable in §7.6 and non-null in §6.10. | Missing input and invalid computation collapse into null; an aggregate can ignore failed evaluations and return an apparently ordinary result. Independent adapters must invent unit/failure/capability behavior. | Extend one authoritative operation/result contract with the actually required policies and units; default to a typed error for scalar evaluation failures, or return mandatory typed outcome data for tolerant batch analysis. Generate capability matrices from binding declarations and conformance scope. Do not infer auxiliary implementations from a signature. | Compare scalar, batch, UDF, native, NL and Pyomo behavior for missing inputs, invalid domains, unsupported derivatives, natural-unit conversions and each conditional binding. Reconcile matrix entries and observation nullability mechanically. |
| **R4-11 — P2: exact-pushdown verification detects extra rows but not missing rows** | DM-53, DM-54, DM-60 | Blueprint §§5.4/24.1 reapply the advertised predicate to returned batches and check for survivors. A provider returning an empty batch passes that test for every filter. The recorded two-element `IN` rewrite also does not establish a universal expression-shape guarantee. | An implementation can silently prune matching facts; the prescribed test reports success, and incomplete inference propagates downstream. | Compare complete query results with pushdown disabled, including row multiplicities and values. Use DataFusion physical predicates/Arrow filters to avoid a second predicate semantics implementation; advertise unsupported shapes conservatively. | Exact versus Unsupported providers on empty/nonempty inputs, disjunctions, nulls, long `IN` lists, aliases, dictionary keys, projection omitting filter columns and limits. A deliberately over-pruning provider must fail the test. |
| **R4-12 — P2: sorted insertion does not canonicalize protobuf metadata** | DM-15, DM-48, DM-59 | Blueprint §§4.3/14.2 claim sorted insertion into `HashMap` makes plan bytes reproducible. E5 creates 32 fresh maps in sorted key order: 32 protobuf encodings, one IPC encoding. The previous review/map recommendation did not fix the actual iteration-order problem. | Deterministic plan-evidence tests remain flaky; fingerprints differ across equivalent plans. This is now an evidence/replay defect, not a stale memo-key defect: the blueprint correctly removed plan fingerprints from reuse keys. | Keep semantic memo keys. Either make plan bytes explicitly noncanonical evidence, or canonicalize the protobuf representation recursively before encoding using a controlled representation. Sorting map insertion alone is insufficient. | Encode with independently constructed schemas and fresh processes; compare field and schema metadata at nested levels. Do not assert cross-version protobuf stability. |
| **R4-13 — P2: a query memory pool is credited with a broader resource guarantee** | DM-30, DM-35, DM-39, DM-59 | Blueprint §14.3 says a bounded `FairSpillPool` turns exhaustion into a typed error, but §5.3 collects an entire relation into one batch and native/Arrow allocations have no stated reservation path. DataFusion `try_grow` is fallible; `grow` is infallible. Arrow's optional pool has infallible `reserve`/`resize` and is not a rejection mechanism. | Query operators can stay within their pool while canonicalization, results or concurrent snapshot sessions exhaust process memory; concatenation can also exceed List/Utf8 offset capacity. | Bound the guarantee to accounted consumers. Share a `RuntimeEnv`/budget where appropriate; reserve before platform allocations and track peaks. State a phase-1 relation-size limit or design a versioned fixed-boundary canonical streaming format before claiming larger scale. | Budget-failure tests for query, canonicalization and result ingestion separately; two concurrent snapshots; non-spillable consumers; cancellation and release. Report whole-process peak memory beside pool reservations. |

### Applicability and principle verdicts

All twelve groups apply because this proposal defines a semantic model, compiler,
execution lifecycle, persistence protocol, interchange boundaries and extension
system. That broad scope is the reason for the coverage below. Distributed
coordination, an untrusted binary-plugin service and an accelerator backend are
not added to scope. Their absence is not a defect.

“Satisfied” means the inspected **Proposed** design identifies the mechanism,
boundary and rejection behavior for the stated scope. It does not upgrade the
platform to Implemented or Tested. “Unresolved” identifies a decision or evidence
gap; it is not an implicit pass. The grouped rows provide a verdict for each
applicable principle without using a numerical score.

| Group | Satisfied | Violated | Unresolved | Mechanism or limiting evidence |
|---|---|---|---|---|
| 1 — Meaning and authority | DM-02, DM-03, DM-05 | DM-01 | DM-04 | Change-set authority and separate physical layouts are explicit; composite meaning and opaque-kernel contract remain incomplete, R4-08/10. |
| 2 — Types and invariants | DM-09 | DM-06, DM-07, DM-08 | DM-10 | Membership/domain relations reject invalid combinations; extension/cast/null boundaries fail, R4-01/05/08/10. Several policy-bearing text/options fields still require fully specified interpretation. |
| 3 — Identity and revisions | DM-11, DM-13 | DM-12, DM-15 | DM-14 | Stable IDs and separate cases/results are specified; identity/integrity and null bytes are inconsistent, R4-03/04. |
| 4 — Declarations and composition | DM-16, DM-17, DM-19, DM-20 | — | DM-18 | Templates, explicit provider selection and non-mutating inspection have mechanisms; demand versus expansion ordering needs resolution, R4-06. |
| 5 — Derivation and lowering | — | DM-24 | DM-21, DM-22, DM-23, DM-25 | Stage ownership, complete pass inputs, and generated operation contracts need R4-02/05/06/07/08/09/10. Provenance declarations alone cannot repair an invalid derivation. |
| 6 — Execution and effects | DM-26, DM-27, DM-29 | DM-28 | DM-30 | Preparation, typed stages and owned workspaces are explicit; conditional effects and complete failure outcomes are not, R4-02/10/13. |
| 7 — Reuse and concurrency | DM-34 | DM-31, DM-32 | DM-33, DM-35 | Graph meanings are distinct; incomplete per-instance keys and cross-consumer budgets limit reuse/concurrency claims, R4-07/13. |
| 8 — Execution representation | DM-37, DM-38, DM-39 | DM-40 | DM-36 | Coarse typed transfer and native numerics fit their consumers; §24.3 explicitly withholds performance claims. Layout benefits remain unmeasured; R4-09/13 qualify precision and cost. |
| 9 — Boundaries and extensions | DM-45, scoped to authored data and built-in/explicitly selected kernels | DM-41, DM-42, DM-43 | DM-44 | Agent edits share change-set gates; arbitrary compiled plugin loading is deferred. Mechanical conversion and truthful capabilities still need R4-01/02/05/10. |
| 10 — Provenance and explanation | DM-46, DM-49, DM-50 | DM-48 | DM-47 | Source maps, derivations, semantic diffs and lifecycle spans are identified; deterministic evidence and mandatory failure distinctions need R4-04/10/12. |
| 11 — Evolution and verification | DM-51, DM-52, DM-55 | DM-53, DM-54 | — | Explicit versions, unknown-registry rejection, generated artifacts and discoverable entry points are specified. Tests must exercise the actual boundary and complete result equality, R4-01/11. Future meaning-changing migrations still need their own contract. |
| 12 — Architectural discipline | DM-56, DM-57 | DM-59, DM-60 | DM-58 | Shared declarations and deferred optional engines reduce duplicate decisions. Overstated evidence and ineffective regression controls need correction; §8 compares a smaller viable compiler. |

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| Current skeletal repository | Intended boundaries exist; no working process-model lifecycle establishes extension locality | Foundation checks cannot certify engineering behavior | Lowest present implementation, but does not deliver Slice A | No end-to-end modeling measurement | Baseline only |
| Blueprint revision 4 as written | Strong declaration/generation intent; incomplete contracts force adapter and pass authors to make hidden semantic choices | R4-01–R4-13; especially admission, conditional execution, hashing and reuse | Broad staged compiler plus three backends and fine reuse before those guarantees are settled | Benefits are hypotheses under §24.3 | Revise before implementing affected contracts |
| **Simpler viable compiler with the same semantic model and phased capabilities** | Keep typed relations, indexed math IR, native/NL/Pyomo boundaries and contracted kernels; use complete immutable stage bundles and conservative whole-stage memoization first; generate only adapters with a current consumer | Easier dependency/recovery argument; still requires physical/numerical semantics and conformance tests | Less fine-grained invalidation and plan-fingerprint machinery; reporting uses pinned views where materialization is unnecessary | Potential extra recomputation; must be measured against its reduced preparation/memory cost | Recommended starting implementation after correctness corrections; refine per-instance reuse when a representative workload establishes value |

The simpler alternative preserves the stated observable outputs and future
capability scope. It changes internal granularity and ordering of investment.
It does not replace the semantic model with Pyomo objects or force Newton
iterations through a relational engine. An initially coarser memo is a deliberate
DM-33 tradeoff and should be recorded if adopted, with refinement triggered by
measured rebuild cost and complete dependency fixtures.

### Library capabilities to use more extensively

The following recommendations are **Proposed**. “Interface-checked” establishes
the available mechanism; it does not establish its benefit or complete domain
semantics. Most use already pinned families and do not call for another engine.

| Priority / capability | Concrete consumer and principle | What the library can supply | Platform-owned contract and limit | Recommendation and verification |
|---|---|---|---|---|
| **L1 — active analyzer and recursive validation** | Rule/catalog and bundle admission; DM-07, DM-42 | `AnalyzerRule`, `LogicalPlan::apply_with_subqueries`, Arrow `try_extension_type`, and the DataFusion extension registry | Recursive field validity, semantic compatibility, admission ordering and rejection of unsupported versions; the registry must be called | **Adopt next. Interface-checked**, E1. Generate one validator from registry contracts and install it in the ordered engine profile; test query paths, nested fields and projections. [Analyzer source](https://github.com/apache/datafusion/blob/55.1.0/datafusion/optimizer/src/analyzer/mod.rs), [plan traversal](https://github.com/apache/datafusion/blob/55.1.0/datafusion/expr/src/logical_plan/tree_node.rs). |
| **L2 — built-in conditional physical execution** | Guarded property evaluation; DM-24, DM-28 | `Expr::Case` lowers to `CaseExpr`; E2 exercises row exclusion | Native branch/mask semantics and truthful optimization restrictions; UDF arguments do not become lazy from flags | **Adopt next. Tested** in E2's two-row case. Reuse the existing conditional execution path before inventing a custom physical operator. [Physical planner](https://github.com/apache/datafusion/blob/55.1.0/datafusion/physical-expr/src/planner.rs), [generic UDF evaluator](https://github.com/apache/datafusion/blob/55.1.0/datafusion/physical-expr/src/scalar_function.rs). |
| **L3 — physical predicates and Arrow/Parquet filter machinery** | Snapshot scans and runtime analytics; DM-24, DM-38, DM-53 | `SessionContext::create_physical_expr`, Arrow `filter_record_batch`, Parquet `ProjectionMask`, `ArrowPredicate`, `RowFilter`, and reader projection/filter options | Exact versus candidate pruning, required filter columns, null semantics and limit placement; complete equivalence with an unpruned scan | **Adopt shared predicate evaluation; evaluate storage pushdown by workload. Interface-checked.** Durable Parquet is already a scan source in blueprint §5.4, despite the Arrow map's deferred-item rationale saying otherwise. [Parquet reader](https://github.com/apache/arrow-rs/blob/59.3.0/parquet/src/arrow/arrow_reader/mod.rs), [predicate contract](https://github.com/apache/arrow-rs/blob/59.3.0/parquet/src/arrow/arrow_reader/filter.rs). |
| **L4 — relational aggregation and explicit unnesting** | P8 contribution grouping and membership projections from list columns; DM-18, DM-25, DM-38, DM-56 | `LogicalPlanBuilder::aggregate` and `unnest_columns_with_options`; built-in aggregate expressions | Valid membership, empty/null-list treatment, grouping keys, ordering of collected terms and derivation links | **Specify the missing rule-algebra route now; adopt for these consumers. Interface-checked.** §14.2's small algebra lacks an aggregate/unnest operation although §10.1 groups contributions and §6 stores list membership. Extend only those required operations or explicitly assign them to a contracted native pass. Compare to an independent small reference expansion. [Builder source](https://github.com/apache/datafusion/blob/55.1.0/datafusion/expr/src/logical_plan/builder.rs). |
| **L5 — shared runtime, reservations and memory attribution** | Concurrent snapshots, canonicalizer and result assembly; DM-30, DM-35, DM-39, DM-50 | `SessionStateBuilder::with_runtime_env`, `MemoryConsumer`/`MemoryReservation::try_grow`, `TrackConsumersPool`, `PeakRecordingPool` | Accounting before allocation, non-spillable behavior, shared ownership and a whole-process envelope; an Arrow pool alone is infallible tracking | **Adopt explicit accounting and measure peaks. Interface-checked.** Keep separate snapshot catalogs on a shared budget where appropriate. Test release after failure/cancellation. [Memory API](https://docs.rs/datafusion-execution/55.1.0/datafusion_execution/memory_pool/index.html), [Arrow pool source](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-buffer/src/pool.rs). |
| **L6 — field-aware UDF typing and scalar fast paths** | Generated constitutive adapters; DM-06, DM-25, DM-37 | `ReturnFieldArgs.arg_fields`, `ScalarFunctionArgs.arg_fields`, `ColumnarValue::Scalar`, `number_rows` | Input quantity checks and declared conversions before output metadata; required per-row quantity handling; all-scalar output cardinality | **Deepen the already adopted mechanism. Interface-checked.** Derive input rejection as well as return metadata; preserve scalar inputs rather than expanding every constant into an array. Measure allocations for scalar/mixed/batch cases. [UDF API](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/trait.ScalarUDFImpl.html). |
| **L7 — safe buffer views and bounded coalescing** | Parameter binding and Python streams; DM-29, DM-36, DM-37 | `PrimitiveArray::values` exposes a slice-aware `ScalarBuffer`; `BatchCoalescer` controls transfer batches | Ownership/lifetime, null rejection, immutable publication and explicit copying; a nonzero slice offset alone does not imply noncontiguity | **Evaluate after correctness. Interface-checked.** Relax §18.2's offset-zero rule only through safe buffer APIs and lifetime tests. Coalesce transfer batches without concatenating the entire dataset. [Primitive arrays](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-array/src/array/primitive_array.rs), [coalescer](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-select/src/coalesce.rs). |
| **L8 — narrowly proved predicate preimages** | Repeated analytics over computed quantities; DM-24, DM-32, DM-43 | `ScalarUDFImpl::preimage` supplies an input interval for a supported output predicate; existing interval/ordering hooks can aid planning | Equivalent predicate semantics, half-open endpoints, selected branch, floating precision and absence behavior; a conservative pruning range is insufficient for a replacement predicate | **Evaluate later, starting with exact discrete/affine cases. Interface-checked.** This is not a general thermodynamic inversion or automatic support for arbitrary inequalities. Differentially compare full results before enabling pruning; measure skipped decoding. [Pinned UDF source](https://github.com/apache/datafusion/blob/55.1.0/datafusion/expr/src/udf.rs). |
| **L9 — explicit-schema Arrow readers** | Future tabular observations/imports already contemplated in §6.10; DM-41, DM-52, DM-56 | Arrow CSV and JSON `ReaderBuilder::new(schema)` accept a declared schema; JSON strict mode and CSV header validation are available | Units, target binding, source spans, required columns, exact numeric policy and foreign-key validation remain platform responsibilities | **Correct the map's blanket rejection; adopt only for a declared tabular interface. Interface-checked.** Schema inference is optional, not inherent. This does not replace the YAML/TOML modeling language or a specialized IDAES state-file adapter. [JSON reader](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-json/src/reader/mod.rs), [CSV reader](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-csv/src/reader/mod.rs). |

Already adopted and worth retaining: `RowConverter` for composite key ordering,
`Schema::project`, extension metadata and generated Arrow/Python registrations,
explicit engine rule lists and configuration, immutable snapshot providers,
structured DataFusion error translation, native graph algorithms, and kernel-local
`num-dual`. Registration, method availability, and selected configuration should
be described at exactly the strength they establish.

Do not add Flight, distributed query orchestration, a second dataframe engine,
or a general plugin service to solve these findings. `salsa`, `egglog`, FeOs and
additional physical encodings should retain their explicit consumer/measurement
triggers. Arrow `RunEndEncoded`, view arrays and custom allocation can be valuable
later, but none repairs lost meaning at an admission boundary.

### What remains ordinary code

Physical quantity algebra, semantic admissibility, canonicalization, four-valued
inference and provenance, demand closure, sparse derivative preparation, matching,
branch-aware numerical execution, solver callbacks and the snapshot commit
protocol remain platform responsibilities. Arrow supplies representations and
kernels; DataFusion supplies plans and execution machinery. Neither library
defines process-engineering meaning on behalf of the registry.

## 9. Verification and measurement plan

The retained [probe source, runner and receipt](../evidence/blueprint-rev4-2026-09-13/README.md)
make the library counterexamples reproducible. The full regeneration recipe would
rewrite unrelated capability evidence and Python exports; no recipe isolates
these characterization questions. The bounded runner therefore creates an
ephemeral crate from workspace pins and a copied lockfile, checks resolution
against that lock, and runs it offline with the pinned Rust toolchain.

**Executed command:**

```bash
bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
```

**Mode and baseline:** Rust 1.98.1; dev profile; Arrow 59.3.0 with
`arrow/force_validate` explicitly enabled by the scratch `force-validate` feature;
DataFusion 55.1.0; offline locked execution; failure baseline **zero**. Final result:
**6 characterization groups completed, 0 assertion failures**. This proves the
named library behavior, including failures of the proposed assumptions. It does
not mean six platform acceptance gates passed.

The initial scratch run stopped at the prototype's missing protobuf provider
codec after E1–E4; that harness defect was corrected before recording the final
successful receipt. No full Rust suite, Python suite, solver parity, or benchmark
was run as part of this documentation review.

**Document and evidence checks:** `just docs`, `just lint-typos` and
`just lint-license` completed with exit status 0 in documentation/static mode
(failure baseline: zero). The pinned Rust formatter's
`rustfmt --check --edition 2024` accepted `probe.rs`; `/usr/bin/shellcheck`
accepted `run.sh`. A focused check found 0 failures across the five review/evidence
files, ten local links, eleven required sections and three recorded input digests;
`git diff --check` also succeeded. The recipes emitted the existing host-shell
diagnostic `/etc/bash.bashrc: line 7: PS1: unbound variable` before succeeding;
host startup configuration was outside this review's edit scope.

| Claim or risk | Evidence label | Test / analysis / benchmark | Conditions and expected result | Current result or remaining gap |
|---|---|---|---|---|
| Extension registration validates all plans | **Tested**, library only | E1 `extension_validation` | Registered validator; known malformed and unknown extension fields; ordinary `SELECT` | Direct checks reject; query succeeds; factory invoked zero times during query. Claimed universal mechanism refuted. |
| UDF flags enforce lazy branches | **Tested**, library only | E2 `conditional_evaluation` | Two positive integer rows; excluded failing branch; Immutable custom UDF with both flags | Custom UDF errors; built-in `CASE` succeeds. |
| `safe: false` means no numeric loss | **Tested**, library only | E3 `cast_fidelity` | Fractional Float64 and integer above exact Float64 range | Both casts succeed with altered values. |
| Canonical bytes ignore null payload and match storage encoding | **Tested**, library only | E4 `representation_identity` | Same PK order, take and concatenate, equal nullable values, V5/alignment-64/no-compression stream | Null payload affects bytes; IPC file and Parquet digests differ from stream. |
| Sorted metadata insertion stabilizes protobuf | **Tested**, library only | E5 `sorted_metadata_proto` | 32 independently built sorted-insertion maps; fixed provider codec | 32 protobuf encodings and one IPC encoding in the retained run. The count may vary; more than one refutes the guarantee. |
| Real-algebra rewrites preserve required floating results | **Tested**, arithmetic only | E6 `floating_point_rewrites` | `log(exp(1000))`; addition of 1e16, -1e16 and 1 | Infinity versus 1000; 1 versus 0. No platform optimizer was run. |
| Complete and terminating compiler schedule | **Proposed** | `demand_seed_closure`, `stage_bundle_graph` | Method/law/initializer-only demands, finite recursion, explicit stage inputs | Needs normalized seed or joint-stage decision before implementation. |
| Reuse equals clean recomputation | **Proposed** | `incremental_vs_clean_binding_matrix` | Change one package/domain/binding/value/policy at a time, including absence and provider candidates | Compare full declared outputs and lineage, not only cache-hit counts. |
| Semantic conversion and composite physical types | **Proposed** | `head_semantic_admission`, `quantity_composition` | Lossy numeric conversions; different unit/reference/basis; scaled differences and weighted means | Must reject unsupported conversions and preserve declared result types. |
| Exact pushdown is complete | **Proposed** | `pushdown_vs_unpruned` | Deliberately over-pruning provider plus representative filters/projections/limits | Full results and multiplicities match reference; malicious empty output is detected. |
| Crash, retry and cancellation preserve a coherent result | **Proposed** | `publish_fault_matrix`, `cancelled_stage_outcomes` | Fault after each artifact/manifest/ref step; native callback and NL process interruption | Old/new complete snapshots only; explicit terminal status and retry attempt; no mixed or partial accepted result. |
| Backend values, derivatives and failures conform | **Proposed** | Slice A plus adversarial operator/kernel cases | Independent analytic/finite-difference fixtures and native/NL/Pyomo comparisons | Shared lowering agreement alone is not an independent oracle. Include invalid/branch/nonsmooth cases and unsupported modes. |
| Added library use reduces total cost | **Proposed** | Slice A cold/warm and scale benchmarks | Same output contract; whole-stage versus fine memo, predicate pushdown, buffer views, scalar/batch inputs | Report parse, validation, preparation, canonicalization, transfer, solve, persistence, recovery, inspection, pool/process peak memory, and provenance size. No gain measured here. |

## 10. Exceptions and unresolved decisions

No MUST-level exception is accepted by this review. The gate failures require
revision or explicit narrowing of the affected supported behavior. Existing
deferrals for distributed coordination, optional kernels and additional engines
remain outside this review's implementation demand.

| Decision required | Scope / reason and alternatives | Consequence and compensating control | Evidence | Accountable role / revisit trigger |
|---|---|---|---|---|
| Logical versus physical artifact identity | Canonical stream hash plus encoding checksum, or decode-and-canonicalize verification | Readers must know which identity they check; test all supported encodings and interrupted commits | E4; R4-03/04 | Architecture and persistence maintainers; before canonicalizer/store implementation |
| Demand-seed stage and stage artifact ownership | Pre-realization seed extraction versus an explicit joint fixed point | No implicit late discovery or hidden backward stage writes; validate dependency graph and closure | Blueprint §§9.6/14.1; R4-06 | Compiler maintainer; before P6/P7 contracts are implemented |
| Numeric and quantity-operation policy | Guarded symbolic semantics and selected evaluation policy; complete result-type rules | Persist assumptions, reject unsupported combinations, test failure behavior and numerical comparison classes | E6; R4-08/09 | Math/quantity maintainer; before P10 and tape contracts are frozen |
| Tolerant batch failure mode | Fail operation versus mandatory typed per-row outcomes | A null by itself cannot denote both missing input and invalid evaluation | R4-10 | Kernel/runtime maintainers; before adapter generation |
| Coarse initial reuse and relation-size envelope | Complete whole-stage keys initially; bounded one-batch canonicalization or a new canonical streaming version | Potential extra recomputation or explicit size rejection; no silent stale hits or unbounded promise | R4-07/13; §8 alternative | Compiler/runtime maintainers; refine after Slice A measurements and dependency conformance |
| Deterministic plan-evidence contract | Canonical protobuf representation versus explicitly noncanonical diagnostic bytes | Preserve useful semantic memo keys; no false cross-process fingerprint claim | E5; R4-12 | Rule-engine maintainer; before engine reproducibility acceptance |

Owner entries identify the responsible role for a future decision; this review
does not assign a person or authorize a dependency/design change. Adoption that
changes D1–D14, identity, boundary or commit contracts must follow the repository's
ADR and design-review process. The blueprint and accepted ADRs were not amended.

## 11. Decision and implementation changes

**Decision: Revise.** G1 has a coherent proposed authority model; G2, G3, G4, G6 and
G7 fail on concrete contract or capability evidence, and G5 remains unresolved.
The architecture's central choices are useful, but the current blueprint is not
ready to serve as an unambiguous specification for its affected boundaries.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 — Semantic boundaries | Active extension validation, loss-aware rule admission, complete kernel/result semantics; R4-01/05/10; L1/L6 | DM-06–08, DM-41–44 | Actual import/query/output paths reject invalid data and preserve semantic distinctions | Generated boundary matrix from the same contracts as adapters |
| 1 — Execution semantics | Guarded physical execution and complete numeric/quantity rules; R4-02/08/09; L2 | DM-01, DM-24, DM-28, DM-40 | Conditional and numerical adversarial cases agree under named policies | Per-operator and per-backend value/error/derivative conformance |
| 1 — Identity and consistency | Separate canonical content from encoding integrity; normalize ignored physical data; R4-03/04 | DM-12, DM-14, DM-15, DM-48 | Encoding round trips and publication fault matrix | Canonicalization metamorphic fixtures, format/version compatibility checks |
| 1 — Compilation and reuse | Resolve demand-stage cycle and complete input bundles; R4-06/07 | DM-18, DM-22, DM-31, DM-32 | Stage graph validity, complete property closure, incremental equals clean output | Declared-read enforcement and dependency-change fixtures |
| 2 — Library reuse and regression quality | Shared filter evaluation, bounded aggregate/unnest operations, complete pushdown testing, honest plan evidence; R4-11/12; L3/L4 | DM-25, DM-38, DM-53, DM-54, DM-59, DM-60 | Over-pruning detected; standard relational operations preserve keys and provenance; fingerprint policy holds | Differential query fixtures and descriptor-derived capability tables |
| 2 — Operational envelope | Account for platform allocations and shared budgets; R4-13; L5 | DM-30, DM-35, DM-39 | Typed limit failure within the supported envelope; measured process/pool peaks | Memory and cancellation fixtures under concurrent snapshot workloads |
| 3 — Measured refinements | Buffer views, bounded coalescing, predicate preimages and tabular readers where a consumer exists; L7–L9 | DM-36, DM-37, DM-56, DM-58 | Same semantic outputs and a measured end-to-end improvement | Keep conservative paths as differential references; revisit with workload changes |

The recommended next design revision should settle the priority-1 contracts first,
then express their library bindings and acceptance tests. It should also correct
the capability-map claims identified by E1, E2, E3 and E5; otherwise future agents
will inherit the same unsupported assumptions. More library use is valuable when
it removes a duplicated mechanism while leaving one explicit semantic authority.
