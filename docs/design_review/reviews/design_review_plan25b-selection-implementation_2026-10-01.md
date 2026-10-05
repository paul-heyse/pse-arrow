---
title: "Plan 25b scientific selection implementation review"
date: 2026-10-01
standard: core-3.3/process-simulator-1.3
tier: change
purpose: conformance
evidence: Implemented
decision: Revise
---

# Plan 25b B1/B2/B3 and source costing implementation review

Read-only review of the Oct 1 integrated dirty-main checkpoint, HEAD `332612064d9120dee250d2dd7b8905f087fc9b69`. Source inspected on 2026-10-01 while B3 tests and B4 code were being edited. This is a bounded implementation review against Plan 25b, proposed ADR-0140/0141 and the scoped Sept 30 target review; it is not whole-product qualification. No builds, tests, formatting, commits or repository edits were performed.

## Material findings

### IR25B-01 — Table consumers do not retain the admitted selection closure

Location: `crates/pse-modeling/src/scientific_selection.rs:104-109`, `crates/pse-modeling/src/specialize/value.rs:494-508,801-815`. Concrete consumers: `packages/reference/methods/models/cubic.pse:35-37`, `packages/reference/thermodynamics/models/pcsaft.pse:49-51`, and `packages/reference/domain/models/properties.pse:140-144`.

Trigger: specialize a property consumer that reads selected records via `properties.pure_selection[k,...]` / `selection[k,...]` without directly reading `k.admitted`. The package admission requirement checks membership in `k.admitted.records`, but specialization evaluates only package and record identities. `retain_context` matches only a value exactly equal to the context entity; `member` examines the receiver, not the returned attribute, and the table read has no mechanism to retain the admission products used to authorize that row.

Consequence: the specialized model can carry no selection closure for a scientifically selected property package. Changing dependency edges or scientific closure while preserving the selected coefficient and declaration IDs can then leave numerical/preparation meaning without the required retained closure identity and lineage. A full reactor happens to read a selection context explicitly and does not establish the property-table case.

Close with generic retention of the admission products actually consumed through selected table/package records (without scientific-name dispatch), plus an isolated property consumer test asserting the selected closure survives specialization and a changed unused dependency/edge changes preparation identity. Check that an unrelated package closure is not retained.

### IR25B-02 — Finite-function hashes snapshot closures before this call consumes them

Location: `crates/pse-modeling/src/specialize/functions.rs:497-499`, static argument evaluation at `588-610`, body/guard work at `726-760`.

Trigger: the first call to a function reaches a selection context while resolving its static actual arguments or its body. The V8 hasher frames the global collector before either operation, and does not frame the final consumed closure products after body/guard evaluation.

Consequence: the function ID omits its newly consumed closure; identical numerical bodies and entity IDs but different dependency edges can yield the same finite-function ID. IDs also depend on which unrelated closures were reached before this call. The later dispatch-body hash does not repair a finite-function ID and is only effective if retention already happened.

Close by framing the function's consumed closure products after actual evaluation, with appropriate occurrence scope, and tests for a first isolated call with equal coefficients but changed edges and for changed evaluation order.

### IR25B-03 — Joint-fit membership can bypass atomic closure

Location: `packages/reference/domain/models/properties.pse:38-46,118-119`.

Trigger: declare `fit_group G { members={AB,AC} }`, but omit G from AB's `fit_groups` (whose default is empty), then select AB alone. `required_records` consults only `r.fit_groups`, so the declaration of G's members supplies no closure obligation. No reciprocal-membership requirement rejects this disagreement. Conversely a record may name a group whose members do not include it.

Consequence: a declared inseparable fit can be split without a declared permitted projection due to a missing backlink; independently authored membership directions become conflicting scientific authorities.

Close by deriving one direction from the other or refusing disagreement at admission. Test the omitted backlink, wrong backlink and ordinary complete group / permitted projection. Current joint-fit tests author both directions correctly and do not cover this disagreement.

## Source-traced positive assessment

- B1 distinguishes explicit complete, complete-empty and unknown composition. Element and charge admission checks are independent; sparse complete formulas supply zeros and mass derivation separately requires atomic weights. Apparent and lumped mappings demand complete participants before conserving elements and charge. Elemental control-volume consumers now call guarded authoritative `chemistry.element_count`; the old arbitrary atoms callback is gone in the searched production package scope.
- B2's concrete `reactions.Projection` derives coefficients exclusively from `chemistry.stoichiometry`, refuses missing nonzero participants and unadmitted records, permits inert extras, checks one record per reaction and binds rate/heat extent identities. CSTR and PFR both consume this projection for source and heat terms. No second production coefficient callback was found in inspected package/Rust source. The concrete definition cannot be extended as an interface; the authored refusal test covers that attempted override.
- B3 separates phase-independent pure data from phased records, retains parameterization/family/subject/variant identity independently of source provenance, declares ordered NRTL directions and canonical symmetric records. Explicit prediction has rule/input authority distinct from stored zero and absent required pairs. Closure is finite, canonical and follows reached records with context; conflicting slots refuse. During this review the source requirement was tightened from roots-only to all reached records in the consuming subject/model scope, so that earlier concern is not reported as current.
- Dataset binding extends nonderived attributes while rejecting replacement of kind-bound or derived attributes, in both inline and document record producers. This supports source/coherence bindings without introducing provenance into keys.
- Source costing removes `pressure_extrapolation` Boolean dispatch. Unknown area evidence remains Unknown, pressure and oversize regions are Reported, hard positivity remains a mathematical domain, and seed/campaign callers use explicit independent named permissions. No old Boolean production consumer was found in the searched source. B4 enforcement/runtime observation correctness is outside this completed review.

## Coverage and limits

Inspected source owners include chemistry, properties/interactions, correlations/cubic/NRTL/PC-SAFT parameter access, reaction forms/projection, both reactors, elemental balances, costing/seed/campaign declarations, entity document/inline layout, expression builtins, static evaluator/callbacks, closure collector and finite-function/dispatch hashing. Scientific branches on species/method/phase names were not found in the new generic selection mechanism inspected; this was not an exhaustive Rust architecture audit.

Existing old-API test fixtures were observed in `crates/pse-compiler/src/physical_potential_tests.rs:172-177` and `crates/pse-modeling/src/domain_schema.rs:288` (`pair_selection` / `PairAbsence`); the coordinator was informed while test migration was active. Treat these as an outstanding migration check rather than an independent production defect until that assigned work settles.

Targeted tests read include `scientific_composition_tests`, `scientific_reaction_tests` and the newly arriving `scientific_selection_tests`; none were run by this reviewer. The only prior functional receipt provided in the brief is composition 5/5 with explicit force-validation. Other outcomes are pending and not imported as passing evidence. B4 missing-claim fallback, numerical demand preservation, permission enforcement and generated transport acceptance remain unfinished/excluded, as do native solve behavior, performance, durable history and Plan 25k qualification.

Findings require correction and focused executed controls before B3 implementation acceptance. No additional material B1/B2/source-costing declaration defect was found within the inspected scope.

Current finding dispositions and executed corrections belong to [Plan 25b](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25b-scientific-knowledge-and-applicability.md#execution-checkpoint). This review retains its original snapshot and verdict.
