---
title: "Plan64b: Native scientific intake"
status: complete
date: 2026-11-02
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [design, validation]
doc_retention_reason: "Completed native intake evidence and exact source handoff remain dependencies of Plan64 and H02/H03."
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
| B01 | complete | 64a selected first-case/reference scope | Exact selected source closure, provenance roles and applicability limits retained in the bounded source account |
| B02 | complete | B01 selected sources and interpretations | Bounded Rust extraction/reconciliation produces typed deltas with faithful values, provenance and refusal behavior |
| B03 | complete | B02; existing publication contracts applicable | Ordinary contextual admission, guarded publication and reopened scientific selection work through existing owners |
| B04 | complete | B03 selected revisions; 64c case binding and C08 context contract | Both first-case closures and meaningful scientific/flow context reach ordinary consuming cases with exact retained readback |
| B05 | complete | B03/B04 outputs; C01 ordered-read contract | Required discovery/update-impact questions use proportionate shared reads with exact coverage; failed/unchanged/changed imports discriminate authority and identity |

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

All failure counts use the zero-failure target. Current native test routes explicitly
retain Arrow validation. Earlier adapter, source-impact, read/custody and qualification
controls retain their original scope; current source-to-use acceptance below does not
establish the wider scientific population.

**Tested:** the authenticated captured `pse-runtime` executable ran
`workflow::modeling::scientific_intake::tests:: --nocapture` with the current immutable
scientific-input capsule: three passed, zero failed (`/tmp/pse-bc-intake-3bcc7_a7`).
These independently check original FeOS values/unit normalization, changed/missing/duplicate
source refusal, authored-cell reimport and exact-parent ordinary admission.

**Tested, 2026-11-03:** `scripts/pse test pse-runtime --lib --profile local --features
solver-pounce,solver-ipopt,solver-kinsol,solver-root-isolation,canonical-tests,native-study-tests,compiled-evaluator
-E 'test(ordinary_delivery_scientific_intake_publishes_and_reopens_exact_source)'`
passed one selected canonical journey, zero failures, in 138.422 s
(`01a1a05a-3ac3-760d-adbd-38e68da92c21`). It exercises guarded publication,
fresh revision reopening, exact source readback and the ordinary consuming case through
the actual `prepare_operation -> start -> RunHandle` path. The separately authenticated
captured executable passed `--ignored --exact
tooling::capture::handoff::tests::c09_real_capture_offline_readback --nocapture` with
`C09_CAPTURE_KIND=canonical` on `/tmp/pse-bc-final-canonical-20261103`.
The explicit handoff selection omits source bodies and retains actual invocation,
canonical run/attempt, numerical and selected callback evidence; the immutable input
capsule and authored source retain the source bytes separately.

**Tested:** C08's strict ordinary recycle/design/rating inventory and current direct
design, same-case accepted-design-to-rating and compatible-closure science/readback
controls passed. [64c's verification](64c-process-authoring-and-first-delivery.md#verification-to-date)
owns their commands, conditions, complete selected recordings and intentional refusals.
Its shared scope-end qualification is complete, including the final-source feature refresh
and affected warning rechecks. H02/H03 matched reference execution is separate.

## Outcome

**Implemented:** B01–B05 supply the selected ordinary BT source closure, bounded Rust
FeOS interpretation, contextual parented admission, guarded canonical publication,
fresh readback and source-impact discovery. Ordered/exact reads avoid edit products;
selected history and receiving qualification preserve complete positive and absent
premises. Source population and database work remain outside scalar numerical evaluation.
The replaced one-row Poling transcription prototype and its tests were deleted.

**Implemented and Tested:** admitted response records, exact revisions, full physical
quantity conventions and frozen exposure bounds reach C08 through the same source path.
PR uses energy-only requirements without an unsupported frozen heat-capacity claim.
The [native/reference correspondence](../../packages/reference/ordinary-delivery/sources.md)
owns exact source/configuration/output handoff and scientific limits for H02/H03.

**Mistakes corrected:** the preliminary single-record transcription adapter did not
establish the demanded two-species artifact interpretation. The actual FeOS adapter
replaced it and its tests. The completed path selects independently attributed source
artifacts; transitive eligibility never follows from an oracle row's availability.
Scientific source validity and consuming-case extrapolation remain separate.

**Deliberate boundaries:** the ordinary pressure formulation and caloric/critical inputs
are explicit changed science relative to the oracle fixtures. Perry benzene use retains
its published envelope and explicit extrapolation; unknown empirical envelopes and the
predictive-zero interaction policy remain qualified as such. No stable quantitative
speedup is claimed for intake/read optimizations. Broader source population, H02/H03
execution, assembled P0–P7 acceptance and the collective dependency upgrade remain with
their existing owners. B01–B05 are complete for this selected native scope. The completed
record and source account remain while the coordinator and H02/H03 consume this handoff;
their retention authorizes no additional work.
