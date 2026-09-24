---
id: ADR-0063
title: Declare indexed template and conservation realization contracts
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-09, DM-17, DM-18, DM-22, DM-24, DM-38, DM-43, DM-46]
blueprint: [§6.15, §10.1, §12.3, §14.1]
review: docs/design_review/reviews/design_review_wave2-contracts_2026-09-14.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported law needs a new subject conversion or non-finite domain; scalarization becomes a consumer.
verification: template_instantiation, law_participation_completeness, method_realization_units, authored_to_typed_graph with force-validate; just adr-lint.
---

# ADR-0063: Declare indexed template and conservation realization contracts

## Context

Blueprint §10 describes realized contributions and exhaustive exclusions, while
current template contribution rows describe only authored intent. P7–P10 also need
instance equation ownership and production typing context, neither supplied by the
Wave 1 P10 fixture.

## Scope

Amend blueprint §6.15.4–§6.15.6 with consumed template, contribution, law,
connection and method realization contracts. Activate R-30 for instance equations,
and production P4–P10 ports (tear candidates withdrawn by ADR-0075). P11–P16 and final closure remain deferred.

## Drivers

Retain indexed structure, physical meaning, complete participation and reproducible
source correspondence while supporting generic template extension.

## Options

1. Generate each heater/mixer with Rust constructors: rejected duplicate semantics.
2. Fill missing relations with free-form JSON: rejected uncheckable meanings.
3. Extend the existing typed template/MathIR adapters and use relational participation
   classification: selected; ordinary DAG substitution remains native code.

## Outcome

P7 instantiates validated templates using finite P3/P5 bindings and decided P4
guards. P8 classifies every law candidate exactly once, emits ordered signed
expression descriptors and indexed equations, and preserves explicit exclusions.
Physical conversions are declared operations, never matching unit labels or hashes.
The inferred mathematical family retains pending conversion/smoothing requests
until P10, while requiring actual symbols, domains and gathers. P9 realizes selected methods and checks every advertised provision and dependency;
P10 reads complete declared context and establishes the typed canonical graph.

Expression-role symbol bodies have a single authored template-symbol-expression
relation and a realized symbol-to-root correspondence; they are not fabricated
equations. Actual quantity and dependency-cycle checks precede canonical acceptance.
Instance-owned equations have an explicit source-owner alternative. Indexed
connection equations keep their domain product at P8; concrete scalar provenance
is created only with P12 expansion. Tear selection is not a compiler output: it is a
run-time result of the Pyomo adapter's `select_tears` (ADR-0075).

### Consequences

New relation declarations and status-dependent invariants precede their consumers.
P7/P8/P9 publish complete stage bundles, including empty outputs and provenance.
Unsupported features fail explicitly before publishing a successful graph.

For an outer index declared on one equation or symbol, its actual binder is
`named_id(instance_id, "pse:free-index:v1:" + declaration_id.to_hex() + ":" +
position_decimal)`. Separate expression and filter source fields of that declaration
therefore share the same actual binder only after their declared position and actual
domain have been compared. A lexical inner binder uses the source-bound identity
formula above. These names provide occurrence identity; the complete source-axis,
lexical-scope and actual-domain checks establish validity.

Indexed conditional equations retain each reachable relation leaf and the exact ordered
branch predicates. A branch record identifies its parent indexed equation, source and
leaf equation node; its identity is `named_id(parent_indexed_equation_id,
"pse:equation-branch:v1:" + source_id.to_hex() + ":" + leaf_equation_id_decimal)`.
Scalar predicates select one leaf under the parent identity. Index-dependent branches
have complementary, exhaustive actual P4 predicate masks, combined with any authored
filter. Their equation senses and bounds remain unchanged; no early scalarization or
inequality-to-equality rewrite is permitted.

Every indexed additional expression root carries its complete ordered actual free
index inventory in `compiled.expression_root_indices`, keyed by owner, role, root
ordinal and axis position. Consumers check actual binder/domain correspondence before
typing. Scalar symbol bodies have an empty inventory. The P9 realization entry point
uses the same generic realizer with an admitted complete seed graph and explicit
published additional instance, configuration and path facts; it preserves and remaps
all existing node references while realizing only the selected instances.

`reference.method_state_parameters(method_id, parameter_name)` explicitly identifies
the method template's required SemanticId parameter that receives the requesting
actual state instance. A missing mapping does not imply a parameter named `state`.
State-definition methods reuse the exact existing selected state instance rather than
creating a second one. Other selected method roots carry their actual P6 selection as
source evidence through the same configuration expansion and P4/P5 rule executors.

Selected equation-template methods bind each parameter-role symbol through
`compiled.method_parameter_bindings`. The binding records the actual method instance,
symbol declaration and local domain-member index; the original property-package owner,
exact parameter-kind name and source entity-index tuple; and the complete declared
quantity type with the actual source value and unit. P7 substitutes only these explicit
rows. The source coefficient remains inspectable and is converted using the declared
complete physical contract; a symbol's optional initial guess is never a parameter
value. P10 checks the source key/value and physical correspondence independently.
The complete ordered `reference.method_parameter_axes` inventory declares how each
local parameter axis contributes to the source data key: `member` contributes the
actual domain-member identity; `ref_entity` contributes its declared entity reference;
`phase_species_pair` contributes the actual material-member pair in phase/species order.
The axis position must match the method parameter's actual symbol/domain contract.
No domain-kind or label fallback chooses this mapping. Scalar parameters have no axis
rows; every indexed parameter axis has exactly one mapping.

`inferred.path_targets` retains each actual ordered `path_domains` vector alongside
its `path_index`: a member key always includes its domain, so consumers must not infer
an axis by a global member-identity lookup. P7 and P10 compare those factors against
the actual resolver traversal and collected group product.

Finite arithmetic subscripts retain an explicit `compiled.group_reindexings` record
and complete `compiled.group_reindexing_members` correspondence. The record identifies
the original normalized source/node, ordered free source binders, and fixed source
binder/member assignments. For each tuple in the sole P5-valid product, the shared P4
coordinate evaluator computes the original ordered subscripts and requires the exact
source-group member. The resulting group preserves free indices; it does not scalarize
equations. P10 repeats that evaluation against actual source rows and compares every
output/source tuple and provider. The identity is `named_id(instance_id,
"pse:group-reindex:v1:" + source_id.to_hex() + ":" + source_node_id_decimal + ":" +
concatenated_sorted_fixed_binder_hex_equals_member_hex_semicolon)`. Identity never
certifies the coordinate correspondence.

### Compensating controls

The methods document exposes the existing `reference.kernel_specs` relation with
explicit persisted `kernel_id` identity and no entity-registration kind. Its full
signature, physical units, outcome/effect policy, derivative declarations and
implementation-backed adapter declarations remain ordinary source rows. Source
admission validates those actual fields; neither registration nor an artifact
digest establishes an installed or executable backend.

Kernel method input selection is explicit in
`reference.method_kernel_inputs(method_id,input_name,dependency_ordinal)`. Every
declared input has exactly one mapping to an actual method dependency; descriptor
names or relative list positions never imply that correspondence. Coefficient
domain axes bind the declared ordered method-parameter domain kinds to unique
actual domains of the requesting state, refusing ambiguous bindings, then use the
same explicit parameter-axis projection as equation-template methods.

`compiled.kernel_output_symbols` records each generated output symbol's exact
requirement, state scope, method, kernel binding, output ordinal and actual index.
The symbol identity is `named_id(state_scope_id, "pse:kernel-output:v1:" +
method_id.to_hex() + ":" + output_ordinal_decimal + ":" + concatenated_index_hex)`.
This explicit `generated_semantic` route retains the complete physical output type,
an actual KernelCall body and method-output root; it never invents an authored
template declaration. `method_realizations` may pair a kernel binding and output
ordinal with this actual output symbol. Descriptor registration and preserved
implementation identities do not establish any available numeric execution route.
The actual kernel binding identity is `named_id(state_scope_id,
"pse:method-kernel-binding:v1:" + method_id.to_hex() + ":" + concatenated_index_hex)`;
the full ordered inputs and parameter values are still validated independently.
The existing scalar `ParameterBinding` carrier is preserved. An indexed parameter
descriptor remains inspectable but is refused for binding until a real array/group
parameter carrier exists; no repeated names or scalar type relabeling encodes an
array. A scalar descriptor fixture certifies binding only, not execution or a
derivative/backend implementation.

Complete candidate partition checks, ordered-domain validation, actual quantity
inference, demand/provision reconciliation, and incremental-versus-clean replay.
No constructor-only P10 context or fake kernel execution certificate is accepted.

`authored.template_contribution_contracts @2` adds the optional paired
`transfer_port_name` and `transfer_member_ordinal`. Both are absent for an ordinary
contribution. A transfer claim resolves the named port owned by the realized
instance and its exact member ordinal to one actual connection endpoint; a
forwarding port is equivalent only when its complete ordered state-tuple/owner
inventory, member declaration, physical contract and domains agree. Orientation
selects the corresponding incoming or outgoing endpoint. The realized expression
must be the actual member SymbolRef or Gather, with every concrete coordinate
reading the same provider; arbitrary arithmetic cannot assert a transfer.
Missing or ambiguous endpoints and duplicate same-orientation claims for the same
actual owner, connection and member are refused. Connection identity alone cannot
justify internal cancellation, which additionally requires distinct opposing
contributions with compatible full law/subject/physical meaning (ADR-0063).

### Confirmation

The named tests exercise independent templates, altered domains, excluded and
internal contributions, missing providers, natural-unit conversions and actual
source-to-P10 compilation. This record makes no execution claim yet.

## Pros and cons

Typed intermediate facts make negative decisions and extensions inspectable.
Their storage and compilation costs must be measured with the whole workflow.

## More information

[Wave 2 plan](../plans/04-wave-2-semantic-compilation.md), ADR-0040–0043,
ADR-0047, ADR-0054 and ADR-0062.

## Status history

- 2026-09-14 — proposed before implementation; formal decision PR pending.
- 2026-09-23 — tear-selection sentence and the R-30 tear-candidate activation replaced by ADR-0075, while still proposed.
