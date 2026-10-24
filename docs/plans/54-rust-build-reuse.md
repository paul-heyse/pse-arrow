---
title: "Plan 54: Rust build reuse"
status: in-progress
date: 2026-10-24
adrs: []
review_sources: [docs/design_review/reviews/design_review_rust-build-reuse_2026-10-24.md, docs/functional_reviews/rust-build-reuse-tooling-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/54-rust-build-reuse.md
doc_topics: [current-work, design, environment, preparation, performance]
---

# Plan 54: Rust build reuse

## Target and authorization

**Proposed:** the best technically justified Rust build-reuse system for this codebase's
current host: developer and agent edit loops, selected correctness tests, worktrees,
isolated experiments and Python extension installation. Optimize complete useful workflows,
correct reuse, interruption behavior and operating burden. A higher hit percentage alone
does not establish a better system.

The maintainer requested a **new series**, independent of other open plans, selected
"current host, broad inquiry" and authorized creation of these documents on 2026-10-24.
The subsequent instruction authorized execution of 54a's A01–A03 on 2026-10-24; its
[Outcome](54a-compiler-cache-and-environment.md#verification-and-outcome--2026-10-24) records
accepted bounded implementation and controls. Remaining packages retain their planned state;
investigation selection does not pre-approve the mechanism it may recommend. Existing plans are consumed foundations and interfaces,
not a reason to narrow this series or a second owner for its work.

This coordinator owns the combined design, finding and inquiry dispositions, shared
decisions and completion boundary. Companions own their package state and local evidence:

| Companion | Detailed responsibility | Handoff |
|---|---|---|
| [54a: Compiler cache and environment](54a-compiler-cache-and-environment.md) | Exact tools, supervised cache mode, equivalent caller environments | Selected executable and reliable compilation ownership |
| [54b: Cargo freshness and dependency graph](54b-cargo-freshness-and-dependency-graph.md) | Content fingerprints, hygienic macros/manifests and strict shear | Accurate freshness and dependency declarations with preserved validation |
| [54c: Cross-location reuse and investigations](54c-cross-location-reuse-and-investigations.md) | Safe relocation, cache economics, non-cacheable work and wider capabilities | Supported decisions and bounded adoption conditions |
| [54d: Measurement, qualification and adoption](54d-measurement-qualification-and-adoption.md) | Snapshot-local targets, end-to-end comparisons and joined acceptance | Trustworthy benefit evidence and published support limits |

## Basis and focused foundation assessment

**Interface-checked:** authoring inspected HEAD
640035fbf7c840f05d1fa82eb4b67b57244efa80 and the supplied working-tree
[design review](../design_review/reviews/design_review_rust-build-reuse_2026-10-24.md),
[functional review](../functional_reviews/rust-build-reuse-tooling-2026-10-24.md) and
[library reference](../libraries/rust_caching.md), read end to end in this engagement.
The review's original measurements concern 68e44a37d; they were not rerun for authoring.
Concurrent review drafts and unrelated changes were preserved. These references identify
the inspected inputs, not a source freeze or continuing provenance campaign.

Use Core3.4, its efficient-architecture heuristics and the pse-arrow binding selected by
[standard.toml](../design_review/design_principles/standard.toml). The test-assessment
companion informs the proposed acceptance; this tooling subject does not change physical
or numerical semantics. No new broad diagnostic audit is initiated by plan creation.

| Existing foundation | Assessment and consequence |
|---|---|
| Cargo workspace feature unification, checkout-local targets and incremental workspace crates | Retain as the supported starting design. Feature equality across sampled selections is useful evidence, not proof that every unit or profile is interchangeable. |
| Managed sccache service and client-side compilation | Reuse its ownership, private endpoint and generation handling. Correct logging-mode detection; do not replace it with another daemon manager. |
| Bootstrap, doctor and diagnostic-tool declarations | Extend existing tool installation/selection rather than creating a second pin ledger. A declared binary must be the one actually used. |
| Cargo content fingerprints | The actual pinned Cargo 1.101.0-nightly, commit 3d7cf6e937d6127d0f49881bf689c560b36d35c4, documents both required keys. Installation-context config admission must consume them. |
| Build measurements and copied source snapshots | Retain source isolation, preparation/execution distinctions and owned recovery. Default targets inside each snapshot remove an avoidable environment-key difference. |
| Explicit Arrow validation and selected build/catalog preparation | Preserve actual target/host/build/dev leaf validation and existing retained-build owners. Compiler-cache equivalence does not authorize unvalidated correctness. |
| Python/custom-server installation preparation | Consume existing preparation and controlled-context boundaries from Plans52d/53d. This series changes cache/config compatibility, not their artifact authority or implementation status. |

The new findings are **Interface-checked** source conclusions:

- **CB08:** build measurements export an absolute CARGO_TARGET_DIR for each campaign and
  change it for the second snapshot. sccache hashes this input, independently of OUT_DIR.
- **CB09:** build_environment.configure and compiler_cache.supervised_compile check
  SCCACHE_ERROR_LOG but omit SCCACHE_LOG, whose presence disables upstream client-side
  mode, including an empty value. This can move compiler work outside the intended caller.
- **CB10:** the proposed shear acceptance permits warning diagnostics because it omits
  --deny-warnings. A zero exit is insufficient for the stated zero-finding target.

The supplied report is evidence, not an instruction to reproduce every remedy. Several
corrections from the assessment are incorporated here: multiple detectors are independent
analyses rather than competing semantic authorities; most manifest removals have no
demonstrated compiled-graph reduction; Cargo unit durations are not CPU-seconds; and the
current Rust-basedirs PR does not solve raw OUT_DIR environment dependencies.

## Combined design and representative scenarios

Cargo decides whether a build unit is fresh. sccache may restore an eligible invocation's
outputs after Cargo decides to run rustc. Installation/preparation owners decide whether
their consumed artifact can be reused. Keep these responsibilities distinct; do not add a
bespoke cache-key interpreter, shared cross-checkout target, or generic build service.

The selected direction first removes unnecessary work and accidental differences, then
assesses the remaining expensive work. Client-side execution, explicit tool selection and
isolated measurement are prerequisites for credible later comparisons. Preserve meaningful
differences in validation, target, compiler, profile, features, source and embedded inputs.
Never erase a key input merely to turn a miss into a hit.

| Scenario | Required behavior | Primary acceptance owner |
|---|---|---|
| S01 Tool installation, upgrade or explicit override | Bootstrap, doctor and actual invocation agree on the selected executable; overrides remain visible and deliberate | 54a A01 |
| S02 Logging, scheduler configuration or interrupted compilation | Ordinary builds retain documented direct fallback; explicit caching refuses incompatible mode; compiler children remain caller-owned | 54a A02 |
| S03 Equivalent agent and terminal build | Presentation-only defaults do not create avoidable compiler-cache variants | 54a A03 |
| S04 Touch, branch restoration or genuine input change | Identical tracked Rust contents remain fresh; real, generated and declared build-script input changes invalidate correctly | 54b B01 |
| S05 Macro use, optional validation and legitimate path modules | Accurate declarations, hygienic expansion and unchanged selected Arrow coverage; no test hidden to appease analysis | 54b B02/B03 |
| S06 New campaign, second checkout or artifact recovery | Each snapshot owns its target, shared eligible inputs can hit, recovery deletes only authenticated campaign artifacts | 54d D01 |
| S07 Relocated include output, embedded environment value or changed generated bytes | Safe reuse where established; changed observable values and dependencies miss; no wrong artifact restoration | 54c C01/C02 |
| S08 Sustained worktree activity or expensive non-cacheable work | Assess eviction, lookup/restoration, linking and compile cost across complete workflows before tuning | 54c C03/C04 |
| S09 Joined Rust/Python development | Correctness, interactive installation and selected native paths retain their intended distinctions | 54d D02/D03 |

## Recommendation dispositions

These are planning judgments, not implementation completion. Source R/F/O identities
retain their original scope; CB08–CB10 are additions from this engagement.

| Recommendation | Judgment | Work owner | Resulting direction |
|---|---|---|---|
| R01 / O01, O08 | Improve | 54a A01 | Pin sccache 0.18.0 and shear 1.14.0; connect installation, executable selection and doctor |
| R02 / O11 | Retain | 54a A03 | Remove agent CARGO_TERM_COLOR defaults; verify refreshed equivalent callers |
| R03 / O05, O06 | Improve | 54b B02 | Correct macro ownership; inspect removals and empty targets; preserve valid fixture layouts and feature activation |
| R04 / O07, O08 | Improve | 54b B03 | One strict routine shear check; retire routine machete/udeps without claiming identical analysis coverage |
| R05 / O10 | Improve | 54b B01 | Both Cargo content keys, controlled-context admission and positive invalidation checks |
| R06 / O02 | Retain | 54d D03 | State current reuse conditions and exclusions truthfully |
| R07 / O02, O04 | Correct | 54c C01/C02 | Investigate actual safe OUT_DIR/relocation support; a PR merge is not acceptance |
| R08 / O03, O09 | Retain starting direction | 54c C04 | Keep native workspace unification and mixed incremental strategy; reconsider only on specific workload evidence |

## Finding dispositions

| Finding | Scenario | Disposition | Work owner | Required evidence or trigger |
|---|---|---|---|---|
| F01 | S07 | scheduled | 54c C01/C02 | Resolve supported extent and external limitation; adopt only on compatible-cache and behavioral controls |
| F02 | S03 | resolved | 54a A03 | Both defaults removed; refreshed terminal/Codex inputs match and actual eligible reuse observed; 54a Outcome |
| F03 | S01 | resolved | 54a A01 | Verified managed 0.18.0 default selected; explicit overrides diagnosed; 54a Outcome |
| F04 | S05 | scheduled | 54b B02 | Inspected removals, selected compilations and unchanged validation closure |
| F05 | S05 | scheduled | 54b B03 | Strict shear, retired duplicate routine routes and preserved security checks |
| F06 | S04 | scheduled | 54b B01 | No rebuild for identical bytes and required rebuild for changed consumed inputs |
| F07 | S06/S07 | scheduled | 54d D03 | Published guide matches demonstrated reuse and remaining limits |
| CB08 | S06 | scheduled | 54d D01 | Snapshot-local default targets, no campaign target env and guarded recovery |
| CB09 | S02 | resolved | 54a A02 | Presence handled at both boundaries; real caller ownership/interruption controls; 54a Outcome |
| CB10 | S05 | scheduled | 54b B03 | Warning diagnostics fail the strict hygiene route |

## Inquiry outcomes and remaining questions

| Inquiry | State | Owner | Decision / completion evidence |
|---|---|---|---|
| I01 | open | 54c C01 | Actual safe relocation coverage, raw environment limitations and daemon-basedirs policy; no automatic PR adoption |
| I02 | open | 54c C03 | Attribution of cache errors/eviction and complete lookup/restore versus compile costs after fragmentation fixes |
| I03 | open | 54c C04 | Expensive host/target/profile variants, incremental tradeoffs and binary-owned work; select a bounded improvement or retain current design |
| I04 | open | 54c C05 | Technical value and full operational cost of local tiers/remote/distributed options for this host; no infrastructure assumed |

An inquiry closes with a supported choice, retained limitation or evidence-based no-change
decision, not an obligation to produce an optimization. A viable but unavailable external
capability can be deferred with an owner, concrete capability trigger and future check. It
must not leave this series waiting indefinitely for an upstream release.

## Dependencies and execution route

Begin with A01/A02 to establish selected tools and supervised cache behavior. A03 and B01
can proceed independently with coordinated edits to shared configuration/tests. B02 supplies
the clean manifest/source basis before B03 activates strict shear. D01 can proceed alongside
these packages and must precede comparative campaigns and C01's actual second-location
experiment. C01 then decides whether C02 has an eligible candidate; C03/C04 consume corrected
measurement and inputs. C05 can reach a source-supported no-change decision without deployment.
D02 joins selected packages and investigation dispositions; D03 publishes the actual result.

Package tables live in companions. Coordinator tables link those owners and do not duplicate
their progress. During execution, allocate one writer for shared build_environment,
compiler_cache, build_measurements, manifests and documentation configuration. Source overlap
is distinct from logical dependency; overlapping Cargo work uses the existing admission and
checkout rules. No additional worktree or universal resource manager is selected by this plan.

## Rule decisions and scope boundaries

**No rule change is selected by the immediate implementation target.** Tool pins, logging
handling, macro corrections, strict scope-end shear and Cargo freshness are tooling/refactors
within current contracts. Preserve exact pins, one Arrow type universe, explicit validation,
caller-owned compiler execution, non-direct C/C++ caching and checkout-local Cargo state.

Reintroducing a workspace-hack, adding a library crate to extract binary code, weakening
isolation or changing scientific/producer contracts is **investigation-only**. If recommended,
record the concrete rule impact and obtain the maintainer's decision before incorporating
dependent implementation, following the existing ADR/design route. No such decision is
inferred from "best possible", and missing current consumers do not exclude a library.

Native cache/provider acquisition remains an existing foundation; native compiler-cache
compatibility is covered, not a redesign of solver preparation. Other plans retain their
remaining obligations. Remote/distributed deployment and a new CI platform are not selected.

## Verification and completion

**Proposed:** each companion names targeted checks; D02 owns assembled selected-workflow
qualification and comparative benefit evidence. The failure baseline is zero. Follow the
repository rhythm: targeted compile/unit controls while implementing, ready after tool
selection changes, and the selected integration/static/manual bundle once functional scope
is complete. A cache canary cannot qualify product workflows; a source-only investigation
cannot establish a speedup.

Series completion requires A01–A03, B01–B03 and D01–D03 accepted; C01/C03/C04/C05 have explicit
decisions and C02 is either implemented and qualified or explicitly not adopted/deferred with
its remaining F01 limit. Reconcile findings accordingly: F01 is not resolved by documenting
a limitation. Record implementation, qualification and measured benefit separately. Enduring
meaning moves to the existing build/environment/dependency guides and, only where a contract
changes, through the existing architecture route. The 54a Outcome establishes its bounded
implementation and controls; it does not establish joined qualification or workflow benefit.
