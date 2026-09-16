---
id: ADR-0067
title: Construct semantic execution through native logical plans
status: superseded
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-07, DM-21, DM-22, DM-24, DM-26, DM-27, DM-31, DM-32, DM-37, DM-41, DM-45, DM-46, DM-53, DM-59]
blueprint: [§D10, §D14, §3.3.3, §4.3, §4.4, §4.6, §5.3, §5.4, §6.15.2, §6.15.6, §14.2, §14.3, §14.3.1, §20.1, §20.5, §21.1, §22.1, §22.2, §24.1]
review: docs/design_review/reviews/design_review_provider-contracts_2026-09-15.md#7-principle-findings
evidence: Proposed
supersedes: [ADR-0013, ADR-0025, ADR-0026, ADR-0039, ADR-0048, ADR-0050]
superseded-by: ADR-0068
revisit: A required semantic property cannot be derived or checked by the common computation contract, or a delivered consumer requires a retired wire format
verification: just adr-lint; just docs; docs/plans/06-provider-contracts-hard-pivot.md Verification
---

# ADR-0067: Construct semantic execution through native logical plans

## Context

The fundamental review found independently assembled pass results, repeated local
producer execution, duplicated row validators and source inventories, and semantic
relationships reconstructed after execution. The maintainer chose a hard pivot:
implement the target directly and delete the superseded code, without preserving
old APIs, development stores, optimizer profiles or an old-versus-new test campaign.

The provider review found split hierarchical lookup and admission, silent root
registration failure and independent producer/inspection routes. The maintainer
authorized Plan 06's complete provider-contract pivot on 2026-09-15. The native
hierarchy and bound operation lifecycle now govern every product data operation.

## Scope

Amend blueprint D10/D14 and the construction, admission, ownership, reuse and
publication contracts in the cited sections. Typed relations remain model authority;
native `LogicalPlan`/`Expr` become the sole relational execution representation.
D5/D6/D11 physical and mathematical meaning, explicit identity and publication,
complete dependencies and current-format integrity remain required. Full library
eligibility and advisory dependency policy remain as ADR-0065/ADR-0066 specify.

The supersession is scoped as follows; unchanged requirements stay in the cited
blueprint, and accepted historical arguments are not edited.

| Record | Replaced restriction | Retained requirement |
|---|---|---|
| ADR-0013 | Exhaustive four engine roles/no-Newton placement ceiling | Declared numerical semantics and truthful complete-result pushdown |
| ADR-0025 | Phase-based exclusion of logical extension attribution | Typed data-level derivations and actual plan attribution |
| ADR-0026 | Library bans for Flight/uom | Registry quantity authority; pint validates generated output |
| ADR-0039 | Repeated unconditional validator invocation at internal boundaries | Full quantity algebra, nested semantic fields and actual external admission |
| ADR-0048 | Bounded algebra and deferred feature eligibility | Native predicates, explicit-schema imports and exact multiplicity/null/precision contracts |
| ADR-0050 | Fixed phase-1 row/byte ceiling and preservation of development stores | Sole canonical framing/constants; version actual identity-format changes |

## Drivers

- Connect declarations, exact inputs, derived properties and complete results through construction.
- Delete repeated local parsing, validation, replay and row-shaped execution layers.
- Use native engine semantics while keeping specialized domain algorithms explicit.
- Preserve truthful imports, immutable ownership, failure states and reproducibility.

## Options

1. Retain the current architecture and add another validation layer: rejected because it compounds independent meanings and work.
2. Maintain old and new engines with differential certification: rejected by the maintainer's hard-pivot direction.
3. Build one native-plan composition and private completion route, retaining only code that fits: selected.

## Outcome

Implement blueprint §3.3.3, §4.6 and §14.3.1 as the common path for binding,
preparation, semantic derivation, execution and publication eligibility. Registry
invariants compile once to native obligations. Exact immutable inputs and sound
constructors retain established facts; unresolved properties require explicit
checks. Metadata and planner constraints describe meaning and never certify it.

Bind actual sources, roles, parameters, functions, native algorithms and semantic
configuration before sealing preparation. Only its executor can mint a complete
admitted result. Remove arbitrary successful batch-map assembly, post-admission
validator mutation and the second producer-dispatch/replay path. Current-format
external import and reopen establish actual correspondence through the same
producer construction when required; read-only replay does not publish new physics
or move refs. Normal local completion does not trigger producer re-execution.

The current manifest selects an immutable admission-binding artifact by its encoding
checksum. That artifact records exact parent encodings, the producing operation,
selected policy handles and sealed engine settings; it does not contain its target
manifest, so publication has no checksum cycle. The selected original-document
source is an explicit binding even when the producing pass has no relational input
port for documents. Different invocation bindings can
select different physical manifests even when the logical output membership is
equal. Logical snapshot framing remains unchanged. Reopening admits the actual
bound dependencies and uses the available implementation assembly with the stored
settings; names alone do not recreate custom implementations. Missing bindings and
the superseded target-keyed receipt format are errors, never default-policy replay.
Stage lookup entries retain only exact input/output and attempt-record references.
They do not serialize a second registry, source-text or policy-row inventory.
Reuse compares the admitted dependencies and sealed invocation against the actual
request; lookup hashes and physical references alone cannot establish equivalence.

### Consequences

The target replaces incompatible interfaces and stored development fixtures directly.
There is one current format; retired versions are explicit errors, not an importer
project. Schema evolution between deliberately declared current contracts remains a
product operation. Canonical framing constants remain unchanged unless a separately
declared version changes them; configurable allocation limits are not byte identity.

Native plans cover relational work. Parsing, quantity algebra, MathIR
canonicalization, numerical solvers and conditional publication run as specialized
implementations inside bound native operations. Their inputs, effects, resources,
outputs and completion belong to the same provider framework. Callbacks and OS
primitives remain internal algorithm steps. No second relational algebra, optimizer,
proof language or completed-result framework is added.

### Provider generations and operation policy

One resolved inventory binds relation versions, operation-scoped roles, source
revisions, actual implementations and retained owners. Native root/catalog/schema/
table lookup, direct plans, metadata and Rust/Python inspection project that same
inventory. Domain namespaces are declaration categories, not the complete schema
universe. Generated operation/input/output/metadata scopes avoid role collisions.

Native registration semantics remain faithful. The root's infallible registration
mutates private assembly/attempt state and returns the replaced owner; product
commands enforce policy through a fallible boundary. Prepared work captures exact
generations, and native extensions cannot mutate another prepared or published
generation through shared mutable providers. Declared namespace effects are recorded;
undeclared changes refuse admission/publication. Raw native traits do not provide
immutable capabilities, application policy inheritance or transactions.

Compose semantic requirements, actual support/effects, overridable defaults and
resource limits once across root/catalog/schema/table/invocation scopes. Preserve
quoted names and bind cross-catalog conflicts explicitly. Facts retain actual
premises; metadata and capability views are generated from bindings, never editable
trust flags or a concrete provider roster. The complete native function/planner/
codec surface remains eligible with purpose-specific dependency/effect contracts.

Preparation captures dependencies and obligations before folding or view inlining,
including scalar-only and extension plans. DDL is constructed and admitted before
calling eager effectful handlers. One native operation owns private typed output
ports and an outcome stream; output lookup cannot rerun it. DML affected-row counts
do not establish durable publication. Complete obligations and the conditional
manifest/ref protocol determine visibility and explicit uncertain/retry outcomes.

Resolution establishes source generation and full/subset/negative coverage before
cheap synchronous lookup. External mutable sources require coherent revision
support or explicit capture/observation; cached async lookup is not a remote
transaction. Current-format cold admission shares one traversal context and invokes
the common producer only where correspondence requires it. Admitted inspection
uses provider streams with the original leases and cannot compile or publish.

### Compensating controls

Construction facts have stated premises and limited scope. Candidate data advertises
no unproved uniqueness, reference or coverage properties. Foreign mutable memory is
isolated before retaining facts; partial streams and unfinished obligations cannot
be published. Rewrites preserve full fields, nullability and metadata, explicit bag
and equality semantics, source correspondences and dependencies removed by folding.
The current format rejects malformed data and establishes derived correspondence;
a saved validity marker or matching hash cannot recreate process-owned guarantees.

### Confirmation

Plan 06 names provider conformance, first-principles construction, ownership, source/change, finite closure,
physical, lifecycle and source-to-P10 workflow checks. They target the new code;
there is no legacy equivalence baseline. `just adr-lint` and `just docs` establish
only document validity. A checked plan or passing example is not a universal proof,
and no implementation or performance acceptance is claimed by this proposed record.

## Pros and cons

One construction path makes semantic obligations and deletable duplicate work
explicit. The cost is replacing callers and fixtures together and implementing the
small domain-property transfers that native storage types do not supply.

## More information

- [Plan 06](../plans/06-provider-contracts-hard-pivot.md) is the sole implementation sequence and carries forward Plan 05's unfinished outcomes.
- [Provider review](../design_review/reviews/design_review_provider-contracts_2026-09-15.md) supplies the expanded contracts and independent G1–G7 closure requirements.
- [Fundamental review](../design_review/reviews/design_review_wave1-logical-plan-foundations_2026-09-14.md) supplies the source findings and property arguments.
- [Logical planning capability specification](../capability-maps/datafusion_logical_planning_capability_spec.md) distinguishes construction, analysis, execution and application correctness at the pin.
- Superseded records retain their historical arguments. Their unchanged physical quantity algebra, explicit-schema ingestion, exact pushdown, typed derivation and canonical framing requirements remain in the blueprint; their repeated-validation, placement, phase-envelope and feature-deferral restrictions do not.

## Status history

- 2026-09-14 — proposed before implementation under the maintainer-authorized Plan 05 hard pivot; blueprint revision 37 records the target. Formal ADR acceptance remains separate from implementation authorization.
- 2026-09-14 — supersedes ADR-0013/0025/0026/0039/0048/0050 only as scoped above; their immutable bodies retain historical evidence and the blueprint retains unchanged domain/identity requirements.
- 2026-09-15 — proposed record extended before Plan 06 implementation: provider generations, hierarchical policy, native commands/multi-output completion and common inspection. Blueprint revision 38 records these contracts. The provider review is Revise; G1–G7 implementation acceptance remains open.
- 2026-09-15 — superseded by ADR-0068.
