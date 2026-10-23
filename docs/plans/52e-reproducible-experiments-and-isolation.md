---
title: "Plan 52e: Reproducible experiments and isolation"
status: draft
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
