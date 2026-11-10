---
title: Semantic library and replay fixture boundaries — focused assessment
date: 2026-11-10
tier: change
purpose: target
standard: Core 3.4; ProcessSimulator 1.5
decision: accept-target
disposition_owner: docs/plans/72-surrealdb-functional-improvements.md
---

# Semantic library and replay fixture boundaries

**The scoped target is architecturally sound at Interface-checked strength.** A
third-party concurrency dependency does not establish dependence on the product's
execution owners. The revised ceiling expresses that distinction directly. The
quantity test correction also preserves the distinction between isolated fixture
authority and qualified production authority without inventing another factory or
removing useful refusal tests. One stale architecture sentence needs reconciliation.

This is independent, focused advice on the boundary mismatch exposed during ordinary
implementation. It does not initiate a broad diagnostic engagement, certify Plan72,
accept ADR-0268, or replace any separately due review or qualification.

## Scope and inspected baseline

The source baseline is `main` at
`78dad7db8228c185c4560bf522e86751cb3be95f`, including the uncommitted proposal on
2026-11-10: ADR-0268; the semantic ceiling and quantity unsafe-allowlist addition in
the root manifest; quantity's fixture dev-dependency and constructor substitution;
the added quantity replay selection in the local and scheduled Miri routes; and
blueprint revision 383 with the corresponding workspace/dependency owner edits.
Concurrent Python/reference edits were visible but excluded from this assessment.

Inspection also covered the existing dependency-ceiling walker, unsafe governance
tests, IDs qualification/fixture constructors and record binding, the complete two
quantity replay tests, and compiler/math dependency declarations. The selected core,
heuristics, test-assessment companion and repository binding informed the judgment.
No compilation, family check, governance test, product test or Miri was run by this
reviewer. Executed evidence and current disposition belong to
[Plan72](../../plans/72-surrealdb-functional-improvements.md).

The relevant scenarios are replacing or composing a mathematical library whose
normal closure includes Tokio, and testing receipt membership, interpretation and
cleanup locally. Scientific model definitions, numerical algorithms, native provider
qualification, production storage admission and the enclosing Plan72 design are
outside this boundary.

## Dependency responsibility

The revised `semantic` declaration retains its columnar exclusions and adds
`pse-engine`, `pse-runtime`, `pse-operations` and `pse-backend*`. The existing
`xtask/src/dependency_ceilings.rs` walker follows resolved normal edges transitively,
excluding proc-macro expansion owners, and matches the declared exact names or
trailing-wildcard prefixes. It does not equate the presence of Tokio with use of a
product Runtime, canonical operations or native backend. The compiler's explicit
dependency on the math owner is compatible with that responsibility model.

This is a declaration change within the existing enforcement mechanism. It neither
adds product orchestration to semantic crates nor makes arbitrary effectful library
usage appropriate there. Concrete effects still require appropriate ownership and
operation contracts. New product execution owners would need to be assessed against
this ceiling when introduced; the current names are an enforceable present boundary,
not a timeless inference from all possible crate names.

Keeping the Tokio prohibition would constrain incidental library composition while
failing to name the actual product dependencies being excluded. Removing it without
the new product exclusions would lose a useful enforceable boundary. Splitting or
wrapping a fitting mathematical library solely to hide Tokio would introduce an
adapter or packaging obligation without improving semantic isolation. The proposed
responsibility-based ceiling is the simpler complete choice (AP-01–AP-03, AP-05,
DP-13; heuristics H1, H3 and H4).

## Fixture authority and assurance

The only unsafe assertion found by a search of `crates/pse-quantity/src/**/*.rs` is
the private `authority` helper inside `#[cfg(test)] mod replay_tests`. It owns the
serialized receipt stream and calls the existing
`QualifiedScientificRecipe::from_admitted_record_fixture`. The newly selected
`pse-ids/fixtures` feature comes through quantity's dev-dependency; it is not added
to the normal dependency declaration.

The fixture constructor is nondefault and unsafe, documents complete fixture
ownership and isolation, and delegates to the existing qualification machinery with
the distinct `admitted-record-fixture` binding domain. Integrity hashing remains
insufficient for production admission. `RecordAuthority` still derives membership
from the supplied immutable population; the correction does not create unrestricted
safe reconstruction authority or alter the production constructor.

The two existing quantity tests challenge original backing/outcome preservation,
foreign-stream membership, an unsupported receipt interpretation, and cleanup after
reconstruction failure. They test replay obligations using real domain operations;
they do not independently establish those operations' scientific correctness or
canonical-store qualification. Constructing the supported corrupt fixture cases is
legitimate for this claim. Requiring a full compiler/storage journey would add
unrelated setup and could prevent malformed cases reaching the intended boundary.
Deleting the controls would discard useful assurance; adding a safe public fixture
factory would unnecessarily weaken the authority seam. The existing isolated
constructor is the fitting mechanism (AP-04, AP-06; test-assessment §§1, 3, 5–8;
heuristics H18–H20).

The root allowlist and existing governance tests enforce named-crate eligibility and
nonempty scoped reasons. They do **not** mechanically prove that every future unsafe
assertion in an allowlisted crate is test-only. The present confinement follows from
the inspected source, the narrow allowance and the constructor contract. This is an
evidence limit, not a reason to add a second governance mechanism for this correction.
Cargo feature unification likewise does not itself enforce isolation; the unsafe
contract and caller confinement remain material.

Both local and scheduled Miri declarations add
`resolved::receipts::replay_tests` for quantity while retaining their original IDs/
relations coverage. The local route retains its existing large-description skip;
the scheduled route retains its original unskipped selection. These are inspected
execution declarations, not evidence that Miri passed. Miri also cannot establish the
semantic truth of production admission merely by exercising an unsafe logical mint.

## Scoped judgment and correction

AP-01–AP-03 are satisfied for this target: semantic dependency direction and existing
library composition remain coherent. AP-04 and AP-05 are satisfied at the inspected
boundary: fixture and production authority remain distinct, and the explicit
normal-closure and unsafe controls retain their roles. AP-06 and AP-07 are satisfied
for the local replay scenario: useful membership/refusal controls need no unrelated
runtime, database or solver startup, and the change adds no new production operation
or reuse mechanism. No latency, capacity or build-time improvement is claimed.

The source supports the authority and membership design at Interface-checked
strength. Runtime behavioral adequacy remains unqualified here. PS-G1–PS-G3 and the
numerical-stage details are not newly assessed: this change neither alters a physical
model nor supplies numerical solve evidence. No broader G1–G9 pass is implied.

<a id="f01"></a>
**F01 — Reconcile the remaining Tokio layering statement.** At the inspected
baseline, blueprint §3.2 still says that the listed semantic/numerical crates reach
neither Arrow, DataFusion, Delta nor Tokio. That sentence contradicts revised §3.1
and would leave future readers with conflicting boundary guidance. Replace its Tokio
prohibition with the intended separation from columnar and product execution owners.
This is a documentation correction under AP-05/DP-24; it does not require a different
target design. The coordinator owns its implementation and disposition in Plan72.

The proposed rule impacts are the semantic-ceiling interpretation and quantity's
narrow unsafe eligibility, already routed through proposed ADR-0268 and blueprint
§3.1/§3.2. No additional production authority, crate, dependency family or scientific
contract change is recommended. Acceptance of that ADR and executed qualification
remain separate from this architectural recommendation.
