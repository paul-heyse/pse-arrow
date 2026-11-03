---
title: "Plan64b: Native scientific intake"
status: in-progress
date: 2026-11-02
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
review_sources: [docs/design_review/reviews/design_review_process_simulator_target_2026-11-02.md, docs/plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md]
adrs: []
---

# Native scientific intake

## Purpose and foundation

**Proposed:** deliver Packet A's complete selected-source → Rust interpretation/extraction → typed native delta → ordinary admission → guarded SurrealDB publication → ordinary consuming case → exact reopened readback journey. The [coordinator](64-process-simulator-target.md) owns scope, shared decisions and review dispositions; this companion owns B01–B05 and intake evidence. Authoring source is `1acd8c83c`; no acquisition or numerical execution was performed to create this plan.

Consume the [integrated review](../design_review/reviews/design_review_process_simulator_target_2026-11-02.md), particularly scientific source-to-use closure and Packet A, and [the sourcing contract](../process-simulator-design/delivery/scientific-sourcing-and-catalog.md). This is a native path inside pse-arrow, using existing SurrealDB. It has no thermo-knowledge service or separate accepted-knowledge database dependency. RC01–RC04 were accepted on 2026-11-02; their governing routes remain with 64a before dependent contracts change.

**Interface-checked:** `modeling/documents.rs` declares `SourceDelta`, `SourceDocumentDelta` and `SourceFitDelta`; `modeling/canonical/delta.rs::with_delta` consumes contextual deltas; `ModelingPackage::with_declarations` computes parented declaration edits. Existing `canonical_staging` and research publication own guarded persistence. These mechanisms support the intended composition; their presence does not establish a populated catalog or ordinary eligibility for the campaign fixtures.

Source owners are [documents](../../crates/pse-runtime/src/workflow/modeling/documents.rs), [delta admission](../../crates/pse-runtime/src/workflow/modeling/canonical/delta.rs), [canonical workflow](../../crates/pse-runtime/src/workflow/modeling/canonical.rs), [staging](../../crates/pse-operations/src/canonical_staging.rs), [research publication](../../crates/pse-runtime/src/workflow/research_publication.rs) and [knowledge declarations](../../crates/pse-schema/src/catalog/modeling_knowledge.rs). Extend the existing owner if a demanded operation is missing; do not build a second loader/admission/publication path.

## Target behavior and selected scientific boundary

### Execution requirements from 64a

Consume [64a's operation decisions](64a-scope-and-shared-contracts.md#a03--selected-library-backed-operation-design). Typed declarations remain scientific authority; Serde/Schemars describe boundaries and Arrow/Parquet/DataFusion supply applicable artifact and relational operations. Reuse ordinary SurrealDB staging/publication and registry-generated projections. No second scientific interpreter or general importer is selected; writable Surrealism belongs to G08.

B01 resolves the two-case subset without waiting for full H01. Current BTIdeal/BT_PR oracle banks do not establish ordinary eligibility. Select and attribute each source before adding an adapter; missing records block their dependent case. Preserve formulation, units, basis, datum, applicability and missing-versus-predictive-zero distinctions.

The [first-case source closure](64b-first-case-source-closure.md) owns the inspected per-record gaps and selected bounded published-data interpretation. Failed acquisition is not evidence that a source does not exist; neither an adapter nor a metadata record closes the missing ordinary science.

Execution completes B's native obligations and reference handoff. H02/H03 execution and the collective dependency upgrade remain separate.

The [recycle improvement review](../plan_improvement_reviews/recycle-preparation-performance-2026-11-02.md)
and maintainer's implementation-first direction add the contributions below. The
[coordinator](64-process-simulator-target.md#recycle-improvement-dispositions) owns I01–I12
dispositions; 64c C01/C02 own shared read/preparation corrections, C08 shared precision and
C09 selected diagnostics. B migrates its actual intake/discovery/case consumers and supplies
their scientific premises, without a second inventory, cache, recorder or accuracy resolver.
Plausible optimizations proceed with targeted verification and refinement, not a separate
benefit-approval phase. First-party Rust remains at optimization level 2.

Inventory the actual dependency closures of [recycle-flash](../../packages/reference/campaign/models/recycle-flash.pse) and [exchanger-costing](../../packages/reference/campaign/models/exchanger-costing.pse). Keep the recycle's ideal benzene/toluene formulation distinct from the exchanger's BT_PR/relational-density configuration. The named tests are reference witnesses, not ordinary-input authorizations. The exchanger's explicit unknown-area and pressure-extrapolation allowances, and its limited cost claim, survive the ordinary case definition.

For every consumed record preserve material/phase identity, quantity kind, unit, mass/molar basis, datum, formulation/variant, subject tuple, parameter source and applicability. Retain ordered versus symmetric interactions, missing versus explicit-zero/predictive coefficients, joint-fit identities and uncertainty where present. Separate formulation authority, parameter source, matched reference and independent evidence; one source can fill several roles only when explicitly justified.

The import operates against an exact parent revision. A bounded source-specific interpretation creates ordinary typed declaration/data changes, admission checks their contextual meaning, staging publishes a complete closed selection, and guarded activation makes that immutable revision discoverable. Conflicting or ambiguous interpretations remain attributed findings; they do not silently select a default. Failed or interrupted acquisition/publication must leave the prior admitted revision usable and any incomplete work non-authoritative.

An unchanged reimport preserves scientific identity even if retrieval time or documentary annotations change. A changed coefficient, formulation or interpretation creates the corresponding scientific revision; it must not overwrite historical input meaning. Existing imported content may satisfy the demonstration, but the complete path and reopened consuming case must still be exercised. NIST nitrogen is an optional small source example, not a substitute for a missing benzene/toluene dependency.

Discovery answers only consumer questions: what exact selected definitions/records exist, which dependencies they require, why a required input is absent or ineligible, and which native revision/evidence the answer covers. Extend `runtime.modeling_knowledge` and its name projection only where these queries need it. A source row is not executable science; an admitted record is not use-specific qualification; a limited query cannot establish global scientific absence.

## Package state

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| B01 | in-progress | 64a selected first-case/reference scope | Exact ordinary data closure and source-role decisions recorded; unresolved records have bounded source-selection work |
| B02 | in-progress | B01 selected sources and interpretations | Bounded Rust extraction/reconciliation produces typed deltas with faithful values, provenance and refusal behavior |
| B03 | in-progress | B02; existing publication contracts applicable | Ordinary contextual admission, guarded publication and reopened scientific selection work through existing owners |
| B04 | in-progress | B03 selected revisions; 64c case binding and C08 context contract | Both first-case closures and meaningful scientific/flow context reach ordinary consuming cases with exact retained readback |
| B05 | in-progress | B03/B04 outputs; C01 ordered-read contract | Required discovery/update-impact questions use proportionate shared reads with exact coverage; failed/unchanged/changed imports discriminate authority and identity |

### B01 — Resolve sources and ordinary eligibility

Read each case's full selected scientific closure, actual records and source notes. Produce a case-local decision table identifying existing ordinary inputs, fixture/oracle-only inputs, missing formulations/data and the exact source/revision/locator chosen for each gap. Prefer a structured authoritative artifact or an explicit scientific interpretation over sampling a provider's output and calling it a parameter source. Use the independent reference solely within its allowed clean-room role.

The investigation must settle whether existing record provenance permits ordinary use and whether the selected data reproduce the intended reference configuration. Its outputs are exact input selections and interpretation decisions consumed by B02 and 64c; a missing source, datum, formulation variant or permission blocks only its dependent ordinary case. Do not invent a coefficient or broaden an eligibility flag to remove the block.

For I05/C08, identify the independently sourced caloric/heat-capacity information and
admitted flow/energy characteristics consumed by meaningful precision. Existing FeOS/Perry
records may supply them; inspect their exact formulation, basis and envelope before seeking
new data. Preserve the Perry benzene extrapolation qualifier. A local Cp projection is not
a coupled output-error certificate, and a circulation capacity is not actual solved flow.

### B02 — Implement the smallest actual extraction gap

Use an existing adapter if it supplies the chosen source-to-native interpretation. Otherwise add a source-specific Rust adapter at the existing workflow boundary, retaining source bytes/locator, original values, adapter/interpretation revision and transformations. Code-defined science requires explicit semantic interpretation; retaining its foreign class is not native implementation. Extraction may produce a proposed delta plus attributed unresolved content, but publication cannot label unresolved required content complete.

Exercise a nontrivial unit/basis conversion, missing versus zero coefficient, directional interaction or joint parameter group as relevant to the chosen source. Compare expected native values independently of the conversion under test. Scope parser/adapter tests to that source rather than creating a generic scientific ingestion framework.

Extract genuinely missing C08 scientific context through this same bounded path, retaining
original values, unit/basis and interpretation. Engineering decision resolution belongs to
the existing authored policy, not the extractor. Do not infer an enthalpy allowance from
an arbitrary absolute datum or a provider's current trial value.

### B03 — Admit and publish through native authority

Feed the delta into ordinary contextual admission, preview and guarded publication. Preserve exact source selection, dependency closure, parent identity and eligibility. Reopen a fresh reader to recover typed scientific inputs from the published revision without reconstructing them from catalog metadata or foreign runtime objects. Measurements/fitted coefficients consume the existing research and fit-publication owners.

Migrate all callers of any replaced intake helper together and delete the replaced path once the target tests pass. New storage shapes, if actually required, are registry-authored and regenerated through the declared generator route. Do not hand-edit generated codecs or acquire a parallel version authority.

Consume C01's I08 separation of ordered reads from delta/edit products in publication and
reopened inspection. Consume C02's I09/I12 shared selected-admission improvements, carrying
qualified source context only while its existing invocation/premises hold. A new invocation,
changed source/physical/provider context or changed negative lookup still requires admission.
Preserve guarded parent/CAS semantics, leases and explicit force-validation. If C08 changes
authored engineering defaults, re-admit the affected sources and publish a coherent revision;
historical source/result meaning remains intact. C09's selected recorded journey retains
actual publication/readback evidence and managed child coverage where applicable.

### B04 — Bind the ordinary first cases

Bind published ordinary inputs to the same physical definitions used by the recycle and exchanger witnesses. Hand 64c exact source/revision identities, assumptions, allowances and missing-capability refusals, not fixture constructors that bypass eligibility. Execution in 64c exercises the actual native science; B04 establishes faithful input selection and readback, not a converged result.

At least one complete demanded source path must reach an ordinary consuming case and exact retained readback. Both cases need a complete ordinary eligible closure. The boundary permits reuse of existing native records/adapters; it does not permit dropping the sourcing demonstration because a record already exists.

Supply C08 exact scientific IDs/revisions, caloric differences, units/basis/datum conventions,
validity/extrapolation and independently admitted flow/energy characteristics. C08 owns the
programmatic allowance and goal interpretation; B does not select per-case solver knobs or
a universal molar-enthalpy floor. Requirements must account for actual consuming energy/flow
and useful thermal/output resolution. Bind source context before dependent precision tests;
unrelated source acquisition and C01/C02 implementation can proceed concurrently.

### B05 — Explain availability and source updates

Add only the selected discovery queries required by case authoring and later scientific increments. Show exact native definitions, sources, dependencies and covered revisions. For a source update, expose the scientific delta and affected selected consumers through existing dependency/preview owners. Do not claim all globally possible impacts from a bounded selection.

Compare unchanged reimport, changed scientific content, conflicting source interpretation, absent required record and interrupted publication. The current admitted revision remains authoritative until coherent activation succeeds. Diagnostic/partial content may remain inspectable without becoming ordinary executable science.

For I08, use existing selected pages/windows or exact IDs for case identification and
bounded discovery. Full ordered reads consume C01's shared read owner without eagerly
building edit/dependency products, including warm-path parent copies; genuine edits still
construct their complete products at the edit boundary. Do not page-loop a full read when
that repeatedly decodes ingress. For I09/I12, migrate selected-history and qualified-owner
consumers with C02, preserving newest-first candidate choice, complete positive/absent
premises and changed-context refusal. A bounded displayed result is not a bounded scan.

Acceptance adds exact ordering/identity/provenance and lease lifetime, scarce-capacity
selected reads, complete inventory followed by real edit/publication/reopen, and A/B/A
source history including insertion of a previously absent member. Distinguish typed carried
rows, real declaration/dataset clones and fallback decoding when correcting read extents;
never reduce conservative reservations merely because a multiplier looks large.

## Verification and handoff

**Planned evidence:** targeted adapter/admission/identity tests during implementation; real publication/reopen tests at the relevant integration boundary; 64c ordinary native journeys; and 64h matched-reference/assembled acceptance. Independent values and deliberately wrong/missing/convention-mismatched inputs must discriminate silent acceptance. Commands and modes are selected from the changed packages under AGENTS.md; required Arrow validation remains explicit.

Adopted performance benefits remain unmeasured; the review retains diagnostic prototype
measurements with their coverage and memory limits. Keep extraction and database work outside
repeated scalar evaluation; hydrate the selected immutable dependency closure for native
preparation and reuse it while its premises hold. Implement the scheduled plausible read,
copy and admission optimizations and use measurements to refine them. B01–B05 are in
progress; source population and admission do not establish complete publication/native
acceptance or close P2 scientific breadth or the full IDAES population.

**Proposed additional verification:** B01/B02 independently check selected caloric values
and units/basis; B03–B05 exercise the migrated reads/qualification and publication consumers;
C08 checks cross-flow and unit/datum-invariant precision; C09 retains the successful and
curtailed source-to-case evidence. C06 owns assembled native acceptance. Run targeted owner
tests and touched-package checks during implementation, then the relevant integration and
scope-end checks once functional scope is assembled. Do not repeat the full science campaign
for each source-local optimization or replace independent physical expectations with the
shared resolver's own calculation.

**Current checkpoint — 2026-11-02:** the selected ordinary source artifacts, typed scientific definitions and bounded two-species FeOS adapter are implemented. Unchanged reimport, ordinary admission and the guarded canonical publication/fresh source-to-consuming-case/readback journey pass their focused controls. The unused one-row Poling prototype was deleted after replacement validation. B05's selected-consumer impact, unchanged/changed source, missing-record and oracle-only refusal controls now pass. The shared specialization owner preserves typed provenance, resource and cancellation refusals; typed source edits retire stale authoritative text rather than claiming regenerated source. Discovery retains exact positive and negative source-selection premises and introduces no global catalog. Complete numerical delivery and scope-end qualification remain open. The [native/reference correspondence](../../packages/reference/ordinary-delivery/sources.md) supplies exact H02/H03 inputs and required outputs without claiming reference execution.

The I05/I08/I09/I12 contributions and selected C09 evidence integration above are newly
scheduled planning scope, not additional implemented/tested intake claims. Existing focused
receipts remain valid at their original scope; the new context and migrated consumers still
require their named verification. No source artifact, precision default or storage contract
was changed by this plan revision.
