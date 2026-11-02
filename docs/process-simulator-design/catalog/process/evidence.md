---
title: "Process concepts — Evidence"
---

# Process concepts — Evidence

<a id="source-40b141b5836981aea4eede0e379cbd86"></a>

## Coverage case

Source: [Notion](https://app.notion.com/p/40b141b5836981aea4eede0e379cbd86).

**Cardinality:** One or more contracts and acceptance checks; zero or more attached evidence records.

**Invariants:** Design case and partial source evidence statuses never mean the workflow was executed or passed.

**Area:** Evidence

**Implementation:** Target Notion coverage index; all initial entries are unexecuted design/source evidence records.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/scope-and-open-design.md

**Definition:** A named process workflow selected to demonstrate a specific domain contract and acceptance conditions.

**Owner:** Design/qualification catalog

**Relationships:** Links design pages, implementation owners, source witnesses and later execution results.

**Concept:** Coverage case

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Repository mapping and acceptance workflows](../../process/repository-mapping-and-acceptance.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/scope-and-open-design.md).


<a id="source-40b141b58369814da78dc87cdea79fcb"></a>

## Execution run

Source: [Notion](https://app.notion.com/p/40b141b58369814da78dc87cdea79fcb).

**Cardinality:** Exactly one actual attempt and one immutable request identity per run; zero or more internal steps, outcomes and retained products. A retry is separately identified and linked to the originating request.

**Invariants:** Unexecuted design cases are not runs; cancellation, failure and incomplete execution remain explicit.

**Area:** Evidence

**Implementation:** Existing native runtime/job lifecycles; no new runs created by this design review.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/operations-and-validation.md

**Definition:** One actual attempt to execute an admitted atomic SimulationProblem assembled from a process model, case and Query; linked to its parent Study when orchestrated.

**Owner:** Runtime execution

**Relationships:** References original model revision, effective case, numerical choices and evidence.

**Concept:** Execution run

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Repository mapping and acceptance workflows](../../process/repository-mapping-and-acceptance.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/operations-and-validation.md).


<a id="source-40b141b5836981d4905feac909c4ad7c"></a>

## Lineage and qualification claim

Source: [Notion](https://app.notion.com/p/40b141b5836981d4905feac909c4ad7c).

**Cardinality:** Every material claim has one or more explicit evidence links and stated limitations.

**Invariants:** Source presence does not imply execution; local qualification does not imply full parity or all platforms. A selected required reference invocation with missing prerequisites fails; genuinely unrun scope stays unattempted. Neither is successful qualification or a reason to remove required cases from a frozen population. Industry direction and candidate examples are not industry-wide support claims.

**Area:** Evidence

**Implementation:** Existing provenance and qualified-outcome discipline; preserve exact scope in the domain catalog.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/operations-and-validation.md

**Definition:** Traceability from a reported fact or support claim to original source, occurrence, execution and evidence scope.

**Owner:** Source and evidence owners

**Relationships:** Connects source inspection, tests, runs, reference comparison and publication without merging them.

**Concept:** Lineage and qualification claim

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Repository mapping and acceptance workflows](../../process/repository-mapping-and-acceptance.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/operations-and-validation.md).


<a id="source-40b141b58369811ca4f0df43a0ca12bf"></a>

## Physical validation obligation

Source: [Notion](https://app.notion.com/p/40b141b58369811ca4f0df43a0ca12bf).

**Cardinality:** Zero or more obligations with explicit subject, tolerance, applicability and observation status.

**Invariants:** Scaled residuals or solver termination alone do not establish physical acceptance. Structural admission precedes solving but does not prove uniqueness or global validity. Each new numerical state requires its applicable original residual, bounds, domain and independent closure checks. Expected failure evidence must reach and identify the intended boundary.

**Area:** Evidence

**Implementation:** Existing original-expression checks and closure requirements; target visible process-level reporting.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs

**Definition:** An independently identified original-space condition required for physical acceptance.

**Owner:** Authored model, case or study contract

**Relationships:** Includes conservation, transport agreement, state consistency and validity checks.

**Concept:** Physical validation obligation

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Repository mapping and acceptance workflows](../../process/repository-mapping-and-acceptance.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/crates/pse-modeling/src/specialize/process.rs).


<a id="source-40b141b5836981879d39dcf09d0624bb"></a>

## Result and observation

Source: [Notion](https://app.notion.com/p/40b141b5836981879d39dcf09d0624bb).

**Cardinality:** Zero or more products per run; each has a defined subject, extent and provenance.

**Invariants:** Numerical success, physical acceptance, publication completeness and reference agreement remain separate. No-candidate outcomes retain unavailable assessments and failure/limit meaning. A terminal prefix cannot support unobserved future outputs or full-horizon/annualized totals. Local execution and explicit retained publication preserve their separate effects.

**Area:** Evidence

**Implementation:** Existing results/diagnostic separation; target process-facing interpretation using retained identities.

**Source URL:** https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/operations-and-validation.md

**Definition:** Values and outcomes retained from a run, including physical checks and bounded availability facts.

**Owner:** Runtime result/evidence owners

**Relationships:** Maps derived/native values back to original process quantities and selected output meaning. Engineering assessment states additional boundary, allocation, period and factor assumptions over the selected physical results.

**Concept:** Result and observation

Part of [Flowsheet Process Domain Model](../../process/index.md), version 1, 2026-11-01.

The structured properties carry the concept definition, ownership, relationships and obligations. Read [Repository mapping and acceptance workflows](../../process/repository-mapping-and-acceptance.md) for the full context and examples.

The implementation field distinguishes inspected source from the proposed contract. [Source or modeling reference](https://github.com/paul-heyse/pse-arrow/blob/926655517f6eac4ab52ae7a817a7b72168afdc25/docs/authoritative_design/sections/operations-and-validation.md).
