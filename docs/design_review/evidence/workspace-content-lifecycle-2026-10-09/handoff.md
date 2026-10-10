---
title: Workspace content lifecycle implementation and design handoff
date: 2026-10-09
doc_role: evidence
doc_owner: docs/plans/32-workspace-content-lifecycle.md
doc_provenance: authored
doc_topics: [documentation, storage, environment]
doc_retention: while-dependent
doc_retirement_trigger: owner-release
doc_retention_reason: Resume context for unfinished Plan 32 and its reopened structured-document design.
---

# Workspace content lifecycle handoff

Work is paused at the maintainer's request so it can be resumed later. The existing
implementation is substantial, but **Plan 32 is not complete**. Its document representation
needs reconsideration after the maintainer identified split authoring between Markdown and
external metadata, inadequate treatment of design reviews, and the need to express the
existing skills' content guidance in the structured model.

This is a resume snapshot, not a new plan, approval, design review or disposition ledger.
[Plan 32](../../../plans/32-workspace-content-lifecycle.md) remains the work/status owner.
No conversion to a new structured document format has been implemented or selected.

## Request and boundaries to preserve

The original request was a comprehensive review and better management of repository
content and workspace storage, excluding production/test source bodies and `.venv*`.
It includes relevance, intended use, generation/provenance, content structure, tagging,
discovery, agent instructions/skills, publication, retention, archival/removal and
producer-owned output management. Necessary exact reference searches in excluded source
trees were subsequently authorized for retirement safety; they were not a product review.

The maintainer's clarifications are binding:

- Preserve the carefully reviewed **substantive content directions**: how plans explain the
  intended change, implementation, alternatives, assessment, verification and completion.
  Representation may change; the reasoning must not be reduced to tags or a checklist.
- The skills already give general structure. The structured approach must capture that
  guidance, including design reviews, while retaining its permitted flexibility.
- Multiple plans can be active and execute concurrently. Reconcile scope at the content
  level, with one owner per status fact and an aggregate view. There is no one-plan rule.
- Plan 28 keeps its own product/scientific implementation and qualification scope. Do not
  fold it into Plan 32 or interpret content management as scope consolidation.
- Remove all pre-28 plans and companions without salvage, metadata backfill, old-item
  reconciliation or scope import. In particular, do not infer Plan 27 work belongs in 28.
  Only required retained references/tool consumers need repair.
- Keep concurrent uncommitted work. Do not reset, clean, overwrite or stage unrelated work.
- Focus verification on this documentation/workspace system. The maintainer explicitly
  stopped broad codebase hygiene work while other implementation is active.
- Completion or a metadata tag is not authority to delete evidence, caches or persistent
  state. There is no universal prune command or required deletion quota.

## Where the work stands

The implementation baseline is dirty `main` at
`4c24721e691187e1a5b28398b29722fbde671da8`, verified again for this handoff.
This work has not been committed or pushed. The working tree also contains extensive
independent Plan 28/33 and product changes. Reinspect it when resuming; this snapshot is
not a promise that shared files have remained unchanged.

| Record | Role and current limitation |
|---|---|
| [Original workspace review](../../reviews/design_review_workspace-content-lifecycle_2026-10-09.md) | Dated inventory, findings WCL-F01–F05 and proposed RC01–RC03; predecessor verdict remains Revise |
| [Plan 32](../../../plans/32-workspace-content-lifecycle.md) | Existing target, packets WC00–WC08, finding dispositions and bounded resource pass; still in progress |
| [Scoped target assessment](../../reviews/design_review_workspace-content-lifecycle-target_2026-10-09.md) | Accepted the earlier target at Proposed design strength; does not assess the newly discussed structured-document correction |
| [ADR-0168](../../../adr/0168-workspace-content-lifecycle.md) | Proposed metadata/lifecycle decision; not accepted and not implementation acceptance |
| Blueprint §24.4.1 | Enduring contract added through the visible design-edit route, with revision 140; reflects the earlier target |
| This handoff | Records implementation, limitations and later discussion for resumption; does not replace any of those owners |

RC01 shared interpretation, RC02 explicit asset delivery and RC03 instruction routes were
confirmed before implementation. That confirmation does not settle every newly raised
representation choice. The substantive plan-authoring guidance was preserved: changes to
`create-plan`, `execute-plan` and `plan-execution` were additive lifecycle/routing paragraphs,
with the original content retained as an unchanged prefix. The ADR skill's command examples
were also routed through the docs environment.

The plan table currently records WC00/04/05/06/07 Complete, WC01/02/03 In progress and
WC08 Planned. These are the pre-reconsideration packet states, not acceptance of the new
target. The document implementation and independent correctness corrections are present;
final closure was interrupted before its Outcome and dispositions were completed.

## What has actually been implemented

### Shared metadata interpretation and aggregate plan queries

**Implemented:** `scripts/document_metadata.py` replaces separate publisher/ADR metadata
parsers with one strict, library-backed reader. It preserves original front matter/body
bytes, native fields and date meaning. It rejects duplicate keys, unsafe tags, merge/cycle
ambiguity, invalid controlled fields, conflicting ownership/defaults and invalid references.

The docs group pins `ruamel.yaml==0.19.1` and `markdown-it-py==4.2.0` in `pyproject.toml`;
`uv.lock` was updated narrowly. No wholesale dependency upgrade was performed.

**Implemented, but now challenged as the authoring target:** `docs/lifecycle.toml` contains
vocabulary, collection/bundle defaults, exact-file exceptions, native table selectors,
status interpretation profiles and sparse cross-plan relationships. `scripts/document_lifecycle.py`
provides read-only, versioned JSON inventory, validation, aggregate scope and retirement
preflight. Its active-plan bindings cover retained Plan 28+ content, including companions.
It reads native table cells rather than maintaining a second editable status backlog.
Missing state remains unknown, source/raw status is retained, and item identities are
qualified by document, binding and native row.

However, authors may have to edit both Markdown and external selectors/relationships after
a presentation or ownership change. This is the split semantic authoring the maintainer
objected to. Also, interpreting status prose with regexes is weaker than an explicitly
typed state. A focused reviewer illustrated the risk with “Complete; no pending acceptance”:
a lexical `pending` match can classify that sentence as open despite its meaning. This was
design advice, not a newly executed regression test.

An explicit, closed compatibility boundary handles 22 immutable ADRs with 37 historically
invalid YAML scalar encodings. Exact path/key/raw-scalar digests identify the spans;
normalization occurs in memory before the same strict parse. Original immutable source
syntax and literal meaning remain intact. This is not an error-triggered fallback parser.
The mutable ADR template was corrected directly. Historical reference relocation is a
separate, existing allowed operation; accepted ADR content was not generally rewritten.

### Independent documentation environment and publication

**Implemented:** `scripts/pse_env.py --docs` selects `.venv-docs` distinctly from the product
environment, rejects aliases and visibly replaces an inherited product selector. Ordinary
reads do not synchronize packages, install the project, compile the extension or access the
network. Missing provisioning fails with an actionable instruction before a Bash caller can
fall through to a product interpreter. Rendered `--docs --print` exports preserve docs-first
PATH precedence and are idempotent under repeated evaluation.

Explicit bootstrap uses:

```bash
uv sync --locked --only-group docs --no-install-project
```

Recipe, bootstrap, doctor, ADR and manual CI consumers were migrated. Cold provisioning paths
keep parser imports lazy. The docs group is also included in normal dev tooling. Windows CI
uses explicit docs-only child environment selection; this is not qualification of the Linux
resource wrapper on Windows.

**Implemented:** `scripts/docs.py` and `docs/site.toml` stage explicitly selected assets
instead of copying all non-Markdown files from retained collections. Named bundle definitions
currently reside in `docs/lifecycle.toml`. Source/destination containment checks reject escaping
paths. Failed staging/render/index/replacement preserves the preceding successful site;
successful replacement drops obsolete delivery while preserving original evidence.

Current work is explicitly selected in `docs/site.toml`: Plan 28 and its companions, Plan 32,
and independent Plan 33. Two historical PostgreSQL conversational notes were removed from
Current discovery and classified with their retained ADR rationale owners. These selections
do not authorize execution or infer completion from document status.

### Producer-owned artifacts and storage observation

**Implemented:** `scripts/test_resources.py`, `scripts/validation.py`,
`scripts/validation_receipts.py` and `scripts/validation_result.py` use producer-declared,
sealed artifact meaning in the existing resource ledger. Roles distinguish receipts,
required provenance, consumed evidence, scratch and unknowns. Retained digest/identity and
required provenance are checked before scratch deletion or reusable reference exposure.

Complete successful outcomes, coverage, unchanged source and valid provenance are required
for automatic eligibility. Failed, incomplete, drifted, undeclared and manifest-less legacy
records remain protected. No current real producer output was invented as scratch merely to
produce savings. Historical reports remain displayable under temporary borrows; unregistered
or manifest-less reports are refused as new reusable origins. Existing references, borrowers,
reservations, no-follow traversal, identity checks and actual process drain remain authoritative.
Affected already-loaded old producers/reclaimers must drain before lifecycle cutover.

**Implemented:** `scripts/build_storage.py` now exposes versioned JSON observations for
configured stores and the four selected legacy cohorts. It distinguishes apparent/allocated
bytes, shared ownership, unknown active use, partial/unavailable observations and unknown
reclaimability. It deduplicates hardlinks within a row, does not follow nested symlinks and
uses descriptor traversal with inode checks against replacement races. Rows overlap and
must not be summed. Persistent state/shared skills retain their real owners. The recipe
forwards observer arguments, including `--metadata-only`. There is no disposal operation here.

### Retirement, public routes and instructions

**Implemented:** 19 tracked pre-28 plans/companions and three completed Plan 25b reviews were
removed: 22 files, 915,238 logical source bytes. No old scientific scope or Outcome content
was transferred. Required historical references point to the same original path at an
immutable Git commit; remaining tool consumers were checked. The retirement worker checked
the 22 historical Git objects, 267 repaired immutable links and normalized reference-only
changes in 44 ADRs. These bounded checks do not certify scientific implementation.

Stale public facts in `README.md`, `CONTRIBUTING.md`, `CITATION.cff` and the bug template were
repaired against current owners. Current-work/review indexes, `AGENTS.md`, the docs rule and
process skills route users to the system. Enduring usage is in the existing guides:
[documentation](../../../dev/documentation.md),
[agent environment](../../../dev/agent-environment.md),
[build/storage](../../../dev/build-performance.md) and
[validation](../../../dev/validation-assessment.md).

No automatic authored-document deletion, new archive tree, standing cleanup hook, central
content service or universal prune command was added.

## Storage findings and measurement limits

**Measured:** under the same eight retained publication collections, the prior implicit
non-Markdown selection comprised 1,051 files and 77,626,046 apparent bytes. Explicit selection
comprised 54 assets and 16,325,156 bytes, matching actual staging. Originals removed: zero.
This is a delivery-selection comparison, not a measured reduction in total workspace storage.

The initial `docs/book/` observation was 55,193,327 apparent bytes and 56,995,840 allocated
bytes. A final comparable book-size observation was not completed. Do not manufacture a
before/after site-storage claim from the asset comparison.

The bounded legacy resource pass found no eligible subset and deleted **zero legacy output
bytes**. Plan 32 records the exact observed directory identities and these allocations:

| Cohort | Observed allocation | Remaining owner/trigger |
|---|---:|---|
| `build/plan10/` | 2,197,307,392 bytes | Campaign owner must establish exact outcome, references and durable authored-input/provenance preservation |
| `build/plan11/` | 6,857,015,296 bytes | Same exact owner qualification; filenames/age do not release it |
| `build/assessment/` | 5,179,871,232 bytes | Assessment owner must qualify exact legacy attempts; 30 observed records had zero sealed manifests, including failed/incomplete and pinned attempts |
| `build/workspace-review-2026-10-09/` | 675,840 bytes | Plan/review handoff owner must preserve durable provenance and reconcile consumers before release |

These are non-atomic snapshots of a changing checkout, not current sizes or additive savings.
The original ignored review inventory remains in the final cohort as a retained input.
Compiler/native/shared caches and persistent scientific state were not swept.

Instruction observations compared source bytes under stated startup-route assumptions:
`AGENTS.md` grew from 22,333 to 22,616 bytes; `CLAUDE.md` remained 2,791 bytes. The three
process skills gained additive routing paragraphs while retaining their original prefixes.
No actual API prompt-token, agent-latency or productivity measurement was performed.

## Verification already performed

All pass/failure claims below use a **zero-failure baseline**. They describe the tested
snapshot, which predates later concurrent changes and this handoff. The new structured
representation discussed below has **not** been implemented or tested.

| Evidence | Command/mode and result |
|---|---|
| **Tested**, document package | `scripts/pse-env --docs --resource-class light -- python3 -m unittest scripts.tests.test_document_lifecycle scripts.tests.test_adr scripts.tests.test_docs scripts.tests.test_pse_env scripts.tests.test_recipe_arguments scripts.tests.test_setup.DecisionRecordTests -v`: 64 tests, zero failures |
| **Tested**, resource/receipt package | `scripts/pse-env --resource-class light -- python3 -m unittest scripts.tests.test_test_resources scripts.tests.test_validation scripts.tests.test_validation_result scripts.tests.test_execution_contracts -v`: 138 tests, zero failures |
| **Tested**, integrated docs fixtures | `scripts/pse-env --resource-class light -- just docs-test`: 26 tests, zero failures; includes real mdBook/Pagefind replacement/recovery controls |
| **Tested**, earlier assembled support snapshot | `scripts/pse-env --resource-class light -- just setup-test`: 583 tests, zero failures; not rerun after subsequent edits and concurrent work |
| **Tested**, actual corpus interpretation | `scripts/pse-env --docs --resource-class light -- python3 -m scripts.document_lifecycle validate`: zero errors; one snapshot had 286 discovered documents and 228 validated scope items |
| **Tested**, actual publication | `scripts/pse-env --resource-class light -- just docs`: successful build, 287 indexed chapters; corpus/build counts differ because independent documents were being added concurrently |
| **Tested**, actual search engine | Temporary Node 26.5.0 harness queried the actual generated Pagefind bundle over an owned local HTTP server; `workspace` returned Current/Reference/History results with matching filters, respectively 50/42/37 at that snapshot |
| **Tested**, changed-page links | `lychee --offline --no-progress --include-fragments` over the four changed developer guides, Plan 32, ADR-0168 and target review: 250 links, zero errors, 20 excluded |
| Broader link limitation | Same offline scan of the full book, excluding `404.html`: 289 errors in 15 other inputs; largely source-relative links resolving under `docs/`. No full-book link pass is claimed; these were not mass-repaired |
| Earlier environment check | `scripts/pse-env --resource-class light -- just ready`: both gates passed after docs dependency/environment wiring |
| Earlier turn-end check | `scripts/pse-env --resource-class light -- just turn-end`: ADR index and formatting passed; later changes mean this is historical scope evidence |
| Broader hygiene limitation | `just hygiene` was run before the maintainer stopped broad checks. It failed three gates: `lint-license`, `lint-py`, `typecheck`. No hygiene pass is claimed or needed to continue the focused document design |

The temporary design-edit marker caused the license finding and was removed after protected
edits; the named license rerun passed. In-scope lint/type fixes were made. A scoped Ruff check
of document/observer files passed; subsequent scoped formatting changed two files. The last
whole-project typecheck had one redundant cast in resource code, which was removed without
another broad run. Unrelated concurrent lint findings were left to their owners.

Independent implementation review reported no remaining material correctness findings for
the earlier target after corrections. These included no-follow observer traversal, strict
bound-table validation, reuse refusal, attachment containment, docs interpreter isolation
and PATH precedence. It did **not** review the later proposed document model.

Useful local evidence, if it still exists:

- `/tmp/pse-doc-lifecycle-focused.log`, `/tmp/pse-wc04-unit.log`
- `/tmp/pse-plan32-docs-test.log`, `/tmp/pse-plan32-setup-test.log`
- `/tmp/pse-plan32-lifecycle-validate.json`, `/tmp/pse-plan32-lifecycle-scope.json`
- `/tmp/pse-plan32-docs-build.log`, `/tmp/pse-plan32-search.mjs`, `/tmp/pse-plan32-search.log`
- `/tmp/pse-plan32-focused-links.log`, `/tmp/pse-plan32-offline-links.log`
- `/tmp/pse-plan32-ready.log`, `/tmp/pse-plan32-turn-end.log`, `/tmp/pse-plan32-hygiene.log`
- `/tmp/pse-workspace-storage-observation.json`, `/tmp/pse-plan32-storage-identities.json`
- `build/assessment/20261010T000655.935627Z-ready-3928092-a026a5/`
- `build/assessment/20261010T001415.015702Z-turn-end-4068062-7890f3/`
- `build/assessment/20261010T001449.006125Z-hygiene-4078619-c0104c/`

Temporary files are not a durable evidence archive. No background server from the search
check remains running; its owned exec session was stopped. No unrelated service was stopped.

## Later discussion: why the document target needs revision

The maintainer challenged the external tags/bindings paired with largely unchanged Markdown
as a dual system. Simply moving those selectors into front matter would still leave tools
reconstructing meaning from presentation tables. The proposed correction must unify the
**authored document**, not merely move its sidecar metadata into the same physical file.

The maintainer then asked whether plans should be structured documents, while preserving
content directions. They also identified the missing first-class role of design reviews:
the intended process has a preceding review, findings/recommendations, decisions where
required, a plan, execution evidence, enduring knowledge and lifecycle closure. Generic
classification of reviews does not implement that process.

Finally, the maintainer emphasized that the skills already define general content structure.
The structured approach must capture those directions. A model containing only IDs, statuses
and arbitrary prose blobs would leave too much of the actual process unexpressed.

### Research and focused advice so far

**Proposed, not selected:** a common document model with family-specific semantics, authored
once and supplying generated reading, navigation, aggregate scope and lifecycle views.
Two credible authoring formats were examined:

| Alternative | Fit and tradeoff |
|---|---|
| Structured Markdown with typed local blocks | Keeps full prose directly readable beside local metadata. Work packages, findings, judgments and relationships become explicit authored elements; summary tables become projections. Requires a parser/renderer extension and a semantic schema |
| Canonical YAML with ordered rich Markdown sections | Also provides one authority and strong structural validation. Long prose/code requires container indentation; readable Markdown becomes generated. Worth considering if direct data editing is preferable |
| Existing Markdown plus external bindings | Useful only as a possible bounded legacy boundary; weak as the enduring target because ordinary edits cross into external selectors and prose-status rules |

MyST supports directives with arguments/options and arbitrary Markdown bodies, and its plugin
API can parse those bodies into a document tree. Pandoc supports nested fenced Divs with IDs,
classes and attributes. These establish feasible structured-Markdown mechanisms; neither
provides this repository's plan/review semantics automatically.
[MyST syntax](https://mystmd.org/guide/syntax-overview),
[MyST directive bodies](https://mystmd.org/guide/plugins-ast),
[Pandoc Divs](https://pandoc.org/demo/example33/8.18-divs-and-spans.html).

YAML literal scalars retain multiline content, and JSON Schema can validate typed fields and
object shapes. Cross-document ownership/reference integrity and closure semantics would still
need repository-aware validation.
[YAML literal style](https://yaml.org/spec/1.2.2/#812-literal-style),
[JSON Schema objects](https://json-schema.org/understanding-json-schema/reference/object).

A fresh focused design adviser recommended structured Markdown for direct agent reading and
flexible reasoning, with typed blocks as the authority and generated tables as views. This
is useful advice, **not** a formal acceptance or an operator-selected syntax/library. No
prototype, new dependency, new schema or migration was created during this research.

### Capture the skills' structure without rewriting their content guidance

Use the existing [plan structure and examples](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/create-plan/references/plan-structure.md),
the [create-plan guidance](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/create-plan/SKILL.md)
and the [review template](../../design_principles/core/design-review-template.md) as the source
of the content model. Do not invent a new plan-writing method.

| Existing guidance | Meaning the structured approach must express |
|---|---|
| Purpose, scope and basis | Intended outcome, completion boundary, preceding review/parent, architecture owner, baseline and evidence limits |
| Current foundations | Relevant existing behavior, reusable components, deficiencies and conclusions from the focused assessment |
| Target and contracts | Intended operation, responsibility, inputs/outputs, interactions, guarantees, failure behavior and support limits |
| Choices and alternatives | Rationale, library/composition choices, assumptions, tradeoffs and reconsideration conditions |
| Execution sequence | Coherent work packages, exact prerequisite capability/readiness, owner and delivered completion boundary |
| Migration/adoption | Affected consumers, preservation, replacement/deletion, intermediate states and consequential recovery |
| Verification/acceptance | Claims, revealing scenarios, planned checks versus actual evidence, conditions and local/assembled/measurement boundaries |
| Disposition/current state | Source finding references, one current disposition owner, uncertainty, deferral triggers and next executable work |
| Outcome | What was built, a mistake corrected, deliberate deviations, enduring-meaning transfer and actual evidence |
| Review contract | Scope/baseline/standard, decomposition/contracts, scenarios, mechanisms, foundations/gates, findings, alternatives/library fit, verification, rule impacts/disposition and separate behavioral/architectural/overall judgments |

The existing guidance permits combining, expanding, renaming or omitting suggested plan
sections according to the subject; the same explanation need not appear multiple times.
Review slots are stable content references with flexible presentation. A structured model
should preserve ordered rich content and meaningful section roles without imposing new fixed
headings, counts or repetitive forms. Preserve existing required obligations, including the
Outcome, evidence vocabulary, review judgment and disposition rules. Schema validation can
check declared structure; it cannot judge whether reasoning is good or synthesize acceptance.

### Model the full document and workspace lifecycle

The common model needs family-specific meaning, not a universal task status:

| Family/content | Meaning and authority to retain |
|---|---|
| Design review and supporting inquiry | Reviewed scope, baseline, standard, scenarios, observations, diagnoses, proposed corrections, rule impacts and original verdict; principal versus supporting role |
| Finding disposition | Reference to the original finding, current disposition, responsible decision/work, evidence or accountable deferral; owned once by the designated plan/owner |
| Plan/coordinator/companion and package | Review basis, combined/local target, reasoning, dependencies, implementation/qualification obligations, current state and Outcome; concurrent plans remain independent |
| ADR/decision | Selected rule, authority, rationale, scope, evidence at decision time and supersession; existing immutability/status route remains distinct |
| Contract, instruction and reference | Enduring owned meaning, audience/task route, applicability and replacement/retention conditions |
| Evidence/bundle and generated output | Claim, source/producer, scope/baseline/conditions, origins, delivery selection and retention dependencies |
| Workspace resource reference | Link to the actual producer's artifact/cohort and identity; operational pins, references, borrowers, eligibility and drain remain with that owner |

Relations should explicitly connect a plan to its preceding review, dispositions to review
findings, work to decisions/findings, claims to evidence, dependencies to the specific consumed
scope, and completed content to its surviving owner/archive. Derive inverse navigation rather
than authoring both directions. Stable document/item IDs should survive heading/path changes.

Keep review verdict, ADR status, execution authorization, plan/package progress, evidence
maturity, qualification, publication scope and retention distinct. Resolving a finding must
not rewrite a historical review's Revise verdict to Accept. Implemented does not mean Tested,
Measured or fully qualified. A package can complete while another owner retains assembled
acceptance. Completing a document does not release its evidence resources.

This means the corrected work must include review-generation/review-planning skills and
consumers, not just plan skills. Shared skill content, accepted ADRs, generated outputs and
immutable historical material need their proper ownership/migration boundaries; do not
convert everything blindly because a common model exists.

## Resume from here

First read this handoff and Plan 32's checkpoint, then inspect the live tree. Do not begin by
rerunning broad hygiene or continuing the old external-binding migration.

1. Reconcile the intended scope as a comprehensive **document and workspace** system. Assess
   the implemented mechanisms individually; preserve useful parser/bootstrap/publication and
   producer-lifecycle work while correcting the split authored representation.
2. Develop the common model from the existing skills and references, including review content
   and its handoff to plans. Compare structured Markdown and canonical YAML on a representative
   existing plan **and its preceding review**, preserving full content, semantic distinctions
   and reading/editing behavior. Do not assume an illustrative syntax is already approved.
3. Revise Plan 32's target/packages and proposed decision as needed. Reopen the appropriate
   focused target assessment for the changed contract; the earlier review is evidence for
   its earlier target, not acceptance of this correction. Use the existing decision/design
   route before protected architecture edits. Do not mark ADR-0168 accepted locally.
4. Once the corrected direction is settled, migrate retained mutable Plan 28+ and review
   content through one authoritative interpretation. Remove replaced external selectors,
   duplicate semantic declarations and lexical status inference where replaced. Keep any
   justified immutable/legacy boundary explicit and bounded.
5. Verify targeted document journeys, cross-plan/review relationships, preserved skill content,
   publication/search and lifecycle ownership. Respect the maintainer's narrowed verification
   scope; current product work and broad hygiene remain outside this assignment.
6. Finish the Plan 32 Outcome/dispositions only against the corrected implemented scope.
   Transfer enduring meaning, remove completed work from Current discovery, and retire eligible
   plans/reviews after consumer reconciliation and durable Git archival. Do not delete these
   currently untracked handoff/decision/review bytes before they have a durable home.

The full-book link errors and protected legacy output cohorts remain visible limitations;
they do not authorize unrelated repairs or deletion. No additional implementation, cleanup,
format migration, commit or publication is authorized merely by this handoff's suggestions.
