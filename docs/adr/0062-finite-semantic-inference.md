---
id: ADR-0062
title: Define finite semantic inference and exact support contracts
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-07, DM-08, DM-19, DM-22, DM-31, DM-38, DM-46]
blueprint: [§6.15, §9.6, §14.2, §14.3]
review: docs/design_review/reviews/design_review_wave2-contracts_2026-09-14.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A rule requires non-finite value invention, higher-stratum negation, or row-level cache reuse.
verification: stratified_fixed_point, four_valued_rule_outcomes, demand_seed_closure, incremental_vs_clean_wave2 with force-validate; just adr-lint.
---

# ADR-0062: Define finite semantic inference and exact support contracts

## Context

Blueprint §14.2 declares stratified four-valued inference, but Wave 1 executes one
rule at a time. The existing method resolution fields cannot represent absence,
and a single requester or unlocated support key loses causality needed by P6.

## Scope

**Current construction target:** ADR-0067 and blueprint revision 37 replace this
record's former local replay, row-copy and phase-limited execution mechanisms.
Plan 05 owns implementation; the domain/identity/lifetime requirements retained
below are implemented through its single native preparation/completion route.

Amend blueprint §6.15 with the consumed P3–P6 contracts and their producer
boundaries. This activates R-30 for those contracts; it changes no identity framing,
solver capability, dependency family or whole-stage reuse policy.

## Drivers

Finite demand closure; explicit ambiguity; DataFusion execution of relational
selection; exact provenance; rejection before a false graph can be published.

## Options

1. Keep single-rule execution and procedural per-property selection: duplicates
   policy, loses conflicts and is rejected.
2. Add a second rule language or custom logical node: no demonstrated operator gap.
3. Extend the existing RulePlan compiler with a finite stratum workspace, typed
   support plans and exact value comparison: selected.

## Outcome

Use blueprint §6.15.1–§6.15.3. P3 supplies finite prospective scopes and typed
configuration; P4/P5 cannot discover new template choices after that boundary.
Rules operate over admitted Arrow worktables in the existing shared runtime.
DataFusion performs joins, difference and ordered grouping; Rust owns scheduling,
checked identity framing and bounded graph/IR algorithms.

A fact key is the declared complete key, and its value excludes fields explicitly
marked provenance. Compare actual typed values for convergence and conflicts.
Each head has a mechanically projected typed assertion relation preserving every
competing payload; support edges name the actual assertion. Undecided heads are
terminal within a stratum; same-stratum consumers are rejected. Reject conflicts
abort the entire stratum before publication. This bounds the supported scheduler
without introducing order-dependent retraction semantics.
Distinct support edges are retained independently of fact deduplication; cycles
retain edges, not endlessly expanded proof paths. Negated support refers to the
complete lower-stratum input binding. A digest is never an absence certificate.

P6 selection is separate from P9 realization. An unresolved or ambiguous requirement
has no selected method, realization or template. Candidate rows and all requesters
remain inspectable. Precedence is declared and deterministic; tied distinct methods
are ambiguous, including different versions. Scalar demand uses an empty index
rather than null; unspecified seed indices expand only over declared finite domains.

P3 lowers variadic selector union/intersection to explicit binary fold witnesses,
retaining every source child and its actual ordinal. Empty union is false; empty
intersection is true; singleton folds are identity; difference has exactly two
children. P5 evaluates complete Boolean membership decisions as positive finite
facts. This makes nested differences monotone without reading an unsettled absence.

A state template explicitly declares its complete ordered port members by actual
symbol declaration ID, allowing both variable and expression roles. Port collections
retain every `(port_id,state_index,state_instance_id)` target and their ordered
domain product. A scalar port records its exact state instance; a collection has
no fabricated representative instance. Member correspondence includes the complete
state-index prefix, symbol-local domain product and physical quantity contract.

Finite structural construction within a pass uses a registered constructor and
explicit output contract. Its private workspace retains actual source bindings,
complete source rows/keys for each constructed row, and the constructor implementation.
Each recipe retains its original sealed source session. Composed recipes form a
finite composition of native plans and explicit algorithms retaining exact immutable owners; support expands to actual source leaves without local replay.
An augmented semantic pass selects an explicit input subset in a session fork
sharing the same runtime, allocator, function implementations and semantic settings,
then adds its declared worktables. It never replaces a selected published provider.
Common preparation binds the exact private providers and schema; only complete execution plus residual obligations establishes the result. Constructed
support expands to the original admitted source locations. A caller-provided batch,
digest, or undeclared derived input cannot establish this authority.

### Consequences

Schemas, generated bindings, stage ports and old fixtures change together. Earlier
schema fingerprints are refused; no silent migration of stored Wave 1 artifacts is
promised. New schemas use new versions where existing relation meanings change.

The target derives full semantic fields through the actual observed native
lowering/physical-rule path (§4.6), replacing detached metadata repair. Support
removal invalidates/recomputes affected and dependent strata from remaining actual
inputs, including negative scopes; recursive self-support cannot retain an
ungrounded truth.

### Compensating controls

Exact head admission, finite domain checks, explicit resource ceilings and
cancellation, stratification validation, retained actual dependency bindings, and
current-format external correspondence checks protect the boundary. Unchanged local completions are not replayed.

### Confirmation

The named verification tests compare values and support under shuffled batches,
cycles, nullable predicates, conflicts, candidate additions and clean recomputation.
No test is claimed run by this proposed record.

## Pros and cons

The existing algebra stays the only rule authority. More complete evidence costs
memory and storage; it is budgeted and measured rather than dropped or replaced by
hashes. Per-row incremental reuse remains deferred.

## More information

[Wave 2 plan](../plans/04-wave-2-semantic-compilation.md), blueprint §14.2,
ADR-0040, ADR-0053, ADR-0055 and ADR-0056.

### Singleton state scopes and exact demand occurrences

**Proposed.** P6 frames `state_scope_id = named_id(state_instance_id,
"pse:state-scope:v1")` and admits the singleton correspondence only by joining the
actual selected package state-definition method to the actual instance template.
Different authored selectors reaching the same actual state therefore coalesce
without making an authored selector the state identity. Every requirement keeps its
ordered actual domain members and every requester keeps its separate source support.

A normalized expression demand retains the exact family-local `read_node_id` in
addition to source, declaration and guard. Distinct indexed reads of the same
symbol under the same guard remain distinct occurrences. P6 evaluates that original
read's ordered index expressions under explicit source binder assignments and joins
the exact projected guard tuple to P4 outcomes. An opaque explicit demand has no
read node and keeps its complete authored tuple. Matching names, domains or hashes
cannot substitute for the read-to-tuple correspondence.

### Explicit connection expansion and port collections

**Proposed target.** Successful finite rule execution participates in the single ADR-0067 completed-result capability; its former independent receipt framework is replaced. It retains the admitted rule inventory, exact bindings and sealed source
session, all immutable completed relation rows and derivations. A subsequent local
constructor or rule may consume a member only through this capability, checking
actual full rows, relation membership and the shared execution environment. Callers
cannot wrap arbitrary batches or a mutable result map as completed semantics.
Current-format reopen uses P9's same producer construction only where persisted correspondence requires it; no new physics is published. These
ephemeral receipts introduce no durable substage or late P6 requirement closure.

**Proposed.** Connection expansion requires an actual
`reference.connection_bindings` row keyed by the declared rule template, with the
closed `ConnectionExpansion` vocabulary (`equality` in Wave 2). A template identity
alone does not declare equality. `compiled.port_member_groups` records each
complete port-member collection product; its actual members concatenate the exact
P5 state tuple and P7 symbol-local tuple. P8 creates the groups and P10 validates
their complete owner/member correspondence.

## Status history

The compiler's finite element-coefficient adapter also consumes the existing pinned
DataFusion family directly for `LogicalPlanBuilder` joins over admitted member and
composition relations. This is a relational adapter in the compiler orchestration
layer; the mathematical and quantity leaf crates remain Arrow-free. It shares the
same session and runtime and adds no independent default, SQL rule text, UDF, custom
execution node, crate or dependency family. Coefficient arithmetic follows the
declared projection formula only after the actual relational correspondence is
complete; P10 independently admits the actual source and output values.

- 2026-09-14 — proposed before implementation under the authorized Wave 2 scope;
  formal decision PR and maintainer acceptance remain pending.

### Element projection and inactive coordinate outcomes

An exact source read whose evaluated coordinate has no actual domain member is
recorded as `inferred.read_coordinate_failures`, retaining its original node and
ordered source bindings. The same declared inner/outer guard plans used for normal
demand seeds decide whether that row is a violation. False branches need no target;
true or unknown branches require one. Missing/conflicting guard evidence refuses.
Ambiguity, malformed declarations, cancellation and reservation failures are not
converted into inactive-coordinate evidence.

Element balances retain a free Element axis and explicit Species reductions.
`compiled.element_projection_groups` declares each generated product's ordered
Element/Species domains, tied to the actual law application and contribution.
These products do not rewrite P3 normalized products. Actual coefficient members
are joined to admitted species composition and, for an explicitly declared mass
projection, molecular weight. Generated constant-expression symbols own these
immutable values; they are neither solver guesses nor authored declarations.
P10 independently validates product/member/source correspondence and all complete
quantity contracts before admitting the resulting graph.

### Checked execution representation and constructor provenance

**Implemented historical mechanism; replaced by the ADR-0067 common computation route.** A registered structural constructor receives complete admitted
input bindings. Its output identity and provenance are assigned from the actual
pass, relation and full key; exact source witnesses supply matching derivation
rows. P9 recomputes the existing P4/P5 programs over complete augmented normalized
inputs, using immutable completion receipts between rule and constructor phases.
The historical stage validator replayed this full sequence. The target retains actual construction ownership locally and establishes external correspondence only at its required boundary.

Historical characterization found that DataFusion 55 scalar list extraction
drops nested child metadata. The former execution adapter repaired missing metadata
from an already admitted expression field,
after checking identical storage, field names and nullability and agreement of
every metadata entry still present. Scalar expansion is reserved before allocation
and exported arrays retain the lease. A metadata-bearing build side of a Cartesian
join uses DataFusion's unfiltered inner nested-loop join, whose Arrow take path
preserves those child fields. Its complete schema had to equal the original join's
schema. Generic source and head admission remain strict.

**Tested.** `just test-package pse-rules --test stratified_fixed_point --test
four_valued_rule_outcomes --test ordered_aggregate_unnest`, default nextest with
`pse-relations/force-validate`, ran the original 18 tests with 18 passed, zero
failures and zero skips (baseline zero). The additional completed-receipt, union
literal and integer narrowing regressions and production P3–P5 fixtures remain
pending execution; this receipt does not certify those later additions.

- 2026-09-14 — reconciled with ADR-0067 and Plan 05; prior receipts describe their original code and do not certify the hard-pivot implementation.
