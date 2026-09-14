---
status: evidence-map
blueprint_revision: 5
pins: Cargo.lock
regenerated: null
reviewed: 2026-09-13
---

# Supporting Rust libraries — capability map

**Current binding:** blueprint revision 5. The original survey and version/probe
receipts below retain their historical scope; they were not regenerated in this
update. Earlier revision comparisons and parked recommendations are historical.
The [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md)
and ADR-0039–ADR-0048 determine the corrected design bindings below. Platform
integration remains **Proposed**; interface and probe evidence are labeled at
their original scope. No Python behavioral corpus was re-audited by this revision.

| Current binding | Decision and limit | Evidence / authority |
|---|---|---|
| Artifact-hash memo | Whole-stage inputs first; finer bundles and salsa require R-22/R-01 evidence | ADR-0040–0042; review R4-06/07 |
| Hash ownership and identity | v2 canonical logical hash and a separate encoded checksum; semantic-ID contexts retained | ADR-0045; review E4 |
| Object storage | Verify existing objects and complete manifest membership before conditional publication | ADR-0045; blueprint §20.1 |
| Kernel-local differentiation | Declared derivative implementation/validity; no inference from scalar signature | ADR-0043/0047; actual backend conformance remains open |

**Companion to** `docs/authoritative_design/blueprint.md` §3.3 (supporting libraries and the boundary each must respect), and the fourth of four capability maps — the other three are `arrow-rust.md`, `datafusion-rust.md` and `python-libraries.md`.

**Compiled** 2026-09-13 against the versions in §1.
**Historical survey adjudicated** blueprint **revision 2** (2026-09-13). The previous edition adjudicated revision 1 and never said so.

---

## 0. Purpose, scope, and evidence rules

### 0.1 What this document is

Blueprint §3.3 names **sixteen supporting-library rows** covering **24 Rust crates** (the seventeenth row is Python and belongs to the Python map), each with a one-line role and a one-line boundary. That is enough to justify a dependency and not enough to build against. This document answers, per crate: **what the library can actually do**, **which of its APIs the blueprint's stated requirements land on**, **what we deliberately will not use**, and **where the blueprint has assumed a capability the library does not provide**.

A second pass asks what these libraries offer that the blueprint has *not* claimed, which would improve alignment with `design_principles/DATA_MODEL_DESIGN_CHARTER.md` (DM-01…DM-60, gates G1–G7). A third (§13) enumerates each library independently and asks what is being left on the table.

*(The previous edition said "twenty-two crates" while its own anchor table had 25 rows. The count above is reconciled against §3.3: 24 Rust crates plus the Ipopt C API, which is not a crate.)*

### 0.2 Evidence rules

Every API name, type, signature or behavioural claim carries a provenance marker. **A claim with no marker is a defect in this document.**

| Marker | Meaning |
|---|---|
| **`[rustdoc:crate@version]`** | **the authoritative lane — locally generated rustdoc JSON from the pinned crates; receipt in §1** |
| **`[probe]`** | **a program compiled against the pinned crates, used where documentation asserts behaviour that can simply be measured** |
| `[c7:/org/project]` | context7, naming the library ID that produced the answer |
| `[docs.rs:crate@version]` | fetched directly from docs.rs — used where context7 has no coverage, or to pin a version |
| `[crates.io:crate]` | registry metadata: release dates, version currency, reverse dependencies |
| `[rustsec:ID]` | the RustSec advisory database |
| `[gh:repo@tag/path]` | upstream source read at an exact tag — used for the Ipopt C headers, which have no crate |
| `[UNVERIFIED]` | no authoritative source found; carried as a risk, never filled in from recall |

The previous edition also declared a `[bp:§n]` marker it never used, and used a `[c7 resolve: …]` form it never declared. Both are gone: blueprint assertions are named in prose, and context7 resolutions are ledger rows.

**The `[ref:]` marker has been retired.** It pointed at `docs/library_ref/`, which has been **deleted**. Every one of the previous edition's 35 `[ref:]` citations has been re-sourced against live evidence or removed with its claim — §0.4 records what that cost.

Status words in the "what the blueprint binds to" tables mean:

- **leverage** — the library provides this; use it directly.
- **build on top** — the library provides a foundation but not the capability; platform code is required and its cost is stated.
- **confirmed** — a specific claim the blueprint makes about a library is true as stated.
- **refined** / **erratum** — the claim is wrong or imprecise; the correction is given and the affected blueprint section named.
- **open item** / **open decision** — a question this lookup surfaced that must be answered before or during implementation.

### 0.3 Status vocabulary

Status words in the "what the blueprint binds to" tables mean:

- **leverage** — the library provides this; use it directly.
- **build on top** — the library provides a foundation but not the capability; platform code is required and its cost is stated.
- **confirmed** — a specific claim the blueprint makes about a library is true as stated.
- **refined** / **erratum** — the claim is wrong or imprecise; the correction is given and the affected blueprint section named.
- **adopt** / **evaluate** / **reject** — Pass-2 and Pass-3 recommendations, disciplined by charter §G (common false positives) and **DM-58**: a recommendation that names no principle and replaces no identified hand-written work does not appear.
- **open decision** — a question this lookup surfaced. §18 gives each one a recommendation and the evidence; the decision itself remains the owner's.

### 0.4 What the deleted reference corpus cost, and what replaced it

The previous edition drew on `docs/library_ref/`, a corpus of local deep-dives. **That directory has been deleted.** Thirty-five citations across eleven files became unresolvable, and with them the sole support for three of the ten recommended blueprint amendments.

Every one has been re-sourced or removed. The outcome:

| Deleted source | Citations | Outcome |
|---|---|---|
| `petgraph.md` | 10 | **re-sourced verbatim** from petgraph 0.8.3 rustdoc, and measured by PROBE 3. Amendment B stands, with one fact the deleted reference did not carry. |
| `ipopt_standalone.md` | 6 | **re-sourced** from `IpStdCInterface.h` at COIN-OR tag `releases/3.14.16`. Every claim confirmed; **amendment H's function names were wrong** and are corrected. |
| `arrow_rust_59_datafusion55_advanced_reference_2026-08-23.md` | 6 | **re-sourced — and the claim did not survive.** The `object_store` trait surface it described does not exist at the pinned 0.13.2. See C9.2 and §18. |
| `rust_codegen_syn_quote_proc_macro2_prettyplease.md` | 4 | **re-sourced** from rustdoc and crates.io. Pins confirmed, one drifted (`syn` 3.0.3 → 3.0.5). |
| `datafusion-tracing.md` | 2 | **re-sourced — and amendment D is currently unsatisfiable.** Both crates exist; neither has a 55.1.0 release to match the pinned engine. |
| `rust_parallel_concurrency_stack_reference_2026-08-19.md` | 2 | **re-sourced** from crates.io. Pins confirmed. |
| `ipopt_solver_ontology.md`, `scip_ipopt_options_and_metrics.md`, `datafusion_calculations_rust.md`, `datafusion_55_arrow_59_2_…narrative`, `rust_mir_cpg_continuous_reference` | 5 | **deleted.** The previous edition described these itself as "mentions only, not a deep-dive" and "a usage, not a deep-dive". No claim rode on any of them. |

Two corrections this exercise forced on the previous edition's own text, beyond the amendments:

1. It claimed "**this cluster produced the document's one blueprint erratum**" (C3, `serde_yaml`) and then issued a second one eleven sections later (C4, petgraph). This edition carries **five**, and counts them.
2. Its `object_store` trait surface was taken from a reference describing a **different version** than the one §3.1 pins. That is precisely the failure mode an unresolvable citation hides.

### 0.5 Coverage

Where the previous edition relied on context7 breadth, this one leads with locally generated rustdoc: **26 crates extracted at the pinned versions**, 190,463 declarations, 589 public traits (§1.5). context7 and docs.rs remain for idiom and intent. Three crates that context7 does not carry at all — `num-dual`, `serde_arrow`, `feos-core` — are now covered by the authoritative lane rather than by fallback.

**One named exclusion:** the Ipopt C API has no crate and needs a native library, so it is sourced from the upstream header at an exact tag `[gh:coin-or/Ipopt@releases/3.14.16]` rather than by extraction.

### 0.6 Version anchoring

Anchors in §1 are pinned exactly and resolved into a committed lockfile, not proposed. MSRV/edition-2024 compatibility against the platform's Rust 1.94 floor (§3.1) is recorded where a crate states one.

---

## 1. Version anchors, the pin gap, and the receipts

### 1.1 The pin gap — the largest single finding in this map

Blueprint §3.2 line 277 states that the workspace manifest "**carries every pin once**". Blueprint §3.1 pins five components: the DataFusion family, the Arrow family, `object_store`, `pyo3`/`pyo3-arrow`, and `tokio`.

Of the **24 supporting Rust crates §3.3 names, exactly one — `tokio` — has a version anchored anywhere in the blueprint.** The other 23 have none. §26 **F11** acknowledges two of the 23 (`serde_yaml`, `num-dual`); the remaining 21 are unrecorded.

That includes **`blake3`**, which is the most deeply wired supporting crate in the design — roughly twenty use sites spanning semantic-ID derivation (§5.1), content hashing (§5.3), hash-consing (§7.4), the plan fingerprint (§14.2) and the manifest (§20.2) — and which is additionally **assigned to no crate in §3.2** and used at **two digest widths** (128-bit IDs, 256-bit content hashes) **with no derivation mechanism named**.

The table below is therefore offered as a §3.1 amendment the blueprint can adopt wholesale. Every version was resolved into the committed lockfile of §1.5, not proposed.

### 1.2 Anchors

| Crate | Pinned here | §3.1 today | Note |
|---|---|---|---|
| `serde` | **1.0.229** | — | substrate; `derive` + `rc` |
| `serde_arrow` | **0.15.0** | — | features `arrow-53`…**`arrow-59`**; no `arrow-60` yet, so an Arrow 60 move waits on this crate `[crates.io:serde_arrow]`. Interposes `marrow ^0.3.0` |
| `syn` | **3.0.5** | — | previous edition said 3.0.3; **`syn 3`**, not the `syn 2` most of the ecosystem still uses |
| `quote` | **1.0.47** | — | |
| `proc-macro2` | **1.0.107** | — | `span-locations` for source spans in generated code |
| `prettyplease` | **0.3.0** | — | matched to `syn 3` |
| ~~`serde_yaml`~~ | **rejected** | — | `0.9.34+deprecated`, last released **2024-03-25**. See C3.1 and §18 Q1 |
| **`serde-saphyr`** | **1.2.0** | — | **recommended replacement** (§18 Q1); released 2026-08-30 |
| `toml` | **1.1.6+spec-1.1.0** | — | 1.x stable; `Spanned` is load-bearing (C3.2) |
| `winnow` | **1.0.4** | — | already in the graph via `toml` |
| `petgraph` | **0.8.3** | — | pre-1.0; pin exactly (C4) |
| `salsa` | **0.28.2** | — | §26 **F13** is a live decision (C5) |
| `egglog` | **3.0.0** | — | optional, phase 4. **Three majors in ten months** (C6) |
| `num-dual` | **0.15.0** | — | **conflicts with `feos-core`** — §1.3 |
| `faer` | **0.24.4** | — | |
| `uom` | **0.38.0** | — | MSRV 1.68 |
| `feos-core` / `feos` | **0.10.1** | — | optional, phase 4; pulls `quantity 0.14.1` and `num-dual 0.14.2` |
| `diffsol` | **0.16.2** | — | optional; pulls `diffsol-la`/`diffsol-nl`, **not `faer`** (C8.2) |
| `blake3` | **1.8.7** | — | `rayon` feature for §18.8's budget (C9.1) |
| `rayon` | **1.12.0** | — | |
| `tokio` | **1.53.1** | **1.52+** | the one anchored crate; consistent |
| `tracing` | **0.1.44** | — | |
| `thiserror` | **2.0.20** | — | |
| `miette` | **7.6.0** | — | **does not depend on `thiserror`** `[crates.io:miette]`, so the 1.x/2.x split is not a concern here |
| Ipopt C API | **≥ 3.14** | — | own `-sys` crate; sourced from `IpStdCInterface.h` `[gh:coin-or/Ipopt@releases/3.14.16]` |

### 1.3 Measured: the `num-dual` split is real, and worse than a duplicate

Resolving the manifest above produces **362 packages with 17 names at more than one version**, and one of them is load-bearing `[probe]`:

```text
num-dual    0.14.2, 0.15.0      <- type-identity hazard
thiserror   1.0.69, 2.0.20
syn         1.0.109, 2.0.119, 3.0.5
```

Reverse dependencies, from the lockfile:

```text
feos-core 0.10.1           ->  num-dual ^0.14   (resolved 0.14.2)
quantity  0.14.1           ->  num-dual ^0.14   (resolved 0.14.2)
<this manifest>            ->  num-dual =0.15.0
```

Three facts follow, and the third is the one that matters.

1. **The split is entirely feos's.** Only `feos-core` and its own `quantity` dependency want 0.14. Nothing else in a 362-package graph does.
2. **`feos-core` 0.10.1 sits one minor behind on both of its key dependencies** — `num-dual ^0.14` against 0.15.0, and `quantity ^0.14` against 0.15.0.
3. **The trait's shape changed between the two, so the kernel *source* differs, not merely the type.** From the extraction `[rustdoc:num-dual@0.14.2]`, `[rustdoc:num-dual@0.15.0]`:

   ```text
   0.14.2   trait DualNum<F>: … NumOps<F> … From<F> … DualStruct<F, Real = F> …
   0.15.0   trait DualNum:    … NumOps<Self::Primitive> … From<Self::Primitive> …
                                DualStruct<Real = Self::Primitive> …
   ```

   §18.5 promises "the same body monomorphized over `num_dual::DualNum`". A kernel written for 0.14 is bounded `D: DualNum<f64>`; the same kernel for 0.15 is bounded `D: DualNum<Primitive = f64>`. **Those are different source files.** The choice is not "which version do we depend on" but "which signature does every kernel body in `pse-kernels` carry".

   A secondary confirmation, from this extraction's own tooling: `cargo rustdoc -p num-dual` **fails outright** with "specification `num-dual` is ambiguous" once both versions are present. The hazard is visible to cargo before it is visible to the type checker.

### 1.4 Supply chain and maintenance

A lane the Arrow and DataFusion maps do not need: those are one well-run Apache project each, whereas this map's surface is 24 independently-owned crates.

**RustSec advisories touching this set — three, all patched below the versions in play** `[rustsec:RUSTSEC-2018-0005]`, `[rustsec:RUSTSEC-2023-0078]`, `[rustsec:RUSTSEC-2024-0358]`:

| Advisory | Crate | Patched | Our version |
|---|---|---|---|
| RUSTSEC-2018-0005 | `serde_yaml` | ≥ 0.8.4 | 0.9.34 — unaffected |
| RUSTSEC-2023-0078 | `tracing` | ≥ 0.1.40 | 0.1.44 — unaffected |
| RUSTSEC-2024-0358 | `object_store` | ≥ 0.10.2 | 0.13.2 — unaffected |

`cargo audit` is therefore clean today. **And that is the finding**, because:

> **`serde_yaml`'s deprecation is invisible to `cargo audit`.** It carries **no RustSec unmaintained advisory**. The deprecation is encoded only in the semver build metadata — `0.9.34+deprecated` — and a repository banner, neither of which any standard tool interprets. It still takes ~90 million downloads a quarter `[crates.io:serde_yaml]`.

So a clean `cargo audit` is not evidence that the dependency set is maintained. §24's supply-chain gate needs **`cargo deny`'s maintenance and ban checks**, not `cargo audit` alone — otherwise the one dependency this map flags as an erratum passes CI silently.

Maintenance recency, worth stating because none of it is an advisory `[crates.io:*]`:

| Crate | Last release | Quiet |
|---|---|---|
| `serde_yaml` | 2024-03-25 | ~30 months — **rejected** |
| `ipopt` / `ipopt-sys` | 2024-12-14 | ~21 months — supports the blueprint's "own `-sys` crate" (C11) |
| `miette` | 2025-04-27 | ~16 months |
| `petgraph` | 2025-09-30 | ~11 months |

### 1.5 Extraction receipt

```text
toolchain       rustc 1.100.0-nightly (809936eac 2026-09-12)
flags           -Z unstable-options --output-format json
format_version  61
targets         25 requested; 3 needed explicit name@version specs (see below)
documents       26 (both num-dual majors extracted, for §1.3)
lockfile        362 packages, committed
facts           build/facts/support — 190,463 declarations, 589 public traits,
                113,713 impl relations
```

Three targets — `syn`, `num-dual`, `thiserror` — could not be documented by bare crate name because each resolves at more than one version; they were re-run as `name@version`. That is §1.3's hazard showing up in the build tooling.

Committed under `docs/capability-maps/evidence/rust/`: `support-Cargo.toml`, **`support-Cargo.lock`**, `gen_rustdoc_support.sh`, `rustdoc_gen_support.log`, the five probe programs, and `probe_output_support.txt`.

### 1.6 Probe receipt

The previous edition made no measurements at all. Five programs, eight probes:

| Probe | Settles | Cluster |
|---|---|---|
| **1** | `blake3` — `derive_key` vs truncation, and whether 16-byte truncation is sound | C9.1, §18 Q4 |
| **2** | `serde_arrow` — what a Rust record actually traces to, and what metadata it carries | C2.1, §18 Q5 |
| **3** | `petgraph` — SCC order stability, matching direction-independence, feedback arc sets | C4, amendment B |
| **4** | `num-dual` — `implicit_derivative` against a hand-derived implicit function theorem | C7.1, amendment G |
| **5** | `faer` — the matrix-free operator contract, and which end of the spectrum it targets | C7.2, §18 Q7 |
| **6** | `salsa` — backdating, accumulators, durability | C5, **F13** |
| **7** | `serde-saphyr` and `toml::Spanned` — spans and panic-freedom at the authoring boundary | C3, §18 Q1 |
| **8** | `egglog` — extraction reproducibility | C6, §18 Q10 |

---

## 2. Code generation — `syn`, `quote`, `proc_macro2`, `prettyplease` (C1)

**Re-sourced.** The previous edition rested this cluster on a 172 KB local deep-dive that has since been deleted. Every claim below is now taken from the pinned crates' own rustdoc `[rustdoc:syn@3.0.5]`, `[rustdoc:quote@1.0.47]`, `[rustdoc:proc-macro2@1.0.107]`, `[rustdoc:prettyplease@0.3.0]` and from the registry `[crates.io:syn]`. One anchor had drifted: `syn` is at **3.0.5**, not 3.0.3.

**Version anchors** `[crates.io:syn]`:

| Crate | Pinned version | Role |
|---|---|---|
| `syn` | `3.0.3` | parse Rust tokens/source into typed syntax trees; inspect, visit, mutate, fold, print |
| `quote` | `1.0.47` | produce `proc_macro2::TokenStream` with interpolation, repetition, identifier construction, span control |
| `proc-macro2` | `1.0.107` | compiler-compatible token trees and spans usable outside proc macros — in `build.rs`, libraries, tests and binaries |
| `prettyplease` | `0.3.0` | pretty-print a `syn 3` `File` into readable generated source |

**Note the major version:** `syn 3` (with `prettyplease 0.3` matched to it), not the `syn 2` most of the ecosystem still uses. Any example found elsewhere may target syn 2; the local reference is the authority here.

#### What the blueprint binds to

| Blueprint requirement | API / pattern | Status |
|---|---|---|
| §4.2 "`pse-schema` runs as a build step (a **non-macro** `syn`/`quote`/`prettyplease` pipeline)" | the non-macro codegen pipeline — `proc_macro2::TokenStream` built with `quote!`, parsed to `syn::File`, emitted with `prettyplease::unparse` | **leverage**; `proc_macro2` exists precisely so this works outside a proc-macro crate `[rustdoc:proc-macro2@1.0.107]` |
| §4.2 "emitted source is **committed** under `pse-relations/src/generated/` and diffed in CI" | `prettyplease`'s "deterministic-enough" formatting contract (§20) | **leverage**, with a caveat: a *formatting* change in prettyplease produces a diff with no semantic change. Pin `prettyplease` exactly and treat its upgrade as a regeneration commit. |
| §4.2 generated views, builders, schemas, validators, providers, migrations, docs, JSON Schema | `quote!` with repetition (§16), `format_ident!` for generated names (§18) | **leverage** |
| §4.2 the generated `try_from(&RecordBatch)` that "checks the schema fingerprint" | ordinary generated code | platform logic, not a library feature |
| §23.2 `authoring.*` failures in generator input | `syn::Error` (§11), `quote_spanned!` for diagnostic span targeting (§19) | **leverage** for *generator* errors; note these are build-time developer errors, distinct from §23.2's runtime failure taxonomy |

#### What we deliberately do not use

- **No procedural macros.** §4.2's parenthetical "(non-macro pipeline)" is a deliberate rejection of the derive-macro model, and it is the right one for this platform: a derive macro hides generated code from review, defeats §4.2's "emitted source is committed and diffed in CI", and makes the `no_shadow_structs.rs` governance check impossible to write against real symbols. The local reference covers derive/attribute/function-like pipelines (§22–§24) — **those sections are explicitly out of scope.**
- **The registry is authoritative, the generator is a projection.** §3.3's boundary. A generator that reads anything but `reference.schema_*` has introduced a second source of truth.

#### Gaps and risks

- `prettyplease` formatting stability across versions is the only real operational risk (above).
- `syn 3`'s feature flags (§7) gate large parts of the AST; enable deliberately — `full` is convenient and slow to compile, and a generator emitting only items may not need it.

---

## 3. Boundary serialization — `serde`, `serde_arrow` (C2)

### C2.1 `serde_arrow` — Rust records ⇄ Arrow batches under a declared schema

**Proposed anchor:** `0.15.0` (released 2026-08-09) `[docs.rs:serde_arrow@0.15.0]`
**Arrow compatibility:** feature flags `arrow-53` … **`arrow-59`**; when several are enabled the highest wins `[docs.rs:serde_arrow@0.15.0]`. The blueprint's Arrow 59.2 pin (§3.1) is therefore supported — **this was the single largest compatibility risk in the cluster and it clears.**
**Role:** §4.2 — a `SerdeArrowSchema` is a generated artifact of the registry, consumed by import/export adapters and the Python contract layer; §1.3 places `serde_arrow` adapters directly beneath the schema registry.

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Schema type | `schema::SerdeArrowSchema` — "a collection of fields as understood by `serde_arrow`" | `[docs.rs:serde_arrow@0.15.0]` |
| Schema construction | sealed trait `schema::SchemaLike` with `from_value()`, `from_type::<T>(TracingOptions)`, `from_samples()` | `[docs.rs:serde_arrow@0.15.0]` |
| **Implementors of `SchemaLike`** | `Vec<FieldRef>`, `Vec<Field>`, and `SerdeArrowSchema` itself | `[docs.rs:serde_arrow@0.15.0]` |
| Arrow conversion | `to_record_batch(fields, items) -> RecordBatch`, `from_record_batch(batch) -> Vec<T>`, `to_arrow(fields, items)`, `from_arrow(arrays)` | `[docs.rs:serde_arrow@0.15.0]` |
| Engine-neutral conversion | `to_marrow`/`from_marrow` over `marrow::array::Array` — the recommended path for *libraries* that do not want to pin an arrow major | `[docs.rs:serde_arrow@0.15.0]` |
| Incremental building | `ArrayBuilder` with `push(item)` / `build()`, plus `from_arrow()`/`from_marrow()` initializers | `[docs.rs:serde_arrow@0.15.0]` |
| Serde plumbing | `Serializer` (wraps `ArrayBuilder`), `Deserializer` | `[docs.rs:serde_arrow@0.15.0]` |
| Type-mismatch escape hatch | `schema::Strategy` enum — "strategies for handling types without direct match between arrow and serde", annotated per field | `[docs.rs:serde_arrow@0.15.0]` |
| Strategy metadata key | `schema::STRATEGY_KEY` — the Arrow **field-metadata key** under which a strategy is stored | `[docs.rs:serde_arrow@0.15.0]` |
| Per-field overrides | `schema::Overwrites` — opaque field-path → field-definition mapping | `[docs.rs:serde_arrow@0.15.0]` |
| Schema tracing control | `schema::TracingOptions` | `[docs.rs:serde_arrow@0.15.0]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §4.2 "`SerdeArrowSchema` derived from the registry (never inferred from samples)" | `SchemaLike` is implemented for **`Vec<FieldRef>`** `[docs.rs:serde_arrow@0.15.0]`, so the generated Arrow `Schema`'s fields *are* the serde_arrow schema — no tracing step exists in the path at all | **confirmed, and cleaner than stated**: the blueprint frames this as a discipline ("never inferred"); the API makes it the natural construction. `from_samples`/`TracingOptions` simply never appear in production code. |
| §4.2 import/export adapters over generated record types | `to_record_batch` / `from_record_batch` with the generated schema's fields | **leverage** |
| §21.5 typed Python contracts across the Arrow boundary | Arrow `RecordBatch` is the handoff; serde_arrow is the Rust-side half only | **leverage**, boundary unchanged |
| Row-at-a-time authoring output (§7.7 parser → `authored` relations) | `ArrayBuilder::push` / `build` | **leverage** — a serde-driven alternative to the generated typed builders, for adapter code only |

#### Measured (PROBE 2) — why §3.3's boundary is not optional

§3.3's boundary for this row reads "Explicit schemas from the registry; **no inference from sample data in production**", and §4.2 says the `SerdeArrowSchema` is "derived from the registry (never inferred from samples)". Measured against a Rust record shaped like a `pse.*` relation row `[probe]`:

```text
PROBE 2  struct Row { id: [u8; 16], v: f64, label: String }
  field id     type = Struct([16 x UInt8 fields named "0".."15"])
               metadata = {"SERDE_ARROW:strategy": "TupleAsStruct"}
  field v      type = Float64      metadata = {}
  field label  type = LargeUtf8    metadata = {}
  round-trip equal : true
```

Two findings, and the first is the more serious.

**Traced types are not the registry's types.** A `[u8; 16]` — the natural Rust shape for a `pse.semantic_id` — traces to a **struct of sixteen `UInt8` fields**, not `FixedSizeBinary(16)`. A `String` traces to `LargeUtf8`, not `Utf8`. Nothing here is wrong of `serde_arrow`; tracing infers from the Rust type, which is what §4.2 forbids. But it makes the boundary concrete: **if anyone ever calls `from_type`/`from_samples` in a production path, the resulting schema is structurally incompatible with §4.4's extension types**, and the failure is a wrong storage type rather than an error. The governance grep §4.2 already implies should name `SchemaLike::from_type` and `from_samples` explicitly.

**And the adapter stamps its own metadata.** The traced field carries `SERDE_ARROW:strategy = "TupleAsStruct"`, which survives into the `RecordBatch`. That is open question 5's concern, confirmed: **a batch that arrived through the adapter carries field metadata a registry-built batch does not.** Since §5.3 hashes the canonical IPC encoding, which includes field metadata, two logically identical relations would hash differently depending on their provenance.

The resolution follows from the first finding rather than needing a new rule: because the registry supplies the schema, no `SERDE_ARROW:*` key should ever appear on a platform relation. **Recommendation for Q5: treat any `SERDE_ARROW:*` key on a relation schema as a contract violation** — a governance check, not a hashing-policy decision.

#### What we deliberately do not use

- **`from_samples` and `from_type` are banned in production paths.** §3.3's boundary ("explicit schemas from the registry; no inference from sample data in production") maps exactly onto never calling these two constructors. A governance grep for `from_samples(`/`from_type::<` outside test code enforces it cheaply — recommend adding it beside the existing `no_shadow_structs.rs` check (§4.2).
- **Not the `marrow` path.** `to_marrow`/`from_marrow` exist so libraries can avoid pinning an Arrow major; the platform deliberately pins Arrow 59.2 workspace-wide (§3.1) and wants the real `RecordBatch`, so the `arrow-59` feature and `to_record_batch`/`from_record_batch` are the right surface.
- **serde_arrow is not the relation writer.** Generated typed builders and views (§4.2) remain the interior path; serde_arrow serves *import/export adapters* at the edges, where the peer is a serde type rather than a relation.

#### Gaps and risks

- **Field-metadata key collision — a real design item.** serde_arrow writes its own key, `STRATEGY_KEY`, into Arrow **field metadata** `[docs.rs:serde_arrow@0.15.0]`, the same namespace §4.3 reserves for `pse.semantic.*` and `ARROW:extension:name`. Two consequences to settle: (1) §4.3's "metadata is a carrier" rule must tolerate a foreign key on fields that pass through serde_arrow; (2) §5.3 step 2 strips *volatile* metadata before hashing — `STRATEGY_KEY` must be classified explicitly as volatile (stripped) or contractual (kept), or two logically identical relations could hash differently depending on whether they arrived through an adapter. **Open decision.**
- **Extension types.** The docs reference support for field metadata, extension types, `FixedSizeBinary` and dictionary encoding, but the status detail lives in a separate support-matrix module that was not fetched. Since every blueprint extension type (§4.4) is backed by `FixedSizeBinary`, `Struct`, `List` or `Dictionary` storage, the risk is low — but the round-trip of `pse.semantic_id` (`FixedSizeBinary(16)`) and `pse.quantity_value` (a struct) through serde_arrow should be a **spike before committing**, with `Overwrites` as the fallback for forcing a field's definition.
- `TracingOptions` and `Strategy` are needed only when a Rust type has no direct Arrow match. If the generated record types are shaped from the registry, this should never fire; a `Strategy` appearing in production is a signal the generated type diverged from the relation.

### C2.2 `serde` — the trait layer underneath

**Role:** §3.3 pairs `serde` with `serde_arrow` for schema-governed conversion at import/export boundaries; it is also the derive layer under `serde_yaml`/`toml` (C3) and appears as an optional feature of `num-dual` (C7) and `uom` (C7).

No capability lookup was performed and none is warranted: `serde` is the Rust serialization substrate, its surface (`Serialize`/`Deserialize`/`derive`) is stable and universally documented, and the blueprint asks nothing unusual of it.

**What matters is the boundary, which is entirely a platform rule:**

- A `#[derive(Serialize, Deserialize)]` struct that mirrors a relation's column list is a **governance failure** under §4.2 ("nothing hand-written may duplicate a column list"), regardless of which crate consumes it. The `no_shadow_structs.rs` check must therefore cover serde-derived types, not only plain structs.
- serde types are legitimate for *documents* (package/case YAML and TOML, §22.1), *manifests* (§20.2 JSON), and *external formats* (IDAES `to_json` v4 import, §20.5) — shapes that are genuinely not relations.

---

## 4. Document loading and the expression DSL — `serde_yaml`, `toml`, `winnow` (C3)

This cluster produced the document's one **blueprint erratum**: `serde_yaml` is no longer maintained.

### C3.1 `serde_yaml` — **deprecated; a replacement decision is required**

**Current release:** `0.9.34+deprecated` `[docs.rs:serde_yaml@0.9.34+deprecated]`. The documentation carries a banner stating verbatim: *"(This project is no longer maintained.)"*

§3.3 names `serde_yaml` for package/case document loading and §22.1 makes YAML a first-class authoring format. Adopting an unmaintained crate for the **authoring boundary** — the platform's outermost trust boundary, parsing user-supplied documents — is the one place where "unmaintained" is not a cosmetic concern: it means no security fixes for a parser fed untrusted input.

**Verified replacement candidates** (existence and coverage confirmed; selection not made):

| Candidate | Positioning | Coverage signal |
|---|---|---|
| `serde_norway` | maintained fork of `serde_yaml`, near-drop-in | `[c7:/websites/rs_serde_norway]` — 5689 snippets, High reputation, benchmark 81.17 |
| `serde-saphyr` | strongly typed, **panic-free** YAML deserializer; converts YAML directly into Rust types with no intermediate `Value` | `[c7:/bourumir-wyngs/serde-saphyr]` — 355 snippets, benchmark 83.96 |
| `serde_yml` | fork of `serde_yaml` under active publication | `[c7:/websites/rs_crate_serde_yml]` — 69 snippets, benchmark 75.22 |

**Recommendation to evaluate, not to adopt blind:** `serde-saphyr`'s two stated properties — panic-free, and no intermediate structure — are exactly what an authoring-boundary parser wants, since §23.2 requires `authoring.parse` to be a *typed failure with a source span*, never a panic. `serde_norway` is the lower-effort path if drop-in compatibility outweighs that. **Open decision requiring a spike**; whichever wins, §3.3's table needs updating.

**Boundary (unchanged by the substitution):** §3.3 — "parser output is `authored` relations; no evaluation during parsing."

#### Measured (PROBE 7) — the replacement decision, settled by evidence

Open question 1 asked which YAML crate replaces `serde_yaml`, and set the criterion itself: §23.2 requires typed failures with spans rather than panics, so span support should decide it. Measured `[probe]`:

```text
PROBE 7b  serde-saphyr 1.2.0
  valid input            : parsed ok
  type error             : rejected, with
       error: line 2 column 10: invalid u32
        --> <input>:2:10
         |
       1 | id: pkg.demo
       2 | version: three
         |          ^ invalid u32
  -- hostile input --
  unterminated flow seq  -> typed Err (no panic)
  tab indentation        -> typed Err (no panic)
  duplicate key          -> typed Err (no panic)
  deep nesting (200)     -> typed Err (no panic)
```

`serde-saphyr` produces a caret diagnostic with line and column out of the box — the shape §23.2 wants, and better than `serde_yaml` offered — and refuses all four malformed inputs with typed errors rather than panics. It also ships a `budget` module for bounded parsing and a `localizer` for message control `[rustdoc:serde-saphyr@1.2.0]`, which matter at a boundary that parses user documents.

Against the alternatives `[crates.io:*]`: `serde_norway` has not been released since **2024-12-21**, so replacing one quiet crate with another quiet crate; `serde_yml` is **explicitly deprecated** in its own description. `serde-saphyr` was released **2026-08-30**.

**Recommendation for Q1: `serde-saphyr` 1.2.0.** The decision remains the owner's; the evidence is one-sided.

And the TOML half, which the previous edition correctly called free and unclaimed `[probe]`:

```text
PROBE 7a  toml::Spanned
  id      span = 5..15   value = "pkg.demo"
  version span = 26..27  value = 3
  source slice at the version span : "3"
  type error span = Some(26..33), rendered with a caret at line 2 column 11
```

Byte offsets that slice back to the source exactly, plus spans on errors. §22.1's `package.toml` and §4.4's `pse.source_span` get this for nothing.

### C3.2 `toml` — document loading with source spans

**Proposed anchor:** `1.1.6+spec-1.1.0` `[docs.rs:toml@1.1.6]` — a **1.x stable** crate, one of only two in this document.
**Role:** §22.1 package layout and §22.2 change sets; the TOML half of §1.3's "load (serde_yaml, toml)".

| Capability | Surface | Provenance |
|---|---|---|
| Deserialization / serialization | `from_str`, `from_slice`, `to_string`, `to_string_pretty`; `Deserializer`, `Serializer` | `[docs.rs:toml@1.1.6]` |
| Document model | `Table`, `Value` | `[docs.rs:toml@1.1.6]` |
| **Source spans** | `Spanned<T>` — "a spanned value, indicating the range at which it is defined in the source" | `[docs.rs:toml@1.1.6]` |
| Features | `parse`, `display`, `preserve_order` | `[docs.rs:toml@1.1.6]` |
| Parser engine | depends on `winnow ^1.0.0` and `toml_parser ^1.1.3` | `[docs.rs:toml@1.1.6]` |

**What the blueprint binds to**

| Blueprint requirement | API | Status |
|---|---|---|
| §4.4 `pse.source_span` = `Struct<document_id, start: UInt32, end: UInt32>` populated from authored documents | wrap authored fields as `Spanned<T>`; its range is the `start`/`end` pair | **leverage — this is free and the blueprint does not claim it.** Authoring provenance for TOML documents requires no custom parsing at all. |
| §23.2 `authoring.parse` / `authoring.reference` failures carrying source anchors | `Spanned<T>` for reference errors found *after* parsing (an unknown path is not a parse error — §7.7 resolves paths in P3, so the span must survive parsing to be reported later) | **leverage**, and it settles a real problem: the span must be captured at load time because the resolver runs much later |
| §22.2 change sets against document regions | `Spanned<T>`, `preserve_order` | **leverage** |

**Asymmetry to resolve:** TOML gets spans for free; the YAML path's span support depends entirely on which replacement crate C3.1 selects. Since §4.4 defines *one* `pse.source_span` type for all authored documents, span support should be a **selection criterion** for the YAML crate, not an afterthought.

### C3.3 `winnow` — the expression DSL parser

**Proposed anchor:** `1.0.4` (released 2026-07-13) `[docs.rs:winnow@1.0.4]` — **1.x stable**; MSRV policy is "the last 6 months of rust releases", no fixed number.
**Already in the graph:** `toml` depends on `winnow ^1.0.0` `[docs.rs:toml@1.1.6]`, so the DSL parser adds no new parser engine — one engine, one version, which is what §3.3's "explicitly not added" discipline wants.
**Role:** §7.7 — parse the authored expression DSL into template expression-graph nodes with unresolved paths and source spans.

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Modules | `ascii`, `binary`, `combinator`, `error`, `stream`, `token`, `prelude`, `_topic`, `_tutorial` | `[docs.rs:winnow@1.0.4]` |
| **Operator-precedence parsing** | `combinator::expression` — "parses an expression based on operator precedence" — with `Prefix` (struct), `Infix` (enum, incl. `Left`/`Right`), `Postfix` (struct), each carrying a binding power | `[docs.rs:winnow@1.0.4 /combinator/index.html]`, `[c7:/websites/rs_winnow_…]` |
| Span capture | `stream::LocatingSlice` wraps the input to track byte offsets; `Parser::span()` and `Parser::with_span()` return the consumed range alongside the output as `Range<usize>` | `[c7:/websites/rs_winnow_…]` |
| Threaded state | `stream::Stateful` for carrying global state (e.g. a document id, an interner) through the parse | `[c7:/websites/rs_winnow_…]` |
| Other stream wrappers | `Partial` (streaming input), `Bytes`, `BStr` | `[docs.rs:winnow@1.0.4]` |
| Error position | `ParseError::offset() -> usize` — "the location in `ParseError::input` where parsing failed"; an offset, not an index, and may equal `input.len()` at EOF | `[c7:/websites/rs_winnow_…]` |
| Core combinators | `alt`, `delimited`, `preceded`, `terminated`, `separated`, `repeat` (+ `Repeat::fold`), `opt`, `dispatch!`, `cut_err` (backtrack → unrecoverable), `trace` | `[docs.rs:winnow@1.0.4 /combinator/index.html]` |
| Result types | `Result`, `ModalResult` | `[docs.rs:winnow@1.0.4]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §7.7 grammar layering `affine → term → factor → unary → primary` to express `+ -` / `* /` / `^` / unary `-` precedence | `combinator::expression` with binding powers: `Infix::Left(5)` for `+ -`, `Infix::Left(7)` for `* /`, **`Infix::Right`** for `^` (right-associative), `Prefix` for unary `-` | **leverage, and a simplification the blueprint does not use.** The EBNF encodes precedence structurally through four nested rules; winnow can express it as a precedence table, which is easier to keep consistent with §7.2's operator catalog. The EBNF stays the normative grammar; the implementation need not mirror its shape. |
| §7.7 "any path that does not resolve is an authoring error with a **source span**" | `LocatingSlice` + `with_span()` producing `Range<usize>` per node | **confirmed** — spans map directly onto §4.4's `pse.source_span` `start`/`end` (`UInt32`), the same shape `toml::Spanned` yields |
| §7.7 "the parser produces nodes with unresolved paths" (no evaluation, no resolution) | plain combinators returning AST nodes | **leverage**; §3.3's "no evaluation during parsing" is a code-review rule, not a library feature |
| §23.2 `authoring.parse` with a position | `ParseError::offset()` | **leverage** |
| §7.7 constructs beyond arithmetic — `sum`/`prod`/`integral` binders, `d(expr)/dident`, `kernel.f(...)`, `if/then/else`, unit suffixes `{K}` | ordinary combinators (`delimited`, `separated`, `dispatch!`) as the **operand** parser handed to `expression` | **leverage** — `expression(operand)` takes an arbitrary operand parser, so the binder and function forms compose cleanly with the precedence table |
| Threading the `document_id` through so every span becomes a complete `pse.source_span` | `Stateful` | **leverage** |

#### What we deliberately do not use

- **Not `Partial`.** Authoring documents are read whole; streaming-input support adds failure modes for nothing.
- **The parser never resolves, evaluates, or type-checks.** §3.3's boundary and §7.7's P3 split — unit inference (§8.3) and path resolution belong to canonicalization, not to the grammar.
- **`cut_err` is a discipline, not an optimization.** Once a construct is unambiguously entered (`sum(` has been consumed), failing must be unrecoverable so the error reports the real problem rather than backtracking into a misleading alternative. This directly serves §23.2's requirement that failures name the culprit.

#### Gaps and risks

- Both `toml` and `winnow` are 1.x with stable APIs — the **lowest-risk crates in this document**. The cluster's risk is concentrated entirely in the `serde_yaml` replacement decision (C3.1).
- winnow's MSRV policy is a rolling window rather than a fixed floor; with the platform on Rust 1.94 this is not a constraint, but it does mean winnow may raise its floor without a major-version bump.

---

## 5. Graph algorithms — `petgraph` (C4)

**Re-sourced and measured.** The previous edition rested this cluster on a 598 KB local deep-dive that has since been deleted — including the *only* support for recommended amendment **B**. Every claim below is now taken from petgraph 0.8.3's own rustdoc `[rustdoc:petgraph@0.8.3]` and from **PROBE 3** `[probe]`. Amendment B survives intact, with one fact the deleted reference did not carry.
**Role:** §12.5 topology closure (`inferred.topology_edges` "built in `petgraph`, persisted as rows"); §15.2–15.3 structural analysis; §14.1 pass-DAG checks; §17.4 topological unit ordering for sequential-modular initialization.

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §15.3 "Block triangularization: Tarjan SCC on the matching-projected digraph (reversed), condensation DAG, topological order" | `algo::tarjan_scc`, `algo::kosaraju_scc`, `algo::condensation`, `algo::toposort` | **leverage** `[rustdoc:petgraph@0.8.3]` |
| §15.3 "topological order made **deterministic** by ordering ties on semantic ID" | `tarjan_scc`/`kosaraju_scc` return `Vec<Vec<NodeId>>` where **"node order inside each component is arbitrary, while SCC order is postorder / reverse topological sort"** | **confirmed necessary, with a sharper reason than the blueprint gives.** The non-determinism is not only in tie-breaking *between* components — it is *inside* each component, where petgraph guarantees nothing. §5.1's "re-running compilation on an unchanged snapshot reproduces identical IDs" therefore requires sorting within every block by semantic ID, not just across blocks. `[probe]` — see the measurement below |
| §15.3 "Connected components for independent subsystems" | `algo::connected_components` | **leverage** |
| §14.1 pass pipeline is a DAG; §12.5 acyclicity checks; §4.2's `acyclic` invariant kind | `algo::is_cyclic_directed`, `algo::toposort` (which returns a cycle error) | **leverage**; note `visit::Topo` "only visits nodes that are not part of cycles", so for possibly-cyclic input use `toposort`'s error or `DfsPostOrder`/SCC `[rustdoc:petgraph@0.8.3]` |
| §17.4 `plan.sequential_modular@1` — "order units topologically with tears" | `toposort` + the platform's tear selection | **leverage** |
| §15.3 **"Maximum matching: Hopcroft–Karp (own implementation; `petgraph` lacks it)"** | see claim audit below | **refined — decision upheld, reason corrected** |

#### Claim audit: "petgraph lacks maximum matching"

petgraph **does** ship maximum matching: `algo::maximum_matching(graph) -> Matching<G>` plus `greedy_matching` and the `Matching` type `[rustdoc:petgraph@0.8.3]`. But the decision to hand-write Hopcroft–Karp is still right, for a stronger reason than the blueprint states:

- `maximum_matching` uses **Gabow's algorithm for general graphs**, documented complexity **O(|V|³)**, auxiliary space O(|V| + |E|), and it **treats the input as undirected** regardless of orientation `[rustdoc:petgraph@0.8.3]`.
- The equation×variable incidence graph (§15.3) is **bipartite**, where Hopcroft–Karp runs in O(|E|√|V|). At process-model scale the difference between O(|E|√|V|) and O(|V|³) is the difference between a usable and an unusable structural check.

**Recommended erratum to §15.3:** replace "`petgraph` lacks it" with "`petgraph` ships only general-graph matching (Gabow, O(|V|³)); the incidence graph is bipartite, so Hopcroft–Karp's O(|E|√|V|) is required."

#### Measured (PROBE 3)

**(a) The matching claim — amendment B — survives verbatim.** petgraph 0.8.3's own rustdoc for `maximum_matching` reads `[rustdoc:petgraph@0.8.3]`: *"Compute the maximum matching using **Gabow's algorithm**… The input graph is **treated as if undirected**. The algorithm runs in **O(|V|³)**. An algorithm with a better time complexity might be used in the future."* Measured `[probe]`:

```text
PROBE 3b  directed eq->var graph, maximum_matching : [(0,3), (1,2)]  size=2
          every edge reversed                      : [(0,3), (1,2)]  size=2
          same matching either way                 : true
```

Reversing every edge changes nothing, which is what "treated as if undirected" means in practice. **The decision to hand-write Hopcroft–Karp stands**, for the stated reason: O(|E|√|V|) against O(|V|³) on a bipartite incidence graph.

One fact the deleted reference did not carry: `maximum_matching` **panics** if `g.node_bound()` is `usize::MAX` `[rustdoc:petgraph@0.8.3]`. Unreachable at realistic sizes, but a panic is not a typed failure (§23.2), so the platform's own implementation should not inherit the habit.

**(b) SCC ordering — the determinism rule is confirmed, with a sharper demonstration** `[probe]`:

```text
PROBE 3a  graph with a 3-cycle {0,1,2}, a 2-cycle {3,4}, and an acyclic tail
  tarjan_scc, insertion order 1 : [[5], [4, 3], [2, 1, 0]]
  tarjan_scc, insertion order 2 : [[5], [4, 3], [2, 1, 0]]
  kosaraju_scc, same graph      : [[5], [3, 4], [0, 1, 2]]
```

`tarjan_scc` is stable across insertion orders — but `kosaraju_scc` returns **the same components in the opposite intra-component order**. The order within a component is therefore an artifact of the algorithm, not a property of the graph, and petgraph guarantees nothing about it. §15.3's rule — order ties on semantic ID — is required, and §5.1's "re-running compilation on an unchanged snapshot reproduces identical IDs" depends on sorting *within* every block, not only across blocks. Swapping one petgraph algorithm for another would otherwise silently change every derived ID.

**(c) A capability §12.5 does not claim: `feedback_arc_set`** `[probe]`:

```text
PROBE 3c  flowsheet-shaped recycle 0->1->2->3->1
  greedy_feedback_arc_set : [(3, 1)]      <- exactly the recycle edge
  toposort before removal : Err(Cycle(NodeIndex(3)))
  condensation            : 4 nodes -> 2
```

A tear set **is** a feedback arc set. §12.5 and §15.3 list tear selection among the hand-written algorithms; petgraph ships `greedy_feedback_arc_set`, and on a recycle it returns precisely the tear. See §13.

#### What we deliberately do not use

- **Graphs never persist.** §3.3's boundary — "graph facts persist in Arrow; matching and DM decomposition are implemented in `pse-structural`". petgraph structures are built per analysis from `compiled.incidence` and dropped; `inferred.topology_edges` is the relation of record (§12.5).
- **petgraph node indices are not identity.** They are artifact-local at best and reused after removal; §5.1's three identity forms admit no fourth. Map `NodeIndex ↔ semantic ID` explicitly at the boundary of every analysis. `[rustdoc:petgraph@0.8.3]`
- **Not `GraphMap` keyed by semantic ID**, tempting as it looks: `GraphMap` implements `NodeCompactIndexable` but the identity mapping is better kept explicit and sorted, since determinism (above) requires a canonical order anyway.
- **Not DM decomposition from petgraph** — it has none; §15.3's coarse Dulmage–Mendelsohn is platform code over the matching, as the blueprint says.

#### Gaps and risks

- petgraph 0.8.3 is pre-1.0 and was last released 2025-09-30 `[crates.io:petgraph]`; algorithms continue migrating toward graph-trait-based implementations, so signatures may shift across minor versions. Pin exactly.
- `maximum_matching`'s docs note "a better-complexity algorithm may be used in the future" `[rustdoc:petgraph@0.8.3]` — worth re-checking at upgrade time, though a bipartite-specialized implementation remains ours regardless.

---

## 6. Incremental compute — `salsa` (C5)

**Proposed anchor:** `0.28.2` (released 2026-08-03) `[docs.rs:salsa@0.28.2]`. MSRV not stated — **open item** against the 1.94 floor.
**context7 coverage:** the richest of any crate in this document — three sources, used for different purposes: `/websites/salsa-rs_github_io_salsa` (the book, for the algorithm and idiom), `/websites/rs_salsa` (docs.rs, for the API and internals), `/salsa-rs/salsa` (the repo).
**Role:** §14.3 — "each pass is a `salsa` query keyed by the content hashes of its declared inputs and its own version… the memo stores the output artifact hashes, which the artifact store resolves"; §14.4 — the invalidation matrix; §2 D14 — incrementality follows declared dependencies.

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Database definition | `#[salsa::db]` on the database struct/trait; `Storage` (the concrete implementation), `DatabaseImpl` (a default), the `Database` trait | `[c7:/websites/rs_salsa]`, `[docs.rs:salsa@0.28.2]` |
| Inputs | `#[salsa::input]` structs; the `Setter` trait for field mutation; `set_with_durability(db, key, value, Durability::HIGH)` | `[c7:/websites/salsa-rs_github_io_salsa]` |
| Tracked queries | `#[salsa::tracked] fn f(db: &dyn Db, key: K) -> V` — salsa records which inputs the body reads and memoizes the result | `[c7:/websites/salsa-rs_github_io_salsa]` |
| Interning | `#[salsa::interned]` — data stored in the database, the struct is a newtyped integer id | `[c7:/websites/salsa-rs_github_io_salsa]` |
| **Backdating** | if a re-executed query returns a value equal to the previous one, the memo's "last changed" revision is backdated, so dependents are **not** invalidated | `[c7:/websites/salsa-rs_github_io_salsa]` |
| Equality requirement | tracked return values must implement `PartialEq` for backdating; `no_eq` opts out | `[docs.rs:salsa@0.28.2]` |
| Durability | `Durability` (e.g. `HIGH`, `NEVER_CHANGE`) — a dependency of durability *D* unchanged since last verification verifies the memo without per-dependency checks | `[c7:/websites/salsa-rs_github_io_salsa]`, `[c7:/websites/rs_salsa]` |
| Memo content | value (or dependencies only), revision last verified, revision value last changed, **minimum durability of dependencies**, complete dependency set or an untracked marker | `[c7:/websites/salsa-rs_github_io_salsa]` |
| **Accumulators** | `#[salsa::accumulator]` — auxiliary outputs (canonically diagnostics) collected while a tracked query runs, "stored alongside the query's memoized result but **do not contribute to that result or its equality**"; `salsa::Accumulator::accumulate(value, db)` | `[c7:/websites/rs_salsa]` |
| Cancellation | `CancellationToken` with `cancel()` / `is_cancelled()`; `Database::cancellation_token()`, `trigger_cancellation()`, `unwind_if_revision_cancelled()`; `Cancelled` as the unwind payload | `[c7:/websites/rs_salsa]`, `[docs.rs:salsa@0.28.2]` |
| Concurrency | queries are claimed per key; a second thread requesting an in-flight query blocks on it rather than duplicating work (`ClaimResult::{Claimed, Running, Cycle}`) | `[c7:/websites/rs_salsa]` |
| Cycles | detected at claim time with recovery via `cycle_fn` / `cycle_result` options on `#[salsa::tracked]` | `[c7:/websites/rs_salsa]`, `[docs.rs:salsa@0.28.2]` |
| Memory control | `lru` option on tracked functions; `Database::trigger_lru_eviction()` | `[c7:/websites/rs_salsa]`, `[docs.rs:salsa@0.28.2]` |
| Escape hatch | `report_untracked_read()` — marks the query for re-execution in every revision | `[c7:/websites/rs_salsa]` |
| Observability | `Event` / `EventKind` notifications; `DatabaseKeyIndex` identifies a query instance; `ingredient_debug_name()` | `[docs.rs:salsa@0.28.2]`, `[c7:/websites/rs_salsa]` |
| Return modes | `returns(ref \| clone \| copy \| deref \| as_ref \| as_deref)` | `[docs.rs:salsa@0.28.2]` |
| Profiling aid | `synthetic_write(durability)` — acts as though an input changed, forcing a new revision | `[c7:/websites/rs_salsa]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §14.3 "each pass is a `salsa` query keyed by the content hashes of its declared inputs and its own version" | `#[salsa::tracked] fn pass_x(db, key: PassKey) -> PassOutput` where `PassKey` carries input content hashes + pass version | **leverage** |
| §14.3 "the memo stores the output **artifact hashes**, which the artifact store resolves" | the `PartialEq`-for-backdating requirement `[docs.rs:salsa@0.28.2]` | **confirmed, and load-bearing.** Storing hashes rather than `RecordBatch` payloads is what makes backdating work: a `[u8; 32]` comparison is exact and O(1), so a pass that re-runs on changed inputs but produces byte-identical output **stops the invalidation cascade dead**. Had the memo held batches, `PartialEq` would be expensive and semantically wrong. The blueprint's choice is validated by the mechanism, not merely compatible with it. |
| §14.4 invalidation matrix ("a value change invalidates only artifacts that depended on that value being a constant", §2 D14) | tracked dependency recording + backdating | **leverage** — the matrix is emergent from what each pass reads, provided passes read through salsa |
| §14.3 "a pass either succeeds with **findings** (severity below error) or fails with a typed failure" | `#[salsa::accumulator]` | **leverage, previously unmapped.** Findings are exactly "auxiliary outputs collected while a query runs that do not contribute to the result or its equality" `[c7:/websites/rs_salsa]` — so accumulating findings does not defeat backdating. The blueprint does not name this mechanism; it should. |
| §6.13 / §23.2 `runtime.cancelled`, lifecycle and cancellation in `pse-runtime` | `CancellationToken`, `unwind_if_revision_cancelled()`, `Cancelled` | **leverage** — note cancellation propagates by **unwinding**, so pass bodies must be unwind-safe and must not leave partial artifacts (which §14.3 already requires: "partial outputs are discarded") |
| §14.3 "independent passes run concurrently on `rayon`" | per-key claiming; a concurrent request for the same query blocks rather than duplicating | **confirmed compatible** — rayon-parallel pass execution will not double-execute a shared pass |
| §3.1/§20 snapshot immutability — "a session pins a snapshot so schemas cannot change mid-query" | `Durability::NEVER_CHANGE` for reference/registry inputs within a snapshot | **leverage** — durability turns per-dependency verification into a single check for the large immutable substrate |

#### Measured (PROBE 6) — what F13 actually turns on

§26 **F13** says `salsa` duplicates artifact-hash memoization and leaves it open. The question is decidable, and the answer is that the two overlap in *purpose* and not in *mechanism* `[probe]`:

```text
PROBE 6  token_count = 3, doubled = 6
         -- input replaced with different text, same token count --
         token_count = 3, doubled = 6      (backdating: `doubled` need not re-run)
         -- accumulator --
         token_count(src2) = 4, accumulated findings = 1
            Finding("suspicious token in \"this is bad \"")
         -- durability: LOW / MEDIUM / HIGH available --
```

**salsa keys on revision counters and durability, not on content hashes.** Its unit is the in-process query; it backdates when a recomputed value is unchanged, so downstream queries are skipped even though the input changed. §14.4's invalidation matrix is expressed entirely semantically and never names salsa — which is precisely why F13 looks like duplication. The two mechanisms answer different questions:

| | salsa | the artifact hash (§14.4, §20.1) |
|---|---|---|
| Unit | in-process query | persisted artifact |
| Key | revision counter + durability | content hash of declared inputs |
| Scope | one process lifetime | across processes and machines |
| Buys | fine-grained skipping, backdating, cycle handling, cancellation | reuse after restart, reproduction (§20.4) |

Neither subsumes the other. The artifact hash cannot skip a sub-computation within a pass; salsa cannot survive a restart. **F13's real question is therefore not "is salsa redundant" but "is intra-pass granularity worth a second memoization system"** — a DM-58 question about demonstrated need, and one §24.3's benchmarks can answer.

Two salsa capabilities the blueprint does not name and should:

- **`#[salsa::accumulator]`** — findings travel alongside a query's value without being part of its return type, which is exactly §23's "a pass either succeeds with findings or fails" shape. Measured above. This is recommended amendment **E**, and the mechanism works as that amendment describes.
- **`CancellationToken`** and **`Cycle`** — §18.3 and §20.1 both specify interruption behaviour, and §14.2's rule engine runs to a fixed point. salsa has first-class machinery for both `[rustdoc:salsa@0.28.2]`.

#### What we deliberately do not use

- **salsa never holds the authored model.** §3.3's boundary ("caches compiler computations, never the authored model") is enforceable and cheap: `#[salsa::input]` holds *content hashes and identifiers*, never relation payloads. The artifact store (§20.1) is the only thing that holds bytes, and salsa memos point into it.
- **Not `#[salsa::interned]` for semantic IDs.** Semantic IDs are already 128-bit opaque values with cross-artifact meaning (§5.1); re-interning them into database-local integer ids would create a second identity space that dies with the database — precisely the artifact-local-ordinal mistake D4 separates out.
- **`report_untracked_read` is a defect, not a tool.** Any pass that reads the artifact store or the filesystem outside a declared input makes itself re-execute every revision and silently voids §14.4. Recommend a governance check that it appears nowhere in `pse-compiler`.
- **Not salsa's cycle recovery for the rule engine.** §14.2's fixed-point executor iterates *DataFusion plans* to a fixed point with provenance rows; that is a different mechanism at a different layer from salsa's query-graph cycle recovery. Keep them unconflated — though `cycle_fn`/`cycle_result` remain available should the pass DAG itself ever need it (it should not; §14.1 declares a DAG).

#### Gaps and risks

- **MSRV unknown** against the 1.94 / edition-2024 floor — verify at `cargo add`.
- **API churn.** salsa 0.28.x is pre-1.0 and has been restructured substantially (the current generation is macro-attribute based; older tutorials describe a different query-group API). Pin exactly, and treat any web-sourced salsa example as possibly pre-dating this generation.
- **Accumulators are feature-gated** (`feature = "accumulator"` guards appear in the source) — enable deliberately if findings ride this path.
- **Unwind-based cancellation** interacts with FFI: a cancellation unwinding through the Ipopt C-API callback boundary (C11) is undefined behaviour. The solve driver must check cancellation between callbacks, never unwind through one. **Design rule to record.**
- **Durability assignment is a design decision the blueprint has not made:** which inputs are `HIGH`/`NEVER_CHANGE` (reference packages, the schema registry within a snapshot) versus low (case values). Getting this right is most of the performance benefit. **Open item.**

---

## 7. Equality saturation (optional, phase 4) — `egglog` (C6)

**context7 ID:** `/egraphs-good/egglog` (115 snippets, benchmark 60.25) — the thinnest coverage of any crate here that context7 does cover.
**Status in the blueprint:** optional, **phase 4** (§25); §7.4 canonicalization is otherwise hand-written.
**Role:** §3.3 — "equality-saturation rewrites under explicit type/domain guards", bounded to "only rewrites justified by physical type, domain, and numerical policy".

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Engine | `EGraph::default()`; equality saturation fused with Datalog — successor to `egg` | `[c7:/egraphs-good/egglog]` |
| Program surface | `parse_and_run_program(None, "(datatype Expr (Num i64) (Add Expr Expr))")` — datatypes, sorts, constructors, rules and schedules are written in **egglog's own S-expression language** | `[c7:/egraphs-good/egglog]` |
| Native Rust rules | `add_ruleset(eg, name)`, `rust_rule(eg, name, ruleset, vars![…], facts![…], closure)` — a rule whose action is a Rust closure, with `ctx.value_to_base::<T>()`, `ctx.add(...)`, `ctx.union(...)` | `[c7:/egraphs-good/egglog]` |
| Declarative rules | `(rule ((Add a b)) ((union (Add a b) (Add b a))) :name "commutativity")` | `[c7:/egraphs-good/egglog]` |
| Running | `run_ruleset(&mut eg, "name")`; `(run n)`; `(run-schedule (saturate …) …)` for staged saturation | `[c7:/egraphs-good/egglog]` |
| Reading back | `eg.update(\|fs\| …)` with `fs.add(...)`, `fs.eclass_of("Num", 9_i64)`; extraction of the best term under a default or **custom cost model**, and retrieval of multiple variants | `[c7:/egraphs-good/egglog]` |
| Container sorts | `(sort MathVec (Vec Math))` for vector-valued arguments | `[c7:/egraphs-good/egglog]` |
| Deletion / subsumption | `(delete …)`, subsumption rulesets | `[c7:/egraphs-good/egglog]` |

#### Fit assessment against the blueprint

| Blueprint requirement | Assessment |
|---|---|
| §7.4 canonicalization rewrites over the math IR | **capable** — the IR's operator catalog (§7.2) maps onto egglog datatypes and constructors, and `rust_rule` allows a guard to be evaluated in Rust (checking §8 physical type, domain, numerical policy) rather than expressed in egglog's language |
| §3.3 "only rewrites justified by physical type, domain, and numerical policy" | **`rust_rule` is the mechanism that makes this possible.** A pure declarative egglog rule cannot consult the quantity registry; a Rust-closure rule can. If egglog is adopted, **every** rewrite that touches units, domains, or numerics must be a `rust_rule`, not a `(rule …)`. |
| Extraction back to a canonical form | **capable** — a custom cost model is the natural place to encode the blueprint's canonical-form preference (§7.4) and numerical-conditioning policy |
| §5.1 determinism — "re-running compilation on an unchanged snapshot reproduces identical IDs" | **unverified and the key risk.** Equality saturation explores to a fixed point and extraction picks a minimum-cost term; whether ties break deterministically across runs is not established by the fetched material. Since §5.1 IDs are derived from the expression structure, a non-deterministic extraction changes IDs and breaks reproduction (§20.4). **This must be settled before adoption, not after.** |

#### Measured (PROBE 8) — extraction *is* reproducible; the trap is elsewhere

Open question 10 asked whether egglog's extraction is deterministic across runs, because §5.1 identity and §20.4 reproduction would both depend on it. Five identical runs of the same program, compared under three renderings `[probe]`:

```text
PROBE 8  5 identical runs -> distinct results by rendering
  Debug   ({:?})                          : 5
  Display ({})                            : 1
  snapshot_stable_under_proof_encoding    : 1
  extracted term : (Mul (Add (Var "x") (Num 2)) (Var "y"))
```

**The extraction is deterministic.** What varies is the `Debug` rendering of the internal `TermDag`, whose `nodes` field is a hash set — so its iteration order changes per process. The extracted term itself, and egglog's own `snapshot_stable_under_proof_encoding`, are identical every time.

This is the same trap the companion DataFusion map found in `datafusion-proto`'s field metadata, and the same one the Arrow map found in `Schema` metadata: **hash iteration order leaking into a rendering that something downstream might hash.** The rule generalises — if any egglog output ever reaches an ID or a content hash, it must go through `Display` or `snapshot_stable_under_proof_encoding`, never `{:?}`.

`[UNVERIFIED]` — one process, one egglog version. Cross-process and cross-version stability are not measured, and §26's phase-4 timing makes the cross-version question the live one: egglog has released **three majors in ten months** (0.5 → 1.0 in 2025-10, 2.0 in 2026-02, 3.0 in 2026-08) `[crates.io:egglog]`, so whatever is assessed now will not be the API phase 4 meets.

#### The structural cost, stated plainly

egglog requires the IR to be **re-encoded into egglog's own datatype/sort language** `[c7:/egraphs-good/egglog]`. That is precisely the "second model language" the blueprint rejects elsewhere (§13.6 for `diffsol`, D6 for DataFusion `Expr`). The encoding is mechanical and can be generated from §7.2's operator catalog — but it is a real, maintained translation layer with its own drift risk, and the blueprint's §3.3 boundary line does not acknowledge it.

**Recommendation:** keep egglog exactly where the blueprint puts it — phase 4, optional, behind a feature flag — and make adoption conditional on two spikes: (1) determinism of extraction, (2) a generated encoding from the operator catalog so the two representations cannot drift. Hand-written canonicalization (§7.4) must remain complete on its own; egglog is an optimization, never a dependency of correctness.

---

## 8. Numerics — `num-dual`, `faer`, `uom` (C7)

The cluster carrying the most blueprint weight and the least prior coverage. `num-dual` and `faer` are both fallback-lane crates (§0.3).

### C7.1 `num-dual` — generic forward-mode derivatives inside kernels

**Proposed anchor:** `0.15.0` (released 2026-08-12) `[docs.rs:num-dual@0.15.0]`
**Role:** §18.5 kernel adapters — "the same body monomorphized over `num_dual::DualNum` types"; §18.2 step 5 — kernels contribute second derivatives as hyper-duals; §9.8 — FeOs SAFT/cubic models are written over `num-dual` and so derivatives arrive natively.

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Generic numeric trait | `DualNum` — the bound a kernel body is written against so one body compiles for `f64` and for every dual type | `[docs.rs:num-dual@0.15.0]` |
| First derivatives | `Dual`/`Dual64`/`Dual32`; vector forms `DualVec`/`DualVec64`, `DualSVec` (static), `DualDVec` (dynamic) | `[docs.rs:num-dual@0.15.0]` |
| Second derivatives | `Dual2`/`Dual2_64`, `Dual2Vec`/`Dual2SVec`/`Dual2DVec`; `HyperDual`/`HyperDual64` and `HyperDualVec` for mixed partials | `[docs.rs:num-dual@0.15.0]` |
| Third derivatives | `Dual3`/`Dual3_64`, `HyperHyperDual`/`HyperHyperDual64` | `[docs.rs:num-dual@0.15.0]` |
| Driver functions | `first_derivative`, `second_derivative`, `third_derivative`, `gradient`, `jacobian`, `hessian`, `partial_hessian` | `[docs.rs:num-dual@0.15.0]` |
| **Implicit differentiation** | `implicit_derivative`, `implicit_derivative_binary`, `implicit_derivative_vec`, and an `ImplicitDerivative` struct for models closed by an external solver | `[docs.rs:num-dual@0.15.0]` |
| Partial application | `partial()`, `partial2()`, `partial3()` to bind extra (parameter) arguments before differentiating | `[docs.rs:num-dual@0.15.0]` |
| Dimension-generic helpers | a `Gradients` trait for gradient/Hessian over static or dynamic dimension | `[docs.rs:num-dual@0.15.0]` |
| Optional integrations | `nalgebra`, `ndarray`, `serde`, `numpy`/`pyo3` feature gates; a `linalg` module over dual-valued matrices | `[docs.rs:num-dual@0.15.0]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §18.5 "the same body monomorphized over `num_dual::DualNum` types" — one kernel body, several derivative adapters | write the body as `fn f<D: DualNum<f64>>(…) -> D`; instantiate at `f64`, `Dual64`, `HyperDual64` `[docs.rs:num-dual@0.15.0]` | **confirmed** — the crate is built precisely around this pattern |
| §18.2 step 5 "kernels contribute second derivatives when available (`num-dual` hyper-duals)" | `HyperDual`/`HyperDual64`, `Dual2`; `hessian` / `partial_hessian` drivers | **confirmed** |
| §18.2 step 4 "implicit kernels apply the implicit function theorem on the selected branch" | `implicit_derivative*` + `ImplicitDerivative` — the crate already implements IFT for solver-closed models | **confirmed, and stronger than the blueprint assumed**: this is library capability, not platform code to write |
| §9.8 FeOs derivatives "natively" | FeOs is written over the same `DualNum` bound (C8) | **confirmed** — see C8 |

#### Measured (PROBE 4) — the implicit function theorem is library capability

§9.8's Helmholtz route and §18.2 step 4's implicit kernels both need the derivative of a root that is defined implicitly. Measured against a cubic-EOS compressibility root, `Z³ − Z² + (A − B − B²)Z − AB = 0`, differentiating the vapour root with respect to the attraction parameter `[probe]`:

```text
PROBE 4  solved Z(A0)          = 0.023366337826   residual = 0.000e0
         dZ/dA by hand (IFT)   = -0.014355061869
         implicit_derivative   = -0.014355061869
         agrees with hand IFT  : true
         second_derivative f'' : -1.859802
```

`num_dual::implicit_derivative` reproduces the hand-derived implicit function theorem exactly. **Recommended amendment G is confirmed by measurement**: this is library capability, not platform code to write, and the blueprint should say so where §18.2 step 4 says "implicit kernels apply the implicit function theorem on the selected branch".

The surrounding family is wider than §3.3's "forward-mode and hyper-dual derivatives" suggests: `ImplicitFunction` and `ImplicitDerivative` give the structured form, `implicit_derivative_binary`/`_vec`/`_sp` the multivariate forms, and `hessian`/`partial_hessian` the second-order forms §18.2 step 5 needs `[rustdoc:num-dual@0.15.0]`.

**One API note that follows from §1.3.** The probe's kernel body is bounded `D: DualNum<Primitive = f64>`. On num-dual 0.14 — the version `feos-core` forces — the same body is bounded `D: DualNum<f64>`. This is what "the kernel source differs between the two pins" means concretely.

#### What we deliberately do not use

- **No whole-problem differentiation.** §3.3's boundary ("kernel-local; never a dense dual vector over the whole problem") is confirmed to be the right call for a second reason the blueprint does not state: `num-dual` is **forward-mode only** `[docs.rs:num-dual@0.15.0]`. Forward mode costs one sweep per independent variable, so a dense dual over all solver variables would be O(n) sweeps per residual evaluation. The per-equation **reverse** adjoint sweep of §18.2 step 4 is ours to write; `num-dual` cannot supply it.
- **Not the `nalgebra`/`ndarray` integrations by default.** The evaluation program owns its layouts (§3.3, D11). Feature-gate these off unless a kernel genuinely needs matrix-valued duals; `gradient`/`hessian` signatures are expressed over `OVector`/`OMatrix`, so using the drivers pulls `nalgebra` in — kernels that want to avoid it differentiate with the scalar types directly.

#### Gaps and risks

- MSRV / edition-2024 compatibility not stated in the fetched docs — **open item**, verify at `cargo add` against MSRV 1.94.
- Third-derivative types exist but the blueprint never asks for them; do not adopt speculatively.
- The `linalg` module over dual-valued matrices is a possible shortcut for small implicit kernels — noted, not committed.

### C7.2 `faer` — sparse and dense linear algebra for diagnostics and workspaces

**Proposed anchor:** `0.24.4` (released 2026-06-24) `[docs.rs:faer@0.24.4]`
**Role:** §15.4 `num.condition_number`; §15.5 SVD toolbox; §3.3 "sparse LU/QR, dense SVD, condition estimates; solver workspaces".
#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Modules | `linalg`, `sparse`, `mat`, `col`, `row`, `diag`, `perm`, `stats`, `matrix_free`, `io`, `utils`, `prelude` | `[docs.rs:faer@0.24.4]` |
| Dense types | `Mat<T>`, `MatRef<'_, T>`, `MatMut<'_, T>` | `[docs.rs:faer@0.24.4]` |
| Dense decompositions | `llt()`, `lblt()` (block LDL), `partial_piv_lu()`, `full_piv_lu()`, `qr()`, `col_piv_qr()`, `svd()`, `thin_svd()`, `singular_values()`, `self_adjoint_eigen()`, `eigen()`, `self_adjoint_eigenvalues()`, `eigenvalues()` | `[docs.rs:faer@0.24.4]` |
| Sparse types | `SparseColMat`, `SymbolicSparseColMat`, `SparseColMatRef`; `Triplet`; `try_new_from_triplets(nrows, ncols, entries) -> Result<Self, CreationError>` with sorting, dedup and symbolic construction handled | `[c7:/sarah-quinones/faer-rs]` |
| Sparse solvers | `faer::sparse::linalg::solvers::{Llt, Lu, Qr}` | `[docs.rs:faer@0.24.4]` |
| Solve traits | `Solve`, `SolveLstsq`; factor-level `solve_in_place_with_conj(conj, rhs, par, stack)` | `[docs.rs:faer@0.24.4]`, `[c7:/sarah-quinones/faer-rs]` |
| Explicit memory control | `MemStack` via the `dyn_stack` module — factorization and solve take a `&mut MemStack` | `[docs.rs:faer@0.24.4]`, `[c7:/sarah-quinones/faer-rs]` |
| Threading | `Par` enum passed per operation | `[docs.rs:faer@0.24.4]` |
| Matrix-free operators | `matrix_free` module for operator-defined linear algebra | `[docs.rs:faer@0.24.4]` |
| Tuning | `Auto` trait for algorithm hyperparameters | `[docs.rs:faer@0.24.4]` |
| Numerical robustness claim | state-of-the-art Cholesky (LLT/LDLT/Bunch-Kaufman), QR ±col pivoting, LU partial/full pivoting, SVD, eigendecomposition; fused kernels for memory-bound paths | `[c7:/sarah-quinones/faer-rs]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §15.4 `num.condition_number` = `‖J‖_F·‖J⁻¹‖_F` "via `faer` sparse LU" | `sparse::linalg::solvers::Lu` + `Solve` | **build on top.** faer ships **no condition-number estimator** `[docs.rs:faer@0.24.4]`. `‖J‖_F` is a direct pass over the values; `‖J⁻¹‖_F` as written requires solving against all `n` unit vectors — O(n) sparse solves per diagnostic. See risk below. |
| §15.4 "pseudo-inverse for non-square" | `sparse::linalg::solvers::Qr` with `SolveLstsq` | **leverage** — least-squares solve is the supported non-square route |
| §15.4 "a 2-norm estimate via the smallest singular values" | dense `singular_values()` / `svd()` | **leverage** for small; see §15.5 row |
| §15.5 "dense `faer` SVD for small problems, iterative for large" | `svd()` / `singular_values()` cover *small*. For *large*, **`matrix_free::eigen::partial_svd` does ship** — but it computes the singular values of **largest** magnitude by its own documentation `[rustdoc:faer@0.24.4]` | **refined.** The previous edition said no iterative SVD exists; one does, aimed at the wrong end of the spectrum. §15.5 wants the smallest `k`, which needs shift-invert over the sparse LU |
| §3.3 "solver workspaces" | `MemStack` (`dyn_stack`), `Par` | **leverage** — faer's explicit-workspace API is exactly the "derived execution layouts, not model truth" boundary, and `Par` is the knob §18.8's thread budget must own |
| §15.3 incidence built from `compiled.incidence` rows | `Triplet` + `try_new_from_triplets` | **leverage** — triplets are the natural landing shape for Arrow incidence rows (row ordinal, column ordinal, value) |

#### Measured (PROBE 5) — the iterative SVD route §15.5 leaves unnamed

§15.5 says "dense `faer` SVD for small problems, **iterative for large**" and names no implementation; §26 **F17** lists the iterative SVD route as open. faer 0.24.4 does have a matrix-free tier — `faer::matrix_free` with `conjugate_gradient`, `bicgstab`, `lsmr`, and `eigen::{partial_eigen, partial_self_adjoint_eigen, partial_svd}` `[rustdoc:faer@0.24.4]` — so the question is whether it answers F17.

The operator contract works and is small `[probe]`:

```text
PROBE 5  LinOp::apply matches a direct product    : max|diff| = 0.000e0
         BiLinOp::transpose_apply matches A^T * y : max|diff| = 0.000e0
```

Five `LinOp` methods plus three `BiLinOp` methods, and a sparse Jacobian never has to be materialised. That is the right shape for §15.5 operating on a Jacobian the evaluation program already knows how to apply.

**But it does not answer F17 on its own.** faer's own documentation for `partial_svd` says it computes *"an estimate of the singular values … with the **largest** magnitude"* `[rustdoc:faer@0.24.4]`. §15.5 wants the **smallest** `k` singular values — rank deficiency below a tolerance, and the small-singular-value/vector pairs that identify the culprit variables and equations. That is the opposite end of the spectrum, and a Krylov method reaches it only through shift-invert, i.e. by applying `A⁻¹`, which requires a sparse factorisation.

So the answer to F17's iterative route is **two components, not one**: `faer`'s sparse LU to apply the inverse, and `matrix_free` to iterate against it. Recording that is more useful than "use `matrix_free`", which would have been the wrong shortcut.

`[UNVERIFIED]` — `partial_svd` was not executed. Its dimension preconditions (`max_dim < min(m, n)` against a `v0` of length `ncols`) were not satisfiable in this harness, and the finding above rests on the crate's documentation plus the measured operator contract rather than on a completed run.

#### What we deliberately do not use

- **No model truth in faer types.** §3.3's boundary holds: `SparseColMat` instances are per-diagnostic scratch built from `compiled.incidence`/Jacobian values and discarded; the sparsity pattern of record is the Arrow relation, not the `SymbolicSparseColMat`.
- **Not faer's own parallelism defaults.** Every call takes `Par` explicitly; §18.8 requires the platform, not the library, to choose.

#### Gaps and risks

- **Condition number cost.** The IDAES-parity definition (Frobenius norm of the inverse) is inherently O(n) solves. Either accept it as an opt-in expensive diagnostic, or add an estimator (e.g. Hager–Higham 1-norm estimate) as platform code. **Open decision** — the blueprint states the formula but not the budget.
- **Iterative SVD: present, but aimed the wrong way.** §15.5's "iterative for large" does have a faer entry point — `matrix_free::eigen::partial_svd` — which targets the **largest** singular values while §15.5 needs the smallest. Options: implement Lanczos bidiagonalization over `matrix_free`, add a dedicated crate, or restrict the SVD toolbox to problems small enough for dense SVD and say so. **Open decision.**
- **Pre-1.0 API churn.** 0.24.x has no stability guarantee; faer has restructured its API across recent releases. Pin exactly and treat upgrades as a reviewed change, exactly as §3.1 treats the Arrow family.
- context7 coverage for faer is thin (20 snippets), so this inventory is mostly docs.rs-sourced; API shapes below module level should be re-checked at implementation time.

### C7.3 `uom` — compile-time dimensional checking inside kernels

**Proposed anchor:** `0.38.0` (released 2026-06-23), MSRV rustc 1.68.0 `[docs.rs:uom@0.38.0]` — comfortably under the platform's 1.94 floor.
**Role:** §3.3 "compile-time dimensional checks inside selected Rust kernels", with §8's dynamic registry authoritative.

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Custom system definition | `system!` macro — declares base quantities, base units and dimension symbols; `quantity!` declares a quantity and its units against that system; `unit!` adds a unit to an existing quantity | `[c7:/iliekturtles/uom]` |
| Core type | `Quantity<D, U, V>` — dimension, units, value storage | `[docs.rs:uom@0.38.0]` |
| Storage types | `f32`, `f64`; all signed/unsigned integer widths; `bigint`/`biguint`; `rational`/`rational32`/`rational64`/`bigrational`; `complex32`/`complex64` | `[docs.rs:uom@0.38.0]` |
| Construction / conversion | `Quantity::new::<unit>(value)`, `get::<unit>()`, dimension-checked arithmetic that yields the derived quantity type | `[c7:/iliekturtles/uom]` |
| Formatting | `into_format_args(unit, DisplayStyle::Abbreviation)`, `uom::fmt` | `[c7:/iliekturtles/uom]` |
| Parsing / serialization | `str` module for string-slice handling of quantities; optional `serde` feature (off by default) | `[docs.rs:uom@0.38.0]` |
| Cost model | dimensional analysis entirely at compile time, zero runtime cost | `[docs.rs:uom@0.38.0]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §3.3 "compile-time dimensional checks inside selected Rust kernels" | `Quantity<D, U, V>` with SI type aliases inside a kernel body | **leverage**, kernel-local only |
| §8 dynamic registry stays authoritative | — | **confirmed as a necessity, not a preference.** uom resolves units *only* at compile time and provides **no dynamic/runtime unit lookup** `[docs.rs:uom@0.38.0]`. It structurally cannot host §8's user-extensible quantity registry, where packages add quantity types at authoring time. |

#### What we deliberately do not use

- **uom never sees a user-defined quantity.** Package-authored quantity types (§6.2, §8.1) exist only at runtime; they have no `uom` type and must not acquire one.
- **uom is not the unit-inference engine.** §8.3's inference runs over the dynamic registry against `pse.dimension_vector`; uom checking inside a kernel is a second, independent guard on hand-written code — a belt-and-braces check, not the mechanism.

#### Gaps and risks

- **Dimension-basis mismatch.** `pse.dimension_vector` (§4.4) is 8-dimensional — the seven SI base dimensions plus **currency**. uom's pre-built SI system does not carry currency, so any kernel mixing cost with physics cannot be typed by the stock SI system; it would need a custom `system!`. Since costing (§19.5) is relational rather than kernel-resident, the practical answer is likely "uom covers physics kernels only, costing stays outside it" — **open item to confirm**.
- Deciding *which* kernels get uom typing is unresolved; §3.3 says "selected" without a selection rule. Propose: kernels whose inputs are fixed SI quantities and whose bodies are hand-written (EOS, correlations), not generated adapters.

---

## 9. Domain providers (optional) — `feos-core`/`feos`, `diffsol` (C8)

Both are optional capability providers. This cluster contains the document's most consequential compatibility finding.

### C8.1 `feos-core` — native Helmholtz/SAFT/cubic thermodynamics

**Proposed anchor:** `feos-core 0.10.1` (released 2026-07-24) `[docs.rs:feos-core@0.10.1]`. context7 has **no coverage** — `FeOs` resolves to unrelated libraries `[c7 resolve: FeOs]` — so this section is docs.rs-sourced.
**Role:** §9.8 — "optional provider crate under `KernelSpec` for SAFT and cubic models written over `num-dual`, giving derivatives natively"; §3.3 — "capability providers under `KernelSpec`; coverage and conventions validated, not assumed".

| Capability | Surface | Provenance |
|---|---|---|
| Model decomposition | `Residual` and `IdealGas` traits separating residual and ideal-gas Helmholtz contributions; a `Total` trait combining them | `[docs.rs:feos-core@0.10.1]` |
| Equation of state | `EquationOfState` struct — "an ideal gas model and a residual Helmholtz energy model" | `[docs.rs:feos-core@0.10.1]` |
| Property evaluation | `State` — encapsulates a thermodynamic equilibrium state and computes properties from it | `[docs.rs:feos-core@0.10.1]` |
| Phase equilibrium | `PhaseEquilibrium`, `PhaseDiagram`, `PhaseDiagramHetero`, `SolverOptions` | `[docs.rs:feos-core@0.10.1]` |
| Differentiation | depends on **`num-dual ^0.14`**; `IdealGasAD` trait for parameter sensitivity | `[docs.rs:feos-core@0.10.1]` |
| Dependencies | `nalgebra`; optional `ndarray`, `rayon`, `rusqlite` | `[docs.rs:feos-core@0.10.1]` |

#### ⚠ Compatibility finding: `num-dual` version split

**`feos-core 0.10.1` depends on `num-dual ^0.14` `[docs.rs:feos-core@0.10.1]`, while the current `num-dual` is `0.15.0` `[docs.rs:num-dual@0.15.0]`.** For a 0.x crate these are semver-**incompatible**, so Cargo will resolve *two* copies of `num-dual` into the graph if `pse-kernels` takes 0.15 and `pse-kernels-ext` takes feos-core.

This is not a build-size annoyance — it is a **type-identity failure of exactly the kind §3.1 forbids for Arrow majors**. `DualNum` from `num-dual 0.14` and `DualNum` from `0.15` are distinct traits over distinct types. §18.5's whole premise ("the same body monomorphized over `num_dual::DualNum` types") and §9.8's "giving derivatives natively" both assume one dual-number vocabulary spanning platform kernels and FeOs models. Two copies break that silently at the type level.

**Resolution options, in order of preference:**
1. Pin `num-dual` to **`0.14`** workspace-wide so FeOs and platform kernels share one vocabulary. Cost: forgo 0.15 additions — importantly including whatever changed in the `implicit_derivative` family (C7.1).
2. Adopt a newer `feos-core` if one exists that tracks `num-dual 0.15` — **verify at `cargo add`**; 0.10.1 was the version read here.
3. Keep them separate and accept that FeOs kernels are opaque at the `KernelSpec` boundary, converting values at the edge rather than sharing dual types. This costs the native-derivative benefit that motivates FeOs in the first place.

**Add `num-dual` to §3.1's pin table** and add `cargo tree -d` coverage for it alongside the existing Arrow check. This is the actionable item of the cluster.

#### Boundary

§3.3 and §9.8 already say it: FeOs is a capability provider under `KernelSpec`, its coverage (property kinds, conventions, reference states) is declared per kernel and validated by parity tests. Nothing about `State`, `PhaseEquilibrium` or `EquationOfState` becomes a platform type — §9.5's phase equilibrium and §6.x material relations stay authoritative, with FeOs supplying kernel evaluations only.

**Gap:** the `Cargo` feature list and any Python-binding feature were not enumerated in the fetched content — **open item**, minor.

### C8.2 `diffsol` — native DAE trajectory backend

**context7 ID:** `/martinjrobins/diffsol` (396 snippets). Version not pinned during this pass — **open item**.
**Role:** §13.6 — "a native DAE integrator fed directly from the evaluation program (residual and Jacobian callbacks); it is a backend binding; the IR is unchanged"; §18.7 — an option for `DAE_INTEGRATE`; §25 phase 4.

| Capability | Surface | Provenance |
|---|---|---|
| Problem construction (closures) | `OdeBuilder`; `Problem::new_implicit_closure(rhs, jacobian_vector_product, init)` | `[c7:/martinjrobins/diffsol]` |
| Problem construction (traits) | `NonLinearOp::evaluate(&self, y, t)`, `NonLinearOpJacobian::evaluate_jacobian_product(&self, y, t, v)` for custom RHS structs | `[c7:/martinjrobins/diffsol]` |
| Mass matrix / DAE | `LinearOp` trait with `state_size`, `apply`, `apply_transpose`; builder support for a mass-matrix-vector product | `[c7:/martinjrobins/diffsol]` |
| Solvers | ODE and **semi-explicit DAE** solvers, adaptive step size, event handling, sensitivity analysis | `[c7 resolve: diffsol]` |
| Optional DSL | **DiffSL**, a JIT-compiled model DSL | `[c7 resolve: diffsol]` |

#### Fit assessment

| Blueprint requirement | Assessment |
|---|---|
| §13.6 "fed directly from the evaluation program (residual and Jacobian callbacks)" | **supported** — the closure and trait routes both take Rust callbacks `[c7:/martinjrobins/diffsol]` |
| §3.3 / §13.6 "no second model language" | **confirmed achievable, and it requires a deliberate choice.** diffsol ships DiffSL, a JIT'd model DSL — adopting it would be exactly the second model language the boundary forbids. The closure/trait path avoids it entirely. **State this explicitly when adopting: DiffSL is not used.** |
| §18.2's sparse per-equation Jacobian feeding diffsol | **fit check needed.** diffsol's nonlinear interface asks for a **Jacobian-vector product** (`evaluate_jacobian_product`), not a sparse Jacobian. The evaluation program produces sparse rows (§18.2 step 4), from which a JVP is a straightforward multiply — but it is an adapter, and whether diffsol can also consume an explicit sparse Jacobian (for its implicit solvers' linear algebra) was not established. **Open item before committing.** |
| §13.4/§13.6 "semi-explicit DAE" scope | diffsol advertises ODEs and **semi-explicit** DAEs `[c7 resolve: diffsol]`. Whether the discretized process models of §13 always fall in that class — versus general index-1 implicit DAEs — is a modelling question the blueprint should answer before promising `DAE_INTEGRATE` on this backend. **Open item.** |
| §18.7 alternatives | PETSc TS through the NL backend remains the other route; keeping both is the blueprint's stated position and this lookup gives no reason to change it |

**Boundary:** unchanged — a backend binding lowered from the IR. `nalgebra` types (`DVector`, `DMatrix`) appear in diffsol's trait signatures, so they enter at the backend edge only and must not propagate into `pse-numerics`' own layouts (D11).

---

## 10. Identity and storage — `blake3`, `object_store` (C9)

### C9.1 `blake3` — content hashing and derived identity

**Proposed anchor:** `1.8.7` `[docs.rs:blake3@1.8.7]`. A 1.x crate — the only post-1.0 dependency in this document, and the only one whose API can be treated as stable.
**Role:** §5.1 derived entity IDs (`blake3_128(domain ‖ …)`); §5.3 `content_hash = blake3(ipc_bytes)` and `snapshot_id`; §4.3 `pse.contract.fingerprint`; §20.2 manifest hashes; §4.4 the `pse.content_hash` extension type (`FixedSizeBinary(32)`).
**Re-sourced** from `blake3` 1.8.7's own rustdoc `[rustdoc:blake3@1.8.7]` and measured by **PROBE 1** `[probe]`.

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Regular hash | `Hasher::new() -> Self`, `update(&mut self, input: &[u8]) -> &mut Self`, `finalize(&self) -> Hash` (idempotent) | `[c7:/websites/rs_blake3]`, `[docs.rs:blake3@1.8.7]` |
| Keyed hash (MAC/PRF) | `Hasher::new_keyed(key: &[u8; 32]) -> Self` | `[c7:/websites/rs_blake3]` |
| Key derivation | `Hasher::new_derive_key(context: &str) -> Self` — "the context string should be hardcoded, globally unique, and application-specific" | `[c7:/websites/rs_blake3]` |
| Extendable output | `finalize_xof(&self) -> OutputReader` — arbitrary-length output | `[docs.rs:blake3@1.8.7]` |
| Multithreaded hashing | `update_rayon(&mut self, input: &[u8]) -> &mut Self` — **`rayon` feature** | `[docs.rs:blake3@1.8.7]` |
| Memory-mapped hashing | `update_mmap(path)`, `update_mmap_rayon(path)` — **`mmap` feature** (the latter also needs `rayon`) | `[docs.rs:blake3@1.8.7]` |
| Streaming from a reader | `update_reader(&mut self, reader: impl Read) -> Result<&mut Self>` | `[docs.rs:blake3@1.8.7]` |
| State inspection / reuse | `count(&self) -> u64`, `reset(&mut self) -> &mut Self` | `[docs.rs:blake3@1.8.7]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §5.3 step 5 `content_hash = blake3(ipc_bytes)` over a serialized IPC stream | `Hasher::new()` + `update` per buffer, or `update_reader` over the IPC writer's output | **leverage** — incremental `update` means the IPC bytes never need to be materialized as one contiguous buffer |
| §5.3 hashing large relation artifacts | `update_rayon`, `update_mmap_rayon` | **leverage**, with a caveat: these consume the `rayon` pool, which §18.8 says is budgeted. Hashing during a solve must not steal the solver's threads. |
| §5.1 domain-separated derived IDs — `blake3_128("pse:symbol:v1" ‖ parent ‖ …)` | two options, and **the blueprint does not say which**: (a) `Hasher::new()`, update with the domain-tag string then the components, `finalize()`, truncate to 16 bytes; (b) `new_derive_key("pse:symbol:v1")` — the crate's own documented mechanism for an application-specific, globally unique context string | **design decision.** Option (b) is what the `derive_key` mode exists for and gives domain separation by construction rather than by convention. Option (a) is what the blueprint's notation literally describes. Pick one and state it, because the two produce different bytes and the choice is frozen forever by §5.1's "survives forever" contract. |
| §5.1 / §4.4 128-bit `pse.semantic_id` from a 256-bit function | `finalize_xof()` reading 16 bytes is the *documented* way to get a shorter digest; truncating `finalize()`'s 32 bytes is the common way | **partially unverified.** The fetched `Hasher` page does not state that prefix-truncation below 32 bytes is safe `[docs.rs:blake3@1.8.7]`. BLAKE3's extendable output is designed so any output length is a valid hash, but this should be confirmed against the specification before freezing, since §5.1 IDs are permanent. **Open item.** |
| §4.3 `pse.contract.fingerprint` = blake3 of the `RelationSpec` rows | same as §5.3 | **leverage** |

#### Measured (PROBE 1) — which 128-bit derivation, and is truncation sound?

§5.1 derives semantic IDs as `blake3_128("pse:named:v1" ‖ package_id ‖ qualified_name)` and never says how the 128 bits are obtained. There are two routes and they do not agree `[probe]`:

```text
PROBE 1  material = "pse:named:v1|pkg-01|flowsheet.heater.inlet"
  hash(material)[..16]              = 221090e3e5fabe979c6d73507758c9b7
  derive_key(ctx, material)[..16]   = d19bb5f9d890907fab83f83e9cbb59a1
  the two routes agree              : false
  finalize_xof -> 16 bytes          = 221090e3e5fabe979c6d73507758c9b7
  xof(16) == hash[..16]             : true
  Hasher::new_derive_key == derive_key : true
  different context -> different bytes : true
```

Two things follow.

**Truncation is sound, and it is not an accident.** BLAKE3 is an extendable-output function: asking the XOF for 16 bytes returns exactly the first 16 bytes of the 32-byte digest. Taking a prefix is therefore a defined operation with the security of a 128-bit digest, not a hopeful shortening. That closes the "is 16-byte truncation sound?" half of the open question.

**But the two routes give different bytes, so §5.1 must name one.** The recommendation is **`derive_key(context, material)`**, because §5.1 is already doing domain separation by hand — prefixing `"pse:named:v1"`, `"pse:symbol:v1"`, `"pse:equation:v1"` and so on to the hashed material. `derive_key` is BLAKE3's purpose-built mechanism for exactly that, and the probe confirms a different context yields different bytes. Concatenating a prefix into the message achieves the same end less explicitly, and leaves the separator's encoding as an unstated part of the identity contract.

Either choice is defensible; **not choosing is not**, because §5.1 IDs are permanent and the two routes are irreconcilable after the fact.

#### What we deliberately do not use

- **Not keyed hashing.** `new_keyed` is a MAC — identity here must be reproducible by anyone holding the same content (§20.4 reproduction), so a secret key would defeat the purpose.
- **Not `Hash`'s constant-time equality for ordinary lookups.** `blake3::Hash` compares in constant time (an anti-timing-attack property); semantic IDs are not secrets, and hot paths that index by ID should use the raw `[u8; 16]`/`[u8; 32]` inside Arrow `FixedSizeBinary` rather than round-tripping through `Hash`.

#### Gaps and risks

- **Canonicalization, not hashing, is the hard part.** §5.3's five preparatory steps (PK ordering, batch concatenation, dictionary expansion, volatile-metadata stripping, NaN canonicalization with `-0.0` preserved) are entirely platform code over Arrow; blake3 contributes only step 5. Test the canonicalizer, not the hash.
- Feature flags `rayon` and `mmap` are off by default — enable deliberately and record them in the workspace dependency table.

### C9.2 `object_store` — the artifact store

**Version note — settled, and the previous edition's API description was wrong.** The pin question is closed: DataFusion 55.1.0 **requires** `object_store ^0.13.2`, so **0.13.2 is not a choice but a constraint**, and 0.14.1 must not be used. Revision 2's §3.1 records this correctly; the companion DataFusion map carries the reasoning.

**But re-sourcing this cluster produced an erratum.** The previous edition described the trait surface from a local reference that has since been deleted — and that reference described a **different version**. At the pinned **0.13.2**, `object_store` does not have the API the previous edition listed. Version 0.13.0 reshaped the trait: `copy` and `copy_if_not_exists` were merged into `copy_opts`, `delete` became `delete_stream`, `rename` became `rename_opts`, and `put`/`get`/`head` became `put_opts`/`get_opts` `[docs.rs:object_store@0.13.2]`.

This is exactly the failure mode an unresolvable citation hides: a confident API list, checked against nothing, describing a version the design does not use.

**Role:** §20.1 artifact store layout, atomic and conditional writes, ref update protocol; §20.2 manifest.

#### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| **Trait surface at the pinned 0.13.2 — ten methods** | required: `put_opts`, `put_multipart_opts`, `get_opts`, `copy_opts`, `delete_stream`, `list`, `list_with_delimiter`; provided: `get_ranges`, `list_with_offset`, `rename_opts` | `[docs.rs:object_store@0.13.2]` |
| **Methods that do *not* exist at 0.13.2** | `put`, `get`, `head`, `delete`, `copy`, **`copy_if_not_exists`**, **`rename`**, **`rename_if_not_exists`** — all removed or merged in the 0.13.0 reshape | `[docs.rs:object_store@0.13.2]` |
| Put modes | `enum PutMode { Overwrite (default), Create, Update(UpdateVersion) }` — all three confirmed present at the pin | `[docs.rs:object_store@0.13.2]` |
| Conditional-write token | `UpdateVersion { e_tag: Option<String>, version: Option<String> }` | `[c7:/apache/arrow-rs-object-store]` |
| Put entry point | `async fn put_opts(&self, location: &Path, payload: PutPayload, opts: PutOptions) -> Result<PutResult>` — "atomic; either all data is written or the operation fails, preventing partial writes"; `PutResult` carries `e_tag`, `version`, extensions | `[c7:/apache/arrow-rs-object-store]` |
| Error signals | `Error::Precondition` (version mismatch), `Error::AlreadyExists` (`Create` on an existing object), `Error::NotModified` (conditional fetch) | `[c7:/apache/arrow-rs-object-store]`, `[docs.rs:object_store@0.14.1]` |
| Backends supporting conditional put | `LocalFileSystem`, `InMemory`, `AmazonS3`, `GoogleCloudStorage`, `MicrosoftAzure` | `[docs.rs:object_store@0.14.1]` |
| Local atomicity | `LocalFileSystem`: "all operations are atomic, and readers cannot observe partial and/or failed writes" | `[docs.rs:object_store@0.14.1]` |
| Optimistic-concurrency idiom | read → compute → `put_opts(PutMode::Update(version))` → retry on `Precondition` | `[c7:/apache/arrow-rs-object-store]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §20.1 "`object_store` provides atomic single-object writes and conditional updates (`PutMode::Update` with the ref's version)" | `put_opts` + `PutMode::Update(UpdateVersion { e_tag, version })`, `Error::Precondition` on mismatch | **confirmed, exactly as stated** |
| §20.1 ref update: `refs/<name>.json` compare-and-swap | the documented optimistic-concurrency loop; `PutResult.e_tag` from the previous write is the token | **leverage** |
| §20.1 idempotent encoded objects | `PutMode::Create` plus verified existing-object checksum/length/format | **Proposed integration:** AlreadyExists is successful only after verification; a name alone does not prove integrity (ADR-0045) |
| §20.1 "the local single-writer mode is the initial implementation" | `LocalFileSystem`, documented atomic | **confirmed** |
| §20.1 mmap-able IPC artifacts | `get_range` / `get_ranges` for footer-then-body reads; note `object_store` returns bytes, it does not hand out a mapping — §5.4's "answered only from cached metadata" statistics path needs the footer range, which `get_range` provides | **leverage**, with the caveat that "mmap-able" describes the *file format* choice, not an `object_store` capability |
| §20.3 / §20.4 reproduction reading pinned manifests | `get_opts` with conditional fetch, `NotModified` | **leverage** |

#### What we deliberately do not use

- **Never a rename as a commit primitive** — and at 0.13.2 the temptation is smaller than it was, because **`rename` no longer exists**; only `rename_opts` does `[docs.rs:object_store@0.13.2]`. The rule still holds and still belongs in §20.1: a rename is a copy-plus-delete on object stores, not an atomic move, so a commit must be a conditional write to the final path (`PutMode::Create` for content-addressed artifacts, `PutMode::Update` for the ref). §20.1's protocol already does this; stating it prevents someone "optimising" it into a temp-file-and-rename.

  **Recommended amendment J is therefore refined, not retracted:** the advice is right, but it should name the mechanism (`PutMode`) rather than a method (`rename`) that the pinned version does not have.
- **The snapshot protocol stays platform code.** §3.3's boundary holds: `object_store` gives single-object atomicity; multi-relation consistency comes from the write-artifacts → write-manifest → CAS-the-ref ordering, which is ours.

#### Gaps and risks

- **The API erratum** (above) is the actionable item in this cluster: any §20.1 pseudocode or prototype written against `put`/`get`/`copy_if_not_exists` will not compile at the pinned version.
- **S3 multi-writer coordination.** The docs reference S3 needing conditional-write support or DynamoDB-based commit coordination for some use cases `[docs.rs:object_store@0.14.1]`; the detail was not pinned down. Since §20.1 starts single-writer local, this is deferred — but it must be settled before any multi-writer or S3 deployment, because the ref CAS is the platform's only serialization point. **Open item.**
- `put_multipart` is unused by the current design (artifacts are written whole); revisit only if relation artifacts grow beyond a comfortable single-put size.

---

## 11. Runtime and failure — `rayon`, `tokio`, `tracing`, `thiserror`, `miette` (C10)

### C10.1 `rayon` and `tokio` — the thread budget

**Re-sourced.** `tokio` **1.53.1** and `rayon` **1.12.0**, both confirmed current `[crates.io:tokio]`, `[crates.io:rayon]` and resolved into the committed lockfile (§1.5). §3.1's `tokio 1.52+` is consistent — and `tokio` is the **only** crate in this entire map that §3.1 anchors at all (§1.1).

**Role and boundary** are almost entirely a platform concern, and §18.8 already states it well: *one* configuration owns the DataFusion `tokio` runtime, the `rayon` pool for batch kernels and pass parallelism, and the solver's linear-algebra threads (HSL/MUMPS/PETSc); oversubscription is a configuration error, not a runtime surprise.

Three cross-cluster facts this document surfaces that §18.8 must account for, because each quietly consumes the `rayon` pool:

| Consumer | Mechanism | Cluster |
|---|---|---|
| `blake3::Hasher::update_rayon` / `update_mmap_rayon` | artifact hashing, `rayon` feature | C9 |
| `faer`'s `Par` parameter on every factorization and solve | diagnostics and condition estimates | C7 |
| salsa's per-key claiming under rayon-parallel passes | blocks rather than duplicates — benign, but it means pass parallelism is bounded by the pass DAG, not by the pool size | C5 |

The rule §18.8 states ("defaults leave the solver single-threaded while a solve is running") should be extended: **hashing and diagnostics must not steal the pool during a solve either.**

### C10.2 `tracing` — structured observability

**Role:** §23.1 — spans per pass run, per rule evaluation, per plan stage, and per solve; "events carry the semantic IDs involved"; metrics (pass durations, row counts, cache hits, solver iterations) emitted with the same identifiers.

| Capability | Surface | Provenance |
|---|---|---|
| Function instrumentation | `#[instrument]` — creates and enters a span per call; span name defaults to the function name, level `INFO`, and **all function arguments are recorded as fields by default** | `[c7:/websites/rs_tracing]` |
| Instrumentation options | `name`, `target`, `level`, `skip(...)`, `fields(...)` with expressions evaluated at span entry, `err` | `[c7:/websites/rs_tracing]` |
| Async support | `#[instrument]` handles span entry/exit across `await` points correctly | `[c7:/websites/rs_tracing]` |
| Late field recording | `Span::current().record("name", &value)` | `[c7:/websites/rs_tracing]` |
| **Field pre-declaration requirement** | fields must be declared at span creation; use `field::Empty` as a placeholder. Recording an undeclared field produces **no error and no effect** | `[c7:/websites/rs_tracing]` |
| Subscriber composition | `tracing-subscriber` layers | `[c7 resolve: tracing]` (`/websites/rs_tracing-subscriber`) |
| OpenTelemetry bridge | `tracing-opentelemetry` | `[c7 resolve: tracing]` (`/websites/rs_tracing-opentelemetry`) |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §23.1 spans per pass / rule / plan stage / solve | `#[instrument]` on the pass entry point, or manual spans where the boundary is not a function | **leverage** |
| §23.1 "events carry the semantic IDs involved"; metrics recorded with the same identifiers | `fields(...)` at creation + `Span::current().record(...)` | **leverage, with a trap.** Outcome values a pass only knows at the *end* — output content hashes, row counts, findings counts, cache hit/miss — must be declared as `field::Empty` when the span opens, or the later `record` is silently discarded `[c7:/websites/rs_tracing]`. This is the single most likely way §23.1 gets quietly implemented wrong. |
| §23.1 "provenance is separate from observability… linked by pass run and run IDs but never merged" | span fields carrying `pass_run_id` / `run_id` | **leverage**; the separation is a platform rule, and `tracing` has no opinion |

**Two extensions the blueprint does not list — re-sourced, and the recommendation is now qualified** `[crates.io:datafusion-tracing]`, `[crates.io:instrumented-object-store]`:

> **Amendment D is currently unsatisfiable.** Both crates exist and both are version-locked to the DataFusion release line — `datafusion-tracing` **55.0.0** and `instrumented-object-store` **55.0.0**, both published 2026-08-24. **Neither has a 55.1.0 release**, and §3.1 pins the engine at **55.1.0**. The previous edition hedged exactly this ("verify that release exists before depending on it") on the strength of a reference that has since been deleted; verified, it does not. Adopting these means either moving the engine back to 55.0.0 — contradicting the owner's decided pin — or running a plan-instrumenting crate one minor out of step with the plans it instruments, which is the least forgiving kind of mismatch. The companion DataFusion map reaches the same conclusion from the engine side and offers `JoinSetTracer` as the first-party alternative.

- **`datafusion-tracing`** — instruments DataFusion through its own extension points: a `PhysicalOptimizerRule` that wraps each physical node (`instrument_with_spans!` and level-specific variants), plus rule-phase instrumentation across analyzer/logical/physical optimizer phases (`instrument_rules_with_spans!`), with `InstrumentationOptions`/`RuleInstrumentationOptions` and `pretty_format_compact_batch` for batch previews. **Version-locked to the DataFusion release** (`datafusion-tracing 55.0.0` would be required alongside DataFusion 55) — verify that release exists before depending on it.
- **`instrumented-object-store`** — spans for object-store operations, which is §20.1's artifact store.

Both map onto the two subsystems §23.1 most wants visibility into, through supported extension points rather than custom wrappers. **Recommend adding them to §3.3** (with the version-lockstep caveat).

#### What we deliberately do not use

- **`#[instrument]`'s default "record every argument".** A pass takes a snapshot handle and possibly large inputs; blanket recording is both noisy and a way to leak whole structures into logs. Use `skip(...)` and name fields explicitly.
- **Observability is never provenance.** §23.1's separation is load-bearing: `provenance.*` relations explain *derivation* and are content-addressed and reproducible; spans explain *execution* and are not. A field in a span is not a provenance record.

### C10.3 `thiserror` and `miette` — the failure taxonomy

**`thiserror`** 2.0.20 `[rustdoc:thiserror@2.0.20]`; its surface (`#[derive(Error)]`, `#[error("…")]`, `#[from]`, `#[source]`) is stable and unremarkable. The interesting half is `miette`.

**`miette` role:** §3.3 — "typed failure taxonomy with source anchors"; §23.2 — a failure-class table where "failures carry the relation rows and source spans involved; they are never flattened into 'run failed'".

| Capability | Surface | Provenance |
|---|---|---|
| Derive | `#[derive(Diagnostic)]` on any `std::error::Error`; composes directly with `thiserror`'s `#[derive(Error)]` | `[c7:/websites/rs_miette]` |
| Diagnostic metadata | `#[diagnostic(code(...), severity(...), help(...), url(...))]`; `code` is documented to use **Rust path format (`foo::bar::baz`)** | `[c7:/websites/rs_miette]` |
| Source attachment | `#[source_code] src: NamedSource<String>` — the source text snippets are cut from; a plain `String` works when file names do not matter | `[c7:/websites/rs_miette]` |
| Labelled spans | `#[label("...")] bad_bit: SourceSpan`; `SourceSpan` constructible from `(offset, len)` | `[c7:/websites/rs_miette]` |
| Trait surface | `Diagnostic::{code, severity, help, url, source_code, labels, related, diagnostic_source}`; `labels()` yields `LabeledSpan`s, `related()` yields further `Diagnostic`s | `[c7:/websites/rs_miette]` |
| Ad-hoc diagnostics | `diagnostic!(severity = …, code = …, help = …, labels = vec![LabeledSpan::at_offset(…)], url = …, "message")` → `MietteDiagnostic` | `[c7:/websites/rs_miette]` |
| Reporting | `miette::Result`, `Report`, custom reporter via `miette::set_hook()` | `[c7:/websites/rs_miette]` |

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §23.2 failure classes named `authoring.parse`, `compile.property`, `solve.infeasible`, `internal.invariant`, … | `#[diagnostic(code(...))]` in Rust path format | **leverage — a near-exact fit.** The taxonomy's dotted names *are* diagnostic codes; the table in §23.2 becomes the code registry. |
| §23.2 `severity ∈ {error, warning}` (also §4.2 invariants) | `Severity` / `#[diagnostic(severity(...))]` | **leverage** |
| §3.3 / §23.2 "source anchors" — `pse.source_span` (§4.4) from authored documents | `#[source_code] NamedSource` + `#[label] SourceSpan` | **leverage**; `SourceSpan` is `(offset, len)` while `pse.source_span` stores `(start, end)` — a trivial conversion, but one that must be written once and only once, since `winnow`'s `with_span()` and `toml::Spanned` both yield `Range<usize>` (C3) |
| §23.2 "failures carry the **relation rows** … involved; never flattened" | `related()` for multiple diagnostics; `help()` / `url()` for the explanation | **partially** — miette models *source-anchored* failures beautifully. A failure whose evidence is *a set of violating relation rows* (§4.2's invariant validators return violating keys) has no natural miette representation. **Design item:** findings are relations (`compile.findings`-style rows) and a miette `Diagnostic` is the *rendering* of one, not its storage. Keep the row the record and the diagnostic the presentation, or the taxonomy stops being data (§23.2's own premise). |

#### What we deliberately do not use

- **`miette::Result` in library crates.** miette's own documentation is explicit: use its `Result` throughout an application "but **NOT your libraries!** Those should always return concrete types!" `[c7:/websites/rs_miette]`. Every `pse-*` crate is a library and must return its own concrete error enums (`thiserror` + `Diagnostic` impls); only the CLI and the top-level driver return `miette::Result` and install a reporter. This also keeps §23.2's classes typed and matchable rather than erased into a report.
- **Not miette's fancy graphical rendering as the machine-readable output.** Rendered output is for humans at a terminal; `runtime`/`provenance` relations are what programs and §20.4 reproduction read.

#### Gaps and risks

- miette's version was not pinned during this pass — **open item**, record an anchor at `cargo add`.
- The rows-versus-diagnostic design item above is the one substantive decision in this cluster.

---

## 12. Solver FFI — the Ipopt C API (C11)

**Re-sourced from the header itself.** The previous edition rested this cluster on a local reference that has since been deleted, and flagged one signature as read second-hand. Everything below now comes from `IpStdCInterface.h` at **COIN-OR tag `releases/3.14.16`** `[gh:coin-or/Ipopt@releases/3.14.16]`. Every claim survived; one recommended amendment had the wrong function names.

**The complete exported C surface is eleven functions**, which is small enough to state in full: `CreateIpoptProblem`, `FreeIpoptProblem`, `IpoptSolve`, `AddIpoptStrOption`, `AddIpoptNumOption`, `AddIpoptIntOption`, `OpenIpoptOutputFile`, `SetIpoptProblemScaling`, `SetIntermediateCallback`, `GetIpoptCurrentIterate`, `GetIpoptCurrentViolations`. This section covers the **Rust binding layer** and the two capabilities §18.3 depends on. (The previous edition also pointed at two local documents on option semantics and metrics; both are deleted, and no claim in this map rested on either, so the pointers are removed rather than replaced.)

**Crate decision:** §3.3 specifies an own `-sys` crate. A search found no established Rust binding worth preferring — context7 resolves `ipopt-sys` only to the upstream C++ project `/coin-or/ipopt` `[c7 resolve: ipopt-sys]`. **The blueprint's decision stands.**

#### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §18.3 "binds the Ipopt C API (`CreateIpoptProblem`, `IpoptSolve`, `AddIpoptNumOption`/`StrOption`/`IntOption`)" | declared in `IpStdCInterface.h` under `$PREFIX/include/coin-or`; `FreeIpoptProblem` completes the lifecycle | **confirmed** `[gh:coin-or/Ipopt@releases/3.14.16]` |
| §18.3 callbacks `eval_f`, `eval_grad_f`, `eval_g`, `eval_jac_g`, `eval_h` over the evaluation program | the five C callback typedefs, each taking `UserDataPtr user_data` — the handle through which the §18.2 tape and workspace reach the callback | **confirmed**; `user_data` is the entire Rust-side plumbing |
| §18.3 "iteration statistics parsed from the intermediate callback into `runtime.iterations` (objective, primal and dual infeasibility, `mu`, step size, regularization, restoration flag)" | `SetIntermediateCallback`; the callback receives algorithm mode, objective, `inf_pr`, `inf_du`, `mu`, step/`d` norm and unscaled infeasibilities, and **returns false to stop the solve** | **confirmed** `[c7:/coin-or/ipopt]` |
| §18.3 richer per-iteration state | **`Ipopt_get_curr_iterate(nlp)` and `Ipopt_get_curr_violations(nlp)`** — C-interface counterparts of `TNLP::get_curr_iterate()`/`get_curr_violations()`, callable *inside* the intermediate callback, available since Ipopt **3.14** | **leverage, and it improves on §23.1.** §23.1 plans to capture "solver output … with parsed structure (iteration lines, restoration entries)". Text parsing is brittle and locale/format dependent; these two functions return the current primal/dual iterate and infeasibilities as data. Use the structured API for `runtime.iterations`, and keep text capture only for messages that have no structured equivalent. |
| §18.2 step 6 domain guards — "a recoverable evaluation error … the Ipopt driver reports it as an evaluation failure (Ipopt then restores or fails)" | every callback returns `Bool`; returning false signals an evaluation failure | **confirmed** — this is precisely the mechanism, and it is why IDAES needs `halt_on_ampl_error` while the native driver does not |
| §17.5 continuation and §17 initialization plans re-solving perturbed problems | `warm_start_init_point yes`, `warm_start_bound_push`, `warm_start_mult_bound_push`, `mu_init` | **leverage** `[c7:/coin-or/ipopt]` — documented for exactly the "similar active set, repeated solve" case that continuation steps and sequential-modular initialization create |
| §18.3 bound handling for `pse.bound` (`{finite, unbounded}`, §4.4) | `CreateIpoptProblem` copies bounds; values at or beyond `nlp_lower_bound_inf` / `nlp_upper_bound_inf` are treated as infinite | **confirmed** `[gh:coin-or/Ipopt@releases/3.14.16]` — the extension type's `unbounded` kind maps onto these sentinels at the driver boundary, which is the right place for a sentinel to exist |

#### Re-sourced (PROBE-free, from the header)

Two claims the previous edition could not fully support, now first-hand `[gh:coin-or/Ipopt@releases/3.14.16]`.

**`Intermediate_CB`'s exact signature** — the previous edition read it "through the Java interface's parameter list and the ChangeLog" and flagged it. It is twelve typed parameters:

```c
typedef bool (*Intermediate_CB)(
   ipindex  alg_mod,              /* 0 is regular, 1 is restoration */
   ipindex  iter_count,
   ipnumber obj_value, inf_pr, inf_du, mu, d_norm,
   ipnumber regularization_size, alpha_du, alpha_pr,
   ipindex  ls_trials,
   UserDataPtr user_data);
/* "If this method returns false, Ipopt will terminate the optimization." */
```

Every field §18.3 says it parses — "objective, primal and dual infeasibility, `mu`, step size, regularization, restoration flag" — **is a typed callback argument**, not a value scraped from text. And the documented return-`false` semantics is exactly §18.3's cancellation mechanism, confirmed at the source.

**Recommended amendment H had the wrong function names.** It proposed preferring `Ipopt_get_curr_iterate` / `Ipopt_get_curr_violations`. Those are the **C++ `TNLP` method names** (`get_curr_iterate`, `get_curr_violations`). The C interface exports them as **`GetIpoptCurrentIterate`** and **`GetIpoptCurrentViolations`**. The recommendation stands; the names are corrected.

#### Binding-layer rules (platform decisions this lookup surfaces)

- **`IpStdCInterface.h` is the source of truth for typedefs**, and the exact set at 3.14.16 is `ipindex`, `ipnumber`, plus the legacy aliases `Index` (`int`), `Number`, `Bool` (**`bool`**, i.e. C99 `stdbool`, not an `int`), `UserDataPtr` (`void*`), `IpoptProblem` (an opaque `struct IpoptProblemInfo*`), and `enum ApplicationReturnStatus` `[gh:coin-or/Ipopt@releases/3.14.16]`. `Bool` being a real `bool` matters for the `-sys` crate's FFI declarations. Whether the `-sys` crate uses `bindgen` at build time or hand-declared `extern "C"` items is an open choice: bindgen tracks the installed header (safer across Ipopt builds), hand-declaration keeps the build hermetic and reproducible (§20.4 cares about reproducibility). **Open decision** — recommend bindgen with the generated bindings committed and diffed in CI, mirroring §4.2's treatment of generated relation code.
- **Index style.** `CreateIpoptProblem` takes an index-style argument (`0` = C style) `[gh:coin-or/Ipopt@releases/3.14.16]`; the sparsity pattern from `compiled.incidence` is 0-based, so C style is correct and must be asserted, not assumed.
- **Never unwind through a callback.** A Rust panic — or a salsa cancellation unwind (C5) — crossing the C boundary is undefined behaviour. Callbacks must catch, record the failure in the `user_data` state, and return `false`; the driver then converts that into `solve.evaluation_error` or `runtime.cancelled` (§23.2) after `IpoptSolve` returns. **This is the single most important rule in this cluster** and neither §18.2 nor §18.3 currently states it.
- **Version floor.** `Ipopt_get_curr_iterate`/`Ipopt_get_curr_violations` require **Ipopt ≥ 3.14** `[c7:/coin-or/ipopt]`. §20.3 already records solver versions in the run environment; add a capability probe alongside the existing `linear_solver = ma57` probe (§18.3) so an older Ipopt degrades to text-parsed iterations rather than failing.

#### Gaps and risks

- **Linking and distribution.** How Ipopt (and HSL/MUMPS) is located, linked and redistributed is untouched by the blueprint and by this lookup — static vs dynamic, `pkg-config`, vendored build. It determines whether `pse-backend-native` builds on a fresh machine. **Open item, out of scope for a capability map but on the critical path.**
- The exact `Intermediate_CB` C typedef was read through the Java interface's parameter list and the ChangeLog `[c7:/coin-or/ipopt]`; take the precise C signature from `IpStdCInterface.h` at implementation time.
---

## 13. Pass 3 — the under-leverage sweep

Passes 1 and 2 both start from the blueprint, so neither can find a capability it never gestured at. This pass takes each library on its own terms and asks what it offers that bears on a named blueprint requirement and is not claimed. Charter §G and **DM-58** apply throughout: a row naming no principle and replacing no identified work does not appear.

The denominator is §1.5's extraction — **26 crates, 589 public traits**. Most of that surface is irrelevant to this design and is dismissed by cluster rather than enumerated.

| Capability | Crate | DM / gate | Adjudication |
|---|---|---|---|
| **`algo::feedback_arc_set`** | petgraph | **DM-38**, DM-25 | **adopt.** §12.5 and §15.3 both list tear selection among the hand-written algorithms. A tear set *is* a minimum feedback arc set, and PROBE 3c shows `greedy_feedback_arc_set` returning exactly the recycle edge on a flowsheet-shaped graph. This does not remove the need for a *policy* (which tears are acceptable, what they cost), but it removes the search. The strongest single find in this map. |
| **`algo::ford_fulkerson` / `maximum_flow`** | petgraph | DM-38 | **note, and it refines amendment B.** Maximum bipartite matching reduces to max-flow, and petgraph ships both. So "Hopcroft–Karp is required" is true of the *complexity* — O(\|E\|√\|V\|) against a flow formulation — not of the *capability*. Worth stating so the amendment's reasoning is exact. |
| `algo::{tred, dominators, articulation_points, bridges, condensation, is_bipartite_undirected}` | petgraph | DM-10, DM-53 | **evaluate.** `condensation` is already implied by §15.3's DAG step. `tred` (transitive reduction) would make §14.1's pass-dependency DAG minimal and therefore readable; `is_bipartite_undirected` is a cheap assertion on an incidence graph that §15.3 assumes is bipartite and never checks. |
| **`faer::matrix_free`** (`LinOp`/`BiLinOp`, `lsmr`, `conjugate_gradient`, `bicgstab`, `partial_svd`) | faer | **DM-36**, DM-38 | **evaluate — it is half of F17's answer.** See C7.2: the operator contract works and is eight methods, but `partial_svd` targets the *largest* singular values while §15.5 wants the smallest. The route is sparse LU plus matrix-free iteration, not matrix-free alone. |
| `linalg::svd::pseudoinverse_from_svd_with_tolerance` | faer | DM-40, **DM-59** | **evaluate.** §15.4's condition number is `‖J‖_F·‖J⁻¹‖_F` "via `faer` sparse LU (pseudo-inverse for non-square)". This is that pseudo-inverse, with an explicit tolerance argument — which is exactly the "declare the approximation" discipline DM-40 asks for and §15.4 currently leaves implicit. |
| **`num-dual`'s `ImplicitFunction` / `ImplicitDerivative` / `implicit_derivative*`** | num-dual | **DM-25**, DM-52 | **adopt.** Confirmed by PROBE 4. Recommended amendment G already spotted it; the measurement makes it a binding rather than a hope. |
| `hessian`, `partial_hessian`, `Dual3`, `HyperHyperDual` | num-dual | DM-40 | **evaluate.** §26's "exact Hessian cost for large distributed models" risk row assumes second derivatives are expensive to obtain. Third-order forms exist too, which matters only if a method ever needs them. |
| **`#[salsa::accumulator]`** | salsa | **DM-47**, DM-30 | **adopt if salsa is kept.** Measured in PROBE 6. Recommended amendment E. Findings travel with a query's value instead of being threaded through every return type. |
| **`salsa::CancellationToken`, `salsa::Cycle`** | salsa | **DM-30**, DM-35 | **evaluate if salsa is kept.** §18.3 and §20.1 both specify interruption; §14.2's rule engine runs to a fixed point. Both have first-class machinery here rather than being platform concerns. |
| **`toml::Spanned`** | toml | **DM-47**, DM-46 | **adopt.** Measured in PROBE 7a. §4.4 defines `pse.source_span` and §23.2 requires failures that carry source spans; this supplies them for `package.toml` at no cost. |
| **`serde-saphyr`'s `budget` module** | serde-saphyr | **DM-30**, G3 | **adopt with the crate.** Bounded parsing — explicit limits on nesting, aliases and allocation — at a boundary that parses user-supplied documents. §23.2 wants typed failures; a resource bound is what turns a hostile document into one. |
| `proc-macro2`'s `span-locations` | proc-macro2 | DM-46 | **evaluate.** §4.2 commits generated source and diffs it in CI. Span locations let a generated-code diagnostic point back into the `RelationSpec` row that produced it, which is DM-46's lineage requirement applied to generated artifacts. |
| `blake3::Hasher::update_rayon` | blake3 | DM-39 | **evaluate** — recommended amendment I already covers extending §18.8's thread budget to hashing; the API is the reason it is actionable. |
| `rustsec` / `cargo deny` maintenance gate | — | **DM-31**, **G7** | **adopt.** §1.4's finding: `cargo audit` is clean and cannot see `serde_yaml`'s deprecation. A supply-chain gate that cannot detect the one dependency this map rejects is not a gate. |
| `egglog::CommandOutput::snapshot_stable_under_proof_encoding` | egglog | **DM-48**, DM-15 | **adopt if egglog is adopted.** PROBE 8: the stable rendering exists precisely because the `Debug` one is not. |
| `uom::str` runtime unit parsing | uom | DM-41 | **evaluate, narrowly.** §8's registry is authoritative and uom is compile-time, which §3.3 states correctly. `uom::str` parses a unit string into uom's compile-time types at runtime, which is the only bridge between the two — relevant if a "selected Rust kernel" ever needs to accept a registry unit. |
| **Rejected: `quantity`** (arrives with feos) | feos-core | **G1**, DM-02 | **reject.** `feos-core` 0.10.1 pulls `quantity ^0.14` `[crates.io:feos-core]` — a **third** units representation alongside `uom` and §8's dynamic registry. §3.3's boundary already says §8 is authoritative for quantities; adopting feos imports a units system that does not know that. |
| **Rejected: `diffsol`'s linear algebra** | diffsol | DM-58 | **reject as a second stack.** `diffsol` 0.16.2 depends on `diffsol-la` and `diffsol-nl`, **not `faer`** `[crates.io:diffsol]`. Adopting it adds a second linear-algebra implementation to a design that has already chosen one. It also pulls `petgraph ^0.8.3`, which at least agrees with C4's pin. |
| **Rejected: `egg`** | — | DM-57 | **reject.** The alternative e-graph crate. If equality saturation is ever adopted, the choice between `egg` and `egglog` is a real one — but it is a phase-4 choice, and C6's structural objection (a second model language) applies to both. |
| ~460 further public traits | all | — | **not extension points for this design** — dismissed as a group |

---

## 14. Principle-alignment register (supporting libraries)

Capabilities and disciplines that would improve alignment with `DATA_MODEL_DESIGN_CHARTER.md`. Every row names a principle and the hand-written work or risk it removes. Rejections are included, because a recorded rejection is worth as much as an adoption.

**The previous edition had no register and cited no DM principle at all** — the only one of the four capability maps in that position. Its findings were real; they were simply never connected to the charter. Numbering is local to this map.

### Adopt

| # | Capability or discipline | Cluster | DM / gate | What it removes or prevents |
|---|---|---|---|---|
| 1 | **Pin all 24 supporting crates in §3.1** | §1.1 | **DM-31**, DM-48, DM-51 | §3.2 claims the manifest "carries every pin once"; §3.1 anchors **one** of the 24. An unpinned dependency is an undeclared input that can change meaning between builds — and §20.4's reproduction contract is stated over exactly such inputs. |
| 2 | **Pin `blake3` and assign it a crate in §3.2** | C9.1 | **DM-02**, DM-11, DM-48 | It is the most deeply wired supporting crate — ~20 sites across identity, hashing, hash-consing, plan fingerprints and the manifest — with no pin, no owning crate, and two digest widths whose derivation is unnamed. |
| 3 | **Name the 128-bit derivation: `derive_key(context, material)`** | C9.1 | **DM-11**, DM-15, DM-48 | Measured (PROBE 1): the two candidate routes give different bytes, and §5.1 IDs are permanent. Truncation itself is sound — BLAKE3 is an XOF, so a 16-byte prefix is a defined 128-bit digest — but which route is used must be written down. |
| 4 | **Replace `serde_yaml` with `serde-saphyr`** | C3.1 | **G3**, DM-07, DM-47 | Measured (PROBE 7b): typed errors with caret diagnostics, and four classes of hostile input refused without a panic. The incumbent is unmaintained since 2024-03 and sits at the boundary that parses user documents. |
| 5 | **A `cargo deny` maintenance gate, not `cargo audit` alone** | §1.4 | **DM-31**, **G7** | Measured: all three RustSec advisories touching this set are patched below our versions, so `cargo audit` is clean — and **cannot see** `serde_yaml`'s deprecation, which lives only in semver build metadata. A supply-chain gate blind to the one rejected dependency is not a gate. |
| 6 | **Decide the `num-dual` pin, and state it as a source-level constraint** | C7.1, C8.1 | **DM-31**, G6 | Measured (§1.3): `feos-core` forces `num-dual 0.14` alongside our 0.15, and the trait's *shape* differs between them, so every kernel body's bound differs. Not "two copies of a type" — two versions of the source. |
| 7 | **Ban `SchemaLike::from_type` / `from_samples` on any platform path** | C2.1 | **DM-02**, **G2** | Measured (PROBE 2): tracing a `[u8; 16]` yields a 16-field struct, not `FixedSizeBinary(16)`. §4.2 already forbids inference; the governance grep should name the functions. |
| 8 | **Treat any `SERDE_ARROW:*` field-metadata key as a contract violation** | C2.1 | **DM-15**, DM-48 | Measured: the adapter stamps `SERDE_ARROW:strategy` into field metadata, which §5.3 hashes — so provenance would change a relation's content hash. Settles open question 5 without a hashing-policy exception. |
| 9 | **`toml::Spanned` for `package.toml`** | C3.2 | **DM-47**, DM-46 | Byte offsets that slice back to source, and spans on type errors. §4.4 defines `pse.source_span`; this supplies it for free. |
| 10 | `#[salsa::accumulator]` for pass findings | C5 | DM-47, DM-30 | **Deferred**, despite the historical probe. Current pass attempts are sidecar evidence; salsa requires R-22/R-01 adoption (ADR-0042). |
| 11 | **`petgraph::algo::feedback_arc_set` for tear selection** | §13 | **DM-38**, DM-25 | Measured (PROBE 3c): returns exactly the recycle edge. §12.5 and §15.3 list tear selection as hand-written; the search is library work, the policy is ours. |
| 12 | **`num-dual::implicit_derivative` for implicit kernels** | C7.1 | **DM-25**, DM-52 | Measured (PROBE 4) against a hand-derived IFT: exact agreement. Amendment G, now with evidence. |
| 13 | **Sort within every SCC block by semantic ID** | C4 | **DM-15**, **DM-48** | Measured (PROBE 3a): `tarjan_scc` and `kosaraju_scc` return the same components in *different* intra-component orders. Nothing in petgraph guarantees the order, so §5.1's reproducible-IDs claim depends on the platform imposing one. |
| 14 | **Never hash a `Debug` rendering** | C6, §13 | **DM-48**, DM-15 | Measured (PROBE 8): egglog's extraction is deterministic while its `Debug` output is not, because the underlying container is a hash set. The Arrow and DataFusion maps found the same class of defect in `Schema` metadata and `datafusion-proto`. The rule is general and belongs in §5.3. |
| 15 | **State the Ipopt panic/unwind rule in §18.2 and §18.3** | C11 | **G5**, DM-30 | A Rust panic crossing a C callback boundary is undefined behaviour. §18.3 describes cancellation via the intermediate callback but never states the rule. Amendment F. |
| 16 | **Correct amendment H's function names** | C11 | DM-43, **DM-59** | The C interface exports `GetIpoptCurrentIterate` / `GetIpoptCurrentViolations`; the previously recommended `Ipopt_get_curr_*` are the C++ `TNLP` spellings. The recommendation is right and the identifiers were not. |

### Evaluate

| # | Capability | Cluster | DM | Why it is not yet an adopt |
|---|---|---|---|---|
| 17 | `faer::matrix_free` + sparse LU for §15.5's smallest singular values | C7.2 | **DM-36**, DM-40 | Half of §26 F17's answer. `partial_svd` targets the largest end of the spectrum; reaching the smallest needs shift-invert. Two components, and neither is yet costed. |
| 18 | `pseudoinverse_from_svd_with_tolerance` | §13 | DM-40, DM-59 | §15.4's condition number needs a pseudo-inverse and an explicit tolerance; this has both. Open question 6 asks for the budget first. |
| 19 | `salsa::CancellationToken` and `salsa::Cycle` | C5 | DM-30, DM-35 | First-class machinery for two things §18.3, §20.1 and §14.2 specify as platform concerns — but only if **F13** resolves in salsa's favour. |
| 20 | `petgraph::algo::{tred, is_bipartite_undirected, dominators}` | §13 | DM-10, DM-53 | `tred` makes §14.1's pass DAG minimal and readable; `is_bipartite_undirected` is a cheap assertion on a graph §15.3 assumes is bipartite and never checks. |
| 21 | `proc-macro2` `span-locations` for generated-code lineage | §13 | DM-46 | Would let a diagnostic in generated source point back at the `RelationSpec` row that produced it. Only worth it once generated code is large enough to need it. |
| 22 | `uom::str` as the registry→kernel bridge | C7.3 | DM-41 | The only mechanism connecting §8's runtime units to uom's compile-time types. Needed only once a "selected Rust kernel" exists — and §3.3 never says which kernels those are (open question 13). |
| 23 | `blake3::Hasher::update_rayon` | C9.1 | DM-39 | Amendment I's mechanism. Measure before parallelising a hash that may not be the bottleneck (**DM-58**). |
| 24 | `num-dual`'s `hessian` / `partial_hessian` / third-order forms | §13 | DM-40 | Bears on §26's exact-Hessian cost row. Whether third-order is ever needed is a modelling question, not a library one. |

### Reject, with reason

| # | Capability | Cluster | Why rejected |
|---|---|---|---|
| 25 | **`quantity`** (arrives with `feos-core`) | C8.1, §13 | A **third** units representation alongside `uom` and §8's registry. §3.3 already names §8 authoritative; importing a units system that does not know that is the second-authority failure **G1** describes. |
| 26 | **`diffsol`'s linear-algebra stack** | C8.2, §13 | `diffsol` 0.16.2 pulls `diffsol-la`/`diffsol-nl`, **not `faer`**. Adopting it adds a second linear-algebra implementation to a design that has chosen one — **DM-58**. |
| 27 | **`datafusion-tracing` / `instrumented-object-store`** at present | C10.2 | Amendment D's crates exist but top out at **55.0.0** against a pinned engine at **55.1.0**. A plan-instrumenting crate one minor out of step with the plans it instruments is the least forgiving kind of mismatch. Revisit when a 55.1.0 release exists. |
| 28 | **`egg`** as an alternative to `egglog` | §13 | A phase-4 choice that C6's structural objection — a second model language — applies to equally. |
| 29 | **`serde_norway`, `serde_yml`** as `serde_yaml` replacements | C3.1 | `serde_norway` has not been released since 2024-12; `serde_yml` is deprecated in its own description. Replacing a quiet crate with a quieter one. |
| 30 | **`serde_yaml`** itself | C3.1 | Unmaintained since 2024-03, deprecation encoded where no tool reads it. The one erratum the previous edition and this one agree on completely. |

---

## 15. Acceptance-gate review (supporting libraries)

The earlier survey's gates described revision 2. Current document-stage gates are
assessed in the [revision-5 review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md).
The following are **Proposed** platform contracts; historical library receipts do
not certify their implementation.

| Gates | Current mechanism | Remaining acceptance |
|---|---|---|
| G1–G2 | Registry-derived schemas; no traced schema or foreign unit authority; typed conversions and recursive admission (blueprint §4, §8, ADR-0039) | Invalid metadata, lost quantity context and inferred-schema negative fixtures |
| G3–G4 | Typed parser/kernel failure, declared pure kernel bindings, guarded execution; no adopted salsa query runtime (blueprint §18, ADR-0042/0043/0047) | Real parser/adapter entry points, branch failures, undeclared input rejection |
| G5 | Complete objects and manifests, checksum verification on existing objects, conditional refs; no unwind through solver callbacks (blueprint §20, §18.3, ADR-0045) | Truncated-existing-object, interruption, CAS and callback fixtures |
| G6 | Complete stage input keys and pinned derivative implementations; fine reuse/automatic tracking deferred (ADR-0041/0042/0043) | Changed domains/bindings/fixed values versus clean compilation; derivative conformance |
| G7 | Roles remain bounded by actual consumers; optional FeOs/egglog/observability engines retain register triggers | Numerical routes and performance need their own exercised fixtures/measurements; map receipts are narrower evidence |

## 16. Leverage matrix

One row per blueprint requirement that rests on a supporting library. Read this first.

| Blueprint requirement | Crate | API | Status |
|---|---|---|---|
| §4.2 generated views/builders/schemas from `RelationSpec`, non-macro, source committed | `syn`/`quote`/`proc_macro2`/`prettyplease` | non-macro codegen pipeline, `prettyplease::unparse` | leverage |
| §4.2 `SerdeArrowSchema` "never inferred from samples" | `serde_arrow` | `SchemaLike` impl for `Vec<FieldRef>` | **confirmed** — inference is not merely banned, it is absent from the path |
| §4.2 import/export adapters | `serde_arrow` | `to_record_batch` / `from_record_batch`, `arrow-59` feature | leverage |
| §4.3 field metadata namespace | `serde_arrow` | `STRATEGY_KEY` writes into field metadata | **open decision** — classify as volatile or contractual for §5.3 hashing |
| §7.7 expression DSL precedence (`+ -` / `* /` / `^` / unary) | `winnow` | `combinator::expression` with `Prefix`/`Infix::{Left,Right}`/`Postfix` binding powers | leverage — simpler than the EBNF's nested-rule encoding |
| §7.7 / §4.4 `pse.source_span` from authored text | `winnow`, `toml` | `LocatingSlice` + `with_span()`; `toml::Spanned<T>` | leverage — spans are free on both paths |
| §22.1 YAML document loading | `serde_yaml` | — | ⚠ **erratum** — crate unmaintained; replace (C3.1) |
| §12.5 topology, §15.3 SCC/condensation/toposort | `petgraph` | `tarjan_scc`, `condensation`, `toposort`, `is_cyclic_directed` | leverage |
| §15.3 deterministic block order | `petgraph` | intra-component node order is **arbitrary** | **confirmed necessary** — sort within blocks by semantic ID |
| §15.3 maximum matching | `petgraph` | `maximum_matching` exists but is Gabow, general-graph, **O(\|V\|³)** | **refined** — Hopcroft–Karp still ours; correct §15.3's stated reason |
| §15.3 Dulmage–Mendelsohn | `petgraph` | none | build on top (as the blueprint says) |
| §14.3 passes memoized on content hashes | `salsa` | `#[salsa::tracked]` + `#[salsa::input]` | **Available but deferred:** R-22/R-01; not the current compiler binding (ADR-0042) |
| §14.3 "the memo stores the output artifact hashes" | `salsa` | `PartialEq`-driven **backdating** | **Available but deferred:** R-22/R-01; not the current compiler binding (ADR-0042) |
| §14.3 pass findings | `salsa` | `#[salsa::accumulator]` — excluded from result equality | **Available but deferred:** R-22/R-01; not the current compiler binding (ADR-0042) |
| §6.13/§23.2 cancellation | `salsa` | `CancellationToken`, unwinding | leverage — **never unwind through the Ipopt FFI** (C11) |
| §14.3 rayon-parallel passes | `salsa` | per-key claiming; second caller blocks | **Available but deferred:** R-22/R-01; not the current compiler binding (ADR-0042) |
| §3.1 snapshot immutability inside a session | `salsa` | `Durability::NEVER_CHANGE` | leverage — assignment is an **open item** |
| §7.4 equality-saturation rewrites (phase 4) | `egglog` | `EGraph`, `rust_rule`, `run_ruleset`, cost-model extraction | capable; **determinism unverified** — gate adoption on it |
| §18.5 one kernel body over `DualNum` | `num-dual` | `DualNum` bound; `Dual`/`Dual2`/`HyperDual` | **confirmed** |
| §18.2 kernel second derivatives | `num-dual` | `HyperDual`, `hessian`, `partial_hessian` | leverage |
| §18.2 implicit kernels via the implicit function theorem | `num-dual` | `implicit_derivative*`, `ImplicitDerivative` | **library capability, not platform code** |
| §18.2 per-equation **reverse** AD | `num-dual` | forward-mode only | build on top — confirms §3.3's kernel-local boundary |
| §15.4 condition number | `faer` | `sparse::linalg::solvers::Lu` + `Solve`; **no condition estimator** | build on top — **O(n) solves; budget is an open decision** |
| §15.4 non-square pseudo-inverse | `faer` | `sparse::linalg::solvers::Qr` + `SolveLstsq` | leverage |
| §15.5 SVD toolbox, small problems | `faer` | `svd()`, `thin_svd()`, `singular_values()` | leverage |
| §15.5 "iterative for large" | `faer` | `matrix_free::eigen::partial_svd` **exists, but targets the LARGEST singular values** `[probe]` | **refined** — §15.5 wants the smallest; the route is sparse LU (shift-invert) **plus** `matrix_free`. Half of §26 F17 (C7.2) |
| §3.3 solver workspaces, thread budget | `faer` | `MemStack` (`dyn_stack`), `Par` | leverage |
| §3.3 compile-time unit checks in kernels | `uom` | `Quantity<D,U,V>`, `system!`/`quantity!`/`unit!` | leverage, kernel-local |
| §8 dynamic registry stays authoritative | `uom` | **no runtime unit lookup exists** | **confirmed as a necessity**, not a preference |
| §4.4 8-dimensional basis incl. currency | `uom` | stock SI has no currency dimension | open item — custom `system!` or keep costing outside uom |
| §9.8 FeOs provider | `feos-core` | `Residual`/`IdealGas`/`Total`, `EquationOfState`, `State`, `PhaseEquilibrium` | leverage — ⚠ **`num-dual ^0.14`** (C8.1) |
| §13.6 native DAE backend | `diffsol` | `OdeBuilder`, `new_implicit_closure`, `NonLinearOp(Jacobian)`, `LinearOp` mass matrix | leverage via closures — **do not adopt DiffSL** |
| §18.2 sparse Jacobian → diffsol | `diffsol` | interface asks for a Jacobian-**vector product** | open item — adapter fit check |
| §5.3 logical and encoding hashes | `blake3` | `Hasher::update` / `update_reader` | pinned mechanism; v2 framing/normalization is platform code and requires metamorphic tests (ADR-0045) |
| §5.1 `blake3_128` derived IDs | `blake3` | `new_derive_key(context)` vs. truncating `finalize()` | **design decision** — choose before freezing; truncation safety `[UNVERIFIED]` |
| §5.3 hashing large artifacts | `blake3` | `update_rayon`, `update_mmap_rayon` | leverage — must respect §18.8's thread budget |
| §20.1 atomic writes, conditional ref update | `object_store` | `put_opts` + `PutMode::Update(UpdateVersion)`, `Error::Precondition` | **confirmed exactly as stated** |
| §20.1 idempotent artifact writes | `object_store` | `PutMode::Create` plus existing-object verification | logical identity and physical integrity are separate; ADR-0045 |
| §20.1 local single-writer mode | `object_store` | `LocalFileSystem` — documented atomic | confirmed |
| §20.1 commit protocol | `object_store` | write the final path conditionally; **`rename` does not exist at the pinned 0.13.2** | **refined** — amendment J should name `PutMode`, not a removed method (C9.2) |
| §3.1 `object_store` API surface | `object_store` | 0.13.0 merged `copy`/`copy_if_not_exists`→`copy_opts`, `delete`→`delete_stream`, `put`/`get`/`head`→`*_opts` | **erratum** — the previous edition described a different version (C9.2, ledger row 60) |
| §3.1 pin coverage | all 24 supporting crates | §3.1 anchors **one** (`tokio`) while §3.2 claims "every pin once" | **erratum** — §1.1, amendment A′ |
| §5.1 `blake3_128` derivation | `blake3` | `derive_key` vs truncated `finalize` give **different bytes**; truncation itself is the XOF prefix and is sound `[probe]` | **erratum** — §5.1 names neither; amendment K |
| §12.5 tear selection | `petgraph` | `algo::feedback_arc_set` returns exactly the recycle edge `[probe]` | **adopt, unclaimed** — §12.5 lists this as hand-written (§13) |
| §18.2 implicit kernels | `num-dual` | `implicit_derivative` matches a hand-derived IFT exactly `[probe]` | **adopt, unclaimed** — amendment G, now measured |
| §4.2 "never inferred from samples" | `serde_arrow` | tracing a `[u8;16]` yields a **16-field struct**, and stamps `SERDE_ARROW:strategy` `[probe]` | **confirmed necessary, and unenforced** — amendment L |
| §23.2 typed failures with spans at the authoring boundary | `serde-saphyr`, `toml` | caret diagnostics with line/column; four hostile inputs refused without panic; `toml::Spanned` byte offsets `[probe]` | **adopt** — settles open question 1 |
| §24 supply-chain gate | — | `cargo audit` is clean **and cannot see** `serde_yaml`'s deprecation `[rustsec:*]` | **gap** — amendment M |
| §14.3/§18.8 pass and kernel parallelism | `rayon`, `tokio` | one owned thread budget | leverage — extend the rule to hashing and diagnostics |
| §23.1 spans per pass/rule/stage/solve | `tracing` | `#[instrument]`, `fields(...)`, `Span::record` | leverage — **declare late fields as `field::Empty`** |
| §23.1 DataFusion and object-store visibility | `datafusion-tracing`, `instrumented-object-store` | extension-point instrumentation | **recommend adding to §3.3** (version-locked to DataFusion) |
| §23.2 failure classes as dotted codes | `miette` | `#[diagnostic(code(a::b::c), severity(...), help(...))]` | **near-exact fit** — §23.2's table becomes the code registry |
| §23.2 source-anchored failures | `miette` | `#[source_code] NamedSource`, `#[label] SourceSpan` | leverage |
| §23.2 failures carrying violating relation rows | `miette` | `related()`; no row-set concept | **partial** — rows stay the record, the diagnostic is the rendering |
| library error types | `miette` | `miette::Result` is for applications, **not libraries** | design rule — `pse-*` crates return concrete errors |
| §18.3 in-process Ipopt | Ipopt C API | `CreateIpoptProblem`/`AddIpopt*Option`/`IpoptSolve`/`FreeIpoptProblem`, five callbacks | confirmed |
| §18.3 iteration statistics | Ipopt C API | `SetIntermediateCallback` + **`Ipopt_get_curr_iterate`/`Ipopt_get_curr_violations`** (≥3.14) | **leverage — structured, better than §23.1's text parsing** |
| §18.2 domain-guard evaluation failures | Ipopt C API | callbacks return `Bool`; false = evaluation failure | confirmed |
| §17.5 continuation re-solves | Ipopt C API | `warm_start_init_point`, `warm_start_bound_push`, `mu_init` | leverage |

---

## 17. Evidence ledger

Every lookup performed, in order. `c7` = context7 plugin; `WebFetch` = docs.rs/crates.io; **`rustdoc`** = the local extraction of §1.5; **`probe`** = a program compiled against the pinned crates.

Rows 1–52 are the previous edition's. **The five rows whose instrument read `local` cited files that no longer exist**; each now records what replaced it, and none is load-bearing any more. Rows 53–64 are this edition's.

| # | Cluster | Instrument | Library ID / URL | Query | Outcome |
|---|---|---|---|---|---|
| 1 | C4 | c7 resolve | `petgraph` | Rust graph data structures, SCC, topological sort, bipartite matching | 4 candidates; selected `/websites/rs_petgraph` (1692 snippets, High) |
| 2 | C7 | c7 resolve | `faer` | Rust sparse LU QR factorization, dense SVD, condition number estimation | 1 candidate `/sarah-quinones/faer-rs` (20 snippets) |
| 3 | C7 | c7 resolve | `num-dual` | Rust generic dual numbers, hyper-dual, forward-mode AD, `DualNum` trait | **no match** — unrelated libraries returned |
| 4 | C7 | c7 resolve | `num_dual` (alt spelling) | generalized dual numbers AD Rust crate feos | **no match** |
| 5 | C7 | c7 query-docs | `/websites/rs_num-dual` (direct ID probe) | `DualNum` trait, `Dual`, `Dual2`, `HyperDual` types | **not found** — ID guessing confirmed to fail cleanly |
| 6 | C5 | c7 resolve | `Salsa` | Rust incremental computation, memoized queries, tracked functions, durability, revisions | 3 candidates: `/websites/rs_salsa` (796), `/websites/salsa-rs_github_io_salsa` (book, 80.2), `/salsa-rs/salsa` (234) |
| 7 | C8 | c7 resolve | `diffsol` | Rust ODE DAE solver BDF SDIRK mass matrix Jacobian sparse | 1 candidate `/martinjrobins/diffsol` (396 snippets) |
| 8 | C2 | c7 resolve | `serde_arrow` | Rust structs to Arrow `RecordBatch` via serde, `SerdeArrowSchema`, explicit schema | **no match** — resolved to `serde` / `apache/arrow` instead |
| 9 | C7 | c7 resolve | `uom` | Rust compile-time dimensional analysis, type-safe quantity arithmetic, SI base units | 1 candidate `/iliekturtles/uom` (58 snippets, High) |
| 10 | C7 | c7 query-docs | `/sarah-quinones/faer-rs` | sparse LU and QR factorization and solving linear systems | sparse Cholesky solve, `try_new_from_triplets`, decomposition list |
| 11 | C7 | WebFetch | `https://docs.rs/num-dual/latest/num_dual/` | version, `DualNum` trait, all dual types, differentiation drivers | 0.15.0; full type and driver inventory incl. `implicit_derivative*` |
| 12 | C7 | WebFetch | `https://docs.rs/faer/latest/faer/` | version, modules, matrix types, decompositions, sparse solvers, `Par`, `MemStack` | 0.24.4; `sparse::linalg::solvers::{Llt,Lu,Qr}`; no condition-number or iterative-SVD API |
| 13 | C7 | c7 query-docs | `/iliekturtles/uom` | defining a custom quantity system with `system!` and `quantity!` | macro forms for custom systems and added units |
| 14 | C7 | WebFetch | `https://docs.rs/uom/latest/uom/` | version, MSRV, `Quantity` parameters, storage types, compile-time vs runtime resolution | 0.38.0, MSRV 1.68.0; compile-time only, no dynamic unit lookup |
| 15 | C2 | WebFetch | `https://docs.rs/serde_arrow/latest/serde_arrow/` | version, arrow feature flags, schema module, conversion functions | 0.15.0; `arrow-53`…`arrow-59` ✔ |
| 16 | C2 | WebFetch | `https://docs.rs/serde_arrow/latest/serde_arrow/schema/index.html` | `SchemaLike` implementors, `TracingOptions`, strategy metadata keys | `Vec<FieldRef>`/`Vec<Field>`/`SerdeArrowSchema`; `STRATEGY_KEY`, `Strategy`, `Overwrites` |
| 17 | C5 | c7 query-docs | `/websites/salsa-rs_github_io_salsa` | database with tracked functions, inputs, interned structs; setting inputs and invalidation | `#[salsa::db/input/tracked/interned]`, `set_with_durability` |
| 18 | C5 | c7 query-docs | `/websites/salsa-rs_github_io_salsa` | backdating on equal recomputed values; durability and revisions | backdating semantics; memo contents; durability-based shallow verification |
| 19 | C5 | c7 query-docs | `/websites/rs_salsa` | cancellation, parallel execution, accumulators, LRU eviction | `CancellationToken`, `#[salsa::accumulator]`, per-key claiming, `trigger_lru_eviction`, `report_untracked_read` |
| 20 | C5 | WebFetch | `https://docs.rs/salsa/latest/salsa/` | version, macros, traits, tracked-return bounds, `no_eq`/`lru`/`cycle_fn` | 0.28.2; `PartialEq` required for backdating; `returns(...)` modes |
| 21 | C9 | c7 resolve | `object_store` | conditional put, atomic writes, `LocalFileSystem`, `get_range` | `/apache/arrow-rs-object-store` (451 snippets) |
| 22 | C9 | c7 query-docs | `/apache/arrow-rs-object-store` | conditional put with `PutMode::Create`/`Update` and `UpdateVersion` | full `PutMode` enum, `put_opts` contract, optimistic-concurrency loop |
| 23 | C9 | c7 query-docs | `/apache/arrow-rs-object-store` | which backends support conditional put; `LocalFileSystem` atomicity; S3 config | inconclusive on per-backend support — escalated to docs.rs (row 28) |
| 24 | C9 | ~~local~~ → **superseded by row 60** | `arrow_rust_59_datafusion55_advanced_reference_2026-08-23.md` §14.11 | `PutMode`, commit protocol, `rename` caveats | conditional-put semantics + "never rename as a commit primitive" advisory |
| 25 | C9 | c7 resolve | `BLAKE3` | `Hasher`, keyed hashing, `derive_key`, XOF, rayon | `/websites/rs_blake3` (1063 snippets, benchmark 87.15) |
| 26 | C9 | c7 query-docs | `/websites/rs_blake3` | `new`, `new_keyed`, `new_derive_key`, `update`, `finalize` | constructor and update signatures |
| 27 | C9 | WebFetch | `https://docs.rs/blake3/latest/blake3/struct.Hasher.html` | version, full method list, feature gates, truncation safety | 1.8.7; `update_rayon`/`update_mmap*` feature gates; truncation **not documented** |
| 28 | C9 | WebFetch | `https://docs.rs/object_store/latest/object_store/` | version, per-backend conditional-put support, `LocalFileSystem` atomicity | 0.14.1; all five backends support conditional put; local documented atomic |
| 29 | C3 | c7 resolve | `winnow` | parser combinators, spans, `Located`, `Stateful`, precedence | 3 candidates; selected `/websites/rs_winnow_x86_64-pc-windows-msvc` (5004 snippets) |
| 30 | C3 | c7 query-docs | `/websites/rs_winnow_…` | `LocatingSlice`/`Stateful` and the `with_span` combinator | span capture as `Range<usize>` |
| 31 | C3 | c7 query-docs | `/websites/rs_winnow_…` | operator-precedence expression parsing; `ParseError` offset | `combinator::expression` example; `ParseError::offset()` |
| 32 | C3 | WebFetch | `https://crates.io/crates/serde_yaml` | version, deprecation status | **no usable content returned** (SPA shell) — escalated to docs.rs |
| 33 | C3 | WebFetch | `https://docs.rs/serde_yaml/latest/serde_yaml/` | version, deprecation banner, API | **0.9.34+deprecated**; *"(This project is no longer maintained.)"* |
| 34 | C3 | c7 resolve | `serde_norway` | maintained Rust YAML serde library | three viable replacements: `/websites/rs_serde_norway` (5689 snippets), `/bourumir-wyngs/serde-saphyr` (355), `/websites/rs_crate_serde_yml` (69) |
| 35 | C3 | WebFetch | `https://docs.rs/toml/latest/toml/` | version, `Spanned`, features, underlying parser | 1.1.6+spec-1.1.0; `Spanned<T>`; depends on `winnow ^1.0.0` |
| 36 | C3 | WebFetch | `https://docs.rs/winnow/latest/winnow/` | version, MSRV, modules, `combinator::expression` presence | 1.0.4; crate-root page inconclusive on `expression` — escalated |
| 37 | C3 | WebFetch | `https://docs.rs/winnow/latest/winnow/combinator/index.html` | does `expression` exist, with `Prefix`/`Infix`/`Postfix`? | **confirmed present** in 1.0.4 |
| 38 | C11 | ~~local~~ → **superseded by row 61** | `ipopt_standalone.md` §20.5 | C interface flow, callback signatures, bound semantics | full C-API coverage; `IpStdCInterface.h` as source of truth |
| 39 | C11 | c7 resolve | `ipopt-sys` | Rust FFI bindings to the Ipopt C interface | only `/coin-or/ipopt` (upstream C++) — no established Rust binding |
| 40 | C11 | c7 query-docs | `/coin-or/ipopt` | `SetIntermediateCallback`, `Ipopt_get_curr_iterate`/`Ipopt_get_curr_violations` | structured iterate/violation API, ≥3.14; warm-start options |
| 41 | C4 | ~~local~~ → **superseded by rows 57, 58** | `petgraph.md` §16–§18 | matching algorithms; SCC output ordering | `maximum_matching` = Gabow, O(\|V\|³); intra-component order **arbitrary** |
| 42 | C1 | ~~local~~ → **superseded by row 56** | `rust_codegen_syn_quote_proc_macro2_prettyplease.md` | version anchors, structure | syn 3.0.3 / quote 1.0.47 / proc-macro2 1.0.107 / prettyplease 0.3.0; §21 non-macro pipeline |
| 43 | C10 | ~~local~~ → **superseded by row 62** | `datafusion-tracing.md`, `rust_parallel_concurrency_stack_reference_2026-08-19.md` | DataFusion/object-store instrumentation; tokio & rayon pins | `instrument_with_spans!`, `instrument_rules_with_spans!`, version lockstep; tokio 1.53.1, rayon 1.12.0 |
| 44 | C10 | c7 resolve | `miette` | `Diagnostic` derive, `SourceSpan`, composing with `thiserror` | `/websites/rs_miette` (1887 snippets) |
| 45 | C10 | c7 query-docs | `/websites/rs_miette` | deriving `Diagnostic` with code/severity/help/url and labelled spans | full derive form, `Diagnostic` trait methods, libraries-return-concrete-types rule |
| 46 | C10 | c7 resolve | `tracing` | spans, events, structured fields, `instrument`, subscriber | `/websites/rs_tracing`; also `tracing-subscriber`, `tracing-opentelemetry` |
| 47 | C10 | c7 query-docs | `/websites/rs_tracing` | `#[instrument]` with `skip`/`fields`/`err`; recording fields after span creation | **fields must be pre-declared (`field::Empty`) or `record` is silently dropped** |
| 48 | C6 | c7 resolve | `egglog` | equality saturation with Datalog, rewrite rules, extraction | `/egraphs-good/egglog` (115 snippets) |
| 49 | C6 | c7 query-docs | `/egraphs-good/egglog` | datatypes, conditional rewrite rules, saturation, cost-model extraction | `EGraph`, `parse_and_run_program`, `rust_rule`, `run_ruleset`, extraction API |
| 50 | C8 | c7 resolve | `FeOs` | SAFT/PC-SAFT Helmholtz EOS, phase equilibrium, num-dual | **no match** — unrelated libraries returned |
| 51 | C8 | c7 query-docs | `/martinjrobins/diffsol` | DAE with mass matrix; residual/Jacobian callbacks from Rust rather than the DSL | `OdeBuilder`, `new_implicit_closure`, `NonLinearOp(Jacobian)`, `LinearOp` |
| 52 | C8 | WebFetch | `https://docs.rs/feos-core/latest/feos_core/` | version, core traits, phase equilibrium, AD integration, features | 0.10.1; **depends on `num-dual ^0.14`** — the version-split finding |
| 53 | §1.2 | crates.io API | every crate in §1.2 | current version, release date, maintenance recency | all anchors confirmed current; `syn` had drifted 3.0.3 → **3.0.5**; `serde_yaml` last released **2024-03-25** `[crates.io:serde_yaml]` |
| 54 | §1.3 | cargo | `cargo generate-lockfile` over the §1.2 manifest | does the pinned set resolve coherently? | **362 packages, 17 names at >1 version**; `num-dual` **0.14.2 + 0.15.0** side by side; reverse deps show the split is entirely `feos-core` + `quantity` |
| 55 | §1.5 | **rustdoc** | locally generated, 26 documents, format_version 61, lockfile committed | full API surface of the pinned crates | 190,463 declarations, 589 public traits, 113,713 impl relations → `build/facts/support` |
| 56 | C1 | **rustdoc** | `syn@3.0.5`, `quote@1.0.47`, `proc-macro2@1.0.107`, `prettyplease@0.3.0` | re-source the deleted codegen reference | pins confirmed; `syn 3` (not the `syn 2` most of the ecosystem uses) confirmed |
| 57 | C4 | **rustdoc** | `petgraph@0.8.3` | re-source amendment B | *"Gabow's algorithm… treated as if undirected… O(\|V\|³)… a better time complexity might be used in the future"*, verbatim; **plus: panics if `node_bound() == usize::MAX`** |
| 58 | C4 | **probe** | `probe_graph.rs` | SCC order stability; matching direction-independence; feedback arc sets | PROBE 3 — `tarjan_scc` vs `kosaraju_scc` disagree on intra-component order; matching identical under full edge reversal; `greedy_feedback_arc_set` returns exactly the recycle edge |
| 59 | C9.1 | **probe** | `probe_identity.rs` | blake3 derivation routes and truncation soundness | PROBE 1 — `hash[..16]` ≠ `derive_key(ctx, ·)[..16]`; `finalize_xof(16) == hash[..16]`, so truncation is the XOF prefix and is defined |
| 60 | C9.2 | WebFetch | `https://docs.rs/object_store/0.13.2/` | the trait surface **at the pinned version** | **erratum** — ten methods; `rename`, `copy_if_not_exists`, `rename_if_not_exists`, `put`, `get`, `head`, `delete`, `copy` **do not exist**; 0.13.0 merged them into `*_opts`. `PutMode::{Create, Update, Overwrite}` confirmed present |
| 61 | C11 | **gh** | `coin-or/Ipopt@releases/3.14.16` → `src/Interfaces/IpStdCInterface.h` | re-source the deleted Ipopt reference | 11 exported functions; `Intermediate_CB`'s 12 typed parameters first-hand; `index_style` "0 for C style" verbatim; bound-infinity semantics verbatim; **amendment H's names corrected** to `GetIpoptCurrentIterate` / `GetIpoptCurrentViolations` |
| 62 | C10.2 | crates.io API | `datafusion-tracing`, `instrumented-object-store` | does amendment D's version lockstep hold? | both at **55.0.0** (2026-08-24); **no 55.1.0 exists** against a pinned engine at 55.1.0 — amendment D is currently unsatisfiable |
| 63 | §1.4 | **rustsec** | `rustsec/advisory-db` | advisories against this dependency set | three — `serde_yaml` RUSTSEC-2018-0005, `tracing` RUSTSEC-2023-0078, `object_store` RUSTSEC-2024-0358 — **all patched below our versions**. **No unmaintained advisory exists for `serde_yaml`**, so its deprecation is invisible to `cargo audit` |
| 64 | C2, C3, C5, C6, C7 | **probe** | `probe_identity.rs`, `probe_parse.rs`, `probe_compile.rs`, `probe_numerics.rs` | PROBES 2, 4, 5, 6, 7, 8 | `serde_arrow` traces `[u8;16]` to a 16-field struct and stamps `SERDE_ARROW:strategy`; `implicit_derivative` matches a hand IFT exactly; faer's `BiLinOp` contract verified; salsa backdates and accumulates; `serde-saphyr` refuses four hostile inputs without panicking; egglog extraction is deterministic while its `Debug` rendering is not |

---

## 18. Open questions, with recommendations

The earlier survey's amendment requests are reconciled below. Evidence sections
and receipts remain the original observations; this is a decision update, not a
new full-library probe run. Current pins are owned by Cargo.toml and Cargo.lock.

| Topic from the survey | Current disposition | Remaining verification / trigger |
|---|---|---|
| YAML replacement | `serde-saphyr`, ADR-0021; typed located failures (blueprint §22.1) | Platform authoring negatives; historical PROBE 7b is a library characterization |
| `num-dual`, FeOs and units | One pinned `num-dual` family; FeOs conditional under ADR-0022/R-06; no `uom` or foreign quantity authority (ADR-0026) | Recheck the exact dependency graph and semantic adapter before optional FeOs adoption |
| `object_store` surface and publication | Pinned options APIs; `PutMode::Create` and conditional ref updates; logical hash separate from encoded checksum (ADR-0045) | Existing-byte verification, restore and interrupted publication; R-10 before multi-writer/cloud support |
| Identity and canonicalization | `pse-ids` owns semantic-ID derive-key contexts and v2 logical/encoded hashing (blueprint §5.1/§5.3) | Recursive null/metadata normalization, exact membership and framing fixtures; R-23 for a larger envelope |
| `serde_arrow` and `SERDE_ARROW:*` | Registry supplies explicit schemas; unregistered metadata is rejected (blueprint §4) | Generated boundary conformance, including nested types |
| Salsa durability, accumulators and backdating | Available library mechanisms, **deferred** under ADR-0042/R-01; current compiler uses complete stage keys | R-22 complete finer input bundle and measurements first; changed lineage cannot silently authorize a skipped stage |
| `faer` condition estimates and smallest singular values | Consumer-specific diagnostics in blueprint §15.4–§15.5; sparse LU and explicit operator/conditioning assumptions remain necessary | End-to-end smallest-value/condition-number accuracy and cost; the operator probe alone is insufficient |
| `diffsol` and its numerical stack | Optional later-phase backend in blueprint §13.6/§25 | Qualify its actual Jacobian, solver, dependency and failure routes before claiming backend support |
| `egglog` | Optional; no `Debug` hashing or unguarded numerical identities (blueprint §7.3, R-05) | Cross-process/version extraction and numerical equivalence; prior same-process observations are narrower |
| Ipopt C bindings and structured iterates | Generated owned `-sys` bindings and capability-resolved structured iterate APIs (ADR-0028/0038, blueprint §18.3) | Actual callback/interruption/linkage acceptance for each supported platform; setup preflight is not numerical parity |
| Thread and memory budgets | One declared thread budget, shared runtime, fallible platform reservations (ADR-0046) | Concurrent consumers and release; pool peaks alongside process peaks |
| Maintenance, pins and optional instrumentation | Family/maintenance gates and ADR-0037 register triggers remain | Library upgrades require new evidence; no supporting-library re-resolution is claimed here |

Use [plan 02](../plans/02-blueprint-revision-5-contracts.md) for the revision-5
implementation handoff. Unchanged specialized numerical/provider questions remain
unverified until their actual consumer is implemented and exercised.
