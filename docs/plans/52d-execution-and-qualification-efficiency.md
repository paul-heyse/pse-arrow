---
title: "Plan 52d: Execution and qualification efficiency"
status: in-progress
date: 2026-10-23
adrs: []
review_sources: [docs/functional_reviews/agent-effectiveness-opportunities-2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, environment, validation, performance, diagnostics]
---

# Plan 52d: Execution and qualification efficiency

## Target and existing owners

**Proposed:** observe actual work without repeated reconstruction, resume expensive checks at
the right unit, avoid configuration-only rebuilds when safe, and explain host pressure.
Owns O15–O17/O20 and I07; [the coordinator](52-agent-effectiveness-and-engineering-workflows.md)
owns state and RC03/RC04/RC07. No new workload scheduler or capacity governor is selected.

Use `activity.py`, `build_observation.py`, `run_inspection.py`, `validation.py`,
`validation_receipts.py`, `validation_scope.py`, `cargo_feature_check.py`, `surreal_source.py`,
`host_admission.py` and `storage_health.py`. Gate-level reuse and stable native result readers
already exist. Current storage-health cgroup fields are predominantly memory observations;
PSI/CPU/I/O/placement interpretation is an addition, not another name for delivered detail.

## Work packages

<a id="D01"></a>
### D01 — Bounded transition watch over exact owners

Add a long-polling watch to existing observation interfaces, selecting an exact invocation,
admission attempt or unit. Its cursor binds selection and observed revisions; it is derived
observer state, not a workflow checkpoint or lease. Return meaningful changed facts and native
result references. Reconnect by reconciling current owners, not replaying guessed intermediate
states. Distinguish running, returned, result complete, observation unavailable/lost and terminal
without protected detail. Process disappearance alone proves neither success nor drain.

Use owner-published snapshots first. [systemd D-Bus signals](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html)
are an optional wakeup optimization; dropped signals still reconcile snapshots. Promise every
transition only if the source actually retained it. Watcher cancellation detaches the watcher,
not the watched work. `just activity` remains independent of workload admission and compiler setup.
A local desktop notification may subscribe to selected terminal events without a messaging service.

Acceptance: actual build→test→terminal, interrupted observer reconnect, unit disappearance without
a terminal, admission unavailable, and environment-shaped failure. Verify that canceling the watch
leaves the run active and that a terminal links its genuine result. Target existing host observer
and result-reader tests plus one representative real consumer. G01 composes this without another
background service or alternate interpretation of terminal truth.

<a id="D02"></a>
### D02 — Exact feature enumeration and combination-level resumption

Extend the existing validation assessment owner with pinned
[cargo-hack0.6.45 enumeration](https://github.com/taiki-e/cargo-hack/blob/v0.6.45/README.md).
Qualify `--print-command-list` decoding as its explicitly unstable interface. Preserve the full
actual depth-two powerset and separate no-default/default/all-feature behavior selected by the
current recipes. Never reconstruct the powerset independently or shell-evaluate emitted text.
Freeze literal argv, tool version, ordering, source/config/native/feature context and per-attempt
completion using existing validation receipt identity.

Execute combinations through `cargo_feature_check` and the existing correctness/environment route.
Retain completed, failed, interrupted, not-run and reused attempts. An interrupted unchanged
campaign resumes its unfinished combinations; a repair initially invalidates the full applicable
Rust-product input closure. Narrower dependency-complete invalidation is a future improvement,
not a premise for first delivery. Preserve original failures and identify every reused result.
Do not call a composed result a fresh all-combination execution.

Keep one Cargo driver for overlapping graphs in one checkout/target. Native partitions do not
isolate dependency units; the documented pinned-Cargo deadlock remains. Distinct checkouts have
their own target directories and are used only when their actual isolation benefit justifies
cold/storage costs. This package does not change sixteen compiler jobs within a driver.

Acceptance: independently verify the pinned enumerated set, interrupt after selected combinations,
resume unchanged input without rerunning those completed, then change a real Rust input and show
correct invalidation. A failed/missing combination prevents full qualification even if every
reported combination passed. FND00's RC03 reuse-contract slice precedes adoption. Pure receipt/
enumeration tests and a small actual matrix establish the mechanism; the full required feature
campaign runs once in H03 after functional scope.

<a id="D03"></a>
### D03 — Safe installation equivalence and rebuild explanation

Resolve I07 before changing reuse in `surreal_source.inputs/installation_name`. Raw Cargo/toolchain
TOML hashes currently select a new custom-server installation/target namespace for a comment-only
change. Compare canonical parsed TOML identity within the installer's real context against this
conservative scheme. Separate the effective build key from retained raw provenance; do not alter
canonical/scientific hashing or qualification evidence identities.

Canonicalization must retain every effective parsed value/key, value type and ordered array,
relative-path/root context, toolchain, upstream revision, exact patch bytes, features, profile,
platform and relevant native/environment overrides. Existing `build_environment.effective_flags`
is useful for its limited precedence rules, not a full Cargo interpreter. Unknown/unsupported
semantics conservatively invalidate. Do not expand this into a general Cargo configuration engine
merely to save a comment-only rebuild.

Acceptance fixtures include comments, formatting/table-key order, ordered flags, changed linker/
relative path, native settings and toolchain/patch changes. A real installer consumer must reuse
an artifact only within the understood equivalence domain; changed executable-affecting inputs
must refuse reuse. If safe equivalence cannot be established, keep raw identity and provide a
clear rebuild reason and explicit reviewed candidate inspection, without silent binary aliasing.
Record I07's supported domain/fallback. Quantitative build-time gains need a measurement; source
inspection alone establishes the avoidable namespace change.

<a id="D04"></a>
### D04 — Pressure, effective placement and selected host experiments

Extend existing observation owners with host/selected-cgroup
[Linux PSI](https://www.kernel.org/doc/html/latest/accounting/psi.html), `cpu.stat`, `io.stat`,
`memory.events`, actual cpusets and ancestor restrictions. Counters/rates retain scope identity,
interval and reset semantics. Missing fields/permission/unavailable scopes remain explicit;
aggregate low CPU is not by itself an explanation. Do not infer causal wait categories from
wall-minus-CPU or combine unrelated process counters.

The observed machine is Ryzen9950X3D, sixteen physical cores/two unequal L3 groups, 192GB RAM,
ext4 NVMe and RTX5090. Existing timing/functional classes already select placement. Show that
actual context rather than apply new defaults. Optional cache-group/thread-width experiments
require a named latency/build question, matched competing-work context and E03/F01 measurement
interpretation. An inner request cannot silently exceed an outer allocation.

Acceptance: a controlled idle wait, observable pressure/throttling and unavailable-field case
produce distinguishable facts with effective placement. A diagnostic environment can simulate
kernel records for pure interpretation tests; an actual bounded observation validates wiring.
Keep device/model observations optional. No automatic topology tuning, global memory limit or
claim that one cache group is universally faster.

## Dependencies and verification

**Proposed:** D01–D04 are independently useful. G01's early vocabulary slice accompanies actual
watch/result consumers; it is not an enclosing prerequisite. D02 needs exact enumeration and
reuse readiness; D03 changes only installation equivalence after its decision. E03/F01 consume
D04 observations where meaningful. Scope-end static/full feature qualification belongs to 52h;
no repeated full powerset is scheduled per package. Delete replaced polling/reuse interpretations
once real callers migrate while preserving useful one-shot snapshots and direct CLIs.


## Execution checkpoint — 2026-10-23

D01/D04 are **Implemented** and **Tested** at their selected boundaries. The isolated executor's
`scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.test_activity scripts.tests.test_activity_watch scripts.tests.test_pressure_observation scripts.tests.test_storage_health`
passed 45 tests against baseline zero. `scripts.tests.plan52_observation_check` passed two actual
controls: real build→test→frozen terminal, watcher interruption/reconnect, finite CPU quota/idle
pressure, direct activity projection and disappearance. Artifacts are in
`/home/paul/.cache/pse52-observation-20261023T1905/`. This is functional wiring evidence, not a
performance benefit or assembled Plan52 qualification claim. No tuning or scheduler was added.

D02 captures literal check argv through the existing Cargo adapter; pinned cargo-hack's displayed
command list loses argument quoting and is not an execution input. The same tool still owns the
complete ordered enumeration. D03's selected target includes one actual new complete-context
custom-server build, followed by safe formatting-equivalence reuse. Both remain in progress.
