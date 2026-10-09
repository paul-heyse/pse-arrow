# Preparation-key baseline replay

[basis-key-baseline-integration.patch](basis-key-baseline-integration.patch) restores the
pre-cached-prehash `BasisKey` behavior measured by
[basis-key-cost-probe.rs](basis-key-cost-probe.rs). Including the probe alone in the final
implementation would measure a cached-u64 hash callback and construction that computes an Fx
prehash. Those are different boundaries from the original full-field hash and construction.

The patch is a disposable replay change, not a production alternative. It removes the private
prehash field and test-only override, returns the original full-field `Hash` implementation,
and restores construction directly into `Arc<BasisKey>` without computing a prehash. The
existing `size_of::<BasisKey>()` accounting consequently uses the earlier key layout. Equality,
request/dependency/context ownership and durable identities remain unchanged. It includes the
original ignored probe under the existing runtime fixture module. The cached-prehash collision
test remains in the source but is disabled with a replay-only false `cfg`, because its field,
override and eight-byte callback assertions do not apply to this baseline. Reversing the patch
restores that control. Permanent J3 and many-body settlement controls are preserved.

## Provenance and limits

The original measurement ran on 2026-10-09 from dirty `main` based on
`46545b2ad3e2999b4335692ac49af6091438c3fc`, before cached prehash was selected. The terminal
identity was `4658d2faab1f492fbbdb1bbba4c5aa9c`; the recorded command, conditions and operation
boundaries are in [the hashing inquiry](hashing-results.md). The
[raw samples](basis-key-results.json) report 103 hash writes totaling 1,430 bytes, median
full-field hash time 0.535 microseconds and median key construction time 0.270 microseconds.
These are original observations, not targets for a replay or measurements of the final key.

The original complete dirty checkout and executable were not frozen as a standalone artifact.
This patch restores the known hashing/construction behavior in the current followup source;
it does not recreate that executable or its timing environment. Other integrated corrections,
selected Fx factorable interning, tool/lock changes and concurrent host work can affect new
measurements, especially complete warm preparation. Do not present a new run as an exact
artifact reproduction or a matched before/after speedup. Capture its own source revision,
patch state, lockfile/toolchain, execution profile, hash counts and raw samples. Do not
subtract isolated hash timings from complete warm preparation to infer an exact latency share.

The patch was regenerated against the final cached-prehash source after the integrator's
scope-end formatting on 2026-10-09. These file digests identify that formatted input snapshot.
Read-only `git apply --check` passed against it; no patch was applied and no replay workload
was run. Check applicability again against the actual replay revision if the source changes.

| Input | SHA-256 |
| --- | --- |
| `crates/pse-runtime/src/math/preparation.rs` | `8aa6ee64a5c921ac57ce056274609020d57c1c1f807666e373e170d1576d6d70` |
| `crates/pse-runtime/src/workflow/modeling/reuse_tests.rs` | `7e7561047b98f724831f6e5725f3322714cb3f112bba6ba23c0d88d02a18b444` |
| `basis-key-cost-probe.rs` | `e24283335dea2ffb54af6ad4eaf2fa15fd46849c3e869b19e1a6e314ebf1eeeb` |
| `basis-key-results.json` | `a4e1edada78b0045fbf118e04320f03532901a31a560f49a004597dbca5e22ad` |

## Replay procedure

Use an isolated disposable checkout of the integrated followup revision, preserving concurrent
work in the main checkout. Keep the pinned toolchain and exact lockfile. This patch requires
no manifest or lock changes; the final runtime's unused direct Fx dependency can remain.
From the repository root:

```bash
git apply --check docs/design_review/evidence/graph-hash-followups-2026-10-09/basis-key-baseline-integration.patch
git apply docs/design_review/evidence/graph-hash-followups-2026-10-09/basis-key-baseline-integration.patch
just unit-native-package pse-runtime pse-runtime/native-solvers,pse-runtime/canonical-tests graph_hash_basis_key_cost_inquiry --run-ignored only --success-output immediate --profile local --status-level pass
```

The original command additionally used `--final-status-level pass`; that only printed
unselected statuses and is unnecessary here. The recipe supplies the native environment,
locked resolution and explicit force-validation. The probe emits `BASIS_KEY_INQUIRY` with
seven same-process samples: 20,000 hash calls, 20,000 constructions and 20 warm preparation
calls per sample, in that order. Preserve the emitted result as new evidence rather than
replacing the original sample file. Sample ranges are descriptive, not confidence intervals.

Restore the production path before running its cached-prehash acceptance controls:

```bash
git apply -R --check docs/design_review/evidence/graph-hash-followups-2026-10-09/basis-key-baseline-integration.patch
git apply -R docs/design_review/evidence/graph-hash-followups-2026-10-09/basis-key-baseline-integration.patch
```

No replay workload was run when this replay artifact was authored. Current implementation and
acceptance status remain with [Plan 28k](../../../plans/28k-graph-kernels-and-hashing-investigations.md).
