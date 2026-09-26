# Architecture-first standard: governance document review

## 1. Scope, drivers and coverage

| Field | Scope |
|---|---|
| Subject | ADR-0094; Core 3.0 principles/template; process-simulator 1.1 routing; skills, roles and tracking guidance |
| Boundary | Review/governance documents and their configuration; no product architecture review |
| Standard | Core 2.0 retained obligations and the maintainer-approved Core 3.0 amendment; process-simulator scientific obligations preserved |
| Tier / purpose | Design / target, document consistency only |
| Reviewer / date | Codex, author review / 2026-09-25; not independent certification |
| Decision | Accept the workflow specification within this document scope; effectiveness remains unpiloted |
| Disposition owner | [Plan 17](../../plans/17-architecture-first-design-review.md#finding-dispositions) |

The maintainer approved the architectural direction and then explicitly reserved pilots for
separate work. This record checks the governance specification and its integration; it does
not apply the new process to a product extension or subsystem as a pilot. Proposed benefits
remain Proposed. Documents and configuration are inspected; check results live in Plan 17.

## 2. Decomposition, ownership and dependencies

The core owns foundations, refinements, evidence and review decisions. The template owns
review shape and follow-up fields. The simulator profile adds scientific obligations and
conditional detail; the binding owns repository routes and scenario seeds. Skills direct
application without duplicating the full standard. Canonical role instructions generate
native roles. Plans own current adopted-finding dispositions; execution packets own their
observations; indexes link these owners. ADRs retain decision-time rationale and support.

Dependencies remain core → profile → binding in applicability, with profiles referring only
to core and the binding referring to repository authorities. No new registry, service or
architecture-scoring framework is introduced.

## 3. Contracts, authority and constraints

`standard.toml` selects current versions; historical reviews retain the versions they used.
Core §J names changes from 2.0 rather than silently repurposing DP/G identifiers. AP-01–AP-06
and G9 add architectural obligations; existing G1–G8 and PS gate meanings remain applicable.
The ADR linter accepts AP alongside DP/PS and historical DM identifiers. ADR acceptance still
uses the existing decision-PR route; this author review does not accept an ADR on its own.

## 4. Document-change cases

These are document consistency cases, not executed process pilots or product scenarios.

| ID | Change to the workflow | Required document relationship |
|---|---|---|
| <a id="s01"></a>S01 | Make architecture consequential to review decisions | Foundations, finding consequences and G9 must agree across core, template and skills |
| <a id="s02"></a>S02 | Reorder the core review slots | Profile and reference slot numbers must follow; scientific obligations remain present |
| <a id="s03"></a>S03 | Link a finding to implementation work | Plans own disposition, packet evidence is linked, and indexes do not copy live progress |
| <a id="s04"></a>S04 | Assess a library integration or early internal API change | Full eligibility remains; integration cost is assessed; explicit evolution does not freeze immature APIs |

## 5. Mechanisms and execution

Existing Markdown/TOML declarations, ADR indexing, native-role generation and document building
remain the mechanisms. The sole executable logic change admits the AP identifier family to
existing ADR syntax validation. Plan scaffolding adds navigation/disposition fields without
another tracking system. No solver, cache, publication, physics or runtime behavior is changed.

## 6. Architectural assessment and gates

These verdicts assess the specified governance organization, not its effectiveness in use.

| Foundation | Document evidence | Verdict |
|---|---|---|
| AP-01 | Core, profile, binding, skill and plan responsibilities are separated | satisfied for specification |
| AP-02 | Versioned slots/identifiers and historical scope are explicit | satisfied for specification |
| AP-03 | Existing layers and tools compose; no new review service | satisfied for specification |
| AP-04 | Version selection and finding status have named owners | satisfied for specification |
| AP-05 | Decision rules, scope, evidence and disposition transitions are explicit | satisfied for specification |
| AP-06 | Change tier and conditional mechanism detail bound the reviewer context | satisfied for specification; practical cost unmeasured |

| Gate | Verdict | Evidence or scope reason |
|---|---|---|
| G1 | pass for specification | Version and status owners; no independently editable score |
| G2 | pass for specification | Core §J preserves versioned interpretation and separate claim strengths |
| G3 | not applicable | No product validity/enforcement behavior changed |
| G4 | not applicable | No effectful product operation changed |
| G5 | not applicable | No runtime consistency/recovery protocol changed |
| G6 | not applicable | No computation, transformation or reuse guarantee changed |
| G7 | pass for specification | No pilot, runtime or independent-certification claim |
| G8 | pass for specification | Existing tooling retained; no bespoke generic platform introduced |
| G9 | pass for specification | Six foundations independently affect decisions; S01–S04 documents agree |
| PS-G1 / PS-G2 / PS-G3 | not applicable | Scientific obligations preserved; no numerical/physical implementation assessed |

## 7. Findings

These findings describe the inspected baseline and integration risks addressed by this change.
Their current dispositions are owned by Plan 17; the table is not a live progress ledger.

| ID | Finding | Principles / gate / case | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | Core 2.0 underweights architectural change cost and local testing | AP-01/AP-03/AP-06; G9; S01 | DP-17 is a SHOULD with no gate; template consequence focuses on wrong/ambiguous/unrecoverable outcomes | Review can omit a costly coupling defect despite correct current outputs | Six foundations, architectural consequences and consequential G9 | Read core §2/§A and template decision/finding rules together |
| <a id="f02"></a>F02 | Slot reordering requires coordinated companion updates | AP-02/AP-05; S02 | Profile 1.0 routes physical semantics to slot 2 and stages to slot 4 | New decomposition/scenario sections receive the wrong content | Route profile 1.1 and skill/reference to Core 3.0 slots | Inspect all active slot references |
| <a id="f03"></a>F03 | Active status and evidence vocabulary diverge across workflow documents | AP-04; G1/G7; S03 | Plans index says Plan 16 has not started while its plan records implementation; contributor/PR forms still offer Observed | Readers cannot distinguish current status and supported claim strength reliably | Link status owners and align vocabulary with core §D | Read plan-index links and contributor/PR/issue guidance |
| <a id="f04"></a>F04 | Library eligibility wording obscures integration cost | AP-01/AP-02; G9; S04 | Core 2.0 DP-16 exempts adopting library capabilities from proportionality | Unnecessary integration lifecycle/coupling may escape review | Preserve eligibility while assessing integration ownership/cost in core §F and skill ledger | Compare core, library reviewer and template slot 8 |

## 8. Library fit and ownership cost

No dependency selection or API change is proposed. Existing ADR/configuration/document tools
serve the workflow. The revised ledger specifies integration ownership, exposed contracts,
lifecycle, test setup and upgrade cost; it does not claim any particular library is qualified.

## 9. Alternatives

Keeping Core 2.0 alone leaves F01/F04. A separate architecture score duplicates judgment and
status. Extending the existing layered standard provides the required distinctions with current
tooling. A pilot could assess effectiveness, but the maintainer explicitly excluded its execution
from this task. This document makes no inference about what such a pilot would find.

## 10. Verification

Document reading supports the scoped design assessment. Plan 17 records exact applicable
checks and outcomes. ADR/agent/document checks establish syntax, references and generation
consistency; they cannot establish reviewer effectiveness, change-locality improvement or
product correctness. No product tests or performance measurements are needed for this scope.

## 11. Authority changes, exceptions and disposition

ADR-0094 and blueprint §24.4 record the governance amendment. The blueprint revision explicitly
records that the workflow is implemented and effectiveness is unpiloted. No accepted ADR body
or historical review is changed. No SHOULD exception or unacknowledged MUST gap is used.
Finding dispositions and evidence links live in Plan 17. Formal decision-PR acceptance remains
separate from the maintainer-authorized local implementation.

## 12. Decision

**Accept the specified workflow within this document scope.** Behavioral/semantic adequacy
means the retained obligations and evidence rules remain coherent; architectural fitness
means the six foundations now influence the review and its decision. Neither judgment
certifies the process in use. The maintainer will pilot separately. Product qualification and
independent review are outside this record.
