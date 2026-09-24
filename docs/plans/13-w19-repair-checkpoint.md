---
title: Rust computation W19 repair checkpoint
status: abandoned
date: 2026-09-23
adrs: [ADR-0076, ADR-0077, ADR-0078, ADR-0079, ADR-0080, ADR-0081]
phase: 1
evidence: Tested — current isolated repair controls and static checks; W19 incomplete and W20 unrun
---

# W19 repair checkpoint

**Historical checkpoint; no longer a resume instruction.**
[Plan 14](14-library-owned-process-simulator.md) supersedes Plan 13's execution scope.
Its target-derived acceptance does not inherit the unfinished campaigns below.
Reuse of any repaired mechanism requires evidence of its new target role.

This was the resume point for [Plan 13](13-rust-computation-architecture.md).
The maintainer requested a documentation checkpoint after the current repair batch.
No further functional campaign or performance work runs as part of this checkpoint.
Baseline: zero failures. The shared dirty tree and real Git index are preserved.

## Current boundary

| Scope | Current state | What the evidence establishes |
|---|---|---|
| W00–W17 and L01–L16 | Implemented; isolated controls refreshed | The target compiler, consumers, generated contracts and deletion routes exist; this is not complete product qualification |
| W18 | Complete; seal and preflight refreshed after repairs, with a final refresh after documentation | A seal validates implementation evidence and source identity; it does not award W19 acceptance |
| W19 | Incomplete; first campaign failed and was interrupted | Shared repairs have isolated/static evidence; complete current functional evidence remains required |
| W20 | Not run | No target performance improvement or final G1–G7 verdict is claimed |
| Decision process | ADR-0076–ADR-0081 remain proposed | Authorized local implementation does not establish formal decision/design PR acceptance |

The pivot is a direct replacement. The former compiler operation wrappers, separate
specialization database, hash-only specialization API and ordinary all-pairs containment
outputs remain deleted. Existing engine plans, source witnesses, physical caches and
Delta ownership remain where they serve actual relational/storage work. Historical
plans and unsuccessful receipts remain evidence, not executable compatibility paths.

## First W19 attempt

**Tested, unsuccessful:**

```bash
CARGO_INCREMENTAL=0 just assessment build/plan13/w19-functional-01 --plan 13
```

`build/plan13/w19-functional-01/checks.json` records 49 passed gates, eight failed,
four advisory findings, one unsupported deferred gate, one interrupted gate and
13 not-run gates. Its captured source remained unchanged. Counts below overlap
between gates and must not be summed.

| Gate | Actual result and mode |
|---|---|
| `test` | 2,757 passed, 119 failed, zero skipped/not-run; ci profile, explicit force-validation |
| `governance-tests` | 98 passed, six failed; ci profile, explicit force-validation |
| `native-solver-test` | Four passed, eight failed; pinned Ipopt runner and explicit force-validation |
| `native-compiler-solver-test` | One passed, one failed; pinned Ipopt runner and explicit force-validation |
| Other required failures | `lint-typos`, `codegen-relations-check`, `codegen-rust-contracts-check`, `conformance-fixtures-check` |

Release-test enumeration was deliberately interrupted after the shared causes were
identified, before further expensive qualification of known-failing source. Release
tests, both doctest modes, the inspection fixture, all three campaign Python groups,
four engineering cases, feature combinations/no-default and coverage remain unfinished.
The final required campaign must complete them; interruption grants no exemption.

The unsafe-surface tool completed, but its report includes dependency parser limitations.
Dependency/advisory reports retain their existing advisory authority; `doc-lint` remains
unsupported under R-20. Neither disposition is product acceptance.

## Repairs in the working tree

| Cause | Implemented repair | Qualification still required |
|---|---|---|
| SQL-bearing generated rows reached a pure registry without a native planner | The effectful document loader installs the standard validation owner; custom engine candidates retain their actual function owner. Standalone native fixtures bind explicitly. Four SQL-dependent controls move to `pse-engine/tests/generated_sql_contracts.rs` | Full authoring, physical-fixture generation, compiler and engineering journeys |
| Delta repolled an exhausted shared store stream | Both `unfold` wrappers use Futures `StreamExt::fuse`; nested list/body/delete streams release admission and remain exhausted on repeated EOF | Actual Delta write/reopen, recovery, CDF, maintenance and inspection journeys |
| Artifact descriptor scan lacked persisted relation metadata | The artifact output uses `declare_relation_output` before durable encoding; strict Delta field/metadata admission is retained | Complete product publication and exact reopen |
| Standalone solver factory lacked running-query admission | The fixture factory owns bounded CPU admission from its thread budget; the solver guard remains mandatory | Actual solver and concurrent edit/solve tests |
| Rule schedule declaration/tests retained old assumptions | `compiled.rule_strata` is Derived; tests inspect actual settlement strata and schedule provenance; graph failures retain `ProjectionError` and its witness | Full rule binding/fixed-point, conflict and negative-recursion tests |
| Generated source tests targeted old semantic/builder ownership | Semantic assertions read `pse-model`; Syn inspects the shared native `RowBuilder` alias; bootstrap roots and reflected query-column lookup follow current declarations | Full generation and all-target campaign checks |
| Appendix B named absent, fully specified runtime tear contracts | Register the ADR-0075 result relations with keys/FKs; runtime `TearMethod` and compiler `TearHeuristic` have separate meanings | Registry governance; this adds no Pyomo selection backend or R-32 execution |
| Other stale assertions | Correct the nested-value diagnostic expectation and private canonicalization field spelling; regenerate conformance fixtures and prune obsolete generated cases | Final current-source campaign |

The Delta pin, kernel pin and four dependency families are unchanged. Delta work used
the exact-git skill/source contract. No compatibility API or relaxed validation check
was introduced to obtain a passing isolated result.

## Completed repair validation

**Tested:** all final commands below have zero failures against baseline zero. Logs
live under `build/plan13/w15-w20/`; counts overlap and are not a campaign total.

| Command / receipt | Result and condition |
|---|---|
| `development-units-07` | 530 controls: 348 selected library units, two tool units, nine typed-boundary units, one linked callback, 63 setup tests and 107 Python units; source capture unchanged |
| Native groups in that receipt | Recipe-owned ci profile and explicit `pse-relations/force-validate`; linked callback also uses the pinned Ipopt runner and invokes no solve |
| Development export | 461 distinct binary/test/mode identities cover all 149 required unit case IDs in `build/plan13/development-checks.json` |
| `just unit-package pse-codegen 'binary(native_field_declarations)' --test native_field_declarations --profile ci --success-output final` | 16 passed; explicit force-validation, pure generator contracts (`w19-repair-codegen-tests-02.log`) |
| `just unit-package pse-schema 'binary(registry_admission)' --test registry_admission --profile ci --success-output final` | 16 passed; explicit force-validation (`w19-repair-schema-tests-01.log`) |
| `just check-native-contracts` | Locked all-target force-validation compilation passed (`w19-repair-check-02.log`) |
| `just clippy-default` | Workspace/all-target default-feature Clippy passed with zero project warnings (`w19-repair-clippy-02.log`) |
| `just codegen-contracts`, `just conformance-fixtures` | Pure regeneration completed; no hand edits to generated output |
| `codegen-checks-06.json` | Strict Rust/Python/docs equality passed with a copied temporary index; actual index bytes unchanged |
| `just conformance-fixtures-check` | Fresh invariant fixture comparison passed (`w19-repair-fixtures-check-01.log`) |
| `just family-check`, `just fmt-rust-check`, `just lint-typos`, `just py-sync` | Family ceilings, formatting, spelling and editable force-validation extension refresh passed |

**Mistakes corrected:** the first focused generator run had ten passes and six failures
because its new assertion depended on line wrapping. Syn now checks the actual type
alias. Clippy then reported five diagnostics from one redundant setup call preceding
a local import; that call is removed. Earlier repair generation/check attempts and
development receipts 05/06 remain intact. The final source-qualified receipt is 07.
The upstream `proc-macro-error2` future-incompatibility notice remains; it is not a
project Clippy warning. No integrated repair is labelled qualified from these units.

**Tested, documentation checkpoint:** `just lint-typos`, `just lint-agents`,
`just docs`, `just architecture-manifest --plan 13` and `just fmt-rust-check` pass
against baseline zero. The book builds with its large-search-index size warning;
this is not an API-reference lint or a functional acceptance result. The resumed
environment check found stale environment metadata. `just py-sync` refreshed the
locked environment and editable extension; `just doctor` then passed all checks
(`w19-checkpoint-py-sync-01.log`, `w19-checkpoint-doctor-02.log`). No library pin changed.

`just architecture-seal --plan 13` and `just architecture-preflight --plan 13`
pass with the refreshed development export (`architecture-seal-03.log`,
`architecture-preflight-02.log`). The final documentation is sealed/preflighted
again in `architecture-seal-04.log` and `architecture-preflight-03.log` before
handover. These are implementation/source checks, not a renewed functional campaign.

Current pointers now include the root README/status, documentation overview and
navigation, plan index, Plan 13 packets/inventory/stage contracts, and implementation
addenda on the alignment, target-design and rule-schedule reviews. Plans 10/11 are
labelled historical execution records with their acceptance still carried forward.
The blueprint and ADR decision statuses are unchanged by this documentation checkpoint.

## Former resume sequence (superseded)

The instructions below are preserved as historical context, not work to run before
Plan 14. They do not authorize or require restarting the old campaigns.

1. Read this checkpoint and the W15–W20 packet. Repair with targeted units only
   (AGENTS.md *Execution rhythm*). Refresh development evidence and the W18 seal
   **once**, immediately before the next campaign run — the campaign tooling refuses
   a stale seal, so re-sealing after each intermediate edit buys nothing.
2. Rerun the failed compiler/storage/solver/rule journeys on the repaired source.
   Treat the table above as grouped causes, not proof that every original failure
   has been eliminated. Preserve typed failures, resource limits and independent
   oracles while repairing any newly exposed cause.
3. Complete W19 with a fresh output directory, or use the authenticated continuation
   (`--resume-from`, `--change-reason`, affected `--rerun` gates). Changed declarations
   and shared owners affect native, solver, Python, generation, governance, feature,
   documentation and coverage gates. Never retain an affected prior pass merely
   because it was green. Keep source and docs frozen during each campaign.
4. Qualify strict generation while preserving the shared real index. Eight new Rust
   generated outputs are currently untracked: `artifact_descriptors`,
   `release_checkpoints`, `tear_selection_runs` and `tear_selections` in both model
   and relations runtime directories. The copied-index mode and receipt condition
   are recorded in `codegen-checks-06.json`. **Never export `GIT_INDEX_FILE` across a
   broad campaign:** disposable Git fixtures and build snapshots create their own
   repositories. Use it only for a generation-only continuation after all other
   required gates qualify; failed/unrun gates otherwise rerun automatically.
5. Only after a complete current W19 receipt, run W20:

   ```bash
   CARGO_INCREMENTAL=0 just assessment build/plan13/w20-measurements-01 \
     --plan 13 --phase performance --functional-from <qualified-w19-directory>
   ```

   Complete all 77 performance gates, the isolated cold/edited/feature builds,
   Criterion groups and five end-to-end repetitions. Report absolute values,
   dispersion, work/resource bounds and regressions. Cold-build dispersion remains
   unmeasured after a single cold build. Record independent G1–G7 verdicts before
   declaring the plan complete. Preserve receipts before reclaiming task-owned
   regenerable build output if disk space becomes limiting.

Formal decision/design acceptance remains a separate outstanding requirement.
Existing campaign exclusions—IDAES parity, distribution builds, other platforms and
new solver/Python architecture—remain unchanged. No commit or push is performed here.
