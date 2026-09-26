---
title: Architecture-first design review
status: done
date: 2026-09-25
adrs: [ADR-0094]
phase: 1
review_sources: [docs/design_review/reviews/design_review_architecture-first-standard_2026-09-25.md]
scenario_sources: []
---

# Architecture-first design review

## Context

Implement the maintainer-approved six-foundation review proposal. This plan owns the
process rollout; Plan 16 retains product implementation and qualification. The maintainer
reserved pilots for separate follow-up. No runtime refactor, dependency upgrade, product test
campaign or historical requalification is implied. Workflow and document delivery is complete;
[Plan 18](18-architectural-documentation.md#verification) subsequently resolved the documentation
search warning. The original observations below remain evidence of this rollout's first checks.

## Decisions

[ADR-0094](../adr/0094-architecture-first-design-review.md) records the governance change.
The [standard declaration](../design_review/design_principles/standard.toml) selects the
current core, profile and binding. Existing accepted records and historical reviews stand.

## Plan

This table owns rollout status. Each packet depends on its predecessor.

| Packet | Responsibility | Acceptance | Status |
|---|---|---|---|
| P01 | Core principles and review template | Six foundations, consequential architecture verdict, retained correctness obligations and version mapping | complete |
| P02 | Skills, profiles, roles and binding | Architecture questions lead; detailed mechanisms remain conditional; generated roles agree | complete |
| P03 | Decision and plan tracking | One disposition owner, concrete scenario links, current-status index links, governance amendment | complete |
| P04 | Governance document and applicable final checks | Document consistency, skill validation, setup controls and ADR/agent/document checks | complete; subsequent documentation closure owned by Plan 18 |

Delete superseded active wording that excludes architectural consequences, makes library
integration costs unassessable, or treats specifications as inherently competing authority.
Do not delete historical reviews or obsolete principles referenced by accepted records.

## Verification

**Implemented:** Core 3.0, process-simulator profile 1.1, the review/ADR skills, canonical and
generated roles, plan/ADR/PR/issue scaffolding and governance references agree on the new
method. The governance document review assesses that specification, not process effectiveness.

**Tested:** the following checks ran locally on 2026-09-25. The required failure/warning
baseline is zero; a successful exit does not erase the reported book warning.

| Command / check | Conditions and scope | Result against zero baseline |
|---|---|---|
| `just adr-lint` | ADR front matter, generated index and deferred register; ADR-0094 exercises AP references | 3 checks passed; 0 failures |
| `just lint-agents` | Shared instructions, references, aliases, generated native roles and hooks | 0 failures |
| `just setup-test` | Existing disposable setup/guard/configuration fixtures; no product extension imported | 84 tests passed; 0 failures |
| `just lint-typos` | Configured repository spelling scope | 0 findings |
| `.venv/bin/ruff check scripts/adr.py` | Sole changed Python logic: admit AP identifiers | 0 findings |
| `git diff --check` | Working diff whitespace | 0 findings |
| `just docs` | Whole mdBook HTML build | Exit 0; 0 build errors; **1 warning: search index is very large** |
| `lychee --offline --no-progress --include-fragments` | 49 inputs: changed root/skill guidance, PR template, updated book pages and added navigation targets | 1,612 links examined; 0 errors; 108 excluded by offline scope |
| Focused configuration/scaffold assertions via `.venv/bin/python` | TOML paths/version agreement; duplicate-rejecting issue YAML/unique field IDs; actual `just plan scaffold-check` recipe in a disposable directory | 3 checks passed; 0 failures |
| Skill-creator `quick_validate.py`, portable metadata projection | Three changed skills; retain and separately check existing shared-runtime `user-invocable` and `model-baseline` metadata | 3 portable-schema checks passed; 0 failures |

The stock skill validator rejects those two existing runtime-extension keys when run directly;
the portable-schema check strips them only in temporary copies. It does not establish native
runtime execution or skill effectiveness. `just agent-config-sync` and `just adr-index`
regenerated their owned files successfully. Static checks establish syntax, references and
generation consistency; they cannot establish architectural quality or product correctness.

**Proposed:** improvements in change locality, reviewer judgment and practical review cost.
Process effectiveness remains unpiloted; the maintainer will assess it separately, and it is
not a completion gate for this plan. No product compilation, solver or performance run is
needed for this documentation/governance scope. The startup report's stale native environment
was not repaired or used as product evidence.

## Finding dispositions

This table owns findings raised by the governance document review; source reviews remain
observations. Each row links the scenario, decision or work owner, and closure evidence.
No product pilot or product finding disposition is part of this rollout. Workflow and document
delivery is complete; Plan 18 owns the subsequent zero-warning documentation result.

| Finding | Scenario | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#f01) | [S01](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#s01) | resolved | ADR-0094; P01/P02 | Core §2/§A, template decision/finding rules and review skill |
| [F02](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#f02) | [S02](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#s02) | resolved | P02 | Profile 1.1 review additions and skill/reference slot routing |
| [F03](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#f03) | [S03](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#s03) | resolved | P03 | Index links status owners; contributor/PR/issue evidence guidance matches core §D |
| [F04](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#f04) | [S04](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#s04) | resolved | P01/P02 | Core DP-16/§F and template slot 8 assess integration cost without restricting eligibility |

## Open items

ADR-0094 remains proposed until its decision PR is accepted. Local implementation and review
are authorized; this plan does not manufacture an approval or publish changes remotely.

The original book build exceeded mdBook's search-index warning threshold. That issue is now
resolved by [Plan 18](18-architectural-documentation.md#verification): scoped Pagefind search
replaces the oversized index while retaining useful content. The original warning row remains
unchanged above. Neither plan claims whole-product qualification or pilot effectiveness.

## Outcome (recorded after implementation)

### What was built

**Implemented:** six architectural foundations organize the standard; G9 makes architectural
fitness consequential independently of behavioral/scientific adequacy. Skills, profile slots,
roles and review examples now start from responsibilities, consumed contracts and change
scenarios. Library review assesses integration ownership, test isolation and replacement cost
while retaining full eligibility and deliberate internal contract evolution.

**Implemented:** one plan-owned finding disposition links scenarios, decisions, packets and
evidence. ADR and plan scaffolds, contributor/governance guidance, issue/PR forms and blueprint
§24.4 route that method. Historical records retain their original evidence. Missing navigation
entries for referenced contracts, packets and their supporting documents are now in the book.

**Tested:** the scoped checks in Verification passed with zero failures; the original whole-book
build reported the search-index warning. Plan 18 records its subsequent resolution. These results
do not qualify the product or demonstrate the effectiveness of the review process.

### A mistake made and corrected

**Implemented:** reordering the core template initially left companion slot references using
the prior positions. The profile, companion skill and reference now route physical contracts
to slot 3, scenarios to slot 4 and mechanisms to slot 5. The link check also showed that a
successful book build alone did not prove navigation: referenced execution documents lacked
book entries, and the contributor release link used an obsolete anchor. Those links and
navigation entries were corrected; the scoped offline fragment check now reports zero errors.

### Deviations from the plan, deliberate

The maintainer removed pilot execution from this task. Pilots are separate maintainer work;
no pilot results or product architecture acceptance are claimed. No new tracking service,
architecture score, product refactor or dependency was introduced. The search warning was not
accepted as a baseline; the separately scoped Plan 18 owns its resolution.
