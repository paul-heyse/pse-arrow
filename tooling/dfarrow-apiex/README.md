# dfarrow-apiex

Extracts a source-grounded API corpus for **DataFusion 55.1.0 + Arrow 59.3.0** (pins live in the workspace `Cargo.toml`) from pinned
git worktrees, and renders agent-facing knowledge volumes from it.

It exists because the prose corpus in `docs/capability-maps/` is anchored to a version *claim*
rather than version-pinned *evidence*, and because the pre-existing `agent_reference/`
fact layer is docs.rs-derived, carries no signatures, and has **zero** method or impl records.

## Layout

| Path | Role |
|---|---|
| `extractor/` | Rust binary. `gate` verifies `format_version`; `normalize` turns rustdoc JSON into declaration / trait-contract / impl-relation records. |
| `gen-rustdoc.sh` | Generates rustdoc JSON via an *extraction-root* crate that path-depends on the target packages. |
| `render_v1.py` | Renders V1 — Extension-Point Contracts — from the extracted facts. |
| `verify/` | Compile gate: implements the published extension points against the pinned worktree. |

## Pipeline

```bash
# 0. gate: the toolchain must emit format_version 61 (== rustdoc-types 0.61.0)
cargo +nightly-2026-08-18 rustdoc … && extractor gate <doc.json>

# 1. worktrees pinned at the tags the documentation corpus asserts
scripts/fetch-external.sh        # external/datafusion @ 55.1.0, external/arrow-rs @ 59.3.0 (tags read from Cargo.lock)

# 2. generate rustdoc JSON
./gen-rustdoc.sh external/datafusion df551 datafusion-session datafusion-expr …

# 3. normalize to facts
./target/release/dfarrow-apiex normalize build/rustdoc/df55-default/<crate>.json \
    df55-default build/facts/df55-default

# 4. render volumes
python3 render_v1.py

# 5. compile gate
cd verify && RUSTUP_TOOLCHAIN=nightly-2026-08-18 cargo check --offline --locked
```

## Invariants this enforces

1. **`format_version` must equal `rustdoc_types::FORMAT_VERSION`.** Sniffed before the typed
   parse; the extractor fails closed. A mismatched document deserializes into *something*,
   silently and wrongly.
2. **Rustdoc `Id` is opaque and document-local.** It is never a durable key. Anonymous items
   (impl blocks) are named from a digest of their rendered header instead.
3. **Public is reachability, not `Visibility::Public`.** Routes are built by walking `Use`
   items from the crate root, so re-exported paths are recorded and unreachable `pub` items
   are not counted as public.
4. **Output is deterministic.** rustdoc's `index` is a `HashMap`; records are sorted before
   emission, and an associated item claimed by several blanket impls resolves to a stable,
   real-impl-preferred owner. An unchanged input re-extracts byte-identically.
5. **No machine-specific paths.** Spans are repo-relative; registry paths reduce to
   `registry/<crate>-<version>/…`; attribute text is sanitised.
6. **`#[async_trait]` is re-sugared.** rustdoc records the macro-expanded
   `-> Pin<Box<dyn Future<…>>>` form; the published signature is the `async fn` an
   implementor actually writes.

## Scope and honesty

`evidence/coverage-src.json` carries the denominators. 18 of 49 DataFusion crates and 1 of 28
arrow-rs crates are extracted so far. **Absence from this corpus is not evidence that a
feature does not exist.**

Extraction is rebuild-only: never hand-edit anything under `build/` or the generated
`*.jsonl` / `dfarrow55_*.md` artifacts. Fix the extractor or the renderer and regenerate.
