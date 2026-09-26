---
title: Lightweight architectural documentation and search
status: done
date: 2026-09-25
adrs: [ADR-0095]
phase: 1
review_sources: [docs/design_review/reviews/design_review_documentation-system_2026-09-25.md]
scenario_sources: []
---

# Lightweight architectural documentation and search

## Context

Implement the approved mdBook/Pagefind design with generated publication navigation,
focused reading paths, movable section identities and an incremental blueprint split.
Plain Markdown remains canonical. This plan owns documentation/tooling only; concurrent
product work, native environment repair and design-review pilots remain outside scope.

## Decisions

ADR-0095 records the target and intended replacement of ADR-0033/0036. Formal decision
adoption and remote publication remain separate from local implementation. No source-proof
manifest, semantic code inventory, automatic architectural verdict or historical requalification
is introduced. The user selected incremental extraction: §§0.1, 0.3, 0.4 and 24.4 now;
subsystem sections move only when substantive work next changes those subsystems.

## Architectural drivers and scenarios

| ID | Change / expected behavior |
|---|---|
| S1 | Add a page inside a collection: discover it without a second navigation edit |
| S2 | Move a numbered section: preserve citations and old browser links with one normative owner |
| S3 | Search current guidance: history stays available through an explicit scope switch |
| S4 | Change an implementation body: no documentation proof inventory needs updating |
| S5 | Build docs without the product environment: use standard-library Python and static binaries |
| S6 | Fail a rebuild: retain the last successfully published local artifact |

## Plan

This table owns implementation progress. Packets depend on their predecessor.

| Packet | Responsibility / acceptance | Replaced mechanism | Status |
|---|---|---|---|
| P00 | Proposed decision and scoped review, S1–S6 | Single-file/manual-navigation policy through formal decision route | complete; decision adoption remains separate |
| P01 | Collection discovery, titles, staging, generated SUMMARY, S1/S5/S6 | Hand-maintained SUMMARY and ADR SUMMARY writer | complete |
| P02 | Pagefind body annotations, scopes and accessible component UI, S3 | mdBook built-in search | complete |
| P03 | Task-oriented entry pages and current-work routing, S4 | Duplicated live-status summaries | complete |
| P04 | Shared section resolver and bounded extraction, S2 | Single-file citation assumption and duplicated moved prose | complete |
| P05 | Skills, shared rules, canonical/generated roles and templates, S4 | Active manual-navigation/semantic-proof requirements | complete |
| P06 | Shared tool pins, bootstrap/doctor, CI and qualification, S5/S6 | Direct mdBook publication path | complete |

## Implementation contracts

`docs/site.toml` owns collection roots/order, exclusions, current-work selection and tool pins.
The adapter stages under ignored `build/docs/`, preserving original relative source paths and
edit URLs. Successful builds replace `docs/book/` completely. Failures preserve its prior contents.
The existing ADR README remains generated; SUMMARY exists only in staging. Titles use scalar
front matter, then H1; no metadata backfill is required. Optional rustdoc is copied when present.

Search defaults to Current. Reference includes the remaining mixed blueprint; current entry
pages link its applicable contracts directly. History preserves reviews and previous execution
records. Everything removes the scope restriction. Scope is a reading purpose, not a claim of
implementation. The UI uses Pagefind components and small integration code; no server service,
Node application, custom ranking or custom watcher is required.

`just docs` builds HTML and search; `just docs-test` exercises fixtures; `just docs-serve`
builds and serves (rebuild with `just docs` after edits); `just bootstrap-docs` installs pins.
The CI link check establishes link integrity separately from successful rendering.

The existing blueprint and `authoritative_design/sections/` own section identifiers through
numbered headings. Vacated locations keep explicit old anchors and link stubs, excluded from
ownership discovery. New section files do not duplicate the blueprint revision history.

## Finding dispositions

The scoped review links here. No product findings or pilots are inherited.

## Verification

**Tested:** local documentation/tooling qualification on 2026-09-25, with Python standard
library, mdBook 0.5.4 and Pagefind 1.5.2. Baseline: zero errors and warnings in this scope.
The stale product Python/native environment was not repaired or required by these checks.

| Command / check | Conditions and scope | Result against zero baseline |
|---|---|---|
| `just docs-test` | Disposable repositories; discovery, title fallback, section moves/duplicate ownership, source preservation, version errors, real rendering/indexing, deleted pages, optional rustdoc and failed-index recovery | 8 tests passed; 0 failures |
| `just setup-test` | Bare setup/guard/configuration fixtures, including the seven pure publisher tests | 91 tests passed; 0 failures |
| `just docs` | Whole book, no product build or optional rustdoc required | 268 chapters indexed; 0 errors; 0 warnings |
| `lychee --offline --no-progress --include-fragments --exclude-path docs/book/404.html docs/book` | Whole published site; the existing site-root 404 exception is unchanged | 0 errors; external URLs remain outside this gate |
| `just adr-lint` | Record fields, generated source index and register | 3 checks passed; 0 failures |
| `just lint-agents` | Shared rules, aliases, canonical/generated roles and hooks | 0 failures |
| `.venv/bin/ruff check scripts/adr.py scripts/docs.py scripts/doctor.py scripts/tests/test_docs.py scripts/tests/docs_integration.py` | Changed Python implementation and fixtures | 0 findings |
| `actionlint .github/workflows/docs.yml`; `shellcheck scripts/bootstrap.sh`; `.venv/bin/taplo fmt --check docs/book.toml docs/site.toml`; `uvx zizmor --offline .github/workflows/docs.yml` | Changed workflow, shell and TOML | 0 findings; existing workflow suppressions retained |
| `just lint-license`; `.venv/bin/typos` on documentation and changed tooling/instructions; `git diff --check` | License coverage, task-scope spelling and diff whitespace | 0 findings in this task's scope |
| Skill-creator `quick_validate.py`, portable metadata projection | ADR and design-review skills; existing runtime-extension fields retained in sources, removed only from disposable validator inputs | 2 checks passed; 0 failures |
| `just docs-serve`, local HTTP request | Same build path followed by Pagefind preview | Site served successfully on port 1414; HTTP 200 |
| Chrome browser scenarios | Local root and `/pse-arrow/` prefix; Current/Reference/History/Everything, result anchors, Escape/Ctrl+K and light/dark appearance | All exercised paths passed; no browser warning/error logs |

**Tested:** the repository-wide `just lint-typos` invocation reported two findings in
concurrently edited `crates/pse-catalog/src/delta/contract.rs` (SQL constraint-token spelling). Those product edits
were preserved. This is a scoped documentation closure, not a claim that the whole shared
working tree meets its zero baseline. No Rust, solver, Python-product or scientific campaign
was run; those obligations remain with the product plan.

**Measured:** one cold-origin Chrome search for `AP-06` at
`http://localhost:3001/pse-arrow/` returned the three Current results in 1.118 seconds,
including browser automation overhead. Local server request logs and requested file sizes
showed 445,544 bytes of Pagefind assets through that query (including UI CSS/JavaScript;
filesystem sizes, not compressed transfer measurements). The initial page visit requested
only the component UI assets; search engine/index data loaded when the dialog opened.
This is a local observation, not a performance gate or deployment benchmark.

**Proposed:** reduced reading and maintenance cost under future design changes. The fixtures
establish mechanical publishing behavior; the author review is explicitly not an independent
review, a process pilot or proof of architectural quality.

## Open items

No local implementation item remains. ADR-0095 remains proposed: formal adoption and symmetric
supersession of ADR-0033/0036 belong to the decision PR. Remote CI/deployment and process pilots
were not performed. Plan 17 links this closure while preserving its original warning observation.

## Outcome (recorded after implementation)

### What was built

**Implemented:** a standard-library publisher derives navigation and the section directory,
stages immutable source copies and replaces the local artifact only after rendering/indexing
succeeds. Pagefind provides scoped, on-demand search. The committed SUMMARY and its ADR writer,
built-in mdBook search index and warning-threshold configuration are removed.

**Implemented:** task-oriented entry pages, explicit current-work selection, shared section
resolution and the bounded §§0.1/0.3/0.4/24.4 extraction preserve one normative owner and old
browser anchors. The authorized blueprint edit used `PSE_DESIGN_EDIT=1` and revision 54;
subsystem contracts and accepted ADR bodies were preserved. Skills, shared rules, contributor
guidance, the root README, canonical/generated roles, bootstrap/doctor, recipes and CI use the same publication
and bounded-context model. No source-proof inventory or new product-change gate was added.

**Tested:** the scoped checks above pass. The full book builds without the earlier search-size
warning and retains historical material in searchable collections. Behavioral/product and
process-effectiveness claims remain outside this result.

### A mistake made and corrected

**Implemented:** the first search integration initialized its default filter during page load,
which fetched search data before any search interaction. Moving that initialization to dialog
opening restored on-demand loading; server request logs confirmed the change. Initial flat
navigation also exposed the entire history at once; generated collection landing pages now
group chapters into collapsed branches. Visual checking corrected the scope label's contrast
in dark mode.

### Deviations from the plan, deliberate

The expanded whole-site link check exposed stale relative and workstation-local links in
12 existing reference/history documents. They were corrected to published relative targets,
existing repository files or verified historical Git blobs; local-only references are explicitly
labelled as source paths. Historical observations and verdicts were preserved. No permanent
link-rewriting layer or new checker exclusion was added.

Real-tool integration fixtures live separately from bare setup discovery, so `just setup-test`
does not acquire a documentation-binary prerequisite. Tool pins still have one owner in
`docs/site.toml`, including the test fixtures. No wholesale blueprint migration, new site
framework, process pilot or production proof mechanism was introduced.
