---
title: "Plan 52e: Reproducible experiments and isolation"
status: in-progress
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, validation, diagnostics, performance, storage]
---

# Plan 52e: Reproducible experiments and isolation

## Target and foundation

**Proposed:** give selected investigations reproducible inputs, owned execution and truthful
historical/comparative outcomes without preserving everything. Owns O18/O19/O21/O22;
[the coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns I08–I10 and RC05/RC08.
Use current invocation/native/resource/reproducer/measurement owners; no new scientific engine
or experiment scheduler. Generated data may be regenerated and ordinary work stays on main.

`test_run.py`/`native_tests.py` already freeze source-bound preparation within one invocation;
that is not cross-run archival. `surrealkit.provision/fixture` owns private namespaces,
credentials and drain, while `validate_input` explicitly refuses seed directories pending
settlement qualification. `build_measurements` already supplies isolated source snapshots and
`case_measure` actual numerical populations. Preserve those contracts.

## Work packages

<a id="E01"></a>
### E01 — Selected historical executable replay

Add explicit archive/inspect/replay composition beside existing test preparation/result owners.
Use [nextest archive and run --archive-file](https://nexte.st/docs/ci-features/archiving/) with
selected binaries, metadata, extra needed files and workspace remapping. An archive excludes
source by default and is not a self-contained operating-system image. Associate source cut,
force-validation, features/toolchain, original selection and declared fixtures. Replay produces
its own current invocation/result with historical producer attribution; it does not qualify
edited source. Ordinary regeneration remains first choice when simpler.

I08 first demonstrates one pure Rust case after the checkout changes. A native branch additionally
represents/verifies actual shared libraries, ABI/admission generation, license/config/data and
runtime environment through native owners. Unsupported/missing native closure refuses replay,
not silently runs a different library. If faithful closure is impractical, record native non-adoption
and regenerate in an owned exact historical checkout for that selected investigation.

Acceptance: inspect before execution; remap a selected fixture; replay independently of current
source; reject missing/incompatible dependencies; retain distinct original/replay outcomes and
owned cleanup. Keep archives only for explicit investigations/current references, using current
artifact protection/release. No automatic archive-every-test or global historical preservation.
FND00 handles any new runtime replay document contract before implementation.

<a id="E02"></a>
### E02 — Representative disposable database populations

Extend the existing SurrealKit resource owner with deterministic small seed populations and
independently verified settlement. Model a few projects/revisions, mixed evidence outcomes,
representative graph fan-out/skew and index states needed for a named query/schema question.
Population size is chosen to expose behavior, not copy production lane/parameter counts.
Use Plan51 project export/import only where its actual scientific/project semantics fit.

I09 selects deterministic generation first. For logical
[export/import](https://surrealdb.com/docs/reference/rust/methods/export), preserve exact schema/
engine context and verify fresh target content; a command exit/empty mutation response is not
proof of settlement. Admit seed operations through the existing fixture lifecycle rather than
remove `validate_input`'s refusal and execute arbitrary SQL. Setup interruption must retain enough
owned state for safe disposition, with new credentials and exact namespace/service generation.

Physical copying is unselected by default. It needs a concrete advantage plus a stopped/drained
dedicated fixture, exact engine/binary and complete required files under
[backup/recovery rules](https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery).
The workstation is ext4; do not assume Btrfs/ZFS snapshots or live-copy an AOF directory.

Acceptance: create two isolated fixtures with identical declared logical content and fresh
credentials, independently read back expected rows/relations/permissions, interrupt setup and
exercise owned cleanup without affecting the other fixture/store. A schema/permission/query-plan
experiment consumes the fixture and compares actual output. Delete redundant ad hoc setup only
where the new route has migrated its real consumers; no new fixture manager.

<a id="E03"></a>
### E03 — Regression and matched-condition experiment driver

Compose strict reproducers, selected tests, existing source snapshotting and measurement owners
around an explicit good/bad or comparison predicate. Use a small
[git bisect run](https://git-scm.com/docs/git-bisect) script in an owned isolated checkout when
history traversal would disturb main, preserving current dirty work. Mark genuinely untestable
revisions with exit125; distinguish setup failure from the tested predicate. Revisions lacking
supported native/schema context are not inferred good or bad.

A deterministic semantic case can use ordinary tests/reproducers before E01 archives or C06
reduction exist. For nondeterministic cases, bounded repetition and
[nextest stress selection](https://nexte.st/docs/features/stress-tests) retain mixed/unknown
outcomes rather than feed a single flaky trial into deterministic bisection. rr may diagnose an
actual selected failure afterward. Keep original failures and new candidates independently named.

Performance comparisons declare pairing/order, input/start/cache/build/native settings and
actual sample membership, using D04 pressure facts when relevant and F01 comparison once available.
Noisy/incompatible data yields inconclusive comparison, not an automatic winner. A source commit
narrowed by a predicate is a causal lead under those premises, not universal scientific blame.

Acceptance: one deterministic semantic regression search with an untestable revision, one
intentionally inconclusive noisy comparison, cancellation/drain and preserved current checkout.
Use existing result/measurement IDs and exact subject correspondence; no second experiment
truth ledger or generic DAG engine. A new experiment kind extends its predicate/source adapter,
not every context formatter.

<a id="E04"></a>
### E04 — Optional disposable tooling environments

Owned private directories plus `pse-env` are the selected default. Resolve I10 with a concrete
upstream/parser/tool-version experiment whose dependency/permission isolation adds value.
[Rootless Podman](https://docs.podman.io/en/latest/markdown/podman-run.1.html), user namespace
handling such as `--userns=keep-id`, read-only source and explicit owned writable output are
candidates; Bubblewrap is lighter when namespace separation alone is sufficient.

Pin retained image/package inputs. Never recursively chown the checkout, share target directories
between checkouts, or make containers mandatory for ordinary development. Start with pure tooling;
solver/systemd/cgroup/device/store transport and licensing require separate actual qualification.
A successful parser container does not qualify the product inside it. Expose environment differences
in the result rather than hiding them behind a generic command wrapper.

Acceptance: an isolated tool experiment reads selected source, writes only owned output, cancels
cleanly and disposes only its own resources without root-owned checkout artifacts. If a private
directory suffices, record no additional sandbox adoption and revisit when a concrete dependency/
namespace conflict recurs. Do not introduce a permanent service/image pipeline for one probe.

## Verification and dependencies

**Proposed:** E01/E02/E04 start at existing owners and their contract slices. E03's deterministic
route needs a stable predicate, not completion of archives, models or the whole database stack.
C06 can simplify later cases; F01 compares retained outcomes. Targeted fixture/closure/driver tests
precede a representative real consumer. 52h owns assembled native/database/Python scope and
scope-end checks. Cleanup success, replay fidelity and scientific outcome remain separate claims.

## Targeted execution outcome — 2026-10-24

**Implemented:** `just experiment` composes selected archive/inspect/replay, semantic Git
bisection, retained-sample comparison and optional confined parser experiments through the
existing invocation, resource and measurement owners. SurrealKit owns named deterministic
population submission and independent settlement. Operator workflows and retention rules live
in the [environment guide](../dev/agent-environment.md#selected-reproducible-experiments) and
[substrate guide](../dev/surreal-substrate.md). No scientific runtime document family or second
experiment scheduler was introduced.

**Tested — E01:** an actual force-validated `pse-admission` authored-package case produced
`build/plan52e-evidence/archive-v4`, including its two declared authored fixtures. Producer
`e0717f5349d8424d8206acf41c83e3a0` passed the one selected test. Actual replay from a remapped
workspace passed after the main checkout changed, with independent invocation
`60928e16dbfd4c519cf260fafe7dc364`; `replay-current-head` passed again after subsequent commits.
The archive/replay commands were the environment guide's concrete `archive`, `inspect` and
`replay` sequence under `scripts/pse-env --resource-class build --`. Active replay references
refused reclamation; an earlier completed archive released its disposable binary detail while
retaining its provenance. The native selection `pse-backend-native` / `highs_incumbents_streamed`
refused before compilation or execution because its deployment/ABI/license/config/data closure
is not archived. This is historical executable evidence, not current-source qualification.

**Tested — E03:** `just experiment bisect --repo <owned-history> --good <good> --bad <bad>
--predicate authoring-load-package --output <new-search> --cleanup` traversed an actual private
nine-commit Git history using the real force-validated authored-package test. It visited good
revision 0, bad 8, bad 4, unsupported 2 (exit125), and good 3, and identified the known first bad
commit `0cbd040d28afed64d47b4fb9a0db325785e6b688`. Four product invocations and their driver scopes
recorded complete drain and zero report errors. Cleanup removed only the experiment's temporary
Git checkout and build output; main, its history and independent worktrees remained intact.
`build/plan52e-evidence/e03-actual-20261023/acceptance.json` retains the actual invocation identities
and visited predicates. The two `experiment compare` controls consumed authentic retained
process-cost samples and returned inconclusive/exit125 for uncontrolled cache, reused sample
membership and missing pressure facts; the stricter noise threshold also detected the observed
spread. This establishes honest refusal, not a performance improvement or universal causal claim.

**Tested — E04:** the optional Bubblewrap0.13.0 parser experiment confined selected read-only
inputs and executable/runtime files, disabled networking and exposed owned output/private
temporary state. Actual valid and invalid server-CLI inputs retained their expected outcomes
with complete process drain and cleanup. The full generated schema exposed the 3.3.4 server
CLI's disabled experimental grammar: its `validate` path does not apply the experimental
capability environment setting. Source inspection selected the already pinned
`surrealdb-syn==3.3.4` parser through `xtask --features surrealql-tools`, using its all-features
parse-only profile. The confined library parser accepted the full generated schema and a
Surrealism module call, and rejected malformed input. It executes no module, file or SQL.
`library-parser-valid` / `library-parser-invalid` retain these separate outcomes. The targeted
Rust command `scripts/pse-env --resource-class build -- .venv/bin/python -m
scripts.arrow_validation nextest run -p xtask --bin xtask --no-default-features --features
surrealql-tools --locked -E 'test(surrealql_tools::tests::)'` passed 3 tests against zero expected
failures after correcting a module fixture to the supported `FROM f"..." UNSIGNED` syntax.
Podman and a permanent image/service pipeline remain unselected; revisit if a concrete consumer
requires a different userspace or dependency environment that private directories/Bubblewrap
cannot provide. Native solvers and database execution inside that sandbox remain unqualified.

**Mistakes corrected:** the first archive producer composition bypassed the selected result
composer, and its child launch did not establish drain. The replacement uses the existing
`execute_selected` composer and drained child scopes. Actual replay also revealed nextest's
required extraction directory and workspace-root manifest; both are now staged explicitly.
Initial failed setup/replay artifacts retain their original partial outcomes rather than
manufactured terminal records. The actual database consumer additionally exposed frozen
supervisor receipt checks incorrectly interpreting build-configuration paths relative to a
runtime snapshot. Verified generations now authenticate their closed build receipt without
weakening main-checkout configuration checks or full launch revalidation. A second initialization
failure exposed a stale caller snapshot rolling back the published supervisor and unit-file
acknowledgement. Failure closure now refreshes authoritative configuration. Lost materialization
acknowledgements require exact private unit bytes and independently drained ownership; the
interrupted initializer's private marker can authenticate an unbound generation for drain only,
without persisting or executing that reconstructed selection.

**Deliberate limits:** deterministic regeneration is selected over logical restore and physical
store copying. Native archival is unselected until a concrete investigation justifies complete
deployment closure; regenerate that exact historical revision through the native owner instead.
No automatic archive-every-test retention, arbitrary seed execution, generic sandbox command
wrapper, automatic comparison winner or production-scale fixture was added. Plan52h retains
assembled qualification, hygiene and manual checks; no whole-series qualification or measured
productivity/performance benefit follows from this focused acceptance.

**Tested — tooling controls:** `scripts/pse-env --resource-class build -- .venv/bin/python
-m unittest scripts.tests.test_test_archives scripts.tests.test_regression_experiments
scripts.tests.test_tooling_experiments scripts.tests.test_surrealkit_population
scripts.tests.test_surrealkit scripts.tests.test_surrealkit_persistent_fixture
scripts.tests.test_surreal_source` passed 88 tests against zero expected failures.
`scripts/pse-env --resource-class build -- just ready` passed after the optional parser
dependency edge and diagnostic tool selection changed. Its cleanup observer separately deferred
three pre-existing resources with unsupported server profiles; it did not release or modify
those historical owners. These are focused tooling/environment checks, not assembled
Plan52 qualification.

The E02 actual fresh-service consumer remains in progress. Its immutable-receipt repair passed
focused controls; actual initialization then exposed stale configuration being written back
after a generation update. That failure and its pending allocation remain owned while the
generation/materialization recovery is repaired; population acceptance is not yet claimed.
