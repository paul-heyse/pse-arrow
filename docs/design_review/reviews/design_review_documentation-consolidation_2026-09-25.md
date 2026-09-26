# Design review: documentation retention policy and ADR tooling (ADR-0096)

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | [ADR-0096](../../adr/0096-current-rationale-and-selective-retirement.md) (proposed) and its tooling: `scripts/adr.py` (`review_error`, `relocated_review`, `unrelocate_links`, `supersession_errors`, `lint_highest_issued`, `lint_immutability`), `scripts/check_register.py` (empty register valid), `DecisionRecordTests` in `scripts/tests/test_setup.py`. Neighbors read: `docs/adr/register.md`, the `plan` recipe, the governance workflow checkout depth. |
| Standard | Core 3.0 (AP-01–AP-06, DP-01, DP-22, DP-24; G5, G7, G9); binding pse-arrow. Process-simulator profile 1.1 not applied: no physical, numerical or solver meaning is in scope. |
| Tier / purpose | Change tier; conformance to the target the maintainer already selected in [Plan 19](../../plans/19-current-documentation-consolidation.md) *Decisions*. That target is not reopened. |
| Reviewer / date | Agent review requested by the maintainer's session, 2026-09-26; separate from the implementing edits, not an independent human review. |
| Decisions | Behavioral adequacy: **not adequate** (G5, G7). Architectural fitness: **fit, with one narrow AP-04 violation** (C3). Overall: **Revise** (§12). |
| Disposition owner | Plan 19 *Finding dispositions* (packet MD01). |

**Target (selected, not reassessed):** Git is the archive. There is no archive tree. Obsolete ADRs, plans and reviews retire once their surviving meaning has an owner. Numbering gaps are permitted. The highest issued record stays in the tree.
**Drivers:** a reader reaches current rationale without history (AP-01, AP-04). One owner per fact (DP-01). Retained records stay truthful and checkable (DP-22, DP-24). Checks stay local and cheap (AP-06).
**Not examined:** GOVERNANCE/CONTRIBUTING/skill wording beyond numbering statements; publisher/lychee behavior; blueprint §0/§24.4 content; any corpus retirement decision (MD02–MD04).

## 4. Change scenarios (from Plan 19)

| Scenario | Expected response | Observed path | Result |
|---|---|---|---|
| S2: why does a current choice exist? | The retained ADR answers it. Its evidence stays resolvable and unchanged. | Review field `git:<commit>:<path>` and body permalinks keep evidence reachable after retirement. The lint checks that a file exists at the named commit, not that it is the cited one. | Partly met (C1, C5, C6) |
| S5: remove an old ADR, plan or review | No retained link, supersession, register row or allocator loses required input. | Supersession to retired peers is tolerated. The register ADR column must name retained ADRs. The highest-issued check compares against the `origin/main` tree. | Met pre-merge. The number-reuse guard lapses after merge (C2, C3). |
| S7: close a future workstream | The completed plan retires without a cleanup campaign and no number is reused. | Plans are allocated as tree max + 1 by `just plan`. No check exists. The register uses a prose high-water mark. | Not enforced (C3) |

## 6. Architectural assessment and gates

| Foundation / gate | Evidence | Verdict |
|---|---|---|
| AP-01 Separation of concerns | Shape (`review_error`), identity (`lint_immutability`), allocation safety (`lint_highest_issued`) and pair consistency (`supersession_errors`) are separate small functions over one parser. | satisfied |
| AP-02 Stable contracts | The review grammar `git:<hex12-40>:<path>[#anchor]` is explicit and tested. The permalink form is implicit: any owner/repo, and only `../` relative links (C1, C6). | satisfied, with C1/C6 corrections |
| AP-03 Composition | stdlib only, with Git as the single history source. No second ledger. | satisfied |
| AP-04 Authoritative meaning | ADR Outcome 3 says register IDs follow "the same rule" (highest retained). `register.md` instead declares a hand-written high-water mark (R-33; the highest retained row is R-32). These are two disagreeing statements (C3). | **violated** (narrow) |
| AP-05 Explicit structure | Retirement, relocation and highest-issued rules are explicit in code. What "reachable" means is not (C1). | satisfied, with C1 |
| AP-06 Local reasoning/testability | Fixtures build throwaway Git repos. There are 8 focused tests and no product dependency. | satisfied |
| G5 Consistency and recovery | Relocation can change cited content (C1). Number reuse is undetected once the highest record's retirement lands (C2). | **fail** |
| G7 Truthful capability claims | `evidence: Implemented` and present-tense consequences cover pending retirement work. The claim about plan and register numbering is not true (C3, C7). | **fail** |
| G9 Architectural fitness | Follows AP-04 violated. The correction is textual and local. | fail (narrow) |
| G1–G4, G6, G8 | No product semantics, transformation or library choice is in scope. | not applicable |

**Question 2 (retired-peer rule):** "absent and below the highest retained ID ⇒ retired" is sound under three conditions. Numbering was contiguous through the consolidation baseline (the former lint enforced it). Allocation is max + 1. The highest record is retained. The rule cannot catch a typo that names a different lower number (C4). Requiring a retained record whose successor is retired to retire as well (`superseded-by` must be retained) is sound and keeps retained pairs symmetric.

## 7. Findings

| ID | Sev. | Finding and evidence | Principles / gate / scenario | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="c1"></a>C1 | Medium | **Relocation checks path existence, not identity.** `relocated_review` compares only path and anchor. `review_error` and `unrelocate_links` use `git cat-file -e`, which checks existence in the object store, not reachability. `PERMALINK_RE` accepts any GitHub owner/repo. Probes accepted three relocations: to an earlier commit where the review had different content (P1), to a dangling commit on a deleted branch (P3), and to `github.com/someone-else/fork` (P2). Relocation was also accepted while the local review still existed (P9). ADR-0096 requires "a retired local path" at "a reachable commit". | DP-22, DP-24, G5; S2 | An accepted record's cited evidence can silently change to an earlier draft, a forged commit or another repository while immutability reports clean. Bulk agent relocation makes a wrong-commit choice realistic. | In the relocation branch of `lint_immutability`, require all of the following. (a) `git merge-base --is-ancestor <commit> origin/main`. (b) Blob identity: `<commit>:<path>` must equal `origin/main:<path>`. This is available because the relocation PR still has the file on `origin/main`. (c) Permalink owner/repo must equal the canonical repository, derived once from the origin remote. (d) The old local path must be absent from the working tree. Define "reachable" in Outcome 4 as "an ancestor of `main`". | Fixtures for P1, P2, P3 and P9 are rejected. The existing positive fixtures still pass. |
| <a id="c2"></a>C2 | Medium | **The highest-issued guard is a snapshot diff, not an invariant.** `lint_highest_issued` compares the tree max with the `origin/main` *tree* max. Once a retirement of the highest record lands (lint is not a merge gate and CI is manually dispatched), the next lint is clean and `next_number()` reissues the number (P7b: retire ADR-0003 and merge; `next_number()` = 3). The check is also skipped silently when `origin/main` is absent, whereas `lint_immutability` prints a note. | DP-22, G5; S5, S7 | A retired ID can be reissued, so historical citations to that number become ambiguous. That defeats "IDs are never reused". | Keep the selected retention rule. Derive the issued maximum from history: the max over `git log origin/main --diff-filter=A --name-only --format= -- docs/adr/` and the tree. Fail the lint when the tree max is below it, and have `next_number()` allocate history max + 1. Print a note when history is unavailable. | A post-merge fixture reports the violation, and allocation after it returns 4. |
| <a id="c3"></a>C3 | Medium | **The numbering rule is overstated for plans and contradicted for the register.** Outcome 3 says the rule applies to plan numbers and register row IDs. `just plan` allocates tree max + 1 and nothing checks that the highest plan is retained. Outcome 6 lets the register be empty. `register.md` already retired R-33 and relies on a prose mark ("highest issued id is R-33") that `check_register.py` does not read. | AP-04, DP-01, G7; S7 | Two disagreeing policy statements. Once the register is empty or R-33 is mis-edited, row IDs cited from ADRs and issues can be reused. | Make ADR Outcome 3 state each namespace's actual mechanism. For plans, either extend the C2 history check to `docs/plans/` or drop the claim. For the register, declare the high-water mark once in `register.md` in a machine-readable form. `--lint` then rejects rows above the mark and a mark below the history maximum. | The ADR text matches the checks. A fixture with an empty register plus a mark passes. A row above the mark fails. |
| <a id="c4"></a>C4 | Low | **A retired peer cannot be told apart from a mistyped one.** `supersession_errors` treats any absent ID below the retained maximum as retired (P6: `supersedes: [ADR-0007]` with no history is accepted). | G5; S5 | A wrong historical pointer in a new record goes unnoticed. The only effect is on navigation. | Optionally confirm that the ID was ever added on `origin/main` (the same history query as C2). If shallow, fall back to shape only. Otherwise state the limit in Outcome 4 next to "shape and existence". | A fixture with a never-issued lower ID is rejected, or the limit is documented. |
| <a id="c5"></a>C5 | Low | **Shallow clones get opposite treatment.** In a `--depth 1` clone, `review_error` accepts a fabricated 40-hex commit (S1). `unrelocate_links` rejects a genuine permalink to an older commit (S3, false positive). The governance workflow uses `fetch-depth: 0`, so this is latent outside CI. | AP-02, G5 | Local results depend on clone depth: a false pass in one case and a false failure in the other. | Use one shared helper that returns present / absent / unknown for a commit and path in both checks. Treat unknown the same way in both, with a printed note, never a silent pass. | A shallow fixture gives the same outcome for both forms. |
| <a id="c6"></a>C6 | Low | **The relocation form assumes `../` links.** `unrelocate_links` rebuilds `](../<path>`. A sibling link such as `[ADR-0065](0065-….md)` (present in proposed ADR-0066) relocated to a permalink is rejected once the record is accepted (P5). | AP-02; S5 | A false positive blocks the permitted repair of links to retired ADRs. | Resolve the old relative target against `docs/adr/` and compare normalized repository paths, not reconstructed text. | The sibling-link fixture is accepted. A different-path fixture is still rejected. |
| <a id="c7"></a>C7 | Low | **Evidence and confirmation are overclaimed.** `evidence: Implemented` and "Implemented by Plan 19" cover Outcome 1, 5 and 6, which are pending (MD02–MD07). "Accepted records that cite retired reviews carry Git citations" is stated as present fact, but no retained record uses `git:` yet. The named `just adr-lint` currently exits 1 (see §10), and its per-record retention checks are never reached. Outcome 4 lists status as the only other mutable field, but `superseded-by` and *Status history* are also exempt. | DP-22, G7 | A reader takes corpus retirement and the lint as established. | Label the tooling `Tested` and name `DecisionRecordTests` and its conditions. Mark corpus retirement as Proposed until MD07. Rephrase the consequences as obligations. List every mutable field and section. | The ADR front matter and Confirmation match §10. |
| <a id="c8"></a>C8 | Low | **Status history is exempt, not append-only.** `lint_immutability` skips the section entirely. Rewriting an existing history line was accepted (P8). This predates the change but weakens the retained-record immutability that Outcome 4 restates. | DP-24, G5 | Recorded acceptance conditions can be edited in place. | Require the old section text to be a prefix of the new. | The P8 fixture is rejected. Appending still passes. |

**Strengths:** the rules stay small and local. Git is the only archive: there is no tombstone catalog or ledger. Retained pairs stay symmetric, and the tests confirm that asymmetric pairs and a retired successor are rejected. Review-field relocation already rejects a changed anchor and any concurrent edit (tested). An empty register is valid.

## 10. Verification (evidence and exact commands)

| Claim | Label | Command, mode and result against a zero baseline |
|---|---|---|
| Tooling fixtures pass | Tested | `python3 -m unittest scripts.tests.test_setup.DecisionRecordTests -v`, stdlib, local Linux, temp Git repos: 8 run, 0 failures, 0 errors |
| Repository ADR lint | Tested | `just adr-lint` (`scripts.validation --group adr-lint`): exit 1. 2 of 3 steps failed. `adr-frontmatter-check`: 1 error, duplicate section §0 in `blueprint.md` and `sections/architecture-overview.md`, reported before the per-record checks run. `adr-index-check`: stale `docs/adr/README.md`. `register-lint`: passed. These are 2 failures against the zero baseline and reflect the in-progress migration's tree state. |
| Retention checks on the real corpus | Tested | Read-only Python call of `review_error`, `supersession_errors`, `lint_immutability` and `lint_highest_issued` over `docs/adr/`: 0, 0 and 0 errors for the last three. `review_error` reported 1 error: ADR-0096's review path did not exist. This artifact resolves it. |
| C1, C2, C4, C6, C8 loopholes | Tested | Disposable scratch-directory probes P1–P3 and P5–P9 using the fixture helper `commit_base`. Not committed. |
| C5 shallow behavior | Tested | Scratch `git clone --depth 1 file://…` probes S1 and S3. |
| Remedies | Proposed | Not implemented, per this review's scope. |

## 11. Disposition

Record C1–C8 in Plan 19 *Finding dispositions* under MD01. ADR-0096 is `proposed`, so C3 and C7 are corrected in place. C1, C2 and C5–C8 are small changes to `scripts/adr.py` with fixtures in `DecisionRecordTests`. No blueprint or accepted-record edit is needed.

## 12. Decision

Architectural fitness: the selected design is simple and appropriately local. The rules are Git as archive, retention of the highest record, relocation to an immutable commit, and tolerated historical peers. The only violated foundation is AP-04, where the numbering rule disagrees between the ADR and the register (C3). Behavioral adequacy fails G5: relocation identity and post-merge number reuse (C1, C2). It fails G7: overstated numbering and evidence claims (C3, C7).

**Decision: Revise.** The selected policy is not in question. Acceptance needs C1–C3 and C7 corrected. C4–C6 and C8 are low-cost hardening that may land in the same change.

| Priority | Change | Findings / scenarios | Acceptance evidence | Owner |
|---|---|---|---|---|
| 1 | Relocation identity: ancestor, blob, repository, retired path | C1; S2 | Negative fixtures P1, P2, P3 and P9 | Plan 19 MD01 |
| 2 | History-derived issued maximum for ADRs, and for plans or the register as the ADR states | C2, C3; S5, S7 | Post-merge and empty-register fixtures | Plan 19 MD01 |
| 3 | ADR-0096 text: evidence label, mutable fields, "reachable", per-namespace rule | C3, C7 | The text matches the checks | Plan 19 MD01 |
| 4 | Shallow parity, sibling links, append-only history, typo detection | C4–C6, C8 | Corresponding fixtures | Plan 19 MD01 |

## 13. Follow-up verification (2026-09-26)

Scope: the Plan 19 *Finding dispositions* for C1–C8 were checked against the current `scripts/adr.py`, `scripts/check_register.py`, the `plan` recipe, ADR-0096 and `DecisionRecordTests`. Commands (local Linux, stdlib, zero baseline): `python3 -m unittest scripts.tests.test_setup.DecisionRecordTests -v` ran 10 tests with 0 failures and 0 errors (**Tested**). `just adr-lint` exited 0 with 3 of 3 steps passed and `adr lint: 30 record(s) OK` (**Tested**). Scratch probes subclassed `DecisionRecordTests` in throwaway repositories under the session scratchpad and were not committed (**Tested**).

| ID | Result | Evidence |
|---|---|---|
| C1 | Resolved | Relocation now requires an absent local path, a commit that is an ancestor of HEAD, the same blob as `origin/main`, and this repository's slug. The fixtures reject P1, P3 and P9. P2 was re-probed with an `origin` remote and rejected ("is not this repository"). Residual (N4, low): once the deletion reaches `origin/main`, `same_as_baseline` returns true, and a relocation to an earlier draft is accepted. |
| C2 | Resolved, one gap | Allocation and the highest-issued lint read `git log --diff-filter=A`. The post-merge fixture reports the violation and `next_number()` = 4. Gap N1 is below. |
| C3 | Partial | Plans: `just plan` counts historical additions, but has the N1 gap. Register: `check_register.py` requires the mark and rejects a row above it (fixture). Probe N6 showed that re-adding a retired lower id (R-03 under mark R-07) and lowering the mark with an empty register (R-07 to R-02) both lint clean. So Outcome 3's "an empty register reuses nothing" relies on the prose convention, not on the check. |
| C4 | Resolved | With history, the never-issued lower peer `ADR-0003` is rejected (probe N5). The shape fallback applies only without history. |
| C5 | Partial | `object_state` is shared, and a fabricated commit is no longer accepted by relocation. Probe N3 (`git clone --depth 1`): `review_error` accepts a genuine older `git:` review with a note. A relocation to that same review is rejected while `origin/main` still holds the file, because `same_as_baseline` compares against an empty `rev-parse`. This false failure affects shallow clones only (the governance workflow uses `fetch-depth: 0`). An unknown state in the relocation paths passes without a note. |
| C6 | Resolved | Links are rebuilt with `os.path.relpath` from the record directory. The fixture accepts `0002-peer.md` and rejects a different path. Non-canonical spellings (`./x.md`, `../adr/x.md`) still compare as text and were not probed. |
| C7 | Resolved | `evidence: Tested` names `DecisionRecordTests`. Confirmation leaves corpus retirement to Plan 19. Outcome 4 lists every permitted edit. Nit: Outcome 4 says "reachable from `main`", but the code checks ancestry of HEAD, and blob identity carries the guarantee. |
| C8 | Resolved | The section-prefix check rejects a rewrite (fixture). Residual (N7, low): extending the last line in place ("condition X." to "condition X. Condition waived.") passes, because the comparison is by character prefix, not by line. |

**New issues.**

- **N1 (Medium, latent): rename detection hides an issued number.** `issued_history` and the `plan` recipe use `git log --diff-filter=A` with Git's default rename detection. Probe N1b: one commit deletes `0002` and adds a similar `0004`. Git records that as a rename, not an addition. After `0004` is retired on `main`, the lint is clean and `next_number()` returns 4, which reissues the number. Adding `--no-renames` restores the addition (verified in scratch). Current history is unaffected: the ADR and plan sets are identical with and without `--no-renames`. The scenario is realistic in consolidation, for example a supersede-and-retire commit.
- **N2 (Low): a second relocation in an already-relocated section is rejected.** `lint_immutability` un-relocates permalinks only in the new text. A section that already holds a permalink on `origin/main` therefore mismatches when a further link in it is relocated (probe N2: "section 'Context' changed"). Remedy: apply `unrelocate_links` to the old section too.

**Updated decision: Accept-scoped.** The acceptance conditions from §12 are met for C1, C2 and C7. The remaining gaps are local and low-cost. Scope of acceptance:

- ADR and plan allocation, retired-peer tolerance, review and link relocation, and append-only status history, in full-history clones and for additions that Git does not classify as renames.
- Excluded until corrected in MD01:
  - N1: add `--no-renames` to both history queries.
  - C3 register: reject new rows at or below the `origin/main` mark and any lowering of the mark, or narrow Outcome 3 to what the lint checks.
  - N2.
- C5, N4 and N7 are optional hardening.
