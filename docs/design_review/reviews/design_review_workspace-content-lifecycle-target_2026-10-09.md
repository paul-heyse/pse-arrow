---
title: Workspace content lifecycle — scoped target assessment
date: 2026-10-09
tier: design
purpose: target
standard: Core 3.4
decision: accept
disposition_owner: docs/plans/32-workspace-content-lifecycle.md
---

# Workspace content lifecycle target

**Accept the scoped target at Proposed design strength.** The revised design separates document interpretation, native scope ownership, publication selection and producer-owned disposal. It provides credible implementation boundaries without creating a second editable backlog, a content service or a universal cleanup mechanism.

This judgment concerns [Plan 32](../../plans/32-workspace-content-lifecycle.md), proposed [ADR-0168](../../adr/0168-workspace-content-lifecycle.md) and blueprint §24.4.1. It does not accept an implementation, close the predecessor findings or qualify scientific behavior. The narrow follow-up assessment below also accepts a closed immutable-ADR scalar adapter within the same reader; it does not authorize edits to immutable records.

## 1. Scope, drivers and coverage

This is an independent, design-tier, target-purpose assessment against Core 3.4, its efficiency heuristics and the repository binding. ProcessSimulator 1.5 scientific gates are not applicable: the target changes documentation and support tooling, without changing scientific models or execution contracts.

The inspected baseline is `main` at `4c24721e691187e1a5b28398b29722fbde671da8`, including the concurrent working-tree target revisions on 2026-10-09. The assessment includes explicit Markdown-table bindings, documentation environment override, temporary display-read borrows and lifecycle cutover/drain. A narrow reopening assessed the immutable ADR input boundary exposed during implementation, the initial `scripts/document_metadata.py` reader and the live legacy owner-reference forms. This reopening is target assessment, not implementation acceptance.

The functional target is useful discovery and safe lifecycle management across multiple concurrent plans, retained documentation and producer outputs. Relevant variation includes new document families, evolving native packet tables, large investigation assets, parser replacement and interrupted or concurrent resource use.

Inspection covered the target documents; the predecessor review; publication, ADR, environment, storage-observation, validation-receipt and resource-owner support code; representative retained Plan 28/28e/33 content; the substantive `create-plan` guidance and suggested structure; and the affected ADR frontmatter and bounded invalid-input inventory. Production/test bodies, `.venv*` contents, persistent scientific state and archive payloads were excluded. No repository files were changed and no tests or cleanup were run by this reviewer.

## 2. Decomposition, ownership and dependencies

| Responsibility | Owner and consumed boundary |
|---|---|
| Metadata interpretation | `scripts/document_metadata.py`; preserved source bytes, library-backed parsing, controlled lifecycle interpretation and native family adapters |
| Lifecycle defaults and content bindings | `docs/lifecycle.toml`; references existing collection roots and native content rather than copying their statuses; closed immutable scalar-encoding declarations contain identity, not replacement meaning |
| Publication | `docs/site.toml` and `scripts/docs.py`; collection/search scope and explicitly selected assets |
| Documentation environment | `scripts/pse_env.py`; explicit docs selection, with thin recipes and direct consumers |
| Inventory and aggregate scope | `scripts/document_lifecycle.py`; read-only projections of interpreted metadata and selected native tables |
| Observation | `scripts/build_storage.py`; configured stores and explicit uncertainty, independent of ordinary document reads |
| Artifact disposal | Existing assessment producer and `scripts/test_resources.py`; sealed artifact meaning, references, borrowers, outcome, drain and exact identity |

Dependencies follow these responsibilities. Publication does not determine retention; metadata does not confer release authority; aggregate discovery does not schedule work. Native-generation disposal remains with its existing collector. The historical scalar adapter is a family input boundary within the shared reader, not another parser or consumer-specific fallback.

## 3. Contracts, authority and constraints

The model preserves the consequential distinctions: document role versus native lifecycle status; authored content versus derived views; retention versus delivery; retained provenance versus disposable scratch; successful reconciliation versus failure or interruption; and observation versus authorization.

The metadata reader rejects conflicting definitions and invalid controlled fields while preserving native family semantics. Canonical `doc_*` references are repository-relative. Existing legacy owner values beginning `docs/` are also repository-relative; other relative legacy owner values resolve from the source document's parent. Normalize each representation before precedence, existence and conflict checks. The inspected retained reviews actually use `docs/plans/...` legacy values, so treating every legacy value as source-relative would change their meaning. Historical-owner links do not automatically confer a current actionable owner.

Defaults classify immutable records without rewriting them. The newly exposed historical input format also needs explicit interpretation: the bounded inventory contains 22 immutable ADR documents with 37 native scalar spans that the former flat reader treated as literal strings, plus one invalid scalar in the mutable template. Examples include unquoted `not-required:` review values, titles containing colon-space and leading backticks in verification/revisit values.

For those immutable spans, select a closed declaration by canonical repository-relative document identity, native scalar key and digest of the exact original raw scalar. Before the same strict ruamel parse, quote only matching literal spans in memory. Preserve the original prefix/body bytes and old literal string meaning. Do not derive eligibility from a diagnostic label, document-supplied status or parse failure. Nonmatching and undeclared inputs follow ordinary strict parsing. Duplicate keys remain visible to ruamel; merge/cycle and controlled-field checks still apply. Historical baseline reads use the same canonical document identity and adapter. The mutable template is corrected directly, and new/mutable documents receive no adaptation.

Aggregate scope uses document, binding and native-row identity. Unique heading/header/column selectors identify the actual content owner. Explicit binding vocabularies classify statuses while preserving raw wording and source locations. Missing status remains unknown; similar titles do not establish equivalent scope.

Resource release consumes producer-declared, finalized artifact meaning. Undeclared or legacy artifacts remain protected. Reclassification cannot silently downgrade retained evidence, and display-only reads borrow the selected report and referenced origins before payload access.

## 4. Change scenarios and composition

[Predecessor scenarios S01–S07](design_review_workspace-content-lifecycle_2026-10-09.md#8-revealing-scenarios-and-alternatives) remain the scenario source, subject to the maintainer's recorded legacy-retirement correction.

| Scenario | Assessment of the revised target |
|---|---|
| S01/S07: discover active work or add a document family | Explicit publication selection and native-table bindings preserve concurrent owners; family changes are absorbed by interpretation/adapters rather than repeated consumer parsers |
| S02/S03: retire documents or preserve a live handoff | Ordinary retained Plan 28+ lifecycle remains dependency-aware. The specifically authorized pre-28 removal requires necessary reference repairs, without salvage, old-item reconciliation or scope import |
| S04/S06: retain a large investigation and rebuild delivery | Explicit asset selection prevents retention from automatically enlarging publication; candidate staging and replacement preserve the existing recovery contract |
| S05: a borrower starts or a path changes | Existing resource reservations, borrower/reference exclusion and identity rechecks remain the disposal boundary |
| S06: interrupt a campaign or activate the new lifecycle | Failed/incomplete artifacts remain protected; affected old producers/reclaimers drain before activation; active records are not migrated |
| S07: migrate parsing over historical native syntax | Exact immutable scalar bindings preserve recorded literal meaning and bytes through the shared strict parser; the template and new authoring use valid YAML |

A parser substitution is a mechanism change within the interpretation owner. A new packet-table shape is a binding change unless it introduces genuinely new status meaning. Neither requires changing substantive plan prose or creating another status authority.

## 5. Mechanisms and execution fit

The physical route is proportionate to this workspace. Aggregate scope necessarily reads its selected native content; it does not require repository-wide full-text ingestion or persistent indexing. Normal document reads neither synchronize environments nor initiate host-storage scans.

Publication processes selected delivery inputs rather than every retained evidence payload. Resource disposal reuses an existing owner instead of introducing another registry, lease system or sweeper. Finalized manifests and temporary borrows preserve distinct write, read and cleanup lifetimes.

The separate docs environment adds explicit provisioning and launch-path maintenance. That burden buys independence from the product extension and prevents documentation bootstrap from removing product packages. Explicit `--docs` selection visibly overrides inherited selectors and rejects product-environment aliases before effects.

The closed scalar adapter adds a small amount of historical format knowledge at the actual input boundary. Its declaration carries path/key/raw-scalar identity; the source remains the value authority. It does not retain the old flat parser, infer arbitrary YAML repairs or introduce a whole-document source seal. Matching original spans are normalized before parsing rather than repeatedly trying alternate parsers after errors.

These are qualitative execution-fit conclusions. No latency, productivity, storage-savings or optimal-cache-budget result is established.

## 6. Architectural assessment and gates

Verdicts below assess the written target; they do not certify deployed behavior.

| Foundation | Verdict | Basis |
|---|---|---|
| AP-01 Separation of concerns | Satisfied | Interpretation, scope ownership, delivery, observation and disposal have distinct owners; historical syntax is handled at the interpretation boundary |
| AP-02 Stable contracts | Satisfied | Native meanings and source bytes survive parser changes, including explicitly bound immutable scalar syntax; unsupported or ambiguous interpretations reject or remain unknown |
| AP-03 Composition | Satisfied | Existing publisher, environment selector and resource collector are composed through explicit inputs; one library parse follows family normalization |
| AP-04 Domain model and semantic authority | Satisfied | Consequential lifecycle distinctions govern intended operations; native tables and original scalar bytes retain their respective authorities |
| AP-05 Explicit structure | Satisfied | Bindings, precedence, asset selection, artifact roles and action eligibility are explicit |
| AP-06 Local reasoning/testability | Satisfied | Support-tool fixtures can exercise the target without solvers, databases or product compilation |
| AP-07 Execution fits workload | Satisfied | Selected content/assets and existing lifecycle owners avoid unnecessary ingestion, copying and coordination; historical adaptation is closed and local |

| Gate | Target judgment | Basis |
|---|---|---|
| G1 Authority | Pass | No independently editable aggregate status ledger or duplicate release owner; scalar declarations supply identity rather than replacement values |
| G2 Semantic fidelity | Pass | Native states, raw evidence wording, source locations, bodies, literal historical scalar meaning and relationship kinds are preserved |
| G3 Validity | Pass | Strict metadata/binding rejection and protected unknown resource states precede effects; bounded syntax adaptation preserves the same strict validation |
| G4 Hidden behavior | Pass | Bootstrap, observation and disposal are explicit; ordinary reads do not sync or clean; syntax adaptation never writes the source |
| G5 Consistency and recovery | Pass | Sealing, truthful outcomes, borrows, drain, reservations and replacement recovery have explicit contracts |
| G6 Transformation and reuse | Pass | Derived views identify native sources; artifact reuse retains required origins and protection; immutable syntax normalization preserves the consumed old string meaning |
| G7 Truthful capability claims | Pass | Target remains Proposed; missing state and unqualified legacy resources are not presented as success |
| G8 Library leverage | Pass | Maintained YAML/Markdown parsers supply generic syntax; repository adapters retain domain policy and only the closed historical encoding gap |
| G9 Architectural fitness | Pass | All applicable foundations are satisfied for this bounded target |

Scientific profile gates are not applicable for the scope stated in slot 1.

## 7. Findings

No new material target finding was identified. The narrow historical-input discovery changes the required native adapter and acceptance cases, without changing the target verdict.

The predecessor's WCL-F01–F05 remain observations about the earlier design and implementation. Their current dispositions stay at Plan 32; acceptance of this target does not resolve them. The present parser duplication, implicit asset copying and filename-based compaction remain implementation work until replaced and exercised.

Preserve the target's explicit constraints during implementation: substantive plan-authoring content and examples, concurrent-plan freedom, native status ownership, immutable source bytes, protected unknown artifacts, provenance/reference lifetimes and existing exact cleanup defenses. A general repair fallback, adaptation selected only by self-declared status, or rewriting immutable records would exceed the accepted scalar-adapter boundary.

## 8. Library fit and ownership cost

`ruamel.yaml==0.19.1` and `markdown-it-py==4.2.0` are appropriate proposed generic foundations. They belong to the exact locked docs group; repository-specific lifecycle interpretation belongs in the tooling adapter.

Interface inspection through current primary documentation supports the YAML instance/safe/duplicate-key boundary and CommonMark token parsing with the table rule. See the [ruamel.yaml API](https://yaml.dev/doc/ruamel.yaml/api/) and [markdown-it-py usage](https://markdown-it-py.readthedocs.io/en/latest/using.html). This is not executed qualification of the selected releases.

Libraries do not supply native status meaning, table-binding uniqueness, strict selected-row arity, reference-base interpretation, historical non-YAML literal meaning or artifact eligibility. Those are legitimate application contracts. The proposed adapter must retain them without implementing another general YAML or Markdown grammar.

## 9. Alternatives and tradeoffs

Keeping the existing limited parsers and filename cleanup is initially simpler, but preserves divergent interpretation and artifact-loss risk. A central content service or editable aggregate backlog introduces synchronization and another authority without serving a demonstrated need.

The selected design is the simplest viable alternative for the required aggregate discovery and lifecycle behavior: shared library-backed interpretation, sparse bindings, explicit publication inputs and existing producer-owned disposal. Explicit bindings cost maintenance when native table shapes change, but make ambiguity visible instead of inferring meaning from prose.

For immutable ADR syntax, one-time source normalization would remove adaptation code but change protected bytes and require a governance exception. The closed preparse adapter preserves both the original source and the single-reader route at modest bounded cost. It is preferable here. Correcting the mutable template directly prevents perpetuating the old encoding. Retaining a general legacy parser would carry substantially more interpretation machinery than this gap requires.

Revisit these choices if an actual new family or producer cannot preserve its native meaning or protection through these owners. No speculative service or permanent indexing mechanism is warranted now.

## 10. Verification

**Interface-checked:** the support paths and contracts named in slots 1–3 were inspected. `scripts/docs.py::stage` demonstrates implicit collection asset copying; the predecessor `scripts/adr.py::split_front_matter` and `scripts/docs.py::metadata` demonstrate separate parsing; `scripts/test_resources.py::finish_report` demonstrates filename/suffix classification. The existing resource owner supplies reusable borrow, reference, reservation and no-follow identity checks.

The narrow reopening inspected the initial shared reader, affected live frontmatter, old scalar interpretation and the coordinator's bounded invalid-input inventory. The inventory records 22 immutable ADRs/37 scalar spans plus the mutable template's one span; these are inspected input evidence, not a test run performed by this reviewer. Actual legacy owner fields beginning `docs/plans/` establish their repository-relative representation.

**Proposed:** the replacement target and Plan 32 acceptance cases. Its native-corpus/body-byte, strict binding, docs-environment isolation, publication recovery and exact resource-lifecycle controls address the material failure classes. The immutable adapter additionally needs revealing checks of exact old literal values and unchanged prefix/body bytes; undeclared/new/mutable and changed-span rejection through the strict route; duplicate/merge/cycle rejection despite adaptation; and historical-baseline reads using the same canonical identity. These belong to WC02's existing native-corpus and failure acceptance, not a new gate.

**not_run:** tests, product checks, environment synchronization, publication and disposal by this reviewer. No Tested or Measured implementation claim is made. Execution results, against the zero-failure baseline, belong to Plan 32's targeted work and final scoped acceptance.

## 11. Rule impacts and disposition

There are no additional rule changes beyond the maintainer-confirmed predecessor impacts:

| Rule impact | Target adoption and route |
|---|---|
| [WCL-RC01](design_review_workspace-content-lifecycle_2026-10-09.md#rc01) | Shared lifecycle interpretation/defaults and native-derived scope; ADR-0168 and blueprint §24.4.1, followed by WC02 |
| [WCL-RC02](design_review_workspace-content-lifecycle_2026-10-09.md#rc02) | Explicit asset delivery independent of retention; publication owner/configuration and WC03 |
| [WCL-RC03](design_review_workspace-content-lifecycle_2026-10-09.md#rc03) | Focused runtime routes and lifecycle duties alongside preserved substantive authoring guidance; WC07 |

The maintainer has already selected these directions and the pre-28 removal correction. The bounded scalar adapter preserves the existing immutability rule and original native meaning; it requires no new immutable-write exception. The reference-base correction preserves observed native representation rather than changing ownership. This review requests no further approval or new gate. ADR status adoption remains separate from this architectural judgment.

Plan 32 owns implementation, predecessor finding dispositions and acceptance. Plan 28 and other concurrent plans retain their existing scope and qualification owners.

## 12. Decision

**Behavioral/semantic adequacy:** accepted as specified target contracts, including the closed historical scalar adapter and corrected reference bases; implementation behavior is unqualified.

**Architectural fitness:** accepted for the scoped tooling/document lifecycle target at Proposed strength.

**Overall decision: Accept.** No new material findings or additional rule impacts prevent dependent implementation. The strongest evidence is the explicit ownership/failure design grounded in the inspected support mechanisms and the bounded historical-input evidence. Runtime correctness, corpus compatibility, publication behavior and disposal safety require the execution evidence already owned by Plan 32.

The predecessor review retains its original Revise judgment. This target review neither imports obsolete work into Plan 28/32 nor changes substantive plan-authoring guidance, scientific requirements, immutable ADR bytes or the freedom to execute multiple active plans concurrently.
