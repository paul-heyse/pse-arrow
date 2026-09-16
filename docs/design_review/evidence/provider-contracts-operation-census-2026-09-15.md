---
title: Provider pivot callable operation census
date: 2026-09-15
evidence: Interface-checked
plan: ../../plans/06-provider-contracts-hard-pivot.md
---

# Callable operation census

**Interface-checked — PC00 inventory, 2026-09-15.** The table covers product
entry points and their implementation dependencies. The accompanying
[public Rust declaration inventory](provider-contracts-operation-census-2026-09-15.json)
records the source candidates used to check crate coverage. It deliberately includes
primitive accessors; a declaration's presence is not evidence that an operation is
implemented, reachable from Python, or functionally qualified.

Every product data operation belongs to the provider/native-operation framework.
Pure helpers and callbacks remain implementation details of those operations;
their unit tests do not create an alternative product execution route. No
fundamental architectural exception has been established by this census.

| Operation family / current entry points | Actual implementation locations | Target ownership / remaining work |
|---|---|---|
| Session assembly, SQL, direct logical plans, registered functions, schema edits | `pse-catalog/src/session/{factory,snapshot_session,native,preparation,functions,schema_transform,plan_codec}.rs` | PC01–PC05: binding index, composed policy, native planning/streaming; general codec/remote/function-effect conformance remains |
| Catalog/schema/table lookup, metadata, projection/filter/limit/statistics | `pse-catalog/src/provider/` | PC02–PC05: native hierarchy projections, actual source owners and generation coverage; no concrete provider roster |
| Native DDL, SET, table DML and Copy | `pse-catalog/src/session/{native,policy,preparation}.rs` and pinned DataFusion dispatch | PC06: private DML uses an actual factory, deferred native hooks and admitted replacement generations; CREATE TABLE defaults and retained readers are covered; EXPLAIN integration and broader factory/Copy/terminal contracts remain |
| Multi-output stage execution and publication | `Catalog::{prepare_production,prepare_production_publication}`, `pse-catalog/src/computation{,/ports}.rs` | PC06: actual registered pass and publication both run in native operations; completed output providers cannot replay production; broader command and terminal contracts remain |
| Source completion/publication | `Catalog::{complete_sources,prepare_source_publication}`, `pse-catalog/src/source_production.rs`, compiler `validator/source_producer.rs` | PC06/PC07: model/case publication now runs in one native operation; upstream construction/completion orchestration remains to pivot |
| Store open, ref/snapshot/manifest/revision resolution, cold admission | `pse-catalog/src/store/{open,refs,pinned,traversal,stage_admission}.rs` | PC02/PC09: ref reads/listing, manifest admission and cold traversal use native operations and actual parent providers; broader remote resolution/generation contracts remain |
| Conditional ref updates, changesets, revisions, sidecars and artifacts | `pse-catalog/src/store/{operation,publication,refs,changes,sidecar,publish,documents}.rs` | PC06/PC07: ref, bundle, sidecar, document, ordered artifact, change-set and stage-index operations use native preparation; conditional writes retain complete typed visibility/durability evidence; generated terminal projections and end-to-end source orchestration remain |
| Authored package/document parsing, dependencies, references, P0/P1 | `pse-authoring/src/{document,p0,p1,lower,targets}` | PC07: one owned parse, native source providers, scoped roles, typed findings and source spans |
| Ordered edits, explicit-ID rename, preimages and schema changes | `pse-authoring/src/{change_set,document/edit,document/rename,document/owned_reparse}`, `pse-catalog/src/session/schema_transform.rs` | PC07: before/after bindings and native operation effects; delete duplicate validation/orchestration |
| Model/case source commits and pipeline requests | `Driver::{base_reader,commit,run}`, compiler `driver/{commit,execution,inputs,history}` | PC07/PC09: exact document/parent/policy inputs, native commands, shared physical and cold traversal owners |
| Registry invariants, diagnostics, rule/fixed-point execution | `pse-rules/src/{invariants,validator,plan,strata}` | PC04/PC08: one invariant compiler discharges requirements under the actual operation session; the separate validator factory is deleted; full truth/support/absence qualification remains |
| P3 normalization and P4–P10 construction | compiler `passes/{p3,p4,p5,p6,p7,p8,p9,p10}`, `validator/producer.rs` | PC07–PC09: native operation roots and declared ports, target algorithms and complete scientific/provenance outcomes |
| Physical quantity/material inventory, affine/rational algebra, element/phase/stoichiometric calculations | compiler `quantity_relations/`, `pse-quantity/src/`, `pse-material/src/` | PC08: shared admitted providers feed specialized algorithms inside the bound operation; source and independent physical-value evidence required |
| Indexed MathIR construction, binding, typing, canonicalization, folding and relational emission | `pse-mathir/src/`, compiler P7–P10 | PC08/PC09: retain useful exact graph algorithms under typed native inputs/results, with source/occurrence correspondence |
| Numerical, structural and backend execution | `pse-{numerics,structural,backend-native,backend-nl,backend-pyomo}/src/lib.rs` | These are currently declared boundaries with no callable execution implementation. PC08 must report that unsupported state; this delivery does not invent P11–P14 solving breadth |
| Rust admitted table inspection | `pse-catalog/src/inspection.rs` (`TableReader`), `session/inspection.rs` | PC10: direct `LoadedRelation` slicing deleted; exact member providers use common preparation, policy and owned native streams with DataFusion batch splitting |
| Python store/snapshot/Arrow stream APIs | `pse-py/src/inspection/{runtime,handles,stream,settings}.rs`; Python `pse` inspection facade | PC10: shared provider/session/runtime path, explicit stream ownership, no implicit compilation/admission |
| Engineering qualification | `xtask/src/engineering_inspection.rs`, `tests/support/{engineering_sources,engineering_expectations}.rs`, Python engineering inspection tests | PC11: real heater/mixer × FTPx/FcTP source→P10→persist→cold Rust/Python, independent source/scientific oracles |
| Registry/schema/type/ID/encoding/resource helpers and generated interfaces | `pse-schema`, `pse-relations`, `pse-ids`, `pse-buildinfo`, generated contracts | Canonical declarations and owned implementations used by the operations above; no standalone data authority or alternate execution engine |
| Build, code generation, lint, evidence extraction and parity tooling | `justfile`, `xtask`, `scripts`, test/parity harnesses | Development tooling, outside the product invocation API; retains pinned type-universe, clean-room and generated-source contracts |

## Verification

**Interface-checked:** public Rust declarations were searched across `crates/`
outside generated paths; native trait implementations, Python `#[pymethods]` and the
engineering xtask entry point were inspected separately. Numerical/structural/backend
crate boundaries were opened and have no callable execution implementation.

**Proposed — PC12 closure:** reconcile this table and the final source inventory
against actual consumers, remove each superseded entry route, and attach the
corresponding V01–V16 receipts. This inventory does not close those behavioral gates.
