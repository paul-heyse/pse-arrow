# Canonical server source integration

`pse_host::server` owns the exact upstream revision, engine/module versions, features,
ordered patches and compiler provenance. The source is
[`crates/pse-host/src/server.rs`](../../crates/pse-host/src/server.rs).
`NativeCanonicalService` and the application own actual launch, schema readiness, connections
and physical settlement; host preparation establishes executable identity only.
ADR-0214 records the storage/transport decision. Current acceptance belongs to
[Plan58](../../docs/plans/58-rust-owned-system.md).

Preparation acquires the exact upstream Git object, verifies and applies the declared patches,
verifies the checksum-pinned DiskANN source, and builds with the repository-pinned toolchain.
It records the explicit release `panic="unwind"` override and executable/lockfile digests.
The compiled version includes the patch identity. Immutable publication selects a completed
native generation; failed or interrupted construction preserves the preceding selection.
There is no released-binary fallback or automatic build on selection.

An explicit `--from-release` import can snapshot a matching established source release into
a new immutable generation. It verifies the original complete receipt, recipe/source/patch/
profile tuple and copied binary/lockfile bytes. Original compiler/environment context remains
provenance; a caller's current loader/cache selection does not fabricate a new build context.
Import does not adopt a persistent service or replace its state. See
[installed server reuse](../../docs/dev/surreal-substrate.md#installed-external-server-reuse).

## Patch responsibilities

The Rust server declaration owns application order; this table explains the retained inputs.

| Input | Reason |
|---|---|
| `mx-1.0-minimal.patch` | Exact engine requirement and the three owned `ByteSlice` to `Vec` return conversions. The minimal stage preserves SI. |
| `mx-1.0-controls.patch` | Explicit SSI, finite memory/age/queue settings, typed nonretrying engine failures, native metrics and revealing adapter tests. Persistence failures preserve unknown commit outcomes. |
| `grpc-connection-ownership.patch` | Authentic accepted TCP identity, connection-bound sessions and bounded cleanup through physical socket loss, beneath plain and TLS serving. Cross-connection session reuse refuses. |
| `grpc-session-admission.patch` | Atomic global session-cap reservation across accepted connections; the permit remains charged until native cleanup completes. |
| `grpc-status-details.patch` | Native structured unary gRPC errors in independently bounded 8 KiB binary metadata. Explicit commit conflicts retain typed retry meaning; generic messages never become conflict evidence. |
| `api-failure-rollback.patch` | Ordinary failing API expressions poison their write transaction so earlier writes roll back; intentional API responses retain their explicit response/commit semantics. |
| `canonical-profile.patch` | Persistent startup requires synchronous AOL, fsync on every append and a valid periodic snapshot interval greater than 30 seconds. Invalid settings cannot silently disable snapshots. |
| `diskann-0.56.0.patch` | Explicit `Arc::as_ref()` at the generic batch call fixes compilation without changing vector-index semantics. The installer verifies the original archive first. |
| `pinned-compiler.patch` | Removes redundant manifest fields, inherits lints, uses identical primitive float constants and the delegated atomic method, and moves `cfg_aliases` to the compatible warning-correcting patch release. |
| `surrealism-locked-build.patch` | Locked Cargo build/metadata and an already declared `.cargo/config.toml`; module compilation does not write project configuration. |
| `surrealism-build-features.patch` | Explicit module feature selections pass from the released CLI into its locked Cargo build. |
| `surrealism-global-controllers.patch` | Process-wide fail-fast controller admission follows the physical WASM Store through idle pooling/cache eviction and returns typed capacity refusal. |
| `surrealism-memory-count.patch` | One linear memory per admitted Store bounds aggregate linear memory by controller count times the per-memory allowance. |
| `surrealism-controller-deadline.patch` | Original context/module deadlines survive queueing, cold instantiation, initialization and invocation; cancellation/zero remaining time are checked before allocation/work. |

The upstream integration is described by the
[MX release](https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0),
[SurrealDB adapter](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs)
and [MX options](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/options.rs).
Compiler compatibility follows the pinned Rust sources: deprecated float constants alias the
primitive constants, and the deprecated atomic spelling delegates with unchanged arguments.

## Explicit native construction

From the checkout root, prepare the independent host first:

```bash
scripts/bootstrap-host
scripts/pse-env -- scripts/pse-host server prepare
scripts/pse-host server select
```

To import an established matching release instead of compiling, explicitly select its receipt:

```bash
scripts/pse-env -- scripts/pse-host server prepare --from-release /absolute/release.json
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

The already selected released server CLI owns WASI compilation/packaging. Production and
qualification module identities remain separate. Module construction starts no database.
The [environment guide](../../docs/dev/agent-environment.md#explicit-compilerklu-preparation)
describes variants and the actual supported preparation boundaries.

## Native service and initialization

A Rust application supplies `NativeServiceSpec` with its prepared server, explicitly built
`pse-canonical-service` helper and `NativeModuleSource::Prepared` capability. The fresh
`NativeCanonicalService` owns private state, credentials, launch, readiness and settlement.
Its initialization is explicit native composition, not a Python setup/start fallback.
`just canonical-test MODULE_RECEIPT` and `just native-test MODULE_RECEIPT ...` own disposable
source-built test fixtures; see the
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
