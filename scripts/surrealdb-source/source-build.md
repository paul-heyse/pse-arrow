# Canonical server source integration

`pse_host::server` owns the exact upstream revision, engine/module versions, features,
ordered patches and compiler provenance. The source is
[`crates/pse-host/src/server.rs`](../../crates/pse-host/src/server.rs).
`NativeCanonicalService` and the application own actual launch, schema readiness, connections
and physical settlement; host preparation establishes executable identity only.
ADR-0214 records the storage and transport decision. ADR-0257 and
[Plan68](../../docs/plans/68-surrealdb-v4-cutover.md) own the private v4 candidate.

Preparation acquires the exact upstream Git object, verifies and applies the declared patches,
verifies the checksum-pinned DiskANN source, and builds with the repository-pinned toolchain.
It builds `surreal` and `surreal-ext` together and records the release
`panic="unwind"` override and both executable digests with the lockfile.
The compiled version includes the patch identity. Immutable publication selects a completed
native generation; failed or interrupted construction preserves the preceding selection.
There is no released-binary fallback or automatic build on selection.

An explicit `--from-release` import can snapshot a matching native source-built generation.
Pass its original `.complete.json` alongside `surreal`, `surreal-ext` and `Cargo.lock`.
Import verifies the source, patch order, original build context and each opened file.
Reimport verifies the supplied bytes and reuses an identical immutable selection.
Import does not adopt a persistent service or replace its state. See
[installed server reuse](../../docs/dev/surreal-substrate.md#installed-external-server-reuse).

## Patch responsibilities

The Rust server declaration owns application order; this table explains the retained inputs.

| Input | Reason |
|---|---|
| `surrealdb-v4-runtime-controls.patch` | Canonical MX isolation, bounded persistent startup and typed effects, connection-bound gRPC sessions and cleanup, typed unary errors, and enclosing API rollback. |
| `surrealdb-v4-extension-build.patch` | Locked WASI build and metadata with the caller's explicit feature selection. |
| `surrealdb-v4-extension-runtime.patch` | Process-wide fail-fast admission through physical Store release and the original deadline through invocation preparation. |
| `surrealdb-v4-diskann-resolution.patch` and `surrealdb-diskann-0.61.0.patch` | Select the checksum-verified 0.61.0 source and apply the single explicit `Arc::as_ref()` at the batch call rejected by the pinned nightly. |

The upstream integration is described by the
[MX release](https://github.com/surrealdb/surrealmx/releases/tag/v1.1.0) and
[SurrealDB adapter](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/kvs-mem/src/lib.rs).

## Explicit native construction

From the checkout root, prepare the independent host first:

```bash
scripts/bootstrap-host
scripts/pse-env -- scripts/pse-host server prepare
scripts/pse-host server select
```

To import an established matching release instead of compiling, explicitly select its receipt:

```bash
scripts/pse-env -- scripts/pse-host server prepare --from-release /absolute/generation/.complete.json
```

Neither route starts a catalog. `server select` reads a completed installation without compiling.
Keep the immutable generation/provenance receipt; do not treat a mutable build path as a service
identity. Do not pass `--resource-class compile` just to describe compilation: ordinary work uses
its configured unbounded selection, while explicit finite profiles retain their actual meaning.

The module has its own construction/selection owner:

```bash
just surreal-module
scripts/pse-host study-module select --receipt /absolute/module.json
```

The extension builder selected with the server owns WASI compilation and packaging. Production and
qualification module identities remain separate. Module construction starts no database.
The [environment guide](../../docs/dev/agent-environment.md#explicit-compilerklu-preparation)
describes variants and the actual supported preparation boundaries.

## Native service and initialization

A Rust application supplies `NativeServiceSpec` with its prepared server, explicitly built
`pse-canonical-service` helper and `NativeModuleSource::Prepared` capability. The fresh
`NativeCanonicalService` owns private state, credentials, launch, readiness and settlement.
Its initialization is explicit native composition, not a Python setup/start fallback.
`scripts/pse test` resolves declared fixture inputs and delegates disposable service ownership
to the original Rust fixture owners after source build/capture; see the
[qualification guide](../../docs/dev/validation-assessment.md#individual-commands).

For an existing explicitly owned deployment, the product initializer remains a distinct native
artifact. Build it explicitly with the correctness adapter, then select that state deliberately:

```bash
scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness build \
  -p pse-operations --bin pse-canonical-init --no-default-features --locked
scripts/pse-env -- target/debug/pse-canonical-init --describe
```

`target/debug/pse-canonical-init /absolute/state` performs the admitted initialization protocol;
it is not permission to initialize arbitrary old/user state. The actual deployment/maintenance
owner must supply its private initializer selection and current fences. Description/read paths
do not grant ROOT bootstrap or schema admission. Native maintenance validates exact generated
interpretation/module identity before activation; submitted unknown SQL retains its original
settlement obligation. See
[reconstruction and recovery](../../docs/dev/surreal-substrate.md#explicit-design-phase-reconstruction-and-recovery).

## Historical deep-engine controls and remaining qualification

The retired Python `surreal_mx_check` driver exercised private server gRPC/session caps,
cross-connection refusal, socket loss, synchronous reopen, snapshots, authenticated metrics,
reader release, rollback and acknowledged commit recovery after SIGKILL/incomplete AOL tails.
Its optional strace fault model injected AOL fsync EIO and interrupted snapshot replacement;
corruption and small-catalog-limit controls checked explicit refusal without partial rows.
These are historical control purposes, not currently available Python commands or new native
acceptance. They establish neither device power-loss durability nor broader deployment behavior.

The old Python `fresh-engine`, lifecycle readmission and initializer orchestration are retired.
Do not recreate their destructive effects through a generic shell recipe. Native caller and H
owners must preserve any required exact fault/recovery scenario, authentic unknown outcomes,
physical drain and schema/activation predicates before claiming replacement qualification.
Construction/import and focused native fixtures alone do not discharge those campaigns.
