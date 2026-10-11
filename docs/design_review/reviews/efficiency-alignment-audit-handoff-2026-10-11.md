# Handoff: independent audit of the 2026-10-11 efficiency-alignment review

**Audience.** An independent reviewer asked by the maintainer to audit
[the efficiency-alignment review](design_review_efficiency-alignment-codebase_2026-10-11.md)
(F01–F29, RC01–RC19, decision *Revise*) and to produce a comparative analysis. That analysis has
three parts:

- **corrections** to what the review states;
- **improvements** to what it specifies (diagnoses, corrective contracts, priorities, order, rule
  impacts);
- **additional efficiency items** it missed.

**What this document is.** It gives you context so you can plan your own review with the
`plan-design-review` skill and then run it with `design-review`. It does not tell you what to
conclude. Everything below is either a fact about how the review was produced or an explicitly
labelled coordinator opinion about where it may be weak. Treat the opinions as leads. You may find
that the review is right where this document doubts it, or wrong where this document trusts it.

**Who wrote it.** The coordinator of the original review, on 2026-10-11.

---

## 1. What exists and where

| Artifact | Path | Notes |
|---|---|---|
| Principal review | `docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md` | About 19.5k words. A fresh `design-reviewer` wrote it after independently re-reading the decisive source. It owns the finding IDs, the verdicts and the rule impacts |
| Evidence index | `docs/design_review/evidence/efficiency-alignment-codebase-2026-10-11/README.md` | Method, coordinator source checks V1–V21, reconciliation, inquiry→finding map, probes, coverage limits |
| Inquiry briefs | `…/inquiry-briefs.md` | What each inquiry was asked. Use it to see what was never asked |
| Operation map | `…/operation-map.md` | Facts-only trace of a warm edit→re-solve, study point N+1 and a result reopen. One claim in it is wrong (§3) |
| Ten inquiry reports | `…/{authoring-admission-and-incremental-preparation,mathematical-compilation-and-evaluation,single-solve-native-route,multi-case-workflows-and-capacity,publication-persistence-and-results,relational-and-columnar-layer,build-test-and-development-loop,assurance-identity-and-observation,numerical-library-utilization,data-and-infrastructure-library-utilization}.md` | Inquiries A–H plus L1 and L2. Each is the verbatim inquiry output under a provenance header. Local labels (A-01 …) are not the final IDs |
| Probe | `…/sha256-pass-time.{py,txt}` | Script and output for probe M1 |
| Standard | `docs/design_review/design_principles/` (`standard.toml` → core 3.4, heuristics 1.0, template, process-simulator profile 1.5, `binding/pse-arrow.md`) | The only judging criteria, together with the functional target |

Nothing from this review is committed. The review, its evidence directory and this handoff are
untracked files on `main`.

## 2. How the review was produced

### 2.1 Decisions the maintainer made

These shaped the scope. You may question them, but say so explicitly if you do.

- **Full re-assessment.** A codebase efficiency review already ran on 2026-10-09 (Plan 33, all of
  F01–F11 marked Resolved), and many earlier efficiency findings are tracked in Plan 28 (PE, PA,
  Parallel, EF and US rows) and Plan 34. Every prior disposition was treated as a lead and as
  contrary evidence, never as a conclusion. Principal review §9.3 records a position on each.
- **Scope.** The product (all crates, `python/pse`, native and SurrealDB integration) plus the
  build, test and development loop. Documentation tooling was excluded.
- **Evidence mode.** Static source inspection plus light probes only where doubt could change a
  verdict. There were no campaigns, services, heavy native runs or new benchmarks, because a Plan 28
  qualification campaign was waiting for exclusive host capacity.
- **Library-use review.** This was added mid-review at the maintainer's request. Library questions
  were to use the library skills (`.claude/skills/` and `~/.local/share/library-skills/skills/`),
  Context7, GitHub search and pinned upstream source. `docs/library-utilization.jsonl`, the
  library-catalog MCP and `docs/capability-maps/` are **stale** (the catalog's generator was
  removed) and were deliberately not used. The design-review skill and the binding still point at
  them; that is stale text.

### 2.2 Grounding

The review was judged against the standard and the functional target, not against ADRs, blueprint
decisions, plans, AGENTS.md policies or pins. Those are current design. Every change the review
would make to them is a `review#RCnn` rule impact, and none takes effect until the operator
confirms it in plan creation. The disposition owner is a **new plan created through
`plan-creation`**, which is not yet scheduled. Plan 28 is not extended, and Plans 33, 34 and 35 are
not reopened.

### 2.3 Pipeline

The work ran in five steps, using twelve agents in total.

1. **Operation map.** One read-only `code-mapper` (Opus override) traced three warm operations at
   function granularity.
2. **Inquiries.** Ten inquiries ran in parallel against a common brief:
   - eight `design-reviewer` inquiries (Opus, xhigh), lettered A–H;
   - two `library-research` inquiries (Opus, high), L1 and L2.

   Each inquiry took between about 11 and 21 minutes and used 280k–460k tokens. They were
   read-only: they did no builds and ran no tests, and proposed probes rather than running them.
3. **Coordinator verification.** The coordinator read the decisive source for each consequential
   candidate (V1–V21 in the evidence README), ran five light probes, and reconciled duplicates and
   disagreements in a reconciliation note. Its substance is in the README; the note itself was not
   published.
4. **Principal review.** A fresh `design-reviewer`, which took no part in any inquiry, re-read the
   decisive evidence. It took about 30 minutes and wrote the review. It reported checking every
   diagnosis in source, while taking some counts and searches from inquiries (it says so per
   finding).
5. **Publication.** The coordinator published the review and evidence, adding headers and a label
   map. It did not alter the principal's judgment.

Inquiry boundaries were drawn around responsibilities rather than crates. Inquiry H (assurance,
identity, observation) cut across all components on purpose, to catch repeated checks at seams.
`inquiry-briefs.md` has the exact questions.

### 2.4 What was actually measured

All probes were read-only and run on 2026-10-11 on a shared host with 32 logical CPUs (SHA-NI
present), about 188 GiB of RAM and a warm page cache.

| Probe | What it measured | Result |
|---|---|---|
| M1 | SHA-256 full pass over large binaries (Python `hashlib`) | 0.30–0.39 s per pass over the 727 MB dev `pse-worker` and the 786 MB dev extension |
| S1 | Physical packages present (static check) | 2 |
| G-P1 | Ledger parse plus dump, `python -I`, deliberately not through `pse-env` | 9.3 ms per cycle on the 1.71 MB ledger; validation, lock and write excluded |
| G-P2 | Bridge import time | `scripts.test_resources` ≈ 17 ms, `scripts.surreal_server` ≈ 13 ms |
| G-P3 | Hash of the newest native solver generation | 0.58–0.71 s for 1.2 GB in 2,305 files |
| Census | File counts | 1,190 native operation records; 639 invocation files |
| Maintainer JUnit | Per-test cold setup | Median 1.31 s per test, before the store refusal (n = 107, 16 threads) |

Everything else is source-derived. The per-edit and per-study-start hashing totals in F05 are
arithmetic from the measured per-pass costs, not end-to-end measurements. **No speedup is claimed
anywhere.**

## 3. Facts that are easy to get wrong

These cost the original reviewers time, or contradict a natural assumption. Verify them before you
rely on the opposite.

- **The study-candidate index exists.** The operation map says no index covers `settled` or
  `assigned`. That is wrong: the generated schema defines
  `study_candidates (study, settled, assigned, ordinal)`. The real discovery defect is that
  *blocked* candidates (waiting for a predecessor) are revisited, each costing one `study_scope`
  RPC, on every `work_once` call (F09).
- **The cache reservation default.** Inquiry F says file-cache envelopes default to zero bytes.
  That holds only for an explicitly constructed `CacheSettings`. The default Python runtime passes
  `cache=None`, so `pse-runtime/src/settings.rs` falls back to `CacheBudget::for_memory`, which
  reserves metadata (≤ 50 MiB) and statistics (≤ 16 MiB) envelopes up front (`Envelope::new`
  `try_grow`). L2's claim is the correct one (V20).
- **The value path exists, but only for studies.** `workflow/bindings.rs::admit_overlay` and
  `prepare_bound_operation` bind values onto an unchanged basis. Interactive Python edits can only
  use `with_declarations` (complete replacement), so a parameter tweak mints a revision and rebuilds
  structure before reaching `rebind_scoped` (F01). Reading §14.4's reuse table gives the opposite
  impression.
- **Salsa is a per-call memo.** `MathService::workspace` builds a fresh `CompilerWorkspace` (new
  `salsa::Storage`) on every call, and no production path publishes a second revision into an
  existing workspace. Backdating across edits therefore never happens, despite the §14.4 text (F07).
- **Result reads are mutations on the retention guard.** `canonical_results.rs::read_results` and
  `ResultRead::renew` bump `retention:<problem>` (`UPSERT … generation+1`). Every protected read
  takes the same record `FOR UPDATE`. A Python user opening results can therefore abort concurrent
  worker transactions on that problem (F08). In addition, every execution write of a study point
  touches `study:<id>` through the generated `writer` → `study_cancelled`.
- **SurrealDB 3.3 limits the coordination remedies.** It has no shared lock mode and no predicate
  locks; `FOR UPDATE` applies to record ids only (library skill probes SB091/SB092). Any
  coordination redesign must keep phantom and write-skew protection by key partitioning. Probe D-P1
  (do two `FOR UPDATE` readers of one record conflict?) was not run.
- **Hashing cost depends on the build.** The interactive extension is a **dev-profile,
  force-validated** build (`justfile` `py-sync`/`py-sync-native`): 786 MB, with a 727 MB dev
  worker. Release artifacts are smaller. F05's diagnosis is structural (repeated full
  re-observation), but its magnitudes partly reflect dev builds. F23/RC15 concerns the
  force-validate choice itself.
- **The cubic EOS evaluator is shared across attempts.** `implicit_accelerators.rs::Cubic::admit`
  returns one `Arc<CubicRoots>`, which holds a `Mutex<ExpressionEvaluator>`, and the implicit
  `Factory` clones that `Arc` into every attempt. The review treats this as a DP-19
  nonconformance, not only an efficiency issue (F18).
- **Some prior "Resolved" dispositions are real.** Do not re-raise them without new evidence:
  10-09 F04–F11, EF01/EF03/EF07, PE03, Parallel F04, Graph J3/N12/B8, State F01–F04/F09, Enh
  F04/F05. Others moved rather than disappeared:
  - PA01: hashing per body became hashing per prepare and per Runtime;
  - PE02: the block unit is now set by index metadata rather than a cell count;
  - Parallel F01: serialization replaced the conflict instead of narrowing it.
- **`ensure_writes` reads `config.json` on every mutation.** It is client-side, advisory, and
  cheap next to the RPCs. All reviewers judged it not a finding. Revisit it only if you have a
  write-fence argument.

## 4. Where the coordinator thinks the review is weakest

These are leads for your audit, not conclusions. The coordinator built the briefs, so these are
partly blind spots of the coordinator's own design.

### 4.1 Unmeasured magnitudes driving priorities

The diagnoses rest on structure. Several priorities rest on assumed magnitude:

- F16 is rated High while its callback share is unmeasured.
- F09's O(N²) arithmetic assumes chain-shaped continuation studies.
- F10(c) depends on the actual pool size.
- F11's rows per block are about 26–33 by source estimate only.

The settling probes are listed in review §9.2. The most decision-relevant are:

- **B-P1 / C-P1:** a callback-path `perf` profile on `just case-measure … --case warm-large-1`.
- **A-P1 / H-P1:** count the preparations and observations per value edit.
- **E-P1:** rows per block and empty blocks.
- **D-P1:** SurrealDB `FOR UPDATE` reader semantics.
- **D-P2:** discovery counts.
- **F-P1:** validation-context identity.
- **H-P4:** late executable mappings. If MKL or libffi map executable pages after startup,
  `qualify_receiving` returns `None` and persisted reuse is silently off. That would be a different
  defect from the one F05 describes.

All of these need builds, services or a quiet host. None was run.

### 4.2 Claims resting on inquiry searches the principal did not re-prove

- **F23 per-call assembly rebuild.** It depends on there being no long-lived strong holder of
  `ModelAssembly` in production (V14 confirmed only the `Weak` map).
- **Inquiry G counts.** About 231 Runtime-helper call sites in F26; 818 xtask units and 78 pruned
  crates in F27.
- **F24 absence searches.** The search excluded tests and `cache_service/`. Internal reachability of
  `pse-engine` `round`, `traversal`, `cache/*`, `operation/*` and `provider/*` was **not** traced, so
  part of the engine may serve physical inventory through `prepare_scoped`.
- **F25 hang and exit-status claims (G-08).** These are from source reading only.
- **G-07 (unconfirmed).** The claim is that native and plain routes invalidate the
  `pse-backend-native` build-script unit. It was not raised as a finding; probe G-P4 would settle it.

### 4.3 Remedy soundness, as opposed to diagnosis

Each of these items is open; a tradeoff, risk or condition for each remains unresolved.

1. **F04(ii).** Narrowing description premises to body scope is safe only if `SemanticBodyHash`
   frames every meaning-affecting dependency, including absent names a body probed.
2. **F05 / RC04 / RC05.** Stat-identity premise checks change ADR-0164's meaning of "current
   admission" and Plan 35's executable-association rule. Check the threat model (privileged
   in-place mutation, RTLD_GLOBAL promotion, mmap SIGBUS).
3. **F08 / RC06.** A protocol sketch is given, but its correctness under SurrealDB 3.3 semantics
   was not argued in full. Check reclaim-versus-root-creation phantoms, cancellation races and
   uncertain-commit settlement.
4. **F11 / RC08.** Dropping per-field cells assumes no cross-output predicate workload. The review
   points such workloads to F24 option 1, a DataFusion provider over blocks. Plan 33 EI02 is
   changed by this.
5. **F12 / RC09.** Returning before durability is conditional on cross-run seed reads
   (`run.rs::with_stored_start` → `latest_seed`).
6. **F01.** Which authored fields `admit_overlay` can address is unverified: starts and bounds are
   in question. So is how overlay lineage interacts with reopening an authored revision.
7. **F17 / RC13.** Worker reuse across attempts touches DP-19 attempt ownership.
8. **F23 / RC15.** Building without force-validate may contradict the maintainer's intent for the
   editable extension. The review flags this as a rule impact, not a fact.
9. **F24.** The review deliberately does not choose between committing to an inspection capability
   and retiring the machinery.

### 4.4 Verdict calls that could reasonably go the other way

- **AP-04 is rated "Satisfied".** This is despite the §14.4 specification/implementation divergence
  (F07) and the transient validation-owner identity (F23).
- **G7 and G8 fail "narrowly".** This rests on F24 and F07.
- **PS-G1–PS-G3 are "Unresolved (not re-qualified)".** The status→outcome mapping and diagnostic
  naming were not re-traced.
- **Priority ordering in §11.** For example, F05 is listed as priority 1 alongside F01. The
  prerequisite chain is engagement contract → coordination protocol → publication.

### 4.5 Grain of the 29 findings

Several findings bundle distinct mechanisms under one cause:

- F10 has manifestations (a)–(e);
- F16 merges six inquiry candidates;
- F13 spans studies, horizons and IDAS sensitivities.

Consider whether any should be split for disposition, or whether any merge hides a distinct
obligation.

## 5. Breadth that was not examined, and likely sources of additional items

This is the review's own coverage list, plus coordinator observations from reading all ten
reports.

### 5.1 Not read or read only shallowly

- IBEX root isolation (`root_isolation.rs`, about 4.6k lines).
- PETSc, SCIP, HiGHS, Uno and Clarabel adapter internals.
- `kkt/path/*` continuation and IDAS adjoint routes.
- Dynamics trajectory and shooting encoders.
- `pse-math` `derived.rs`, `factorable.rs`, `collocation.rs`, `surrogate.rs` and regime proof
  transport.
- `pse-authoring` parser performance and `pse-quantity` inference.
- `pse-codegen` internals.
- `pse-engine` session internals.
- `python/pse` beyond its boundary modules.
- Most of `scripts/surreal_server.py` (about 7.3k lines).
- `scripts/tests/` and `tests/`.
- teqp and the parity route.

### 5.2 Leads noticed but never pursued

- **Runtime construction itself.** The 1.31 s median cold setup per Rust test (SharedRuntime,
  registry, DataFusion sessions) was treated as a test cost (F26). Nobody decomposed it, and the
  same construction precedes interactive Python use. It is unknown whether it includes the F05(b)
  observation on the Rust path.
- **Native session lifetime.** Each run and each study point spawns a new native session thread,
  so Ipopt/KLU/FERAL symbolic setup and the TNLP are rebuilt per point (C §4 opportunity). Only
  partly addressed, through F09's lane-affinity remedy.
- **Presolve per attempt** (`Pipeline::new` per multistart start or strategy rung; probe C-P3).
- **Heartbeat** (one task plus two RPCs per interval per run) and **seed persistence** (JSON arrays
  of u64 bits, re-read from the store for in-process continuation; E §4).
- **Arrow C-stream pulls.** `block_on` runs per batch on the consumer's thread, refusing inside
  Tokio. The GIL interaction depends on the consumer.
- **Python-side conversions.** The cattrs/attrs/msgspec round trips per edit are folded into F02
  but not assessed for other APIs.
- **Prior leads not examined.**
  - Parallel F03: process-global PETSc, HiGHS and IBEX locks.
  - State F05–F08: deployment affiliation, recovery oracles, readiness exclusion.
  - The integrated-solve-pipeline and solver-acceleration reviews of 2026-10-03, beyond C's spot
    positions.
  - `prepare_flow` call frequency.
- **Library opportunities not raised as findings.** Symbolica JIT/SIMD and batch evaluation
  (RC-level change, §14.5); FeOS as a production PC-SAFT provider (knowledge-boundary decision);
  `klu_l_refactor` and FERAL symbolic reuse; the vendored SurrealDB SDK patch of about 2.8k lines
  (no pins hold record); DataFusion 55.2 metadata-preserving extension types.
- **Questions the briefs never asked** (see `inquiry-briefs.md`):
  - memory high-water and peak live state for a full 16-case run;
  - process start-up and import time of `pse`;
  - cross-language allocation and copy accounting end to end;
  - log and tracing volume under the default observation policy;
  - cold-start cost of the managed primary and SurrealDB service;
  - the docs build (excluded by scope).

## 6. Working constraints and practical notes

- **Host admission.** `scripts/pse-env` admits workload classes through host admission. At handoff
  time an **exclusive heavy-work owner** (probably the Plan 28 E3/E4/E5 campaign) held the host, and
  `pse-env --docs` was refused, so lifecycle validation of the new files has not run. Do not
  override admission or stop another owner's work. Source reading needs no admission.
- **`pse-env` side effects.** Every top-level `scripts/pse-env` exit rewrites the test-resource
  ledger (F25). For strictly read-only probes, run the interpreter directly (`python3 -I …`), as
  G-P1 and G-P2 did, and state that you did so.
- **Concurrent edits.** Another session was modifying `scripts/test_resources.py` and
  `scripts/tests/test_test_resources.py`, adding a reclaim deferral while service admission is
  closed. `docs/plans/28e-…md` also had maintainer edits. Preserve them. Because of this,
  `just turn-end` (whole-repository `cargo fmt`, `taplo`, `ruff --fix`) was not run after
  publication; the review changes are Markdown only.
- **Subagent mechanics observed.** One `library-research` agent's file write was interrupted by a
  safety classifier. Ask agents to return complete reports as their final message; the coordinator
  can then persist them. Design-reviewer contracts forbid repository edits, so scratch space
  outside the repository is the right place for drafts.
- **Library evidence.** Use the library skills first (`symbolica-faer-oximo`,
  `native-solver-libraries`, `salsa`, `rust-graphs`, `neo4j-surrealdb`, `datafusion`, `pyo3`,
  `serde-arrow` …), then Context7, then GitHub search and pinned source under
  `~/.cargo/registry/src/` and `vendor/`. Check version applicability against the pins in
  `Cargo.toml`. Do not use the stale catalog or capability maps.
- **Maintainer preferences relevant to judgment.**
  - Prefer native and library capability over bespoke code. Never gate library features on the
    existence of a consumer.
  - Aim at the best-in-class target and record conflicting rules as rule impacts.
  - Judgment over ceremony: no mandated probes, checklists or new alignment lints.
  - Delete legacy code as soon as it is replaced.
  - No library or licence is refused.
  - Heuristics are qualitative: no cost models, and no speed claims without measurement.

## 7. Suggested shape for the comparative analysis

This is a suggestion; plan your own.

- **Location.** Publish your audit as a separate principal document, for example
  `docs/design_review/reviews/design_review_efficiency-alignment-audit_<date>.md`, with evidence
  under `docs/design_review/evidence/efficiency-alignment-audit-<date>/`. Do not edit the original
  review. It is the earlier reviewer's judgment, and the operator reconciles both in plan creation.
- **Per-finding positions.** For each of F01–F29, record one of: confirmed, corrected (diagnosis),
  improved (contract, preservation or priority), merged, split, re-scoped or withdrawn. Give the
  decisive evidence for each.
- **Per-rule-impact positions.** For RC01–RC19, record agree, modify or reject, plus any rule
  impacts the original missed.
- **Additional findings.** Use new identifiers (for example `A01`…) under the same finding standard,
  with workload, consequence, corrective contract, preservation constraints and evidence label.
- **Order and conflicts.** Report corrections to remedy compatibility and prerequisite order (review
  §8.2), especially the preparation engagement owner, the coordination protocol, publication shape
  and the inner-loop owner.
- **Probes.** Say which open probes you ran and under what conditions. If host capacity allows,
  running even two or three of the §9.2 decision-relevant probes would materially strengthen
  whichever review is right.
