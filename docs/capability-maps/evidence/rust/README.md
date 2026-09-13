# Rust evidence — arrow 59.3.0 / datafusion 55.1.0 / the §3.3 supporting crates

Reproduction material for the `[rustdoc:…]` and `[probe]` markers in
`../../arrow-rust.md`, `../../datafusion-rust.md` and
`../../supporting-rust-libraries.md`.

**Two independent extractions live here**, each with its own `=`-pinned manifest and
committed lockfile:

| Manifest | Lockfile | Covers | Facts |
|---|---|---|---|
| `apisurface-Cargo.toml` | `apisurface-Cargo.lock` | 20 arrow + 37 datafusion crates | `build/facts/{arrow593,df551}` |
| `support-Cargo.toml` | `support-Cargo.lock` | 26 supporting crates (§3.3) | `build/facts/support` |

This supersedes the material in the parent `evidence/` directory, which described a
13-crate arrow extraction whose output was **not preserved** — every `[rustdoc:]`
marker in the previous edition of both maps was therefore unreproducible. That is the
defect this directory exists to prevent: the lockfile is committed, so the extraction
resolves to the same versions rather than to whatever is current.

## Files

| File | Role |
|---|---|
| `apisurface-Cargo.toml` | Extraction manifest. Every crate in both families `=`-pinned individually. |
| `apisurface-Cargo.lock` | **The resolved graph — 361 packages.** Without this, "re-derivable" means "re-resolvable", which is a weaker claim. |
| `gen_rustdoc.sh` | Generates rustdoc JSON for every target; fails soft per crate and writes a log. |
| `rustdoc_gen.log` | The generation receipt: toolchain, timestamps, per-crate OK/FAIL, output listing. |
| `df_probe.rs` | PROBES A, C, D, E, H — pushdown shapes, proto encoding, float/null semantics, extension-metadata survival, memory-pool failure. |
| `df_probe_b.rs` | PROBE B — `UserDefinedLogicalNode` survival through the optimizer. |
| `df_probe_c3.rs` | PROBE C characterisation — plan-byte stability against the number of field-metadata keys. |
| `df_probe_x.rs` | PROBES X1, X2, X3 — `ExtensionTypeRegistry`, `EXPLAIN` formats, config-validation behaviour. |
| `arrow_probe2.rs` | PROBES 6, 7 — metadata-order effect on canonical IPC bytes; Parquet metadata survival and write determinism. |
| `probe_output.txt` | Captured output of all five programs, as quoted in the maps. |
| `checkmap.py` | The mechanical check the maps claim: marker census, table well-formedness (respecting backtick spans and escaped pipes), contiguous section numbering. Run it against all four maps. |
| `gen_rustdoc_support.sh`, `rustdoc_gen_support.log` | The supporting-crate extraction and its receipt. |
| `probe_identity.rs` | PROBES 1, 2 — blake3 derivation routes and truncation soundness; what `serde_arrow` tracing actually produces. |
| `probe_graph.rs` | PROBE 3 — petgraph SCC order stability, matching direction-independence, feedback arc sets. |
| `probe_numerics.rs` | PROBES 4, 5 — `num-dual`'s implicit function theorem against a hand derivation; faer's matrix-free operator contract. |
| `probe_compile.rs` | PROBES 6, 8 — salsa backdating/accumulators/durability; egglog extraction reproducibility. |
| `probe_parse.rs` | PROBE 7 — `serde-saphyr` and `toml::Spanned` spans and panic-freedom at the authoring boundary. |
| `probe_output_support.txt` | Captured output of the five supporting-crate probes. |

The Arrow-side probes 1–5 remain in the parent directory's `arrow_probe.rs`, unchanged.

## Anchors

```text
arrow family        59.3.0   20 publishable crates extracted (of 23 in the workspace)
datafusion family   55.1.0   37 publishable crates extracted
toolchain           rustc 1.100.0-nightly (809936eac 2026-09-12)
format_version      61
targets             57 requested, 57 OK, 0 FAIL, 60 JSON documents
```

Not extracted, and named rather than silently omitted: `arrow-pyarrow` (requires a Python
interpreter with `pyarrow` at build time, which breaks the offline constraint),
`parquet-geospatial` and `parquet_derive` (both rejected in the Arrow map §10.1).

## Reproduction

```bash
mkdir -p /tmp/apiex/src && cd /tmp/apiex
cp .../evidence/rust/apisurface-Cargo.toml Cargo.toml
cp .../evidence/rust/apisurface-Cargo.lock Cargo.lock     # <- the point
cp .../evidence/rust/gen_rustdoc.sh gen.sh && chmod +x gen.sh
echo '// extraction root' > src/lib.rs
cargo fetch --locked
./gen.sh                      # rustdoc JSON -> target/doc/*.json

# probes
mkdir -p src/bin && cp .../evidence/rust/*probe*.rs src/bin/
cargo run --offline --bin df_probe
cargo run --offline --bin df_probe_b
cargo run --offline --bin df_probe_c3
cargo run --offline --bin df_probe_x
cargo run --offline --bin arrow_probe2
```

To normalize the rustdoc JSON into the queryable `api` / `traits` / `impls` records the
maps cite as `[rustdoc:arrow593@…]` and `[rustdoc:df551@…]`:

```bash
BIN=tooling/dfarrow-apiex/target/release/dfarrow-apiex
$BIN gate /tmp/apiex/target/doc/arrow_schema.json          # must report format_version=61
for f in /tmp/apiex/target/doc/*.json; do
  case $(basename "$f") in
    arrow*|parquet*) prof=arrow593 ;;
    datafusion*)     prof=df551 ;;
    *)               continue ;;
  esac
  $BIN normalize "$f" "$prof" build/facts/$prof
done
```

This writes to **new** profiles. The pre-existing `build/facts/{arrow59-default,df55-default}`
profiles are at **59.2.0 / 55.0.0** — one minor behind both pins — and are never overwritten.
Do not cite them for a claim about the pinned versions.

## Honesty notes — supporting-crate extraction

- **Three targets needed explicit `name@version` specs.** `syn`, `num-dual` and `thiserror`
  each resolve at more than one version in that graph, so `cargo rustdoc -p <name>` fails with
  "specification is ambiguous". That is not a harness defect — it is the `num-dual` version
  split showing up in the build tooling before it shows up in the type checker.
- **Both `num-dual` majors are extracted** (0.14.2 and 0.15.0) so the trait-shape change
  between them can be cited directly. `num_dual.json` is 0.15.0; `num_dual-0.14.2.json` is the
  other.
- **`ipopt-sys` is not extracted.** It needs a native Ipopt install, which is outside the
  offline constraint. Its claims are sourced from `IpStdCInterface.h` at COIN-OR tag
  `releases/3.14.16` instead, and the map says so.
- **`faer::matrix_free::partial_svd` was not executed.** Its dimension preconditions were not
  satisfiable in the harness; the map records the finding as `[UNVERIFIED]` and rests it on
  faer's own documentation plus the measured `BiLinOp` contract, rather than guessing.
- **PROBE 8 corrected a first reading.** The initial comparison used `{:?}` and appeared to
  show egglog extraction as nondeterministic. Comparing `Display` and
  `snapshot_stable_under_proof_encoding` showed the extraction is deterministic and only the
  `Debug` rendering of a hash-set field varies. The probe now reports all three.

## Honesty notes

- **Two probes corrected claims the previous edition made.** PROBE 6 refutes the Arrow map's
  assertion that metadata key order destabilises the canonical IPC encoding (it does not).
  PROBE C locates the real instability in `datafusion-proto`, which blueprint revision 2's
  F9 depends on. Both corrections are recorded in the maps rather than quietly fixed.
- **PROBE A resolves a blocking item, PROBE C opens one.** The `Exact`-pushdown spike that
  blueprint §26 F17 required has been run and the answer is favourable; the plan-fingerprint
  mechanism F9 specifies is not reproducible as written.
- **`arrow-pyarrow` is unmeasured.** The `arrow-pyarrow` vs `pyo3-arrow` decision (Arrow map
  §15 item 5) is still open and this extraction cannot inform the build-complexity half of it.
- Probe programs measure the pinned versions only. A version bump invalidates every quoted
  measurement; re-run before trusting them.
