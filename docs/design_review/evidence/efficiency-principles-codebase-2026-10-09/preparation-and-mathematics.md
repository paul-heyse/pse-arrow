# Preparation and mathematics: supporting assessment

This is bounded independent supporting analysis for the
[principal efficiency review](../../reviews/design_review_efficiency-principles-codebase_2026-10-09.md),
which owns the combined judgment. It does not create a separate product
acceptance or implementation backlog. The inspected preparation architecture has sound
semantic and lifecycle boundaries, but three operations still perform avoidable whole-source
work inside smaller semantic operations. The bounded recommendation is to revise those
physical paths while preserving complete-basis reuse, physical admission, demanded derivatives
and fresh original-space assessment. The known PC-SAFT fact-context refusal is a separate
behavioral limitation; it does not establish that incorrect physical results escape.

The source baseline is clean commit `4c24721e691187e1a5b28398b29722fbde671da8`. Initial
`46545b2ad3e2999b4335692ac49af6091438c3fc` plus dirty graph/hash changes became that committed
baseline during review startup. This reviewer performed read-only source and evidence
inspection, followed by authorized publication of this supporting document. No builds, tests,
probes or formatters were run by this reviewer.

The assessment applies Core/template 3.4, Heuristics for Efficient Architecture 1.0,
ProcessSimulator 1.5 and the selected pse-arrow binding. Coverage includes authored occurrence
products, checked-package selection and publication, Salsa specialization, grouped mathematical
admission, portable reconstruction, retained bases and solver views, support/evaluator
preparation, value rebinding and conditional initialization consumers. Relevant owners are
blueprint §5, §6, §7, §14, §15–§18. It does not independently qualify every property model,
native adapter, dynamics/fitting route or assembled E3/E4/E5 journey. Status and subsequent
work, if adopted, remain with the existing [Plan 28](../../../plans/28-surrealdb-unified-substrate.md)
coordination route, including [28b](../../../plans/28b-selected-compilation-and-reuse.md),
[28f](../../../plans/28f-shared-numerical-preparation.md) and
[28k](../../../plans/28k-graph-kernels-and-hashing-investigations.md).

## Preserved strengths and operation contracts

Current preparation genuinely reuses complete mathematics.
`ModelingPackage::prepare_selected` in
[workflow/modeling/preparation.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/modeling/preparation.rs)
checks the retained exact basis before constructing a pure compiler workspace.
`BasisKey` in [math/preparation.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/preparation.rs)
retains complete request, dependency, root, instance, binding and limit equality. Its Fx
prehash supplies lookup buckets rather than semantic equality. A warm solve can retain
compiled mathematics while refreshing lineage, policy and attempt state.

Reused mathematics and current effects are separated. Selection protection, dependency
qualification, description acknowledgment, source attribution and lineage remain consumer
operations. `PreparedBasis::bind` creates current wrappers; old publication does not become
permission for a new consumer. Immutable allocation owners survive aliases and workspace
rotation. Bounded retention and generation fencing are relevant strengths even though they
do not establish a particular supported capacity.

Source checking, semantic specialization, physical operations, mathematical support, native
artifacts and numerical values are consequential distinct products. Semantic topology can be
prepared independently of executable arithmetic. Demand-specific derivative readiness is not
mistaken for mathematical capability. `PreparedCase::rebind` in
[executable/solve.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace/modeling/executable/solve.rs)
shares plans, structure and artifact requests; it recomputes derived parameters and
value-dependent products when their consumed inputs change. Free-variable starts do not
automatically invalidate structural preparation.

`CasePlan::prepare_with_source_support` in
[assembly.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-math/src/assembly.rs) deduplicates local support and
artifact demands by body, outputs and derivative coordinates.
`PreparedBody::prepare_support` in [execution.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-math/src/execution.rs)
compacts demanded mathematics before support construction. Current J3 portable-body
inventories and description indexing, B8/N13 retained supplier topology, and process-local
hashing changes are present. They must not reappear as unresolved defects from historical
reviews.

The physical model is meaningful rather than dimension-only annotation. Origin-sensitive
points and differences, basis, reference state, subject, indices, operation prerequisites and
explicit conversion authority reach checking and lowering. The original guards precede
symbolic simplification. These are preservation constraints, not a certification of every
reference property's scientific fidelity.

| Physical contract | Meaning and source | Relevant preservation obligation |
| --- | --- | --- |
| Authored variable, parameter and output | Complete registered quantity, canonical representation, basis/datum/subject and index context; blueprint §5 and §6 | Compact views retain original identities and physical contracts. |
| Expression operation | Checked operand/result contracts and operation-scoped prerequisite facts | Occurrence compaction preserves lexical context and actual checked admissions. |
| Numerical allowance | Magnitude in the admitted error/difference contract; blueprint §16 | Affine offsets do not become error magnitudes; prerequisites are supplied rather than bypassed. |
| Conditional block | Original rows, solved columns and predecessor inputs; blueprint §17.1 | Local numerical work retains whole-original validation, explicit fixed roles and transactional stages. |

Well-posedness remains separate from topology and numerical regularity. Declared fixed/free
roles and bound structure precede incidence analysis. Structural matching and BTF supply
complete block scope and deficiencies, not numerical-rank proof. Native routes consume
class/capability and derivative readiness separately. No proposed correction may replace a
fresh numerical-state assessment with a previous solve's success.

## PP01: occurrence preparation amplifies subtree text and hashing

**Implemented source cause.** `ExpressionOccurrence::of` in
[expression/admission.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-modeling/src/expression/admission.rs)
renders an expression once for hashing and again for its retained `syntax`.
`ExpressionOccurrence::in_body` calls `of` for every node, then overwrites each newly computed
body hash with the root hash. Those discarded subtree hashes cannot influence the returned
occurrence map.

This is not an isolated diagnostic helper. `AdmissionRecorder::enter` constructs this map
when checking an expression. `expression::occurrences::bind` in
[occurrences.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-modeling/src/expression/occurrences.rs) additionally
walks nodes and computes `ExpressionOccurrence::of(node).body` to identify admission roots.
Typed function lowering in [typed_math.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/typed_math.rs)
reconstructs occurrence maps for the body, validity/envelope predicates and applicability
inputs whenever the application reaches that lowering path.

**Trigger and consequence.** Nested expressions retain subtree text proportional to the sum
of subtree lengths rather than just authored body length. Recursive rendering also copies
ancestor text repeatedly. Depth limits bound the amplification but do not remove it.
Physical-admission copies and portable function records carry those strings further.
This is a source-proven AP-07/G9 concern under H14, H16 and H26. Its share of complete
preparation latency remains unmeasured; this finding makes no numerical speedup claim.

**Proposed correction.** The immediate slice can preserve current key and wire meaning:
render once inside `of`; in `in_body`, compute the root identity once and directly construct
each occurrence without computing a subtree hash that is immediately discarded. Hash-only
callers should not construct unused retained syntax strings.

A fuller representation should make the immutable body own syntax and deterministic
occurrence structure once, with physical admissions addressed by stable positions. Consumers
retain the required authored AST, lexical environment and exact checked dependencies rather
than recreating them. This direction needs representation design; blindly deleting a string
is not the complete remedy.

There are two distinct syntax products. `CheckedPackage::select` consumes
`CheckedExpression.syntax`, the retained `Syntax` AST, through `dependency_syntax`.
`ExpressionOccurrence.syntax: String` is part of the physical occurrence key. Compacting the
latter must preserve the former and its lexical, positive, absent and membership dependency
consequences. Stable body-relative positions must still distinguish identical syntax in
different binders and synthetic nodes with empty byte ranges. Root/admission association must
remain complete; enumerating fewer subtree hashes is not lawful merely because most are
redundant in a simple fixture.

**Verification.** Compare exact old/new occurrence addresses, physical admissions,
diagnostics and selected dependencies for repeated identical syntax, nested binders, empty
spans, partials and shadowed names. A small deep-versus-balanced expression population probe
can establish copied/retained text growth without native solvers. Easy optimization and
versioned representation replacement have different acceptance obligations.

## PP02: selected closure scans and copies the whole checked universe

**Implemented source cause.** `CheckedPackage::select` in
[check.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-modeling/src/check.rs) performs two full occurrence scans
for every reached declaration: one for recorded dependencies and another for
`dependency_syntax`. Reached tables and kinds scan all declarations to discover supplying
datasets. After computing the closure, it clones the complete `CheckedPackage`, then removes
unselected entries.

The occurrence map is already ordered by `(declaration, role, position)`. Whole-map filtering
discards an existing useful access path. The executing caller is the Salsa `selected` query
in [workspace/modeling.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace/modeling.rs), consumed
by `specialized`. Changed catalog publication can rerun selection even when equal selected
results subsequently stop downstream propagation. Backdating is a strength; it does not
remove the work needed to produce that equal result.

**Trigger and consequence.** Many reached declarations and expressions multiply examined
work even when the entire relevant universe is legitimately selected. A sparse selection
also temporarily clones material it immediately discards. This concerns the checked universe
supplied to this operation; it does not claim current canonical acquisition still hydrates
every repository package. Current canonical selection already narrows that boundary.
The relevant obligations are AP-07/G9, H5, H9, H13 and H16.

**Proposed correction.** Use declaration-key ranges or a declaration-grouped occurrence
view, an admitted reverse dataset-owner relation where useful, and selected-only construction
of the resulting package. Preserve source order where consumed, inherited members,
dataset/refinement relationships, lexical imports, engineering-rule markers, provenance,
entity supplier identity and test taint. Do not replace complete dependencies with a
positive-only graph. A new graph database, durable hash family or general dependency engine
is unnecessary for this correction.

**Verification.** Grow reached declarations and unrelated expressions independently,
compare exact selected products/refusals, and inspect that each declaration retrieves its
own occurrence range. Include dataset refinement, lexical shadowing, imports,
insertion/deletion and absent-name cases. Full-checker incrementality is a separate decision.

## PP03: conditional blocks each materialize the original coordinate universe

**Implemented source cause.** The structural owner already supplies original rows, solved
columns and explicit predecessor inputs. Nevertheless, `CasePlan::conditional` in
[assembly.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-math/src/assembly.rs) clones every source variable,
changes each nonselected variable to fixed, copies every parameter, scans/clones source
instances and constructs another `CaseStructure`. `conditional_blocks` in
[workspace.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace.rs) retains one resulting plan
per block.

There is a schedule-level instance of the same physical-granularity problem:
`Plan::with_execution_dependencies` in
[structural initialization.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-structural/src/initialization.rs)
scans the complete edge set separately for each resulting block to discover inputs.
`PreparedCase::initialization_allocation_bound` explicitly multiplies full plan/structure
populations by `rows + 2`. This is not merely a hypothetical accounting defect: actual
per-block coordinate copies exist.

**Trigger and consequence.** A sparse system with B small blocks and V source coordinates
retains and prepares coordinate metadata proportional to B times V although each block's
useful numerical scope can be tiny. With independent scalar blocks, both counts grow
together. `prepare_with_source_support`, frozen-value validation and value-product binding
perform additional work over copied inventories. Current consumers include authored modeling
initialization in [math/initialization.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/initialization.rs),
automatic block composition in [solves/blocks.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/solves/blocks.rs),
and PETSc block preparation. Existing small-capacity controls establish their stated
accounting/refusal behavior rather than large sparse-block feasibility. No particular
exhaustion threshold or slowdown was measured here. AP-07/G9 and H2, H5, H15–H19 apply.

**Proposed correction.** Retain one immutable original coordinate universe and construct
compact block selections/overlays over it. Preserve the logical meaning that unselected
variables are fixed. Validate complete original bindings at the appropriate operation
boundary, validate each block's consumed inputs and solved coordinates, and retain complete
original post-solve assessment and transactional stage behavior. Index edges by consuming
row or block so predecessor discovery does not rescan all edges for every block.

A narrower alternative is a physically local `CaseStructure` containing solved coordinates
plus the complete arithmetic, guard, validity and provider dependency closure, with explicit
mappings to original values. That requires more careful boundary migration because current
APIs validate the complete scalar inventory. A shared original universe with compact views
is the safer starting design. Neither alternative may omit a bound or provider obligation
merely because it is absent from numeric Jacobian incidence. Whole-original preconditions and
final assessment remain explicit.

No solver iteration, matching algorithm or original scientific assessment needs replacement.
Petgraph already owns SCC/topological mechanics; changing containers alone would leave the
coordinate-copy problem intact.

**Verification.** Prepare increasing small independent-block systems and a chain with real
predecessor inputs; compare exact boundaries/support and count retained coordinate
descriptors. The corrected route retains one original universe plus actual block metadata,
while preserving missing-input/domain refusals, cancellation, stage rollback and final
original assessment. No large campaign is required to distinguish the structural mechanisms.

## PP04: numerical difference inference omits required physical facts

This is an independently source-verified existing unresolved limitation, already recorded at
[Plan 28f's PC-SAFT numerical-fact boundary](../../../plans/28f-shared-numerical-preparation.md#pc-saft-numerical-fact-investigation-boundary),
rather than a newly attributed first failure.

**Implemented source cause.** `engineering_error_quantity` in
[numerics.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-math/src/numerics.rs) infers point self-subtraction using
`NoInvariantFacts`. Numerical `resolve` receives `QuantityRegistry` but no actual
prerequisite inventory. The current
[physical declaration](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/packages/reference/physical/materials/physical.yaml)
registers `LogFugacityCoefficient` subtraction with two `OperandQuantityContract`
prerequisites. [PhysicalPreconditions](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-quantity/src/preconditions.rs)
can check those predicates against actual operands and the admitted operation;
`NoInvariantFacts` necessarily refuses them.

An analogous call exists in `prepare_conditional_boundary_functions` in
[executable.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace/modeling/executable.rs).
Current case, implicit and initialization numerical-policy consumers reach `numerics::resolve`.

**Trigger and consequence.** Otherwise lawful preparation using default numerical
allowances refuses when the operation needs those supplied physical prerequisites. This is
a consumed-contract gap under AP-02/AP-04, not evidence that the physical declaration is
inadequate or invalid subtraction succeeds. The PC-SAFT harness recorded typed refusals at
cold, warm, value and structural stages, but did not identify the first failing caller.

**Proposed correction.** Thread the current immutable physical prerequisite context through
numerical difference resolution and affected boundary preparation, preserving actual
operation/operand checking and context invalidation. Resolve an error contract once per
target where possible. Do not substitute `Scalar`, invent tolerances or accept invariant IDs
as proofs. Not every `NoInvariantFacts` occurrence is wrong: portable reconstruction runs
inside qualified physical receipt replay. A blanket replacement would confuse replayed
proofs with fresh inference.

**Verification.** Exercise present, absent and changed operand prerequisites and the existing
exact numerical allowances; then rerun the unchanged scientifically corrected PC-SAFT
fixture. Trace the first failure before claiming runtime attribution. Until that succeeds,
no PC-SAFT prepared-product or performance claim follows from the completed smoke harness.

## Incremental checking, persistence and library alternatives

Upstream checker granularity remains a separate unresolved optimization decision. Source
publication skips checking only when complete rows, scope and documents are unchanged.
Other publications call the complete checker before updating the Salsa catalog. The existing
[authored-checker inquiry](../graph-hash-followups-2026-10-09/README.md#authored-checker-publication-cost)
corroborates this boundary. Its actual publication samples were approximately 5.1–5.5 ms for
changed scalar fixtures versus approximately 1.3–1.7 ms for subsequent preparation. These
are existing diagnostic measurements, not this reviewer's execution or product-wide latency.

A complete typed incremental checker could improve edit locality but must track membership,
lexical visibility, absent lookups, physical prerequisites, documents, provenance, failure
atomicity and cancellation. A manually maintained positive graph would weaken the current
contract. The simplest present alternative retains the full checker while removing PP01/PP02
avoidable work, then decides whether remaining measured edit cost justifies the complete
redesign. Downstream Salsa reuse does not establish upstream incremental checking.

The actual Recipe persistence inquiry likewise does not justify production persisted Salsa
adoption. Restoring a decoded Recipe leaves mathematical reconstruction and receiving
qualification. Its measured complete snapshot restore/fetch exceeded ordinary Recipe decode
for that fixture. A future persisted product should be selected around work it replaces,
not merely serializability. Native Atom/evaluator persistence requires its own exact-release
capability and receiving-lifecycle inquiry; none is established here.

The live lock contains Salsa 0.28.4/macros 0.28.4/macro-rules 0.28.5, Symbolica/Numerica 3.0.1,
petgraph 0.8.3 and DataFusion 55.1.0. The loaded Salsa skill pins macro-rules 0.28.4; the
math skill pins Symbolica/Numerica 3.0.0. Existing consumer source and the current recorded
probes support the bounded claims above. Those skills alone do not qualify new 3.0.1
serialization or evaluator alternatives. The simplest corrections use current map ranges,
owned immutable views and existing library algorithms rather than new dependency families.

## Bounded judgments, rule impacts and evidence limits

| Foundation | Bounded judgment |
| --- | --- |
| AP-01 | Satisfied: source, physical meaning, mathematics, values, attempt state and effects have coherent owners. |
| AP-02 | Violated at PP04's numerical difference boundary; other inspected preparation contracts preserve consumed distinctions. |
| AP-03 | Satisfied: shared preparation and solver primitives support composition; PP03 concerns physical realization rather than copied numerical policy. |
| AP-04 | Physical model adequate in examined cases; authoritative realization incomplete at PP04's missing fact context. |
| AP-05 | Satisfied in examined keys, demands, typed failures and ownership boundaries. |
| AP-06 | Satisfied in inspected pure compiler/quantity seams, exercisable without unrelated storage/native execution. |
| AP-07 / G9 | Violated by PP01–PP03's concrete repeated-work/materialization mechanisms. |

Physical, numerical and reuse safeguards are source-backed preservation constraints. This
assessment does not establish full G2/PS-G1–PS-G3 qualification across the broader simulator.
PP04 establishes supported-work refusal, while no inspected path established silently
accepted physical inconsistency or solver-status-as-success. Complete-product acceptance
belongs to the principal review's combined judgment.

| Rule impact | Proposed change and consequence if current rule is retained |
| --- | --- |
| PP-RC01 — conditional representation | Shared original universe plus compact views fits blueprint §17.1 and §14.4 while preserving logical fixed roles. No semantic rule change is required. A local-inventory alternative must explicitly revise any contract requiring complete scalar inventories and preserve original-space validation. Retaining complete logical scope does not require retaining per-block physical copies. |
| PP-RC02 — compact occurrence representation | Removing discarded work can preserve current identity/wire meaning. Removing or redefining serialized fields requires deliberate portable interpretation/version handling: `FunctionRecord` serializes the keys, `ADMITTED_RECIPE_INTERPRETATION` is `pse.admitted-body.v1`, and `MathLocalOccurrenceV3` has existing meaning. Retaining those exact contracts leaves the easy optimization available; a broader compact wire must use the decision route rather than reinterpret historical bytes. |
| PP-RC03 — numerical context | Supplying actual prerequisite facts restores existing physical semantics. It requires no weaker quantity rules, numerical precision or scientific scope. Retaining prerequisite enforcement is mandatory; missing facts must still refuse. |

PP02's indexed access and selected-only projection need no rule change if exact outputs and
refusals remain unchanged. Full checker replacement or persisted incremental execution is a
separate decision rather than an implied consequence of these findings.

The corrections fit together without a substrate pivot. Compact occurrence preparation
reduces source and function-lowering overhead; indexed selection reduces closure work;
compact conditional views reduce block preparation and retained state; supplying physical
facts corrects lawful admission. Their closure obligations remain distinct. Source evidence
establishes the mechanisms, not a numerical speedup. The historical 0.535 µs key traversal
and 150.857 ms complete warm preparation are different operation boundaries, not a
before/after result or an exact internal latency share.

No tests or probes were run for this assessment. Existing inquiry receipts retain their
original fixtures and scope. No cancelled feature matrix was restarted, no comprehensive
qualification was claimed, and no remediation was implemented by publishing this document.
