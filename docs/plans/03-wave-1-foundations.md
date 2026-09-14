---
title: Wave 1 — foundations, executed as parallel sub-agent packets
status: in-progress
date: 2026-09-13
adrs: [ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0009, ADR-0027, ADR-0030, ADR-0031, ADR-0039, ADR-0040, ADR-0041, ADR-0042, ADR-0043, ADR-0044, ADR-0045, ADR-0046, ADR-0047, ADR-0048, ADR-0049, ADR-0050, ADR-0051]
phase: 0
---

# Wave 1 — foundations, executed as parallel sub-agent packets

## Context

The repository has finished design and planning (blueprint revision 5, ADR-0001–0048, two
implementation plans, four capability maps with probes) but almost no implementation:
21 of 23 `pse-*` crates are 8–16-line declared boundaries, the schema generator and every
generated tree (`crates/pse-relations/src/generated/`, `docs/generated/`, the non-placeholder
half of `python/pse/contracts/`) do not exist, `cargo xtask golden` does not exist, and four
of the five test families hold one tautological placeholder each. What is real: the
governance machine (workspace pins, `family-check`, 11 governance tests, `codegen --check`
hygiene), the build-provenance chain (`pse-buildinfo` → `pse-py::build_info` →
`pse._build`), the Python extension-type registrar, codec, `Any` lint, numpy boundary and
host probe, the Ipopt `build.rs`, and the ADR/plan/register tooling.

Blueprint §25 defines phase 0 — "Foundations" — as: `pse-schema` registry and codegen;
`pse-ids`; `pse-quantity`; `pse-material`; `pse-mathir` with the operator catalog and
canonicalization; `pse-catalog` over a local store; `pse-authoring` parser and change sets;
passes P0–P3 and P10. Exit criteria: schema governance tests green; expression DSL
round-trips; `pse.canon.v2` metamorphic fixtures and active semantic query admission;
storage identity/integrity fault matrix; quantity/ordered-IR contract fixtures; a complete
snapshot can be published, read back and queried.

Plan 02 hands off seven dependency-ordered packets A–G with fifteen named fixtures. The
revision-5 review (Accept, bounded proposed-design scope) names the "smaller Slice A
runtime" — whole-stage recompilation, only currently consumed backend bindings — as the
viable implementation sequence. This plan cuts that sequence into packets that Opus 5
sub-agents (`impl-plan-exec-subagent` role) execute in parallel with disjoint file
ownership. The packet designs were produced by three design passes (registry/codegen/
authoring/compiler; identity/storage/catalog/runtime; quantity/math IR/DSL) and reconciled
here; the reconciled interface contracts are in §"Interface contracts".

## Decisions

Taken with the maintainer while planning:

| Question | Decision |
|---|---|
| Wave scope | Phase 0 **plus** the load-bearing interfaces plan 02 says must land early: runtime memory reservations (packet E interface and body), the minimal rule compiler for P2 invariants, the compiler pass/port/stage-key skeleton for P0–P3 + P10, generated Python contracts. **Excluded:** P4–P9, P11–P16, kernels, numerics, backends, the Pyomo adapter, `pse.open/compile/solve`. |
| Governance | Merge PR #1, then land the uncommitted revision-5 amendment as one `adr:` PR that also flips ADR-0039–0048 to `accepted` (the revision-5 review carries the Accept verdict the decision rule requires). |
| Topology | One serial **keel** lands shared interfaces; then one PR per packet from its own worktree branch to `main`; required checks gate every packet; the maintainer squash-merges in tier order. |
| New dependencies | One short ADR (ADR-0049) promotes `serde_json` (manifest codec, extension metadata JSON) and `uuid` (`v7`, explicit-policy IDs) to direct `=` pins with §3.1 rows. Both already resolve in `Cargo.lock` (1.0.151, 1.26.1); the graph does not change. |

Taken by the orchestrator while reconciling the three designs (routine choices within the
accepted decisions; each is reversible inside the wave):

| Choice | Decision and reason |
|---|---|
| Where `Opcode` lives | `pse_quantity::enums::Opcode` (the registry types `quantity_operations.opcode` as `enum(Opcode)` and `pse-quantity` sits below `pse-mathir`); `pse-mathir` re-exports it; a conformance test asserts member parity with the registry enum. |
| Hashing from the registry | `pse-schema` gains `pse-ids` as a dependency (`pse-ids` depends on nothing internal, so the edge is downward); the registry fingerprint is `derive_hash("pse:registry:v1", …)`. |
| Reservation interface | `pse_ids::resource::{MemoryReserver, Reservation, ReserveError, CancellationToken}`; `pse-runtime` implements it over DataFusion's `MemoryPool` (packet E interface lands with the keel, body later — plan 02). |
| Manifest struct | Hand-written Rust wire type in `pse-catalog` for phase 0; the registry declares `ManifestSpec` as the authority for the generated Python `msgspec` struct and `docs/generated`; a parity test guards the two. Generating the Rust struct is a wave-2 item (Open items). |
| Extension-type registry for DataFusion | Hand-written in `pse-catalog::session::registry` over the eleven `pse_relations::ext` impls; not generated. |
| Math IR persistence | `pse-mathir` stays Arrow-free; it exposes `MathRelationSink/Source` traits; the adapter over generated builders lives in `pse-compiler` (`mathir_relations.rs`). |
| `lib.rs` and manifest ownership | Keel packets write every crate's `lib.rs` module list, every `Cargo.toml`, and empty documented stub files; fan-out packets only fill module bodies and add test files. No two packets edit one manifest. |
| DSL grammar | Parser accepts a documented superset of §7.7 (equation/predicate entry points, `where` let, `eps=` last argument, `^` right-associative, `-x^2` refused as ambiguous, comprehension form `… for i in …` refused); each item is a revision-6 amendment row. |

**Non-negotiables every packet inherits** (AGENTS.md, `.claude/rules/rust.md`, lints,
governance tests): zero baseline; `unwrap/expect/panic/todo/print/dbg` denied outside tests;
`missing_docs` with `-D warnings`; every `#[allow]` carries `reason =`; bans on
`Field::extension_type()`, `SessionConfig::set_str`, `SchemaLike::from_type/from_samples`,
`IpcWriteOptions::try_with_compression`, `anyhow` in `crates/*`; every `pub enum *Error`
derives `thiserror::Error` + `miette::Diagnostic` with a §23.2 code in Rust-path form
(`compile::math::unit_inconsistent`, `runtime::resource_limit`, …); `pse-ids` is the sole
`blake3` owner and no `{:?}` reaches a hash; `pse-ids/quantity/material/mathir` depend on no
engine crate; engine crates import Arrow through `datafusion::arrow`; generated paths are
written only by `cargo xtask codegen`; every test invocation passes
`--features pse-relations/force-validate` (`just test` does; for packages that do not depend
on `pse-relations` use `just test-package <pkg> -p pse-relations`); tools from `.venv`,
never `$PATH`; `BTreeMap`/sorted `Vec` wherever order reaches output or a hash.

**Superseded behaviours that must not be implemented by accident** (they still read as
prose in older accepted ADRs): a single `RelationSpec.producing_pass_id` (ports own
production); a `cache_key_inputs` list (every input port is keyed, absence included); P6
reading P7/P8/P15 outputs; the two-process plan-byte equality test; one digest for logical
content and encoded bytes (`pse.canon.v1`); the `Exact`-pushdown wrapper oracle used alone
(compare against a complete unpruned result); passive extension registration as
validation; per-session unbounded/greedy pools; `HashMap` metadata iteration order
reaching any hash; `object_store` `put/get/copy_if_not_exists/rename` (only `*_opts` exist
at 0.13.2, and a rename is never a commit).

## Plan

### Stage 0 — landing (maintainer plus one agent, serial)

1. **Merge PR #1** (`chore/complete-repository-setup`, 9 commits ahead of `origin/main`).
2. **Land the revision-5 amendment** from the current working tree (41 changed/untracked
   files: blueprint, ADR-0039–0048, nine supersession edits, capability maps, plan 02, two
   reviews, evidence) as one PR:
   - branch `adr/0039-revision-5-contracts`; title
     `adr: ADR-0039 enforce semantic admission and complete physical quantity operations`;
     labels `adr`, `kind/design-decision`. `governance / adr-lint` requires the `adr` label
     and that title form whenever any `docs/adr/NNNN-*.md` changes; it does not limit a PR
     to one record.
   - before committing, on each of ADR-0039–0048: `status: accepted`; `review:` →
     `docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates`;
     `evidence: Proposed` stays (the design is accepted, the runtime is not); append
     `- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope).` to
     `## Status history`. The records are not on `origin/main`, so the immutability lint
     has nothing to compare.
   - PR description states `PSE_DESIGN_EDIT=1` was used for the authorised blueprint
     amendment and cites the revision-5 review (plan 02 open item).
   - checks: `just adr-lint`, `just adr-index`, `just docs`, `just lint-repo`.
3. This plan file (`docs/plans/03-wave-1-foundations.md`) gets its `docs/plans/README.md`
   row and `docs/SUMMARY.md` entry in packet K-2.

### Execution topology

- **Worktrees.** Each packet runs in its own worktree and branch
  (`git worktree add ../pse-arrow-wt/<packet> -b wave1/<packet> main`; the Agent tool's
  `isolation: "worktree"`). Packets own disjoint file sets; a packet that must touch a file
  it does not own reports the conflict instead of editing (the `impl-plan-exec-subagent`
  contract).
- **One PR per packet** to `main`, Conventional-Commit title with the crate short name as
  scope (`feat(ids): pse.canon.v2 canonicalizer`), the PR template's Evidence section
  filled with §D labels naming tests and exact `just` commands with failure counts against
  baseline zero, and "Not verified" filled honestly.
- **Tiers.** A packet starts when every `depends-on` packet is merged to `main`. The
  orchestrator rebases waiting worktrees onto `main` after each merge. Registry changes and
  their regenerated trees land in the same PR (`just codegen` is an `ask` permission; the
  packet reports the regeneration).
- **Integration checks** before requesting a merge: `just ci-fast`, `just governance`,
  `just codegen-check`, `just family-check`; `just quality` for Python-touching packets;
  `just docs` for docs-touching packets. `just ci-pr` at every tier boundary.
- **Roles.** `plan-scout` preflights each tier's packets against `main` before dispatch;
  `impl-plan-exec-subagent` executes a packet; `precision-integrator` handles a packet that
  must change a contract another packet consumes; `plan-auditor` audits each tier before the
  next starts; `design-reviewer` reviews the implementation at wave exit;
  `architecture-docs-writer` with the `adr` skill executes K-2's decision records.
- **Reports.** Every packet returns packet ID, changed files, command/mode/failure counts,
  unresolved dependencies, evidence labels, and discoveries outside scope (surfaced, not
  acted on). A stub or a focused green check is never reported as terminal acceptance.
- **Decision routing.** A packet that finds it must alter a D-decision, a crate boundary, a
  dependency family or a SHOULD stops and routes through the `adr` skill; the orchestrator
  decides whether to fork the packet.

### Hand-off packet (what every dispatch prompt contains)

1. The packet row below (ID, title, blueprint sections, ADRs, owned files, deliverables,
   acceptance) and the relevant excerpts of §"Interface contracts".
2. The non-negotiables and superseded-behaviour lists above, verbatim.
3. Prerequisite reading: `AGENTS.md`, `.claude/rules/rust.md` (or `python.md`), the cited
   blueprint sections, the cited ADRs, the cited capability-map sections
   (`just lib-outline` first).
4. The exact acceptance commands and the report format.
5. Commit authorisation: commit on the packet branch; never push to `main`; open the PR
   with the Conventional-Commit title given in the row; do not merge.

### Tier 0 — keel

| ID | Title | § / ADR | Depends on | Owned files | Deliverables and acceptance | Size |
|---|---|---|---|---|---|---|
| **K-1** | `pse-ids` core: identities, framing, floats, encoding checksum, reservation interface, snapshot membership | §5.1, §5.3 steps 4–7, §14.3 · ADR-0007, 0030, 0045, 0046 | — | `crates/pse-ids/src/{lib,error,id,derive,float,frame,encoding,resource,contract,snapshot}.rs`, `crates/pse-ids/tests/golden_vectors.rs` | Everything in contracts §I.1 except `canon/`; frozen hex golden vectors for every derive context, `derive_hash`, `snapshot_id`, `encoding_checksum`; `SemanticId::NIL`. `just test-package pse-ids -p pse-relations`; `just governance` (`blake3_owner`, `banned_patterns`, `error_taxonomy`). | M |
| **K-2** | Decision records, revision-6 amendment batch, manifests, test-crate scaffolding | §3.1, §3.2, §4.2, §4.4, §6.3, §6.9, §7.7, §22.2 · new ADR-0049, 0050, 0051 | Stage 0 | `docs/adr/{0049,0050,0051}-*.md`, `docs/authoritative_design/blueprint.md` (revision-6 rows only, `PSE_DESIGN_EDIT=1`), `docs/adr/register.md` (rows for deferred items below), `Cargo.toml`, every `crates/*/Cargo.toml` touched by the wave, `xtask/Cargo.toml`, `benches/Cargo.toml`, `tests/{conformance,engine,lifecycle}/{Cargo.toml,src/lib.rs,src/*.rs stubs}`, `tests/governance/tooling_deps.toml`, `docs/plans/03-wave-1-foundations.md` (this plan, committed with `status: in-progress`), `docs/plans/README.md`, `docs/SUMMARY.md` | **ADR-0049** `serde_json` (`pse-catalog`, `pse-relations`, `xtask`) and `uuid` v7 (`pse-authoring`) as direct `=` pins + §3.1 rows. **ADR-0050** identity framing: u64-LE length-prefixed parts; `derive_key` for IDs/fingerprints; contexts `pse:named:v1`, `pse:registry:v1`, `pse:stage_key:v1`, `pse:settings:v1`, `pse:mathir:node:v1`; `pse.canon.v2` and `pse.snapshot.v2` frame layouts; `Envelope::PHASE1`; IPC alignment 64 / `MetadataVersion::V5`. **ADR-0051** generated trees: `crates/pse-authoring/src/generated/documents.rs` added to ADR-0031's list; `codegen --check` is regeneration equivalence; `GENERATED.sha256` written by xtask. **Revision-6 blueprint rows:** §3.1 rows; §4.4 `pse.enum`/`pse.ordinal_ref` metadata carry `enum_id`/`target_relation_id`; §22.2 `change_ops.row : struct<staged_port: text, staged_ordinal: u64>`; `normalized.package_graph` columns; §6.9 `math_piecewise_linear`, `math_quantity_selections`, `math_broadcasts.bound_index`; §7.7 grammar superset; §7.2 phase-0 boolean guards note; §4.2 generated paths. Manifests: `pse-schema += pse-ids`; `pse-rules += pse-schema`; `pse-compiler += pse-authoring, pse-mathir, pse-quantity, pse-schema`; `pse-runtime += datafusion-execution, datafusion-common`; `xtask += pse-schema, pse-compiler, pse-catalog, pse-authoring, tokio, tempfile, sha2`; dev-deps `proptest`/`insta` on `pse-quantity`, `pse-mathir`, `pse-authoring`; features `fixtures` on `pse-quantity`/`pse-material`, `arbitrary` on `pse-authoring`; test crates get the union of dev-deps and `pub mod` stubs (`conformance::{canon_fixtures,fixture}`, `engine::oracle`, `lifecycle::fault_store`). Title `adr: ADR-0049 pin serde_json and uuid as direct dependencies`, labels `adr`. `just adr-lint`; `just family-check`; `just governance`; `just docs`. | M |
| **K-3** | `pse-schema` registry model, builder, assembly, fingerprint, codegen API; skeleton `lib.rs`/`error.rs` for the A/C crates | §4.1–4.5, §5.1, §6.1, §20.2, §22.2 · ADR-0004, 0031, 0039, 0045 | K-1, K-2 | `crates/pse-schema/src/{lib,error,builder,fingerprint,membership,arrow,ext_metadata}.rs`, `crates/pse-schema/src/model/**`, `crates/pse-schema/src/catalog/{mod,enums_platform,s4_schema,s6_1_identity,s22_change_sets,manifest}.rs` + empty `declare()` stubs for every other `catalog/*.rs` listed in tiers 1–2, `crates/pse-schema/src/codegen/mod.rs` (+ stub `rust/mod.rs`, `python/mod.rs`, `markdown/mod.rs`, `jsonschema.rs`), `crates/pse-schema/tests/{registry_assembles,fingerprint_stable}.rs`; `crates/pse-relations/src/{lib,error}.rs` + stubs (`ext/mod.rs`, `validate/mod.rs`, `cells.rs`, `registry_relations.rs`, `migrate.rs`, `generated/mod.rs` placeholder); `crates/pse-authoring/src/{lib,error,span}.rs` + stubs (`dsl/mod.rs`, `document/mod.rs`, `ids.rs`, `change_set/mod.rs`, `targets.rs`, `p0.rs`, `p1/mod.rs`); `crates/pse-compiler/src/{lib,error}.rs`, `passes/mod.rs` (the `Pass` trait and bundle types) + stubs; `crates/pse-rules/src/{lib,error}.rs` + stubs | Contracts §I.2 model types, `RegistryBuilder`, `catalog::assemble()` with the fixed call list, the six `reference.schema_*` relations, `EXTENSION_TYPES` (11), logical-type catalog, platform enums, `ManifestSpec`, fingerprint/membership/arrow/ext-metadata modules, `codegen::generate()` returning empty trees; every error enum with its §23.2 codes so `error_taxonomy` is non-vacuous. `registry_assembles`; `fingerprint_stable` (insta). `just test-package pse-schema -p pse-relations`; `just check`; `just clippy`; `just governance`. | L |
| **K-4** | Leaf keels: `pse-quantity`, `pse-mathir`, `pse-material` | §4.4, §6.2, §6.4, §6.9, §6.14, §7.1–7.2, §23.2 · ADR-0009, 0030, 0039, 0047 | K-1, K-2 | `crates/pse-quantity/src/{lib,dimension,ids,enums,error,index}.rs` + stubs for every module in §I.6; `crates/pse-mathir/src/{lib,node,payload,graph,error}.rs` + stubs; `crates/pse-material/src/{lib,ids,enums,error}.rs` + stubs | `Ratio`, `DimensionVector` (8 rationals, canonical bytes), `Opcode` (43, `as_str`, `ALL`), registry enums with `as_str`, `IndexSet`, `QuantityError`; `NodeId`, `Payload`, `ExprGraph::insert` with arity/finiteness checks, `MathIrError`; material ids/enums. In-file tests (reduced form, overflow → error, `Opcode::ALL.len() == 43`, NaN literal rejected, arity mismatch). `just test-package pse-quantity -p pse-relations`; `just test-package pse-mathir -p pse-relations`; `just governance`. | M |
| **K-5** | Engine keels: `pse-catalog` types and manifest, `pse-runtime` types | §5.4, §20.2, §23.2, §14.3, §18.8 · ADR-0044, 0045, 0046, 0048 | K-1, K-2 | `crates/pse-catalog/src/{lib,error,failure,contract,snapshot}.rs`, `crates/pse-catalog/src/store/{mod,layout,manifest,clock}.rs`, `crates/pse-catalog/src/provider/mod.rs` + stubs for `store/{encode,verify,refs,publish,membership,open}.rs`, `provider/{list,catalog,schema,table,pushdown,statistics}.rs`, `session/{mod,profile,registry,admission,config}.rs`, `evidence.rs`; `crates/pse-runtime/src/{lib,error,budget}.rs` + stubs `env,reserve,peak,cancel,session_factory` | Contracts §I.4 `CatalogError` + `classify` (the one §23.2 mapping site), `RelationContract`, `Snapshot`/`LoadedRelation`, `Manifest` serde with `validate/frame/encode/decode`, layout paths, `RefName`, `Clock`, `BoxFut`; `ResourceBudget`, `RuntimeError`. In-file tests: manifest round trip, `validate` negatives, classify table. `just test-package pse-catalog -p pse-relations`; `just test-package pse-runtime -p pse-relations`. | M |

Order: K-1 and K-2 in parallel (K-1 codes against the ADR-0050 constants stated here);
then K-3, K-4, K-5 in parallel. Fan-out begins when all five are on `main`.

### Tier 1 — parallel fan-out (14 packets)

| ID | Title | § / ADR | Depends on | Owned files | Deliverables and acceptance | Size |
|---|---|---|---|---|---|---|
| **A-1** | Registry §6.2–§6.5 | §6.2–6.5, §8.1 · ADR-0039 | K-3 | `crates/pse-schema/src/catalog/{s6_2_physical,s6_3_domains,s6_4_material,s6_5_property}.rs` | 33 relations incl. `normalized.domain_products`, `inferred.valid_index_tuples`; section enums. `registry_assembles`; `just test-package pse-schema -p pse-relations`. | M |
| **A-2** | Registry §6.6–§6.7 and §6.14 IDAES enums | §6.6, 6.7, 6.14 · ADR-0040 | K-3 | `catalog/{s6_6_templates,s6_7_instances,s6_14_idaes_enums}.rs` | Templates (17 relations), `template_property_requirements`, `normalized.property_demand_seeds`, instances/scopes/connections, `inferred.*` incl. `undecided`; all 27 §6.14 enums with `idaes_name`. Same command. | M |
| **A-3** | Registry §6.8–§6.9, §7.2–7.3, §6.12, expression families | §6.8, 6.9, 6.12, 7.2, 7.3 · ADR-0009, 0047 | K-3 | `catalog/{s6_8_symbols,s6_9_math,s7_operators,expr_family,s6_12_derived}.rs` | `math_*` incl. revision-6 additions, `symbols*`, `operator_specs`, `Opcode` and related enums; `declare_expr_family(b, ns, prefix)` for `normalized.{template,instance,display,contribution,guard}_expr_*` and `reference.rule_expr_*`; `declare_normalized_copy()` for P3. Same command. | M |
| **A-4** | Registry §5.2, §6.10, §6.11, §6.13, passes P0–P16 | §5.2, 6.10, 6.11, 6.13, 14.1, 14.5 · ADR-0040, 0041, 0043, 0044 | K-3 | `catalog/{s5_2_revisions,s6_10_cases,s6_11_numerical,s6_13_runtime,s14_passes}.rs` | Cases/targets, `kernel_specs`, `numerical_policies`, `pass_specs`/`pass_input_ports`/`pass_output_ports`, `stage_bundles`, `rule_*`, `engine_profiles`, runtime/provenance incl. `closure_report`; all 17 `PassSpec`s with ports per §14.1 (P1 outputs every authored relation; P3+ never authored/reference). Same command. | L |
| **A-7** | `pse-relations` hand-written engine | §4.3, §4.4, §5.3 step 1, §20.5 · ADR-0039, 0045 | K-3 | `crates/pse-relations/src/{ext/**,validate/**,cells.rs,registry_relations.rs,migrate.rs}`, `crates/pse-relations/tests/**`, `tests/conformance/tests/semantic_admission_paths.rs`, `tests/conformance/fixtures/extension_metadata.json` | Eleven `arrow_schema::extension::ExtensionType` impls (`v` versioned; `deserialize_metadata` errors on unknown), canonical metadata codec (serde_json), `PseFormatterFactory`, the recursive validator (`validate_field`, `validate_schema`, `validate_batch`; plural errors), cells↔batch, registry materialization, migration loader. `semantic_admission_paths` validator half (wrong storage, unknown/malformed/versioned/nested metadata, unregistered `pse.*` name, `SERDE_ARROW:*` key, ordinal syntax). `just test-package pse-relations`; `just test-package pse-tests-conformance -p pse-relations`. | L |
| **A-6** | Rust codegen (starts on the keel's example relations; regenerates the full tree once A-1–A-5 merge) | §4.2, §4.4, §22.1 · ADR-0031, 0039, 0051 | K-3, A-7; full tree after A-5 | `crates/pse-schema/src/codegen/rust/**`, `crates/pse-schema/src/catalog/documents.rs`, `crates/pse-schema/tests/codegen_determinism.rs`; outputs (via `just codegen`) `crates/pse-relations/src/generated/**`, `crates/pse-authoring/src/generated/documents.rs` | Per relation: `RELATION_ID/NAME/NAMESPACE/VERSION/FINGERPRINT`, `schema()`, `<Name>Row`, `<Name>View::try_from_batch` (fingerprint + `try_extension_type` + validate), `<Name>Builder`, `validate`; enums with `as_str/from_str/idaes_name`; migrations; generated conformance tests; `serde_arrow_fields()` only after the `serde_arrow_roundtrip` spike passes; `@generated` header prepended after `prettyplease::unparse`. `codegen_determinism` (two runs byte-equal; no `HashMap` in codegen sources); `no_shadow_structs` non-vacuous. `cargo xtask codegen --only relations && just test-package pse-relations`; `just codegen-check`. | L |
| **R-1** | xtask codegen for real | §4.2 · ADR-0031, 0051 | K-3 | `xtask/src/{main,codegen,golden(stub signature),bindgen(dispatch)}.rs`, `.claude/rules/generated.md` (drop the phase-zero paragraph), `justfile` (`codegen-check` doc string) | `codegen` writes every generated file and removes stale ones; `--check` regenerates into a temp dir and diffs both ways plus the untracked-file check; Python target writes `GENERATED.sha256` (sha2); bindgen arm skipped with a notice when `IPOPT_DIR` is unset (`--only bindgen` exits 2), hygiene-only diff for `bindings.rs` until R-3. `cargo xtask codegen && cargo xtask codegen --check`; `just governance`. | S |
| **Q-1** | Units, unit sets, conversions, kinds, bases, reference states, quantity types, registry | §6.2, §8.1, §8.2, §8.4 · ADR-0039 | K-4 | `crates/pse-quantity/src/{unit,unit_set,conversion,kind,basis,reference_state,quantity_type,registry}.rs` | `convert_spec` (the single source for P3/P9/P10 unit-convert edges; compared by `canonical_f64_bits`), `convert_value` without FMA, `UnitSet` rejecting affine base units, `DerivedUnit`, `ConversionRule`, `QuantityTypeKey`, `QuantityRegistryBuilder::build` with every load-time invariant (positive and negative test each), `resolve_key`, `operations_for`, `neutral_dimensionless`. `just test-package pse-quantity -p pse-relations`. | M |
| **M-1** | Operator table, `operator_specs` emission, hash frames, ordered hash-consing, acyclicity, deterministic numbering | §7.2, §7.3, §7.4 steps 1 and 5, §16.3 · ADR-0009, 0030, 0047 | K-4, K-1 | `crates/pse-mathir/src/{opspec,catalog,hash,canonical,topo}.rs` | `OPERATOR_TABLE` (43 rows; `foldable` set = `Add, Sub, Mul, Div, Neg, Abs, Sqrt, UnitConvert`; `rewrite_conditions` naming the disabled rewrites), `emit_operator_specs` via `OperatorSpecSink`; `structural_hash`/`subtree_hash` (`pse:mathir:node:v1`, floats via `canonical_f64_bits`, scope excluded), `CanonicalGraph` with post-order numbering and `listing()`, cycle detection with path. Tests: table order equals `Opcode::ALL`; commutative children not sorted; `-0.0 ≠ 0.0`; cycle → `cyclic_expression`. `just test-package pse-mathir -p pse-relations`. | L |
| **C-dsl** | Expression DSL: AST, parser, renderer, round trip | §4.4 `pse.expr_dsl`, §7.7 (+ revision-6 superset), §9.3, §10.3, §11.3 · ADR-0009 | K-3 | `crates/pse-authoring/src/dsl/{mod,span,ast,error,lexer,parser,unit_expr,render,arbitrary}.rs`, `crates/pse-authoring/tests/{dsl_roundtrip,dsl_examples}.rs`, `crates/pse-authoring/tests/snapshots/` | `parse_expr/parse_equation/parse_predicate`, `render_*`, `Ast` with byte spans and `structural_eq`, `member_references`, `winnow` over `LocatingSlice` with `cut_err` after keywords, budget (≤ 65 535 bytes, depth ≤ 64), even-arity `weighted_mean`, `DslError` with offsets. `dsl_examples`: every blueprint example parses and snapshots. `dsl_roundtrip`: 1024 proptest cases `parse(render(a)) ≡ a`, persistence file committed. `just test-package pse-authoring -p pse-relations`; `just doctest`. | L |
| **B-canon** | `pse.canon.v2` canonicalizer | §5.3 steps 1–5, 8 · ADR-0030, 0045 | K-1 | `crates/pse-ids/src/canon/{mod,order,normalize,metadata,envelope,ipc}.rs`, `crates/pse-ids/tests/canon_unit.rs` | `canonicalize`, `logical_hash`, `stages::{admit_and_order, normalize, metadata_relation, frame_and_hash}`, supported-layout matrix (§I.1), envelope preflight with checked arithmetic and reservations, `RowConverter` ordering `ASC NULLS FIRST`, recursive normalization (dictionary decode with bounds + enum membership, zero-offset rebuild, null payload zeroing incl. nested validity, validity omitted when all valid, i32 offsets), NaN canonicalization on the hashing copy only, zero-row batch emitted. Crate-local proptests incl. E4 verbatim. `just test-package pse-ids -p pse-relations`. | L |
| **B-providers** | Catalog/schema/table providers with truthful pushdown | §5.4 · ADR-0013, 0048 | K-5 | `crates/pse-catalog/src/provider/{list,catalog,schema,table,pushdown,statistics}.rs`, `tests/governance/tests/no_mutating_providers.rs` | Sealed `SnapshotCatalogList`/`SnapshotCatalog`/`NamespaceSchema` (`register_*` → `Sealed`), one generic `RelationTable` (`scan_with_args`, `Schema::project`, `classify` pure and idempotent, `apply_filters` via physical expr + `filter_record_batch`, `Exact` only for key/enum equality, `IS [NOT] NULL`, non-negated `InList`, AND/OR of Exact; any `Cast`/float → `Unsupported`), `statistics_for` with `Precision`, `Constraints::new_unverified`. Governance test asserts the implemented `TableProvider` method set equals `{schema, constraints, table_type, scan, scan_with_args, supports_filters_pushdown, statistics}`. `just governance`; `just test-package pse-catalog -p pse-relations`. | L |
| **E-1** | Shared runtime and DataFusion-backed reservations | §14.3, §18.8 · ADR-0046 | K-5, K-1 | `crates/pse-runtime/src/{env,reserve,peak,cancel}.rs` | `SharedRuntime::build` (`FairSpillPool` → `TrackConsumersPool` → `PeakRecordingPool`; `MemoryLimit::Finite` asserted; `DiskManager` with directory and max size), `PoolReserver` implementing `pse_ids::resource::MemoryReserver` (`try_grow` before allocation; `ResourcesExhausted` → `ReserveError::Exhausted` naming owner and keys; release on drop/cancel), `ResourceReport` with process peak RSS beside pool peak, `CancelSource`. `just test-package pse-runtime -p pse-relations`. | M |
| **Q-material** | Elements, phase-validity predicate, stoichiometry helpers | §6.4, §6.14 · ADR-0039 | K-4 | `crates/pse-material/src/{element,phase_validity,stoichiometry,testdata}.rs`, `crates/pse-material/tests/phase_validity.rs` | `ElementTable`, `species_valid_in_phase` (four-valued minus conflict), `molecular_weight`, `element_balance` (diagnostic, not invariant). Deferred: material systems, Henry, apparent species, basis conversion. `just test-package pse-material -p pse-relations`. | S |

### Tier 2 (10 packets)

| ID | Title | § / ADR | Depends on | Owned files | Deliverables and acceptance | Size |
|---|---|---|---|---|---|---|
| **A-5** | Invariants, assembly checks, rule dependencies | §4.1, §6.1, §6.4, §14.2 rules 2/7 · ADR-0004, 0030 | A-1–A-4 | `crates/pse-schema/src/{checks,rule_deps}.rs`, `catalog/{invariants,inv}.rs` | Auto `unique:*:pk` and `foreign_key:*`; `closure:entity_registered:*`, `closure:stoichiometry_species_in_phase`, `closure:material_system_species_exist` (unnest), `closure:packages.dependencies_resolved`, `acyclic:{entities,instances,cases,model_revisions}` (recursive, depth-bounded), `check:*`, `domain:*`, `cardinality:*`, `rules.stratified_negation`, `stage_graph.*`; `rule_dependencies` rows; float-key and stratification checks at assembly. `just test-package pse-schema -p pse-relations`. | M |
| **A-8** | Python codegen and Python tests | §4.4, §21.5, §6.14 · ADR-0004, 0031 | A-5, A-7 | `crates/pse-schema/src/codegen/python/**`, `python/pse/codec/__init__.py`, `python/pse/tests/{test_no_shadow_contracts,test_generated_manifest_current,test_extension_metadata_canonical,test_relation_rows_structure,test_manifest_fingerprint}.py`, `python/pse/parity/tests/test_enum_parity.py`; output `python/pse/contracts/**` | attrs row classes per relation, enums with `IDAES_NAMES`, `msgspec` `Manifest` from `ManifestSpec`, eleven pyarrow extension types with the revision-6 metadata shapes, idempotent registrar, `REGISTRY_FINGERPRINT`, `GENERATED.sha256`; `pse.codec.converter()` hooks for `SemanticId`/`ContentHash`/`datetime`; `pse.codec.Manifest` re-exported from contracts. Parity test compares `idaes_name` members with IDAES 2.12.0 (container only). `cargo xtask codegen --only python`; `just py-sync && just py-test && just quality`. | M |
| **A-9** | Markdown docs and authoring JSON Schema | §4.2, §22.1 · ADR-0031, 0036 | A-5, A-6 | `crates/pse-schema/src/codegen/{markdown/**,jsonschema.rs}`, `docs/SUMMARY.md` (generated section entries); output `docs/generated/**` | `docs/generated/{README.md, relations/<ns>.md, enums.md, extension_types.md, passes.md (mermaid DAG), rules.md, schema/authoring.schema.json}` with the `@generated` marker. `cargo xtask codegen --only docs`; `just docs`. | S |
| **Q-3** | Quantity operations and the §8.3 inference algebra | §6.2 `quantity_operations`, §8.3 (every row), §7.4 step 4 · ADR-0039 | Q-1 | `crates/pse-quantity/src/{operation,infer,literal}.rs` | `infer(&OpRequest, &[Operand], &QuantityRegistry) -> Inferred` implementing the decision table (built-in rules for literals, additive sums, origin-sensitive ±, neutral scaling, `WeightedMean`, reductions, gather/broadcast, conditional, unit convert, kernel/implicit; registered rules for Mul/Div/Pow/transcendental/derivative/integral with ordered-then-swapped matching and component rules). Tests per row: `T+ΔT` ok, `ΔT+ΔT` ok, `T1−T2` same datum → difference, different datum rejected, `T1+T2` rejected, `α·ΔT` stays difference, ambiguous rule rejected, swapped match recorded, sum of points rejected, `certified_unit_sum` needs invariant. `just test-package pse-quantity -p pse-relations`. | L |
| **M-3** | Guarded literal folding | §7.3, §7.4 step 3, §18.2 · ADR-0047 | M-1, Q-1 | `crates/pse-mathir/src/fold.rs` | `fold_literals` with the seven-clause fold-safety predicate (foldable opcodes only; direct literal children only; never under a `Conditional` branch; finite result; domain checked; unit agreement; bitwise-equal to ordered evaluation), `FoldReport` with `deferred_static_checks`. Tests: `1e16 + (−1e16 + 1)` → 0 (ordered), `Add(Add(x,1),2)` untouched, `Mul(1e308,10)` untouched, `Div(1,0)` static violation at root but untouched under a branch, `Mul(−1,0)` → `−0.0`, `Log(Exp(1000))` untouched. `just test-package pse-mathir -p pse-relations`. | M |
| **C-1** | Authoring loaders, spans, ids | §22.1, §5.1 · ADR-0021, 0049 | K-2, A-6 | `crates/pse-authoring/src/{document/**,ids.rs}`, `crates/pse-authoring/tests/{load_package,hostile_yaml,ids}.rs`, `tests/fixtures/packages/{minimal_explicit,minimal_named}/**` | `load_package` (serde-saphyr with budget, `toml::Spanned`, `SpanIndex` path → `SourceSpan`), generated document structs consumed, `entity_id` for explicit/named policies, `assign_ids` (uuid v7, text edits at the entity span), `parse.missing_id`; four hostile YAML inputs refused without panic. `just test-package pse-authoring -p pse-relations`. | M |
| **B-store** | Artifact store, publication protocol, refs, open/restore | §20.1, §20.2, §20.5, §5.3 step 7 · ADR-0045, 0046 | B-canon, K-5 | `crates/pse-catalog/src/store/{encode,verify,refs,publish,membership,open}.rs` | Protocol: validate batches → canonicalize (keep sorted) → `complete_members` from registry snapshot classes (empties explicit) → `snapshot_id` → encode IPC file (+ Parquet for authored/reference) with finish and decode-verify → `PutMode::Create` (verify checksum/length/format on `AlreadyExists`) → manifest under its own checksum → ref CAS (`PutMode::Update` with prior version; `Create` if absent; `RefUpdatePolicy`); `Catalog::{open, publish_bundle, set_ref, head, list_refs, snapshot, resolve}`; `TrustLevel::Untrusted` decodes, validates and rehashes; stage sidecar `stage_lookup/stage_record` under `stages/<key>.json` (excluded from membership); `RelationContract::from_spec`. In-file tests over `object_store::memory::InMemory`. `just test-package pse-catalog -p pse-relations`. | L |
| **B-session** | Session construction, active admission, engine profile, settings | §4.4, §14.2 rule 5, §14.3, §23.2 · ADR-0039, 0044, 0046 | B-providers, A-7 | `crates/pse-catalog/src/session/{mod,profile,registry,admission,config}.rs`, `tests/engine/tests/{semantic_admission_paths,engine_profile_settings}.rs` | `build_session` (`with_default_features` then typed `ConfigOptions::set`, `information_schema = true`, `explain.format = pgjson`, asserted `enable_ansi_mode = false` and `skip_physical_aggregate_schema_check = false`, shared `RuntimeEnv`, sealed catalog list, extension registry over the eleven impls asserting `add_extension_type_registration` returned `None`, analyzer rules `[pse.admission.first] ++ profile ++ [pse.admission.final]`, explicit optimizer/physical lists, `PseOptions` under `datafusion.pse.*`); `admit_plan` (`apply_with_subqueries`, expressions, nested fields, `TableScan` must be a `RelationTable`); `SnapshotSession::{sql, execute_plan, table_reference, read_back_settings, settings_hash, profile_hash, function_registry_hash}` with observer closures and bounded collection; `phase0_reference_profile`; `RuleCatalog` by name → `config::invalid` on unknown; `ThreadBudget::validate`. `semantic_admission_paths` (E1 regression: ordinary SELECT over wrong storage / unknown name / malformed / versioned / nested / foreign source rejected before execution; direct factory call as positive control); `engine_profile_settings`. `just test-package pse-tests-engine -p pse-relations`. | L |
| **B-fixtures** | Canonicalization conformance fixtures and benchmark | §5.3, §24.1 "Canonicalization", §24.3 · ADR-0045 | B-canon (encoding part after B-store) | `tests/conformance/src/canon_fixtures.rs`, `tests/conformance/tests/{canonical_null_equivalence,encoding_roundtrip_identity,snapshot_membership}.rs`, `benches/src/lib.rs`, `benches/benches/canonicalization.rs` | Generators and the identity-preserving/distinguishing transform matrix (hidden null payload, nested masks, slices, dictionary re-encoding, batch/row order, all-valid bitmap, NaN/signed zero, empty relations); IPC-file/Parquet round trips share logical identity with distinct checksums; membership: empties explicit, changed contract/parent, sidecar excluded, duplicate/missing ports rejected; criterion group per canonicalization stage replacing the placeholder. `just test-package pse-tests-conformance -p pse-relations`; `just bench-smoke`. | M |
| **R-3** (optional) | Ipopt bindgen arm | §18.3 · ADR-0031 | R-1; solver image | `xtask/src/bindgen.rs`, output `crates/pse-ipopt-sys/src/bindings.rs` | bindgen over `IpStdCInterface.h` with the eleven-function allowlist, run with `IPOPT_DIR` (dev image); `--check` becomes regeneration equivalence for `bindings.rs`. If this slips, `bindings.rs` keeps the hygiene-only check and a register row records it. `just parity-container -- cargo xtask codegen --only bindgen --check`. | S |

### Tier 3 (9 packets)

| ID | Title | § / ADR | Depends on | Owned files | Deliverables and acceptance | Size |
|---|---|---|---|---|---|---|
| **Q-4** | Standard package fixture, admission and numeric helpers, proptests | §8.3, §8.2, §14.2 rule 6, §7.3 · ADR-0039, 0026 | Q-3 | `crates/pse-quantity/src/{standard,admission,numeric}.rs`, `crates/pse-quantity/tests/{dimension_props,standard_package}.rs` | `standard_registry()` (feature `fixtures`; named-policy ids; SI + °C/°F/psig/`USD_<year>`; kinds with `addition_kind`; point/difference variants; the six §8.3 required compositions plus swapped `Mul` forms; `Log[mole_fraction]` declared result), `require_same_contract`, exact `i64↔f64` representability, `approx_eq` per §7.3, `is_zero`. Doc states the YAML units package is the authority and this is its test double. `dimension_props`; `standard_package` (all required compositions positive and negative). `just test-package pse-quantity -p pse-relations --features pse-quantity/fixtures`. | M |
| **M-4** | Unit inference driver, index model, equation records, `canonicalize()` | §7.4 (all), §7.5 residual, §8.3, §6.9 · ADR-0039, 0047 | Q-3, M-1, M-3 | `crates/pse-mathir/src/{index,infer,equation,canonicalize,walk}.rs` | Node → `OpRequest` mapping, `SymbolTypeSource`, free-index resolution (`UnboundIndex`), residual typing, `QuantitySelection` rows, `canonicalize(input, Policy::Strict)` in the order acyclicity → indices → bottom-up inference → guarded folding → typed hash-consing → numbering; non-canonical unit may only feed `UnitConvert`/kernel inputs; `NodeVisitor` by operator family. `just test-package pse-mathir -p pse-relations --features pse-quantity/fixtures`. | L |
| **C-2** | Change sets, P0, P1, target resolution | §22.2, §14.1 P0/P1, §6.10 · ADR-0005, 0027 | C-1, C-dsl, A-6 | `crates/pse-authoring/src/{change_set/**,p0.rs,p1/**,targets.rs}`, `crates/pse-authoring/tests/{change_set,p1_stage,targets}.rs` | `ChangeSet`/`ChangeOp` (`row` per revision 6), `AuthoredReader` trait, `apply` (rejects derived writes, named-policy rename, unknown row keys), `p0::resolve` (exact `=x.y.z` requirements only), `p1::stage` (documents → rows via generated builders; every `expr_dsl` parsed with span; entities; targets resolved after staging to `case_spec_targets`/`case_activation_targets`/`observation_targets`; wildcards; `parse.unresolved_target`). Import-path half of `semantic_admission_paths`. `just test-package pse-authoring -p pse-relations`. | L |
| **C-3** | Minimal rule compiler and P2 | §14.2 rules 3/5/6/7/9, §23.2, §6.11 · ADR-0030, 0044, 0048 | A-5, A-7, B-session, B-store | `crates/pse-rules/src/{plan/**,exec/**,invariants,derivations,errmap}.rs`, `crates/pse-rules/tests/**`, `tests/engine/tests/relational_expansion_p2.rs`, `tests/conformance/tests/invariant_fixtures.rs`, `tests/conformance/src/fixture.rs`, `tests/conformance/fixtures/invariants/<name>/{valid,violating}.yaml` | `compile(RuleSpec, PortBinding, &SnapshotSession)` lowering `scan/filter/project/equi_join/anti_join/union/distinct/aggregate(count)/unnest/recursive` (UNION ALL + depth column bounded by seed rows); `rule::float_key` at compile; exact head admission (`rule::head_schema_mismatch`); four-valued root rewrite → head/undecided split; PK sort; `errmap::classify` at one site; `run_invariants` (P2 body) producing findings/undecided/derivations/explain/`rules_fired`; every registry invariant has a valid+violating fixture pair (test iterates `registry().invariants()`). Deferred: strata scheduling across rules, `conflict`, non-count aggregates, `kernel_call`, per-row derivations of recursive results. `relational_expansion_reference` P2 subset vs an independent Rust reference; `invariant_fixtures`. `just test-package pse-rules -p pse-relations`; `just test-package pse-tests-engine -p pse-relations`; `just test-package pse-tests-conformance -p pse-relations`. | L |
| **B-faults** | Publication fault matrix | §20.1, §24.1 "Lifecycle" · ADR-0045 | B-store | `tests/lifecycle/src/fault_store.rs`, `tests/lifecycle/tests/publication_fault_matrix.rs` | `FaultStore: ObjectStore` wrapper with `FaultPlan` (fail-before, truncate/flip-byte, one-shot precondition, cancel-at-step); cases: truncated/corrupt existing object under a correct name, failed writer finish, interruption before each object/manifest/ref, CAS conflict (retry and fail policies), cancellation mid-publish, untrusted restore of a corrupted artifact; assertions: old ref valid, no mixed snapshot visible, `reserved() == 0` after failure. `just test-package pse-tests-lifecycle -p pse-relations`. | M |
| **B-oracle** | `pushdown_vs_unpruned` | §5.4, §24.1 "Pushdown truthfulness" · ADR-0013, 0048 | B-providers, B-store | `tests/engine/src/oracle.rs`, `tests/engine/tests/pushdown_vs_unpruned.rs` | `UnprunedTable` (same data, all `Unsupported`), `OverPruningTable`, `EmptyTable`, `ExtraRowTable`; multiset comparison via `RowConverter` rows; query matrix: key equality, `IN` (2 and 20), `OR`, `AND` with residual, `IS [NOT] NULL` on a nullable reference column, enum equality (dictionary), projection omitting the filter column, `LIMIT`, aliases; the three negative wrappers must fail the oracle. `just test-package pse-tests-engine -p pse-relations`. | M |
| **E-2** | Budget fixtures and session factory | §14.3, §24.1 "Resource envelope" · ADR-0046 | E-1, B-session, B-store, B-canon | `crates/pse-runtime/src/session_factory.rs`, `tests/lifecycle/tests/{query_memory_budget,canonicalization_memory_budget,result_memory_budget}.rs` | `SessionFactory::open_session` (the only path handing a `RuntimeEnv` to `pse-catalog`); `query_memory_budget` (64 KiB pool; `ORDER BY` over a published 200k-row relation → `ResourceLimit` naming consumer and keys; two concurrent sessions share one pool; cancel one → its reservations return to zero; report shows pool peak beside process peak); `canonicalization_memory_budget` (reserver below the preflight estimate fails before allocation; `publish_bundle` under budget publishes no ref; `reserved() == 0` after failure/cancel); `result_memory_budget`. `just test-package pse-tests-lifecycle -p pse-relations`. | M |
| **B-evidence** (optional) | Plan evidence codec and round trip | §14.2 rule 5, §20.2, §6.13 · ADR-0044 | B-session, B-store | `crates/pse-catalog/src/evidence.rs`, `tests/engine/tests/plan_evidence_roundtrip.rs` | `PsePlanCodec` (`try_encode_table_provider` writes `snapshot_id ‖ relation_id ‖ version ‖ logical_hash`; decode resolves only pinned snapshots; extension nodes/UDFs unsupported in phase 0), `PlanEvidence` with checksum, codec/engine versions, `pgjson` EXPLAIN and `rules_fired`, `write_evidence` under `evidence/<checksum>` marked `canonical: false`. `plan_evidence_roundtrip` asserts nothing about bytes. `just test-package pse-tests-engine -p pse-relations`. | M |
| **A-6 (final)** | Full-tree regeneration after A-5 | — | A-5, A-7, A-8, A-9 | as A-6 | `just codegen` over all four targets; `just codegen-check` clean; `just ci-fast`; `just governance`. | S |

### Tier 4 (4 packets) and wave exit

| ID | Title | § / ADR | Depends on | Owned files | Deliverables and acceptance | Size |
|---|---|---|---|---|---|---|
| **Q-5** | `quantity_composition` conformance fixture | §8.3, §24.1 · ADR-0039 | Q-4, M-4 | `tests/conformance/tests/quantity_composition.rs` | Composed expressions through `canonicalize` against `standard_registry()`: heater energy balance, `F·h` preserving the enthalpy reference state, `F·x` preserving basis, `R·T`, `P/(R·T)`, `T/1000{K}` polynomial, `Ea/(R·T)` in `exp`, `T + α·ΔT`, `WeightedMean` of temperatures, gauge/absolute mix rejected, molar/mass mix rejected, unregistered `length×mass` rejected although dimensions combine. `just test-package pse-tests-conformance -p pse-relations`. | M |
| **M-5** | Relation sink/source, snapshots, proptests, `numerical_policy_conformance` (P10 subset) | §6.9, §7.3, §24.1 · ADR-0047, 0030 | M-4 | `crates/pse-mathir/src/relations/{mod,vec_sink}.rs`, `crates/pse-mathir/tests/{hash_consing_props,canonical_snapshots}.rs`, `crates/pse-mathir/tests/snapshots/`, `tests/conformance/tests/numerical_policy_conformance.rs` | `MathRelationSink/Source`, `emit`, `emit_untyped`, `load_untyped`, `load_canonical`, `VecSink`; shuffled insertion orders → identical listings and hashes; emit→load→emit identical; excluded-branch `Div(1,0)`/`Log(0)` never folded or flagged; no reassociation; signed zero survives; NaN refused; `Policy::Strict` is the only value. `just test-package pse-mathir -p pse-relations`; `just test-package pse-tests-conformance -p pse-relations`; `just snapshots-accept` once, reviewed. | M |
| **C-4** | Compiler skeleton and driver: DAG, stage keys, memo, atomic commit, P0–P3 + P10 adapters | §14.1, §14.3, §14.4, §20.1 · ADR-0040, 0041, 0042, 0044 | A-4, C-2, C-3, B-store, B-session, M-5, Q-4 | `crates/pse-compiler/src/{passes/{dag,key,bundle,registry,p0,p1,p2,p3,p10}.rs,driver.rs,memo.rs,records.rs,mathir_relations.rs}`, `crates/pse-compiler/tests/**`, `tests/engine/tests/{stage_bundle_graph,incremental_equals_clean_p0_p3,demand_seed_closure_seeds}.rs` | `StageDag::{build, validate, topo_order}` (acyclic; one producer per port; every derived input names an existing output; no P3+ output to authored/reference; P2 reads only authored/reference + registry); `stage_key` (`pse:stage_key:v1`: pass id/version, registry fingerprint, every port sorted with logical hash or explicit absence, policies, engine profile + function registry hashes; plan bytes never); `Driver::commit` (P0 → P1 → publish model/case bundle without ref → P2 → ref CAS only on zero error findings, ADR-0027) and `Driver::run` (memo via stage sidecar; every output batch validated; all-or-nothing publication with parents = input bindings; `pass_records`); P3 adapter (normalized copies, parsed `normalized.*_expr_*` via `MathRelationSink` adapter, constants to package units via `convert_spec`, demand seeds) and P10 adapter (`canonicalize`); `PipelineRequest.external_bindings` for tests only. `stage_bundle_graph`; `incremental_equals_clean` phase-0 (clean twice → equal hashes and keys; add a species → P1–P3 keys change, P0 reused; absent optional port changes the key; changed engine profile changes P2's key); `demand_seed_closure` seeds only. `just test-package pse-compiler -p pse-relations`; `just test-package pse-tests-engine -p pse-relations`. | L |
| **R-2** | Golden stores and remaining governance tests | §24.1, §5.4, §25 · ADR-0004, 0027, 0031 | R-1, C-4, A-7, B-store | `xtask/src/golden.rs`, `tests/golden/{registry,minimal_explicit}/**`, `tests/governance/tests/{codegen_regeneration,registry_governance}.rs`, `tests/governance/appendix_b_deferred.toml`, `python/pse/tests/test_golden_registry.py` | `cargo xtask golden <name>` publishes a store from a fixture (registry snapshot; `minimal_explicit` package + case through P0–P3, P10) and `--check` compares logical hashes and snapshot ids (physical checksums may differ); `codegen_regeneration` (in `just test`); `registry_governance` (every Appendix B relation registered or listed with a reason; no authored/reference FK into compiled/runtime; no compiler output port to authored/reference; every relation has a snapshot class); Python reads the golden registry store through the generated contracts. `cargo xtask golden registry && cargo xtask golden minimal_explicit`; `just governance`; `just py-test`. | M |
| **W-exit** | Wave verification, implementation design review, plan outcome | §25 · all | everything above | `docs/plans/03-wave-1-foundations.md` (Outcome), `docs/design_review/reviews/design_review_wave-1-foundations_<date>.md`, ADR evidence-label edits by supersession rules (status fields only) | Run the Verification table below in the stated modes; `design-reviewer` reviews the implementation against DM/G gates; `plan-auditor` verifies every packet landed with wiring, not symbols; record what was built, a mistake corrected, deliberate deviations; move `status: done`. | S |

## Interface contracts

Signatures other packets code against. Each is owned by the packet named in brackets; a
change to one of these is a contract change and goes through `precision-integrator`, never
a silent edit. Bodies, private helpers and tests are the owning packet's business.

### I.1 `pse-ids` [K-1, B-canon]

```rust
// id.rs — Copy, Eq, Ord, Hash, Debug; Display = lowercase hex; Debug never reaches a hash
pub struct SemanticId([u8; 16]);   // NIL, from_bytes, as_bytes, try_from_slice, to_hex, parse_hex
pub struct ContentHash([u8; 32]);  // same; to_prefixed() -> "blake3:<hex>", parse_prefixed
pub struct LogicalHash(pub ContentHash); pub struct EncodingChecksum(pub ContentHash); pub struct SnapshotId(pub ContentHash);
pub struct Ordinal(pub u64); pub struct SchemaVersion(pub u32);
// derive.rs — Hasher::new_derive_key(context); each part = u64le(len) ‖ bytes; XOF first 16 bytes for ids
pub mod context { NAMED = "pse:named:v1"; SYMBOL = "pse:symbol:v1"; EQUATION = "pse:equation:v1"; TERM = "pse:term:v1"; CONN = "pse:conn:v1"; NODE = "pse:node:v1";
                  REGISTRY = "pse:registry:v1"; STAGE_KEY = "pse:stage_key:v1"; SETTINGS = "pse:settings:v1"; MATHIR_NODE = "pse:mathir:node:v1"; }
pub fn derive_id(context: &'static str, parts: &[&[u8]]) -> SemanticId;
pub fn derive_hash(context: &'static str, parts: &[&[u8]]) -> ContentHash;
pub struct FramedHasher;  // new(context); part(&[u8]); str(&str); u16/u32/u64 (LE); bool; id(&SemanticId); hash(&ContentHash); finish_hash(); finish_id()
pub struct IndexTuple<'a>(pub &'a [SemanticId]);
pub fn named_id(package_id: SemanticId, qualified_name: &str) -> SemanticId;
pub fn symbol_instance_id(..) / equation_instance_id(..) / law_term_id(..) / connection_equation_id(..) / mesh_node_id(..) / discretized_symbol_id(..);
// float.rs — hashing copy only (ADR-0030)
pub fn canonical_f64_bits(v: f64) -> u64;   // NaN → 0x7ff8_0000_0000_0000; -0.0 preserved
pub fn canonical_f32_bits(v: f32) -> u32;
// frame.rs / encoding.rs
pub trait FrameSink { put_len_prefixed, put_u32_le, put_u64_le, put_fixed }   // impl for Vec<u8> and blake3::Hasher
pub fn encoding_checksum(bytes: &[u8]) -> EncodingChecksum; pub struct EncodingHasher; // impl io::Write (tee for writers)
// resource.rs — implemented by pse-runtime over MemoryPool
pub trait MemoryReserver: Send + Sync + Debug { fn open(&self, owner: &str) -> Box<dyn Reservation>; }
pub trait Reservation: Send + Debug { fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError>; fn shrink(&mut self, bytes: usize); fn size(&self) -> usize; fn release(&mut self); }
pub enum ReserveError { Exhausted { owner, requested, reserved, limit_hint } }   // runtime::resource_limit
pub struct FixedBudget;  // reference impl for tests/benches
pub struct CancellationToken;  // cancel(), is_cancelled(), checkpoint() -> Result<(), CanonError>
// contract.rs — what the canonicalizer consumes (plain values; no registry dependency)
pub struct CanonicalContract { relation_id, schema_version, registry_fingerprint, schema: SchemaRef /* declared, with metadata */, primary_key: Vec<usize>, layouts: Vec<Layout> }
impl CanonicalContract { pub fn try_new(.., primary_key: &[&str], enum_domains: &BTreeMap<FieldPath, Arc<[String]>>) -> Result<Self, CanonError>; pub fn check_batch_schema(&self, &Schema) -> Result<(), CanonError>; }
pub enum Layout { Boolean, Int8..Int64, UInt8..UInt64, Float32, Float64, TimestampNsUtc, Utf8, Binary, FixedSizeBinary(i32), Enum { key: Int8|Int32, members }, List, FixedSizeList, Struct }  // everything else rejected at try_new
pub struct FieldPath(String);  // "" = schema; "0/2" = registry ordinals joined by '/'
pub struct Envelope { max_rows: u64, max_normalized_bytes: u64 }  // PHASE1 = 1_000_000 rows, 256 MiB; lowered() may only tighten
// snapshot.rs — pse.snapshot.v2, plain blake3 over the frame
pub enum SnapshotKind { Model, Case, Stage, Run }
pub struct SnapshotParent { role: String, snapshot_id: SnapshotId }
pub struct SnapshotMember { port, namespace, relation_id, schema_version, logical_hash }
pub struct SnapshotFrame { registry_fingerprint, kind, parents, members }
pub fn snapshot_id(frame: &SnapshotFrame) -> Result<SnapshotId, SnapshotError>;   // duplicate role/port → error; Case requires one "model" parent
pub fn model_port_name(namespace: &str, relation_id: SemanticId) -> String;       // "<namespace>/<32 hex>"
// canon/ [B-canon]
pub const CANON_VERSION = "pse.canon.v2"; IPC_ALIGNMENT = 64; IPC_METADATA_VERSION = V5;
pub struct CanonicalizeOptions { envelope, keep_preimage, keep_sorted, cancel: Option<CancellationToken> }
pub struct CanonicalOutput { logical_hash, row_count, preimage: Option<Vec<u8>>, sorted: Option<RecordBatch> /* original values, PK-sorted */, reservation: Box<dyn Reservation> }
pub fn canonicalize(contract: &CanonicalContract, batches: &[RecordBatch], reserver: &dyn MemoryReserver, opts: CanonicalizeOptions) -> Result<CanonicalOutput, CanonError>;
pub fn logical_hash(contract, batches, reserver) -> Result<LogicalHash, CanonError>;
pub mod stages { admit_and_order, normalize, metadata_relation, frame_and_hash }
// preimage: len‖"pse.canon.v2" · relation_id · u32le version · fingerprint · len‖metadata_stream · len‖data_stream (each StreamWriter(64,false,V5), one batch, finish())
```

### I.2 `pse-schema` [K-3, A-*]

```rust
pub fn registry() -> Result<&'static Registry, SchemaError>;   // OnceLock; assembled from catalog::assemble()
pub struct RelationKey { namespace: Namespace, name: &'static str, version: u32 }      // Display "authored.stoichiometry@1"
pub struct RelationSpec { id: SemanticId /* named_id(REGISTRY_PACKAGE_ID, "relation:<ns>.<name>@<v>") */, key, authority, snapshot_class, derivation_granularity: Option<_>, stability, primary_key: Vec<&str>, columns: Vec<ColumnSpec>, doc, fingerprint: ContentHash }
pub struct ColumnSpec { name, logical_type: LogicalType, nullable, quantity: QuantityContract /* None|Column(id)|PerRow */, fk: Option<ForeignKey>, role: ColumnRole, doc }
pub enum LogicalType { F64, I64, I32, U8, U16, U32, U64, Bool, Text, Timestamp, List(Box<_>), FixedList(Box<_>, i32), Struct(Vec<(&str, LogicalType, bool)>), Ext(ExtensionUse) }
pub enum ExtensionUse { SemanticId, ContentHash, DimensionVector, QuantityValue, Bound, IndexTuple, OrdinalRef { target }, SourceSpan, Enum(&str), ExprDsl, TargetPath }
pub struct ExtensionTypeSpec { name, storage: fn() -> DataType, metadata: ExtensionMetadataShape /* VersionOnly|Enum|OrdinalRef */, metadata_version: u32, doc }
pub const EXTENSION_TYPES: [ExtensionTypeSpec; 11];
pub struct EnumSpec { id, name, idaes_source, members: Vec<EnumMember { name, idaes_name, deprecated, doc }> }
pub struct InvariantSpec { id, name, relation, kind: InvariantKind, rule: &str, severity, doc }
pub struct MigrationSpec { relation, from_version, to_version, steps: Vec<MigrationStep>, doc }
pub struct PassSpec { id, name /* "P2" */, version, inputs: Vec<InputPort { port, relation, source: Pinned | Derived { pass, port }, required }>, outputs: Vec<OutputPort { port, relation }>, preconditions, postconditions, determinism, diagnostics, executes_plans: bool }
pub struct RuleSpec { id, name, version, stratum: u16, head: RuleHead /* Relation(name) | Violations { of, key_columns } */, plan: RulePlan, negation, monotonic, conflict_policy }
pub enum RulePlan { Scan { relation, port }, Filter { input, predicate: RuleExpr }, Project, EquiJoin { keys, null_equality }, AntiJoin, Union, Distinct, Aggregate { group, aggregates /* phase 0: Count */ }, Unnest { column, value_name, null_list, empty_list }, Recursive { name, seed, step, is_distinct, depth_bound } }
pub enum RuleExpr { Col, Lit(Cell), And, Or, Not, Cmp, IsNull, IsNotNull, IsDistinctFrom, IsNotDistinctFrom, InList, Field, ListLen, IsTrue, IsFalse, IsUnknown }
pub enum Cell { Null, Bool, I64, U64, F64, Text, Id, Hash, Enum(&str), List(Vec<Cell>), Struct(Vec<Cell>) }   // the registry's own row model
pub struct Registry;  // BTreeMap-backed: package_id(), fingerprint(), relations() sorted, relation(name), relation_by_id, enums(), enum_spec, logical_types(), invariants(), invariants_for, passes(), pass, rules(), rule, rule_dependencies(), schema_rows() -> Vec<(RelationKey, Vec<Vec<Cell>>)>
pub struct ManifestSpec;  // the authority for the Python msgspec Manifest and docs; Rust struct hand-written in pse-catalog (phase 0)
pub mod arrow { pub fn relation_schema(reg, spec) -> Result<Schema, SchemaError>; pub fn field_for(reg, col) -> Result<Field, SchemaError>; }   // §4.3 metadata attached once at construction
pub mod membership { pub fn members(reg, class: SnapshotClass) -> Vec<&RelationSpec>; pub fn port_name(spec) -> String; }
pub mod fingerprint { pub const FRAME_VERSION = "pse.schema.fingerprint.v1"; pub fn registry(rows) -> ContentHash; pub fn relation(reg, spec) -> ContentHash; }  // derive_hash(context::REGISTRY, ..)
pub mod codegen { pub enum Language { Rust, Python, Markdown } pub struct GeneratedTree { files: BTreeMap<PathBuf, Vec<u8>>, roots: Vec<PathBuf> } pub fn generate(reg, language) -> Result<GeneratedTree, SchemaError>; }
// SchemaError codes: schema::duplicate_declaration, schema::unknown_reference, schema::missing_snapshot_class, schema::missing_granularity, schema::stage_graph, schema::rule_float_key, schema::rule_stratification, schema::codegen
// Registry identity: REGISTRY_PACKAGE_ID = named_id(SemanticId::NIL, "pse.schema"); enums "enum:<Name>"; logical types "logical_type:<rendered>"; invariants "invariant:<relation>:<name>"; passes "pass:<P>@<v>"; rules "rule:<name>@<v>"; rule nodes "rule_node:<rule>@<v>:<preorder path>"
```

### I.3 `pse-relations` [A-7 hand-written; A-6 generated]

```rust
// generated/<namespace>/<relation>.rs — the symbols no_shadow_structs greps
pub const RELATION_ID: SemanticId; pub const NAME: &str; pub const NAMESPACE: Namespace; pub const VERSION: u32; pub const FINGERPRINT: [u8; 32];
pub fn schema() -> SchemaRef; pub fn serde_arrow_fields() -> Vec<FieldRef>;   // the latter only after the serde_arrow_roundtrip spike passes
pub struct <Name>Row { .. }  // serde derive; enums typed; nested <Name><Field>Item structs
pub struct <Name>View<'a> { .. } impl <Name>View<'a> { pub fn try_from_batch(&'a RecordBatch) -> Result<Self, RelationError>; }   // fingerprint metadata + try_extension_type per ext column + validate_batch
pub struct <Name>Builder; // new(), with_capacity(n), push(Row) -> Result<(), RelationError>, finish() -> Result<RecordBatch, RelationError>
pub fn validate(batch: &RecordBatch) -> Result<(), Vec<RelationError>>;
// generated/mod.rs: REGISTRY_FINGERPRINT, relation_by_id(id) -> Option<&'static RelationDescriptor>; generated/enums.rs: as_str/from_str/ordinal/idaes_name/ALL; generated/migrations.rs; generated/conformance_tests.rs
// ext/ — hand-written
pub trait PseExtension: ExtensionType { const SPEC: &'static ExtensionTypeSpec; }
pub struct PseSemanticId; PseContentHash; PseDimensionVector; PseQuantityValue; PseBound; PseIndexTuple; PseOrdinalRef; PseSourceSpan; PseEnum; PseExprDsl; PseTargetPath;
pub struct ExtMetadata { v: u32, enum_id: Option<SemanticId>, target_relation_id: Option<SemanticId> }  // canonical_json(); parse(&str) strict
pub struct PseFormatterFactory;  // impl arrow display ArrayFormatterFactory
// validate/ — hand-written, driven by the runtime Registry
pub fn validate_field(field: &Field) -> Result<(), Vec<RelationError>>;                       // recursive; used by the analyzer rule
pub fn validate_schema(spec: &RelationSpec, schema: &Schema) -> Result<(), Vec<RelationError>>;
pub fn validate_batch(spec: &RelationSpec, batch: &RecordBatch) -> Result<(), Vec<RelationError>>;   // + fingerprint, dictionary bounds and enum membership, bound kinds, finite values, ordinal syntax, required nulls in structs
pub fn batch_from_cells(reg, spec, rows: &[Vec<Cell>]) -> Result<RecordBatch, RelationError>; pub fn cells_from_batch(..);
pub fn registry_relations::materialize(reg) -> Result<BTreeMap<RelationKey, RecordBatch>, RelationError>;   // reference.schema_*, pass_*, rule_* batches
pub trait Migration { relation_id, from_version, to_version, apply(&RecordBatch) -> Result<RecordBatch, RelationError> }; pub fn migrate_to_current(..);
// RelationError codes: schema::fingerprint_mismatch, schema::unknown_registry, schema::storage, schema::extension_type, schema::extension_metadata, schema::unknown_metadata, schema::nullability, schema::enum_member, schema::ordinal_range, schema::arrow
```

### I.4 `pse-catalog` [K-5, B-*]

```rust
pub enum CatalogError { ResourceLimit { consumer, config_keys }, Cancelled, Infrastructure { op, source }, CorruptObject { path, expected, actual }, RefConflict { name }, ManifestInvalid, UnknownRegistry, UnknownVersion, Admission { path, reason }, ForeignSource { table }, Sealed, LogicalHashMismatch, Membership, ConfigInvalid { key, reason }, UserModel, EvaluationError, CompileProperty, Canon(#[from]), Snapshot(#[from]), Reserve(#[from]), Internal }
pub enum PlanOrigin { RuleCompiler, Analytics, KernelUdf }
pub fn classify(err: DataFusionError, origin: PlanOrigin) -> Vec<CatalogError>;   // the ONE §23.2 mapping site; unpacks Collection/Diagnostic/Context/Shared
pub struct RelationContract { canonical: CanonicalContract, namespace, name, key_columns: Vec<usize>, enum_columns, unique_sets, encodings: EncodingPolicy /* IpcFile | IpcFileAndParquet */ }
impl RelationContract { pub fn try_new(..); pub fn from_spec(spec: &RelationSpec, fingerprint: ContentHash) -> Result<Self, CatalogError>; pub fn constraints(&self) -> Constraints; pub fn df_schema(&self) -> Result<DFSchema, _>; }
pub enum TrustLevel { Owned, Untrusted }
pub struct LoadedRelation { contract: Arc<RelationContract>, batch: RecordBatch /* one PK-sorted batch */, member: RelationMember }
pub struct Snapshot { manifest: Arc<Manifest>, relations: BTreeMap<(String, String), Arc<LoadedRelation>> }  // snapshot_id(), relation(ns, name)
pub struct RefName(String);  // ^[a-z0-9][a-z0-9._-]{0,63}$
pub trait Clock { fn now_rfc3339_utc(&self) -> String; } pub struct SystemClock; pub struct FixedClock(pub String);
pub const MANIFEST_VERSION = "pse.manifest.v2";
pub struct Manifest { manifest_version, snapshot_kind, snapshot_id, membership_profile, created_at, schema_registry_fingerprint, relations: Vec<RelationMember>, packages: Vec<PackageRef>, compiler: CompilerRef, engine_profile: Option<ArtifactRef>, numerical_policy: Option<ArtifactRef>, toolchain: ToolchainRef, kernels: Vec<KernelRef>, semantic_parents: Vec<SnapshotParent>, evidence: Vec<EvidenceRecord> }
pub struct RelationMember { port, namespace, relation_id, name, version, logical_hash, rows, encodings: Vec<EncodingRecord { format: ArrowIpcFile|Parquet, writer_version, encoding_checksum, bytes, path }> }
impl Manifest { pub fn validate(&self, registry_fingerprint) -> Result<(), CatalogError>; pub fn frame(&self) -> SnapshotFrame; pub fn encode(&self) -> Result<Vec<u8>, _>; pub fn decode(&[u8]) -> Result<Self, _>; }
// store/ [B-store]
pub struct MemberDraft { port: Option<String>, contract: Arc<RelationContract>, batches: Vec<RecordBatch> }
pub struct BundleDraft { kind: SnapshotKind, members, parents: Vec<SnapshotParent>, packages, compiler, engine_profile, numerical_policy, toolchain, kernels, evidence, set_ref: Option<(RefName, RefUpdatePolicy)> }
pub struct Published { snapshot_id, manifest_checksum, manifest: Arc<Manifest>, ref_head: Option<RefHead> }
pub enum RefUpdatePolicy { Fail, RetryWhileUnchanged { expected_parent: Option<SnapshotId>, attempts: u8 } }
pub struct Catalog;
impl Catalog {
  pub fn open(store: Arc<dyn ObjectStore>, registry: Arc<Registry>, trust: TrustLevel, clock: Arc<dyn Clock>) -> Self;
  pub async fn publish_bundle(&self, draft: BundleDraft, reserver: &dyn MemoryReserver, cancel: CancellationToken) -> Result<Published, CatalogError>;
  pub async fn set_ref(&self, name: &RefName, target: (SnapshotId, EncodingChecksum), policy: RefUpdatePolicy) -> Result<RefHead, CatalogError>;
  pub async fn head(&self, name: &RefName) -> Result<RefHead, _>; pub async fn list_refs(&self) -> Result<Vec<RefHead>, _>;
  pub async fn snapshot(&self, id: SnapshotId, manifest_checksum: EncodingChecksum) -> Result<Arc<Snapshot>, _>; pub async fn resolve(&self, name: &RefName) -> Result<Arc<Snapshot>, _>;
  pub async fn stage_lookup(&self, key: &ContentHash) -> Result<Option<(SnapshotId, EncodingChecksum)>, _>; pub async fn stage_record(&self, key: &ContentHash, target: (SnapshotId, EncodingChecksum)) -> Result<(), _>;   // sidecar stages/<key>.json
}
pub fn complete_members(kind, registry, provided: &[(String, &RelationContract, LogicalHash)]) -> Result<Vec<SnapshotMember>, CatalogError>;
// provider/ [B-providers]
pub struct SnapshotCatalogList; pub struct SnapshotCatalog; pub struct NamespaceSchema; pub struct RelationTable;   // sealed; RelationTable::new(Arc<LoadedRelation>, SnapshotId), identity() -> ProviderIdentity { snapshot_id, relation_id, schema_version, logical_hash }
pub enum FilterVerdict { Exact, Unsupported }  pub fn classify(expr: &Expr, contract: &RelationContract) -> FilterVerdict;   // pure
pub fn apply_filters(state: &dyn Session, contract, batch, filters: &[Expr]) -> Result<RecordBatch, CatalogError>;
pub fn statistics_for(relation: &LoadedRelation) -> Statistics;   // num_rows Exact; total_byte_size Inexact; rest Absent
// session/ [B-session]
extensions_options! { pub struct PseOptions { null_policy: String = "reject", kernel_outcome_policy: String = "typed_error" } }   // prefix datafusion.pse
pub struct ThreadBudget { pool_threads: NonZeroUsize, target_partitions: NonZeroUsize }  // validate(): target_partitions ≤ pool_threads
pub struct ExecutionSettings { batch_size, spill_compression: "uncompressed", max_spill_file_size_bytes, sort_spill_reservation_bytes, time_zone }
pub fn settings_hash(allow_list_version: &str, pairs: &[(String, String)]) -> ContentHash;   // derive_hash(SETTINGS, name-sorted)
pub struct EngineProfile { engine_profile_id, datafusion_version, arrow_version, analyzer_rules, optimizer_rules, physical_rules, semantic_settings, setting_allow_list_version }
pub fn phase0_reference_profile() -> EngineProfile;
pub struct RuleCatalog;  // analyzer(name) / optimizer(name) / physical(name) -> Result<Arc<dyn Rule>, CatalogError>; unknown → ConfigInvalid
pub fn build_extension_registry() -> Result<ExtensionTypeRegistryRef, CatalogError>;   // canonical types + eleven pse.* factories; replacement asserted None
pub enum AdmissionPosition { First, Final } pub struct AdmissionRule;   // AnalyzerRule "pse.admission.first" | "pse.admission.final"
pub fn admit_plan(plan: &LogicalPlan, registry: &dyn ExtensionTypeRegistry) -> Result<(), CatalogError>;   // apply_with_subqueries + expressions; validate_field recursively; TableScan must be a RelationTable
pub fn admit_batch(batch: &RecordBatch, registry) -> Result<(), CatalogError>;
pub struct SessionSpec<'a> { profile: &'a EngineProfile, profile_hash: Option<LogicalHash>, catalogs: SnapshotCatalogList, runtime: Arc<RuntimeEnv>, execution: ExecutionSettings, threads: ThreadBudget, pse: PseOptions, reserver: Arc<dyn MemoryReserver> }
pub fn build_session(spec: SessionSpec<'_>) -> Result<SnapshotSession, CatalogError>;
pub struct SnapshotSession;  // sql(query, origin) / execute_plan(plan, origin) -> QueryReport { batches, rules_fired: Vec<RuleFired>, reservation }; table_reference(snapshot_id, ns, name) -> TableReference; read_back_settings(); settings_hash(); profile_hash(); function_registry_hash(); state(); registry()
```

### I.5 `pse-runtime` [K-5, E-*]

```rust
pub struct ResourceBudget { memory_limit_bytes: NonZeroUsize, spill_dir: PathBuf, max_temp_dir_bytes: u64, top_consumers: NonZeroUsize, threads: ThreadBudget, execution: ExecutionSettings, hashing_may_use_pool: bool /* false */ }  // validate()
pub enum RuntimeError { ConfigInvalid, Infrastructure, Cancelled, Catalog(#[from]), Reserve(#[from]), Internal }
pub struct SharedRuntime { env: Arc<RuntimeEnv> }  // build(&ResourceBudget); reserver() -> PoolReserver; report() -> ResourceReport; pool()
pub struct PoolReserver(Arc<dyn MemoryPool>);  // impl MemoryReserver: MemoryConsumer::new(owner).with_can_spill(false).register(pool)
pub struct ResourceReport { limit_bytes, pool_peak_bytes, pool_reserved_now, top_consumers: Vec<(String, usize)>, process_peak_rss_bytes: Option<u64> }
pub struct CancelSource;  // new(), token() -> CancellationToken, cancel(), cancelled().await
pub struct SessionFactory;  // new(Arc<SharedRuntime>, ResourceBudget); open_session(&Catalog, &[RefName], &EngineProfile, Option<LogicalHash>) -> Result<SnapshotSession, RuntimeError>
```

### I.6 `pse-quantity` [K-4, Q-*]

```rust
pub struct Ratio { num: i16, den: i16 }   // reduced; den > 0; checked_add/sub/mul; ZERO, ONE
pub struct DimensionVector([Ratio; 8]);   // length, mass, time, temperature, amount, current, luminous_intensity, currency; mul/div/pow/root checked; canonical_bytes() -> [u8; 32]; DIMENSIONLESS
pub enum Opcode { Const, SymbolRef, Add, Sub, Affine, WeightedMean, Mul, Div, Pow, Neg, Abs, Exp, Log, Log10, Sqrt, Sin, Cos, Tan, Asin, Acos, Atan, Sinh, Cosh, Tanh, Erf, SmoothMax, SmoothMin, SmoothAbs, SafeSqrt, SafeLog, Conditional, SumOver, ProdOver, MinOver, MaxOver, Gather, Broadcast, Derivative, Integral, KernelCall, ImplicitRef, UnitConvert, PiecewiseLinear }  // ALL: [Opcode; 43]; as_str(); parse()
pub struct UnitId(SemanticId); UnitSetId; QuantityKindId; BasisId; ReferenceStateId; QuantityTypeId; ConversionId; OperationId; DomainId; BoundIndexId; InvariantId;
pub struct BoundIndexRef { bound_index, domain, kind: DomainKind }  pub struct IndexSet(BTreeSet<BoundIndexRef>);
pub struct Unit { id, symbol, dimension, scale_to_canonical, offset_to_canonical, is_affine }
pub struct UnitConvertSpec { from, to, scale, offset }
pub fn convert_spec(from: &Unit, to: &Unit, scale_kind: ScaleKind) -> Result<UnitConvertSpec, QuantityError>;   // the single source for every UnitConvert edge
pub fn convert_value(spec: &UnitConvertSpec, v: f64) -> f64;   // (v*scale)+offset, two roundings, no FMA
pub struct UnitSet { id, base: [Option<UnitId>; 8] }  // new(..) rejects affine/wrong-dimension base units; derived_unit(dim) -> DerivedUnit
pub struct ConversionRule { id, from, to, kind: ConversionKind, kernel, required_parameters, scale, offset }
pub struct QuantityKind { id, dimension, extensive, addition_kind: Additive | OriginSensitive }
pub struct QuantityTypeKey { kind, basis, reference_state, scale_kind: Point | Difference, shape: Vec<DomainKind>, subject_kind }
pub struct QuantityType { id, key, canonical_unit, nominal_magnitude }  // is_neutral_scalar(reg)
pub struct QuantityRegistryBuilder;  // unit/kind/basis/reference_state/quantity_type/conversion/operation/unit_set; build() runs load-time invariants
pub struct QuantityRegistry;  // unit(), unit_by_symbol(), quantity_type(), resolve_key(&QuantityTypeKey) -> Result<QuantityTypeId>, operations_for(Opcode), conversions_between(), neutral_dimensionless()
pub struct QuantityOperation { id, opcode, input_kinds, result_kind, basis_rule, reference_rule, scale_rule, shape_rule, *_source, subject_rule, result_*, input_conversions, precondition_invariants }
pub struct Operand<'a> { quantity_type: QuantityTypeId, indices: &'a IndexSet }
pub enum OpRequest<'a> { Literal { unit, context: LiteralContext }, Add, Sub, Neg, Abs, Affine { term_signs, has_constant }, WeightedMean { normalization, certified_invariant }, Mul, Div, Pow { exponent: Rational(Ratio) | Symbolic }, Sqrt, Transcendental(Opcode), Smooth { opcode, eps }, Conditional, Reduce { kind, bound }, Gather { group_shape, coordinates }, Broadcast { index }, Derivative { domain_unit, domain_kind, order }, Integral { .. }, UnitConvert { spec }, KernelCall { declared_inputs, declared_output }, ImplicitRef { unknown }, PiecewiseLinear { .. } }
pub enum LiteralContext { Free, Additive { sibling: QuantityTypeId }, EquationSide { side: QuantityTypeId } }
pub enum OperationSelection { BuiltIn(BuiltInRule), Registered { operation: OperationId, operand_permutation: Vec<u16> } }
pub struct Inferred { result: QuantityTypeId, indices: IndexSet, selected: OperationSelection, conversions: Vec<OperandConversion { operand, conversion }> }
pub fn infer(request: &OpRequest<'_>, operands: &[Operand<'_>], reg: &QuantityRegistry) -> Result<Inferred, QuantityError>;
pub fn require_same_contract(expected: QuantityTypeId, actual: QuantityTypeId, reg) -> Result<(), QuantityError>;   // quantity half of head_conversion_exactness
pub mod numeric { exact_i64_from_f64, exact_f64_from_i64, approx_eq(a, b, atol, rtol) /* §7.3 */, is_zero }
#[cfg(feature = "fixtures")] pub mod standard { pub fn standard_registry() -> QuantityRegistry; pub mod ids; }
// QuantityError codes: validation::invariant (registry, dimension, unknown id), compile::math::quantity_operation_unsupported, compile::math::unit_inconsistent, compile::math::domain_violation_static
```

### I.7 `pse-mathir` [K-4, M-*]

```rust
pub use pse_quantity::Opcode;
pub struct NodeId(pub u64);
pub struct Node { opcode, payload: Payload, children: Vec<NodeId>, quantity_type: Option<QuantityTypeId> /* None until P10 */, scope: Option<SemanticId> }
pub enum Payload { None, SymbolRef { symbol }, FloatConst { value /* finite */, unit }, IntConst { value }, Affine { constant, terms: Vec<AffineTerm { coefficient, child }> }, WeightedMean { pairs: Vec<(weight, value)>, normalization, unit_sum_invariant }, Reduction { kind, domain, bound_index, filter }, Gather { group, coordinate_map }, Broadcast { domain, bound_index }, Derivative { wrt_domain, order }, Integral { domain, quadrature_policy }, SmoothOp { eps }, Conditional { guard }, KernelCall { kernel_binding, output_ordinal }, ImplicitRef { implicit_system, unknown_ordinal }, UnitConvert(UnitConvertSpec), PiecewiseLinear { breakpoints, input, output } }
pub struct ExprGraph;  // new(); insert(opcode, payload, children, scope) -> Result<NodeId, MathIrError> (arity per OPERATOR_TABLE, literal finiteness, consing on structural hash); node(id); iter(); set_quantity_type() (P10 only); convenience constructors
pub struct OperatorSpec { opcode, family: OperatorFamily, arity, shape_rule, derivative_rule, argument_evaluation, failure_classes, domain_restrictions, smoothness, convexity_rule, monotonicity_rule, sparsity_rule, rewrite_conditions, lowering, foldable }
pub static OPERATOR_TABLE: [OperatorSpec; 43]; pub fn operator_spec(op) -> &'static OperatorSpec;
pub trait OperatorSpecSink { fn operator_spec(&mut self, opcode, arity, quantity_operation_ids: &[OperationId], ..); }  pub fn emit_operator_specs(reg: &QuantityRegistry, sink: &mut dyn OperatorSpecSink);
pub fn structural_hash(node, child_hashes) -> ContentHash; pub fn subtree_hash(node, quantity_type, child_hashes) -> ContentHash;   // derive_hash(MATHIR_NODE, ..); scope excluded
pub struct CanonicalGraph;  // node(id) -> &CanonicalNode { opcode, payload, children, quantity_type: QuantityTypeId, scope, subtree_hash, free_indices }; roots(); equations(); selections(); listing() -> String
pub fn fold_literals(graph: &mut ExprGraph, reg) -> Result<FoldReport { folded, deferred_static_checks }, MathIrError>;
pub trait SymbolTypeSource { symbol_type(symbol) -> Option<QuantityTypeId>; group_shape(group); domain(domain) -> Option<DomainFacts>; kernel_contract(binding); implicit_unknown(system, ordinal) }
pub struct FreeIndex { bound_index, domain, position }
pub struct EquationRecord { indexed_equation_id, owner_instance, equation_decl, qualified_name, product, filter, body, sense, lower, upper, free_indices, residual_quantity_type, law_instance, derivation }
pub struct QuantitySelection { node, selected, conversions, deferred_static_check }
pub enum Policy { Strict }
pub struct CanonicalizeInput<'a> { graph, equations, roots, symbols: &'a dyn SymbolTypeSource, registry: &'a QuantityRegistry }
pub fn canonicalize(input: CanonicalizeInput<'_>, policy: Policy) -> Result<CanonicalGraph, MathIrError>;
pub trait MathRelationSink { expr_node, expr_arg, symbol_ref, float_constant, int_constant, affine, weighted_mean, reduction, gather, broadcast, derivative, integral, smooth_op, conditional, kernel_call, implicit_ref, unit_convert, indexed_equation, free_index, quantity_selection }   // primitive-typed methods; no row structs
pub trait MathRelationSource { /* mirror iterators */ }
pub fn emit(graph: &CanonicalGraph, sink); pub fn emit_untyped(graph: &ExprGraph, roots, sink); pub fn load_untyped(src) -> Result<(ExprGraph, Vec<EquationRecord>), _>; pub fn load_canonical(src) -> Result<CanonicalGraph, _>;
pub struct VecSink;  // test double implementing both traits
// MathIrError codes: compile::math::cyclic_expression, compile::math::unit_inconsistent (Quantity is #[diagnostic(transparent)]), compile::math::domain_violation_static, compile::math::quantity_operation_unsupported (UnboundIndex, UnknownBinding), internal::invariant (Malformed)
```

### I.8 `pse-authoring` [K-3, C-dsl, C-1, C-2]

```rust
pub struct SourceSpan { document_id: SemanticId, start: u32, end: u32 }
pub struct ParseBudget { max_depth, max_aliases, max_bytes }
pub struct Document { document_id /* named_id(package_id, path) */, package_id, path, kind, bytes, content_hash }
pub struct DocumentBundle { header: PackageHeaderDoc /* generated */, header_span: SpanIndex, documents: Vec<(Document, SpanIndex)> }
pub fn load_package(dir: &Path, budget: &ParseBudget) -> Result<DocumentBundle, AuthoringError>;
pub struct SpanIndex;  // span(&DocPath) -> Option<SourceSpan>; DocPath = "/templates/0/symbols/3"
pub mod ids { pub fn entity_id(policy: IdPolicy, package_id, explicit: Option<&str>, qualified_name, at: SourceSpan) -> Result<SemanticId, AuthoringError>; pub fn assign_ids(bundle, next: &mut dyn FnMut() -> SemanticId) -> Result<Vec<DocumentEdit>, AuthoringError>; pub fn uuid_v7() -> SemanticId; }
pub mod dsl {
  pub struct Span { start: u32, end: u32 }
  pub struct Expr { kind: ExprKind, span }  pub enum ExprKind { Number(Number), Path(Path), Neg, Binary { op: Add|Sub|Mul|Div|Pow, lhs, rhs }, Call { function: Function, args, named: Vec<NamedArg /* eps */> }, Kernel { name, args }, Reduce { kind: Sum|Prod|Integral, binder: Binder { var, domain: Path, filter: Option<Predicate> }, body }, Derivative { body, wrt }, Conditional { guard: Predicate, then, otherwise }, Let { bindings, body } }
  pub struct Predicate { kind: Compare | In | And | Or | Not | Atom | Null, span }  pub struct Equation { lhs, sense: Eq|Le|Ge, rhs, span }
  pub fn parse_expr(&str) -> Result<Expr, DslError>; parse_equation; parse_predicate; render_expr(&Expr) -> String; render_equation; render_predicate;
  impl Expr { walk, paths() -> Vec<&Path>, structural_eq (spans ignored, f64 by bits), strip_spans }
  pub enum DslError { Syntax { offset, span, expected, found }, WeightedMeanArity, AmbiguousUnaryPower, NonFiniteNumber, Budget }   // authoring::parse
}
pub mod change_set { pub struct ChangeSet { change_set_id, base_revision_id, author, message, created_at_ns, ops: Vec<ChangeOp>, staged: BTreeMap<SemanticId, RecordBatch> }  pub enum ChangeOp { Insert { relation_id, row_key, staged_ordinal }, Update, Delete, Rename { entity_id, name, qualified_name } }
  pub trait AuthoredReader { fn batch(&self, relation_id) -> Result<Option<RecordBatch>, AuthoringError>; }   // implemented by pse-compiler over a Snapshot
  pub struct CandidateSnapshot { relations: BTreeMap<SemanticId, RecordBatch>, change_sets, change_ops }
  pub fn apply(base: &dyn AuthoredReader, cs: &ChangeSet, reg) -> Result<CandidateSnapshot, AuthoringError>; }
pub mod p0 { pub fn resolve(bundles: &[DocumentBundle], reg) -> Result<PackageGraph, AuthoringError>; }
pub mod p1 { pub fn stage(bundle, base: &dyn AuthoredReader, reg) -> Result<ChangeSet, AuthoringError>; }
pub mod targets { pub fn parse(text, at) -> Result<TargetPath, _>; pub fn resolve(&TargetPath, &TargetContext) -> Result<Vec<TargetRow>, _>; }
// AuthoringError codes: authoring::parse::{syntax, unknown_key, missing_id, unresolved_target, budget}, authoring::reference::{derived_write, rename_named, unknown_row_key}, authoring::pkg::{unresolved, version_conflict}, schema::version_mismatch
```

### I.9 `pse-rules` and `pse-compiler` [K-3, C-3, C-4]

```rust
// pse-rules
pub struct PortBinding { ports: BTreeMap<&'static str, (SnapshotId, SemanticId)> }
pub struct CompiledRule { rule_id, plan: LogicalPlan, head_schema: SchemaRef, key_columns: Vec<String> }
pub fn compile(rule: &RuleSpec, binding: &PortBinding, session: &SnapshotSession, reg) -> Result<CompiledRule, RuleError>;
pub struct RuleOutcome { head: RecordBatch /* PK-sorted */, undecided: RecordBatch, explain_pgjson: String, rules_fired: Vec<RuleFired> }
pub async fn execute(rule: &CompiledRule, session, reg) -> Result<RuleOutcome, RuleError>;
pub struct InvariantReport { findings, undecided, derivations, explain, rules_fired }
pub async fn run_invariants(snapshot: &Snapshot, session, reg, scope: Model | Case | Registry) -> Result<InvariantReport, RuleError>;   // the P2 body
pub mod errmap { pub fn classify(err: DataFusionError, origin) -> RuleError; }   // delegates to pse_catalog::classify
// RuleError codes: rule::float_key, rule::head_schema_mismatch, internal::invariant, runtime::resource_limit, runtime::infrastructure, config::invalid
// pse-compiler
pub trait Pass: Send + Sync { fn spec(&self) -> &'static PassSpec; fn run(&self, ctx: &PassContext<'_>, inputs: &InputBundle) -> Result<PassOutput, CompilerError>; }
pub struct PassContext<'a> { registry, session: Option<&'a SnapshotSession>, policies: &'a PolicySet, runtime: &'a tokio::runtime::Handle, external: &'a ExternalInputs }
pub struct InputBundle { ports: BTreeMap<&'static str, Option<BoundInput { snapshot, relation_id, schema_version, logical_hash, handle }>> }   // None = explicit absence (enters the key)
pub struct PassOutput { ports: BTreeMap<&'static str, Vec<RecordBatch>> /* every declared port, empties explicit */, findings, record: PassRecordDraft }
pub struct StageKey(pub ContentHash);
pub fn stage_key(spec, inputs, reg, policies, engine: Option<(ContentHash, ContentHash)>) -> StageKey;   // derive_hash(STAGE_KEY, pass_id, version, registry_fingerprint, ∀port sorted: port, relation_id, version, 0x01‖hash | 0x00, ∀policy sorted, [profile_hash, function_registry_hash])
pub struct StageDag;  // build(reg); validate() -> Result<(), Vec<CompilerError>>; topo_order(); producer(relation, port)
pub struct Driver;    // new(Arc<Catalog>, Arc<SharedRuntime>); commit(&DocumentBundle, base_ref) -> CommitReport; run(PipelineRequest { through: "P3"|"P10", snapshot, policies, engine_profile, external_bindings /* tests only */ }) -> PipelineReport
// CompilerError codes: compile::stage_graph, internal::invariant, runtime::resource_limit, runtime::infrastructure + re-exported authoring/rule errors
```

## Verification

Phase-0 exit per blueprint §25, each with the command that proves it. Baseline zero for
every command and mode; `Tested` claims name the test and its conditions.

| Exit criterion (§25) | Proof | Command |
|---|---|---|
| schema governance tests green | `no_shadow_structs` non-vacuous, `codegen_regeneration`, `no_mutating_providers`, `registry_governance`, error taxonomy over real enums, pins incl. the new §3.1 rows | `just governance` |
| expression DSL round-trips | `dsl_roundtrip` proptest (1024 cases, committed seed) and `dsl_examples` snapshots | `just test-package pse-authoring -p pse-relations` |
| `pse.canon.v2` metamorphic fixtures + active semantic query admission | `canonical_null_equivalence`, `encoding_roundtrip_identity`, `snapshot_membership`, `semantic_admission_paths` (E1 as a regression) | `just test-package pse-tests-conformance -p pse-relations`; `just test-package pse-tests-engine -p pse-relations` |
| storage identity/integrity fault matrix | `publication_fault_matrix`; the three budget fixtures | `just test-package pse-tests-lifecycle -p pse-relations` |
| quantity/ordered-IR contract fixtures | `quantity_composition`, `head_conversion_exactness` (quantity half in Q-4, head half in C-3), `numerical_policy_conformance` (P10 subset) | `just test-package pse-tests-conformance -p pse-relations` |
| a complete snapshot published, read back and queried | golden stores `registry` and `minimal_explicit`: publish → open → `SELECT` through the catalog; `pushdown_vs_unpruned`; `incremental_equals_clean` (P0–P3) | `cargo xtask golden registry && cargo xtask golden minimal_explicit`; `just test-package pse-tests-engine -p pse-relations` |
| generated trees current | the four generated paths regenerate byte-identically | `just codegen-check` |
| Python boundary | generated contracts pass the `Any` lint, extension round trip with the revision-6 metadata shapes, `GENERATED.sha256`, golden registry readable through generated contracts | `just py-sync && just py-test && just quality` |
| benchmarks build and run | `canonicalization` criterion group replaces the placeholder | `just bench-smoke` |

`just ci-pr` green locally at every tier boundary and at exit; every required check green
on each packet PR; `just features-powerset` and `just test-release` run once at exit
(scheduled jobs, run locally for the record).

## Open items

Decided by recommendation in K-2 unless the maintainer overrides at plan approval:

- **Extension metadata shapes.** `pse.enum` carries `{"v":1,"enum_id":…}` and
  `pse.ordinal_ref` carries `{"v":1,"target_relation_id":…}` in the extension's own
  metadata (revision 6); the Python placeholder types change accordingly.
- **`authored.change_ops.row`** becomes `struct<staged_port, staged_ordinal>` pointing into
  per-relation staged members of the change-set bundle (revision 6); JSON text rejected (D1).
- **Appendix B relations without column lists** (`instance_equations`, `sweep_results`,
  `profile_results`, `run_state`; partial `template_scaling_defaults`,
  `diagnostic_thresholds`, `cost_indices`, `location_factors`) are listed in
  `tests/governance/appendix_b_deferred.toml` with reasons, not invented — except
  `normalized.package_graph`, which P0 needs and revision 6 specifies.
- **Boolean guards.** Phase 0 accepts a `SymbolRef` of a registered boolean kind or an
  `IntConst` 0/1 as a `Conditional` guard; comparison/logical opcodes are a later §7.2
  amendment.
- **Literal ambiguity** (`T − 300{K}`): a unit literal beside an origin-sensitive sibling is
  that kind's difference; beside an equation side it takes the side's type; a bare literal
  resolves only when exactly one registered type matches. Documented as a rule.
- **Derived units per unit set** (slice A): register the derived units a unit set uses as
  `reference.units` rows in the units package; no minted `inferred.derived_units` in phase 0.
- **`Affine`** is constructible and typed but not emitted by P3 or synthesised by P10 in
  phase 0 (§7.4 step 2); it is P8's substrate later.
- **Scope on shared nodes** is excluded from the node hash; conflicting scopes collapse to
  `None`. Revisit before P7 if per-scope ownership is needed.
- **Rust `Manifest` struct** is hand-written in phase 0 with a parity test against the
  registry's `ManifestSpec`; generating it is a wave-2 item under ADR-0051.
- **mmap for IPC artifacts** is not adopted (read via `get_opts` → `Bytes` → zero-copy
  `Buffer`, no `unsafe`); register row under R-24 with a measured trigger.
- **`datafusion-session`** is not declared; the umbrella's `datafusion::catalog::*`
  re-exports suffice.
- **B-evidence** and **R-3 (bindgen arm)** are optional in this wave; if they slip they get
  register rows, and `bindings.rs` keeps the hygiene-only check.
- **Wave 2 preview** (not planned here): P4–P9 rule execution with strata and conflict
  detection, `pse-templates`, the reference units/elements YAML package as the authority for
  Q-4's fixture (with the `standard_fixture_matches_yaml` governance test), `pse.open` over
  the catalog through `pyo3-arrow`, Rust manifest generation, `rename` re-rendering of
  `expr_dsl` from normalized graphs.

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
