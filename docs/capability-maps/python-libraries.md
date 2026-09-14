---
status: evidence-map
blueprint_revision: 5
pins: pyproject.toml and uv.lock
regenerated: null
reviewed: 2026-09-13
---

# Python libraries — capability map

**Current binding:** blueprint revision 5. The original survey and version/probe
receipts below retain their historical scope; they were not regenerated in this
update. Earlier revision comparisons and parked recommendations are historical.
The [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md)
and ADR-0039–ADR-0048 determine the corrected design bindings below. Platform
integration remains **Proposed**; interface and probe evidence are labeled at
their original scope. No Python behavioral corpus was re-audited by this revision.

| Current binding | Decision and limit | Evidence / authority |
|---|---|---|
| Arrow admission | Explicit recursive contract validation, registration/loss checks and typed outcomes | ADR-0039/0043; blueprint §21.1 |
| Pyomo affine lowering | LinearExpression only for a proven affine-in-variables body; constants have degree 0 | ADR-0047; review R4-09 |
| Guarded and opaque bindings | Actual writer/solver capability and numerical-policy preflight; Expr_if construction is not lazy-evaluation proof | ADR-0043/0047; blueprint §21.2 |
| Bounded streams | Safe owning buffers and measured bounded coalescing; no whole-dataset transfer copy | ADR-0047/0048; R-24 |

**Fourth and final map in the series**, completing blueprint §3.3. The Rust side is covered by `arrow-rust.md`, `datafusion-rust.md` and `supporting-rust-libraries.md`; this covers **line 331** — the Python row — plus the adjacent libraries the Python boundary actually depends on.

**Compiled** 2026-09-13.

---

## 0. Purpose, sourcing constraint, and evidence rules

### 0.1 What this document is

Per cluster: what each library can actually do, which APIs the blueprint's requirements land on, what we deliberately will not use, and where the blueprint assumes a capability that does not exist or is under-specified. A second pass asks what these libraries offer that the blueprint has *not* claimed, which would improve alignment with `design_principles/DATA_MODEL_DESIGN_CHARTER.md` (DM-01…DM-60, gates G1–G7).

This boundary deserves the scrutiny. Everything crossing it was already typed once, in Rust, against a registry. §4.4 asserts that Python consumers "register matching `pyarrow.ExtensionType` classes"; §21.5 asserts typed attrs classes with no `Any` fields. Those are **DM-42** and **G2** claims about a language with no compiler to enforce them — so they were measured rather than assumed.

### 0.2 Sourcing constraint

Nothing is taken from existing repository documentation. This is both an instruction and a fact: **`docs/library_ref/` has been deleted**, taking the former `pyomo.md`, `pyarrow.md`, `attrs.md`, `cattrs.md` and `msgspec.md` references with it. Four fresh lanes:

| Lane | Marker | What it is |
|---|---|---|
| **locally generated API surface** | `[api:lib@version]` | The rustdoc analogue: a script walking each library's public API via `inspect`/`importlib`, emitting JSON — modules, classes, callables, signatures, `__all__`, MRO. Authoritative for "does this exist, with this signature, at this version". |
| **locally run probes** | `[probe]` | The `arrow_probe.rs` analogue: scripts that **measure** behaviour the blueprint asserts. Eight were run; their output is quoted inline. |
| context7 | `[c7:/id]` | Idiom and intent |
| PyPI / upstream | `[pypi:pkg]`, `[docs:url]` | Versions, release metadata, dependency requirements |

Checked mechanically: **zero `[ref:]` citations**, and every marker resolves to a ledger row. A claim carrying no marker is a defect.

### 0.3 Status vocabulary

**leverage** · **build on top** · **confirmed** · **refined** / **erratum** · **adopt** / **evaluate** / **reject** · **open item**. Pass-2 recommendations are disciplined by charter §G and **DM-58**: a recommendation naming no principle and replacing no identified hand-written work does not appear.

### 0.4 A correction to this document's own reconnaissance

An early probe of the neighbouring environment suggested that an unregistered `pse.*` extension type would surface in Python as `pyarrow.UnknownExtensionType`. **Measurement showed otherwise** (§3, PROBE 1): it deserializes to its plain storage type. `UnknownExtensionType` exists in the API but is not what this path produces. The corrected finding is materially different — and better — because the degradation is silent at the type level but *detectable* through retained field metadata. Recorded here because the wrong version briefly informed the plan.

---

## 1. Version anchors and environment receipts

### 1.1 Anchors — every library at its current PyPI release

Owner's instruction: use the latest release of every library, not the versions §3.1 names. All eight verified against PyPI at compile time `[pypi:*]`.

| Library | This map | Blueprint §3.1 | Gap |
|---|---|---|---|
| Pyomo | **6.10.1** | "6.9+" | open lower bound → pin |
| pyarrow | **25.0.1** | "15+" | **ten majors** behind → pin |
| attrs | **26.1.0** | *unpinned* | no pin at all |
| cattrs | **26.2.0** | *unpinned* | no pin at all |
| msgspec | **0.21.1** | *unpinned* | no pin at all |
| numpy | **2.5.3** | *unpinned* | arrives via Pyomo/pyarrow |
| scipy | **1.18.1** | *unpinned* | arrives via Pyomo |
| idaes-pse | **2.12.0** | "2.10 for parity tests only" | **2.10 → 2.12** |
| pint | **0.26.1** | *excluded by §3.3* | see §5 — required, not optional |

### 1.2 Two interpreters, and why

`idaes-pse` 2.12.0 classifies Python **3.10–3.13 only** `[pypi:idaes-pse]`; its `requires_python` is unset, so pip would not block a 3.14 install, but it is untested there. Rather than guess, the extraction uses two environments:

```text
py314/   CPython 3.14.7   pyomo 6.10.1, pyarrow 25.0.1, attrs 26.1.0, cattrs 26.2.0,
                          msgspec 0.21.1, numpy 2.5.3, scipy 1.18.1 (+ pint 0.26.1, see §5)
py313/   CPython 3.13.12  the same set plus idaes-pse 2.12.0
                          (pulls pint 0.26.1, pydantic 2.13.5, sympy 1.14.0, networkx 3.6.1)
```

Every library `==`-pinned, mirroring the `=`-pin discipline §3.1 adopted for Rust. IDAES imports cleanly on 3.13.12 `[probe]`.

**This split is itself a §3.1 finding.** §3.1 says "Python 3.11+"; IDAES currently caps the interpreter at 3.13 for anything that must run the parity harness (§24). The platform's own Python package can target 3.14; the parity environment cannot.

### 1.3 Extraction receipt

| Dump | Library | Version | Interpreter | Modules | Items |
|---|---|---|---|---|---|
| `pyarrow.json` | pyarrow | 25.0.1 | 3.14.7 | 28 | 1445 |
| `pyomo.json` | pyomo | 6.10.1 | 3.14.7 | 505 | 5898 |
| `attrs.json` | attrs | 26.1.0 | 3.14.7 | 6 | 61 |
| `cattrs.json` | cattrs | 26.2.0 | 3.14.7 | 20 | 416 |
| `msgspec.json` | msgspec | 0.21.1 | 3.14.7 | 7 | 91 |
| `numpy.json` | numpy | 2.5.3 | 3.14.7 | 69 | 2414 |
| `scipy.json` | scipy | 1.18.1 | 3.14.7 | 156 | 1772 |
| `idaes.json` | idaes | 2.12.0 | 3.13.12 | 191 | 2762 |

Scripts, both requirement sets and all probe programs are committed under **`docs/capability-maps/evidence/python/`** so every `[api:…]` and `[probe]` marker is re-derivable.

---

## 2. pyarrow — PyCapsule interop and the stream boundary (P1)

**Role:** §21.1 — "every table crosses as an Arrow C stream (`__arrow_c_stream__`), never as row objects"; D12 — "one adapter consumes a `CanonicalMathProblem` bundle over the Arrow C stream interface".

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| PyCapsule producer/consumer | `Table.__arrow_c_stream__`, `RecordBatchReader.__arrow_c_stream__`, `DataType.__arrow_c_schema__`, `ExtensionType.__arrow_c_schema__` | `[api:pyarrow@25.0.1]` |
| Stream construction from any producer | `pa.RecordBatchReader.from_stream(obj)` — consumes anything exposing `__arrow_c_stream__`, regardless of which library produced it | `[api:pyarrow@25.0.1]`, `[probe]` |
| Core containers | `Table`, `RecordBatch`, `RecordBatchReader`, `Schema`, `Field`, `ChunkedArray`, `Array`, `ExtensionArray`, `ExtensionScalar` | `[api:pyarrow@25.0.1]` |
| IPC | `pa.ipc.new_stream`, `pa.ipc.open_stream`, file variants | `[api:pyarrow@25.0.1]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §21.1 tables cross as `__arrow_c_stream__`, never as row objects | present on `Table` and `RecordBatchReader` | **confirmed** `[api:pyarrow@25.0.1]` |
| §21.1 "object exposing `__arrow_c_stream__` → pyarrow / polars / datafusion-python" | `RecordBatchReader.from_stream` consumes the protocol generically | **confirmed** — the capsule protocol is implementation-neutral, so the Rust side need not know which Python Arrow library the consumer uses |
| §21.1 "Transfer is one bundle, not per-node calls" | a stream per named table | **confirmed as the right shape** — see DM-37 below |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| Cross the boundary as a **stream**, not a materialised `Table` | **DM-37** (cross expensive boundaries in coarse, typed units) | **adopt, and state it.** §21.1 already says "one bundle"; the sharper rule is that each table should cross as a `RecordBatchReader` the consumer drains, not a fully materialised `Table`. One FFI crossing per relation either way, but the streaming form does not require the whole relation resident on both sides simultaneously. |
| Keep `pyarrow` out of the *contract* | **DM-41** (adapters mechanical), DM-57 | **adopt.** The capsule protocol is the contract; `pyarrow` is one implementation of it. §21.5's typed classes should be built from the streamed batches, never from `pyarrow`-specific types in their field annotations — otherwise the Python contract layer is pinned to one Arrow implementation. |

### Gaps and risks

- **`datafusion` (Python) lagged the Rust engine** in the environment surveyed — 54.0.0 against the Rust side's 55.1.0. It is only a *consumer* of the capsule protocol, so version skew is tolerable by design; worth stating explicitly, since §21.1 names it and a reader could infer the versions must match.
- `pyarrow` 25.0.1 against §3.1's "pyarrow 15+" is a ten-major gap. Nothing observed here requires 15; but an unpinned floor means CI could resolve anything in that range, and extension-type behaviour (§3) is exactly the kind of thing that moved over those releases. **Pin it.**

---

## 3. pyarrow — extension types, metadata, and the loss profile (P2)

The cluster that decides whether the semantic typing built in Rust survives into Python. **DM-06**, **DM-42**, **DM-44**, **G2**.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Extension base | `pa.ExtensionType(storage_type, extension_name)`; bases `BaseExtensionType → DataType` | `[api:pyarrow@25.0.1]` |
| Required overrides | `__arrow_ext_serialize__(self) -> bytes`, `__arrow_ext_deserialize__(cls, storage_type, serialized)` | `[api:pyarrow@25.0.1]` |
| Optional overrides | `__arrow_ext_class__`, `__arrow_ext_scalar_class__` | `[api:pyarrow@25.0.1]` |
| Registry | `pa.register_extension_type(instance)`, `pa.unregister_extension_type(name)` | `[api:pyarrow@25.0.1]` |
| Inspection | `extension_name`, `storage_type`, `wrap_array`, `field`, `num_buffers`, `byte_width` | `[api:pyarrow@25.0.1]` |
| Unresolved marker | `pa.UnknownExtensionType` exists as a type | `[api:pyarrow@25.0.1]` |
| Array/scalar | `pa.ExtensionArray.from_storage(type, storage)`, `pa.ExtensionScalar` | `[api:pyarrow@25.0.1]` |

### Measured: the loss profile (PROBE 1 and 2)

A `pse.semantic_id`-shaped extension over `binary(16)`, with `pse.*` schema and field metadata, round-tripped through IPC and through `__arrow_c_stream__`:

```text
PROBE 1  extension type through IPC
   registered   -> field type: SemanticId          | extension_name: pse.semantic_id
   UNregistered -> field type: FixedSizeBinaryType | extension_name: None | storage: fixed_size_binary[16]
   unregistered field metadata retains ARROW:extension:name: True

PROBE 2  metadata across IPC and PyCapsule
   IPC  schema md : b'rel-1'      IPC  field md : b'qt-demo'
   caps schema md : b'rel-1'      caps field md : b'qt-demo'
   caps id type   : SemanticId
```

| Blueprint claim | Verdict |
|---|---|
| §4.4 "Python consumers register matching `pyarrow.ExtensionType` classes; unknown consumers see storage types" | **confirmed exactly.** With a registered class the type round-trips as `SemanticId`; without one it becomes `fixed_size_binary[16]`. |
| §4.3 metadata "preserved through IPC/Parquet" — the Python half | **confirmed for IPC and for the PyCapsule path**, at both schema and field level. |
| §21.1 the capsule path preserves extension types | **confirmed** — `caps id type: SemanticId`. |
| This document's own earlier claim that unregistered types surface as `UnknownExtensionType` | **refuted** — they surface as the plain storage type. |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Detect degradation via retained `ARROW:extension:name` metadata** | **DM-42**, **G2**, DM-59 | **adopt — the key finding of this cluster.** Loss of an extension type is silent *at the type level*: a consumer sees `fixed_size_binary[16]` with no error. But the field metadata still carries `ARROW:extension:name` `[probe]`. So the Python contract layer can assert, per column, "this field claims extension `X` — is `X` registered?" and fail loudly if not. That converts an invisible G2 degradation into a detectable, testable condition, and it costs one check at the boundary. |
| **Ship and register the `pse.*` extension classes as part of the Python package** | **DM-44** (extensions complete, versioned, conformance-testable) | **adopt.** §4.4 says consumers register matching classes but does not say *who ships them*. If each consumer writes its own, the serialization contract forks. The ten `pse.*` classes should be generated from the same registry that generates the Rust side (§4.2) and shipped in `python/pse/`, with a conformance test mirroring the Rust one. |
| Declare the loss profile per column in the bundle | **G7**, DM-42 | **adopt.** §21.1's `manifest` should record, per table, which fields carry extension types — so a consumer can verify coverage rather than discover a degradation by its absence. |
| `__arrow_ext_scalar_class__` for `pse.quantity_value` | DM-06 | **evaluate.** A struct-backed extension exposed as a bare struct scalar loses its meaning at the point of use. A custom scalar class is the Python-side way to keep it. Only worth it for types a human actually inspects in a notebook. |
| `pa.UnknownExtensionType` | — | **reject as a mechanism.** It exists, but is not what the IPC/capsule path produces `[probe]`; relying on it would be relying on a path we measured not to occur. |

### Gaps and risks

- **Registration is global and process-wide.** `register_extension_type` mutates a global registry; a second registration of the same name raises. A library that registers on import interacts badly with test isolation and with any host process that also registers `pse.*`. **Recommend an idempotent register-once helper in `python/pse/`.**
- Extension metadata is *versioned by nothing*. `__arrow_ext_serialize__` returns opaque bytes; if the Rust-side `Metadata` shape changes, the Python `__arrow_ext_deserialize__` must change in lockstep or silently misparse. Same gap the Arrow map recorded for the Rust side (DM-44, DM-51) — and it is worse here, because the two implementations are in different languages and different repositories' release cycles.

---

## 4. Pyomo — model construction and expression lowering (P3)

**Role:** §21.2 steps 1–2 — the adapter algorithm; §18.5's Pyomo binding row; D12.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Model and components | `ConcreteModel`, `Var`, `Param(mutable=True)`, `Constraint`, `Objective`, `Set`, `RangeSet`, `Suffix`, `Block` | `[api:pyomo@6.10.1]` |
| Affine form | `pyomo.core.expr.numeric_expr.LinearExpression(constant, linear_coefs, linear_vars)` | `[api:pyomo@6.10.1]`, `[probe]` |
| Conditional | `pyo.Expr_if(IF=…, THEN=…, ELSE=…)` → `Expr_ifExpression` | `[probe]` |
| External functions | `pyo.ExternalFunction(library=…, function=…)` → `AMPLExternalFunction`; calling it yields `ExternalFunctionExpression` | `[probe]` |
| Solve | `pyo.SolverFactory(name)`, `.available(False)`, `.solve(model, …)` | `[api:pyomo@6.10.1]`, `[probe]` |

### Measured (PROBE 3)

```text
   LinearExpression      : LinearExpression | value: 2.0
     nargs 3 | polynomial_degree: 1
   Expr_if               : Expr_ifExpression | value: 1.0
   ExternalFunction decl : AMPLExternalFunction | callable in expr: ExternalFunctionExpression
   (warning) Defining AMPL external function, but cannot locate specified library "libpse_kernels.so"
```

| §21.2 step 2 requirement | Verdict |
|---|---|
| `Affine` lowering | **Qualified:** the probe constructs a truly linear expression. Arbitrary Affine children may be nonlinear; use LinearExpression only after proving affine dependence on active variables (ADR-0047). |
| `Conditional` to `Expr_if` | **Interface/probe scope:** expression construction works; guarded evaluation must be qualified for the actual Pyomo writer and selected solver/numerical policy (ADR-0047). |
| "`KernelCall` → … otherwise `ExternalFunction(library, function)`" | **confirmed** as a declaration; see the risk below |
| "`ImplicitRef` never appears (implicit systems stay as equations)" | platform rule, nothing in Pyomo to confirm or refute |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **A missing external-function library is a *warning*, not an error** | **G7** (truthful capability claims), **DM-43**, DM-30 | **adopt a hard check.** `ExternalFunction` with an unresolvable library emitted a warning and still produced a usable expression object `[probe]`; the failure surfaces only at solve time, from the solver, far from the cause. §21.4 already requires an end-to-end test per `pyomo_external_function` kernel — the stronger rule is that the adapter must *verify the library resolves* before building the model, and raise `capability.backend` (§23.2) with the kernel id if not. A warning on stderr is not a typed failure. |
| `polynomial_degree()` as a lowering assertion | DM-24, DM-53 | **Adopt with classification:** compare with the expected degree, 0 for constants and 1 for a nonconstant affine-in-variables form. An Affine opcode alone does not justify degree 1 (blueprint §21.2). |
| Build through `Block`s mirroring `compiled` relation structure | DM-18 | **reject.** Tempting for readability, but D12 is explicit that the adapter "builds no IDAES classes" and "engineering names live in the source map". Blocks would recreate a hierarchy the design deliberately flattened to ordinals. |

### Gaps and risks

- `Param(mutable=True)` is required for §21.2's parameters to be updatable between solves; an immutable `Param` is folded into expressions at construction. The blueprint says `mutable=True` — correct, and worth keeping as a generated-code invariant rather than a convention.
- Pyomo's expression system is large and has several near-equivalent spellings for the same mathematics (`LinearExpression` vs. a sum of products). Since the adapter is generated from the IR, the choice should be made once in the generator, not per operator.

---

## 5. Pyomo — units, suffixes, solve, and results (P4)

This cluster contains the map's sharpest finding: **a direct contradiction between §21.2 and §3.3**.

### The pint contradiction

§21.2 step 1 requires "units attached from the unit set so that `assert_units_consistent` passes". §3.3's closing line states: *"Explicitly not added: … a Python-side units library."*

Measured `[probe]`, on a clean environment with the §3.3-compliant dependency set (no pint):

```text
   m2.T = pyo.Var(units=u.K, initialize=320.0)
   pyomo.common.errors.DeferredImportError: The pint module (an optional Pyomo dependency)
   failed to import: ModuleNotFoundError: No module named 'pint'
```

**Pyomo's units system *is* pint.** `pyomo.core.base.units_container` defers to `pint_module` and raises on attribute access when it is absent `[probe]`. So §21.2's requirement cannot be satisfied under §3.3's exclusion. This is not a transitive-dependency inconvenience — the two statements are incompatible as written.

**Recommended resolution** (and the basis for an amendment): §3.3's exclusion is about **authority**, not about presence. Reword it to say that no Python-side units library is an *authority* for quantities — §8's dynamic registry remains sole authority — while acknowledging that `pint` is required by Pyomo's units machinery and is therefore present in the adapter and parity environments. The boundary rule then becomes enforceable: pint may validate what the adapter emits; it may never define a quantity, and no `pse` relation may be derived from a pint object.

### Capability inventory and measurements (PROBE 4, 5)

| Capability | Surface | Provenance |
|---|---|---|
| Units container | `pyo.units` (`PyomoUnitsContainer`), backed by pint | `[api:pyomo@6.10.1]`, `[probe]` |
| Consistency assertion | `pyomo.util.check_units.assert_units_consistent(obj)` — **not** in `pyomo.environ` | `[api:pyomo@6.10.1]`, `[probe]` |
| Culprit identification | `pyomo.util.check_units.identify_inconsistent_units(block)` → the offending components | `[probe]` |
| Related | `assert_units_equivalent`, `check_units_equivalent`, `UnitsError`, `InconsistentUnitsError` | `[api:pyomo@6.10.1]` |
| Suffixes | `pyo.Suffix(direction=Suffix.EXPORT \| IMPORT)`; `scaling_factor`, `dual`, `ipopt_zL_out`, `ipopt_zU_out` | `[probe]` |

```text
PROBE 4   consistent model      : PASSES
          inconsistent model    : raises InconsistentUnitsError ("kelvin not compatible with …")
          identify_inconsistent_units -> ['bad']        <- names the culprit constraint

PROBE 5   Suffix EXPORT/IMPORT  : declared ok; scaling_factor[x] = 0.001
          SolverFactory(ipopt) available: False
          SolverFactory(cbc)   available: False
          SolverFactory(glpk)  available: False
```

| Blueprint requirement | Status |
|---|---|
| §21.2 step 1 "`assert_units_consistent` passes" | **confirmed working — with pint present**, and **erratum**: the function is in `pyomo.util.check_units`, not `pyomo.environ` as the phrasing implies |
| §21.2 step 3 `scaling_factor` suffix export | **confirmed** |
| §21.2 step 4 `dual`, `ipopt_zL_out`, `ipopt_zU_out` import | **declaration confirmed; round-trip `[UNVERIFIED]`** — no solver is installed in this environment, so the probe could not complete the loop. Recorded as unverified rather than asserted. |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Use `identify_inconsistent_units`, not a bare assert** | **DM-47** (structured evidence), DM-30, §23.2 | **adopt.** `assert_units_consistent` raises; `identify_inconsistent_units` returns the offending components `[probe]`. §23.2's `compile.math` class exists to name the culprit — "unit inconsistency" with a component list is a finding; an exception with a message is not. |
| Units as a **cross-check**, never a source | **DM-02** (one authority per fact), **G1** | **adopt as the boundary rule.** The unit set crosses in the bundle from §8's registry; Pyomo/pint validates that what the adapter built matches. If the two disagree, the registry is right and the adapter has a bug. Making that direction explicit is what keeps pint from becoming a second authority. |
| Solver availability probed before the model is built | **DM-43**, G7 | **adopt.** `SolverFactory(name).available(False)` is a cheap pre-flight `[probe]`. §18.3 already does a capability probe for `ipopt_has_linear_solver`; the same discipline belongs at the Pyomo boundary so a missing solver is a declared capability failure rather than a late crash. |

---

## 6. Pyomo — ecosystem tools and the ASL boundary (P5)

**Role:** §21.3 — "parity testing, and the Pyomo ecosystem: `parmest`, `PyROS`, `GDPopt`, `pyomo.dae` utilities, and `IncidenceGraphInterface`-based cross-checks"; §21.4 opaque kernels and ASL; §18.9's capability matrix.

### Availability — all confirmed present at 6.10.1 `[probe]`

| Module | Public names | Blueprint use |
|---|---|---|
| `pyomo.contrib.parmest.parmest` | 48 | §21.3 parameter estimation; §19.4 |
| `pyomo.contrib.pyros` | 22 | §21.3 robust optimization; §19.8 uncertainty |
| `pyomo.contrib.gdpopt` | 18 | §21.3, §18.7's `GDP` solve plan |
| `pyomo.dae` | 11 | §21.3 DAE utilities; §13.4 discretization cross-checks |
| `pyomo.contrib.incidence_analysis` | 17 | §21.3 cross-checks of native structural results (§15.3) |
| `pyomo.contrib.iis` | 4 | **not named in the blueprint** — see below |

`IncidenceGraphInterface` resolves at `pyomo.contrib.incidence_analysis.interface` `[probe]`, confirming §21.3's naming.

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **`pyomo.contrib.iis` for infeasibility explanation** | **DM-47**, DM-30 | **evaluate.** §15.5 specifies a hand-built minimal-intractable-system procedure (three elasticization rounds plus a deletion filter). Pyomo ships an IIS facility that solves the same problem. Since the Pyomo adapter already exists for parity, this is available as a **cross-check oracle** for the native MIS implementation at near-zero cost — the same role §21.3 already assigns to `IncidenceGraphInterface`. Not a replacement: §15.5's version runs natively, over case overlays, without model cloning. |
| Treat every ecosystem tool as a **declared capability with a version** | **DM-43**, **G7**, DM-31 | **adopt.** §18.9's capability matrix says which operators each backend supports; it does not record *which version* of `parmest`/`PyROS`/`GDPopt` a result came from. These are `pyomo.contrib` packages — explicitly lower-stability than Pyomo's core — and their behaviour changes between releases. A run that used one must record it in `runtime.runs.environment` (§20.3) or the result is not reproducible. |
| Cross-check native structural results against `IncidenceGraphInterface` **in CI, not ad hoc** | **DM-53** (verify equivalence across representations), DM-54 | **adopt.** §21.3 describes this as a use of the adapter; making it a standing parity test over the golden snapshots (§24) turns it from an available technique into a regression control. |

### Gaps and risks

- **`pyomo.contrib.*` is a contrib namespace.** Five of the six tools §21.3 relies on live there. That is a stability profile the blueprint should state, because §18.9's matrix currently reads as though these are equivalent in standing to the native and NL backends. They are not: they are the least stable surface in the entire dependency set.
- §21.4's rule that "Python callback functions are not used for solves because ASL-based solvers cannot execute them" is a *correct and important* constraint, and the `ExternalFunction` probe (§4) shows why it must be enforced early: the object constructs happily either way, and only the solver notices.

---

## 7. attrs + cattrs — the typed contract layer (P6)

**Role:** §21.5 — "typed attrs classes with explicit converters at the boundary, cattrs for structuring, no `Any` fields, and no Python-side model classes that duplicate relations."

**Note:** §21.5 cites `docs/library_ref/contract_substrate_discipline.md` for this discipline. **That file and its directory have been deleted** — so the referenced definition is currently unreadable (erratum E1). What follows measures the discipline's stated properties directly.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Class definition | `attrs.define`, `attrs.frozen`, `attrs.field`, `attrs.mutable` | `[api:attrs@26.1.0]` |
| Field machinery | validators, converters, defaults, `kw_only`, `alias`, `metadata` | `[api:attrs@26.1.0]` |
| Introspection | `attrs.fields(cls)`, `attrs.asdict`, `attrs.astuple`, `attrs.evolve` | `[api:attrs@26.1.0]`, `[probe]` |
| Structuring | `cattrs.Converter`, `structure`, `unstructure`, `register_structure_hook`, `register_unstructure_hook` | `[api:cattrs@26.2.0]` |
| Strictness options | `Converter(forbid_extra_keys=…, detailed_validation=…)`; detailed validation is **on by default since 22.1.0** | `[probe]`, `[c7:/python-attrs/cattrs]` |
| Errors | `ClassValidationError` (exception group of sub-errors); `ForbiddenExtraKeysError` | `[probe]`, `[c7:/python-attrs/cattrs]` |
| Error rendering | `cattrs.transform_error(exc, format_exception=…)` → list of per-field messages | `[c7:/python-attrs/cattrs]` |
| Global strictness idiom | `register_structure_hook_factory(has, …)` + `cattrs.gen.make_dict_structure_fn(cl, c, _cattrs_forbid_extra_keys=True)` | `[c7:/python-attrs/cattrs]` |

### Measured (PROBE 6)

```text
   structure happy path  : VarRow(ordinal=1, semantic_id=b'…', lower=0.0)
   unknown key           : ACCEPTED (silently dropped)
   forbid_extra_keys     : rejected -> ClassValidationError
   wrong type            : rejected -> ClassValidationError
   detailed_validation   : ClassValidationError | sub-errors: 1
   Any field             : ACCEPTED  (nothing in cattrs forbids it)
   attrs introspection   : fields() -> ['ordinal', 'semantic_id', 'lower']
```

| §21.5 requirement | Verdict |
|---|---|
| "typed attrs classes with explicit converters at the boundary" | **confirmed available** |
| "cattrs for structuring" | **confirmed** |
| "no `Any` fields" | **erratum — unenforced.** cattrs accepts an `Any` field and structures whatever it is given `[probe]`. §21.5 states this as a property of the design; it is a convention with nothing behind it. |
| Implicit expectation that structuring is strict | **refuted by default.** A default `cattrs.Converter()` **silently drops unknown keys** `[probe]`. |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Forbid extra keys at every boundary** | **DM-42** (reject silent degradation), **G2**, G3 | **adopt — the highest-value item in this cluster.** Measured: the default silently discards unknown keys `[probe]`, and upstream states the same intent — "by default, cattrs ignores extra keys"; `forbid_extra_keys` "enforces a stricter contract… especially when fields have default values" `[c7:/python-attrs/cattrs]`. At a boundary carrying a versioned contract, an unknown key is *evidence of a version mismatch*; dropping it converts detectable schema drift into silent data loss. Two spellings work: `Converter(forbid_extra_keys=True)` `[probe]`, or the upstream idiom applying it to **every** attrs class at once — `register_structure_hook_factory(has, lambda cl: make_dict_structure_fn(cl, c, _cattrs_forbid_extra_keys=True))`, raising `ForbiddenExtraKeysError` `[c7:/python-attrs/cattrs]`. Prefer the factory: it cannot be forgotten on a new class. |
| **Consume the sub-errors — and do not disable detailed validation** | **DM-47**, DM-30 | **adopt, with a correction to this document's own first pass.** Detailed validation is **on by default since cattrs 22.1.0** `[c7:/python-attrs/cattrs]`, so the work is not *enabling* it — it is (a) never passing `detailed_validation=False` for speed, and (b) actually **consuming** the sub-errors. `ClassValidationError` is an exception group carrying one per bad field `[probe]`, and **`cattrs.transform_error()`** renders it as a list of per-field messages `[c7:/python-attrs/cattrs]` — a ready-made bridge to §23.2's structured findings. Taking only the first message discards exactly the structure that makes a validation failure actionable. |
| **A governance check for `Any`** | **DM-06**, G3 | **adopt.** Since nothing at runtime forbids `Any`, the rule needs a lint: walk `attrs.fields()` of every contract class at import or test time and fail on `Any`/bare `dict`/bare `list`. `attrs.fields()` makes this ~10 lines `[probe]`, and it is the only way §21.5's claim becomes true. |
| **Generate the contract classes from the registry** | **DM-52** (generate mechanical artifacts from shared contracts), DM-23 | **adopt.** §4.2 already generates nine artifacts per relation from `RelationSpec`, including a `serde_arrow` schema for the Rust side. The Python contract classes are the same projection in another language. Hand-writing them re-expresses the column list a second time — which §4.2 calls a governance failure — and guarantees drift. This is the single largest structural opportunity in the Python surface. |
| `attrs.frozen` for anything crossing the boundary | DM-13, DM-20 | **adopt.** Bundle rows are observations of an immutable snapshot; frozen classes make that structural and keep inspection non-mutating (DM-20). |

---

## 8. msgspec — wire contracts, constraints, and schema generation (P7)

**Role:** §21.5 names msgspec among the Python contracts; §20.2's manifest is JSON. The blueprint never says what msgspec is *for* — §9 adjudicates that.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Typed structs | `msgspec.Struct` with `frozen=`, `forbid_unknown_fields=`, `gc=`, `tag=`, `rename=` | `[api:msgspec@0.21.1]`, `[probe]` |
| Codecs | `msgspec.json`, `msgspec.msgpack`, `msgspec.yaml`, `msgspec.toml` — `encode` / `decode(buf, type=T)` | `[api:msgspec@0.21.1]` |
| Constraints | `msgspec.Meta(gt=, ge=, lt=, le=, min_length=, max_length=, pattern=, …)` via `Annotated` | `[probe]` |
| Validation errors | `msgspec.ValidationError` with **JSON-path location** | `[probe]` |
| Schema generation | `msgspec.json.schema(T)` → JSON Schema; `schema_components` for multiple types | `[probe]` |
| Conversion | `msgspec.convert`, `msgspec.to_builtins`, `msgspec.Raw`, `msgspec.structs.*` | `[api:msgspec@0.21.1]` |

### Measured (PROBE 7)

```text
   encode                : b'{"name":"stoichiometry","rows":12,"ratio":1.0}'
   decode round-trip     : Rel(name='stoichiometry', rows=12, ratio=1.0)
   unknown field         : rejected -> ValidationError | Object contains unknown field `BOGUS`
   wrong type            : rejected -> ValidationError | Expected `int`, got `str` - at `$.rows`
   Meta(gt=0) violation  : rejected -> ValidationError | Expected `float` > 0.0 - at `$.factor`
   json.schema(Rel)      : {'$ref': '#/$defs/Rel', '$defs': {'Rel': {'title': 'Rel', 'type': 'object',
                            'properties': {...}, 'required': ['name','rows'],
                            'additionalProperties': False}}}
```

Three properties matter for this design, and all three were measured rather than assumed:

1. **Errors carry a location.** `at $.rows`, `at $.factor` — a JSON path to the offending value. §23.2 requires failures that carry the thing that failed; msgspec supplies it natively.
2. **Constraints are declarative and enforced at decode.** `Meta(gt=0.0)` on a scaling factor is checked at the boundary, not by hand-written code afterwards.
3. **It generates JSON Schema**, with `additionalProperties: False` derived from `forbid_unknown_fields`.

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **msgspec owns the §20.2 manifest** | **DM-42**, DM-47, DM-44 | **adopt.** The manifest is JSON on disk, versioned, read by both Rust and Python, and the place where a malformed or drifted artifact must be caught. `Struct(frozen=True, forbid_unknown_fields=True)` plus `Meta` constraints gives strict decode with located errors, which is exactly what §20.5's "a snapshot whose schema registry fingerprint is unknown is rejected" needs. |
| **`msgspec.json.schema()` vs §4.2's generated authoring JSON Schema** | **DM-23** (derived representations traceable and **non-competing**), DM-52 | **evaluate carefully — this is a competing-authority risk, not a free win.** §4.2 already generates "Authoring JSON Schema for editor validation of package documents" from the Rust registry. msgspec can generate JSON Schema from Python structs. If both exist for the same documents, there are two schemas for one contract and they *will* drift — precisely what DM-23 forbids. **Recommendation: the registry-generated schema is authoritative; msgspec's generator is used only for Python-internal wire types that have no registry counterpart (e.g. the manifest), never for authoring documents.** |
| `Meta` constraints mirroring registry invariants | **DM-07**, G3 | **evaluate.** Attractive — a `scaling_factor > 0` invariant enforced at the boundary. But §4.2 generates invariants as DataFusion plans executed by P2; duplicating them as `Meta` constraints re-expresses the same rule in a second place (DM-56). Adopt only where the constraint guards the *wire format* itself rather than model semantics. |
| `msgspec.Raw` for pass-through payloads | DM-37 | **evaluate** — defers parsing of a nested payload, useful if the manifest carries per-relation blobs the adapter does not need to inspect. |

### Gaps and risks

- msgspec is **0.21.1 — pre-1.0**, the only pre-1.0 library in the Python set. Its API has been stable in practice, but the version number is the honest signal and it should be `==`-pinned, not floated.
- `msgspec.Struct` is not `attrs` — the two class systems do not interoperate, which is the substance of §9.

---

## 9. Adjudication: attrs/cattrs versus msgspec (P8)

§3.3 lists three libraries that overlap heavily. §21.5 names attrs and cattrs and never says what msgspec is for. Leaving this open means the same meaning gets expressed twice at the boundary — **DM-56** ("optimize for fewer independent semantic decisions"), **DM-57** ("a small coherent core"), **DM-58**.

### What the measurements say

| Property | attrs + cattrs | msgspec | Source |
|---|---|---|---|
| Unknown keys, default | **silently dropped** | **rejected** | `[probe]` |
| Unknown keys, strict mode | `forbid_extra_keys=True` | default | `[probe]` |
| Error location | field name via `ClassValidationError` sub-errors | **JSON path** (`at $.rows`) | `[probe]` |
| Multiple errors at once | yes (exception group) | first error | `[probe]` |
| Declarative value constraints | attrs validators (imperative) | **`Meta(gt=…)`** (declarative) | `[probe]` |
| JSON Schema generation | no | **yes** | `[probe]` |
| Codec coverage | none (structuring only) | json, msgpack, yaml, toml | `[api:msgspec@0.21.1]` |
| Works on arbitrary existing classes | **yes** — structures into any attrs class | no — must be a `Struct` | `[api:*]` |
| Introspection for governance | **`attrs.fields()`** | `msgspec.structs.fields` | `[probe]` |

### The adjudication

They are not redundant; they occupy different positions, and the evidence separates them cleanly:

- **msgspec is a wire codec.** Strict by default, located errors, schema generation, four formats. Its natural job is **bytes ↔ typed object at a serialization boundary**: §20.2's manifest, and any JSON the platform reads or writes.
- **attrs + cattrs is an in-memory contract layer.** cattrs structures from *already-decoded* material — which is what the Arrow path produces. Rows arriving from a `RecordBatchReader` are Python objects, not bytes; there is no JSON to decode, so msgspec's strength does not apply, while cattrs' ability to structure into arbitrary attrs classes does.

**Recommended division, to be written into §21.5:**

| Boundary | Library | Why |
|---|---|---|
| §20.2 manifest, and all JSON/TOML the platform reads or writes | **msgspec** | strict decode, located errors, constraints, schema generation |
| Arrow bundle rows → typed Python objects (§21.1, §21.2) | **attrs + cattrs**, with `forbid_extra_keys=True` and `detailed_validation=True` | the input is decoded material, not bytes; cattrs structures into generated attrs classes |
| Anything else | neither — it does not exist | DM-57 |

The rule that keeps this from becoming two authorities: **a given contract is defined in exactly one of the two systems, never both.** The manifest is a msgspec `Struct`; a relation row is a generated attrs class. No type is expressed in both.

This also resolves why three libraries is not over-engineering under DM-58: attrs and cattrs are one thing (classes plus their structuring), msgspec is another (a codec), and each replaces hand-written work that would otherwise exist — provided the division above is stated rather than left to taste.

---

## 10. numpy — the array boundary (P9)

**Role:** §21.1/§21.2 move numeric columns across the boundary; D11 governs layout ownership; §24's parity harness compares native and Pyomo results numerically.

### Measured (PROBE 8) — the null/NaN collapse

```text
   zero-copy with nulls  : REFUSED -> ArrowInvalid
                           "Needed to copy 1 chunks with 1 nulls, but zero_copy_only was True"
   zero_copy_only=False  : [ 1. nan -0.]  | dtype float64 | null became: nan
   no-null zero-copy     : [ 1.  2. -0.]  | shares buffer: True
   -0.0 sign preserved   : True
```

This is the cluster's finding, and it is a **G2** one.

| Fact | Consequence |
|---|---|
| Arrow → numpy with nulls present **converts null to NaN** | §4.4 states `pse.bound` must never use "NaN or null sentinels", and the Arrow map verified that Arrow keeps null, NaN and `-0.0` structurally distinct. **A plain `ndarray` cannot hold that distinction** — float64 has no validity bitmap. The moment a nullable column becomes an `ndarray`, null and NaN are the same value. |
| `zero_copy_only=True` **refuses** rather than silently copying | Good design: the copy is opt-in and visible. |
| `-0.0` survives | consistent with the Arrow side |

| Blueprint requirement | Status |
|---|---|
| D11 "borrow Arrow buffers when layouts permit and copy when they do not" — Python side | **confirmed**: `zero_copy_only=True` succeeds only for a null-free, single-chunk, primitive column `[probe]`; anything else must copy, and says so |
| §4.4's no-sentinel discipline, carried into Python | **at risk** — see below |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Never convert a nullable column to a bare `ndarray`** | **DM-08**, **G2** | **adopt as a boundary rule.** Either assert `null_count == 0` before converting (true for solver vectors by construction — §18.1 says fixed and parameter symbols are bound, and solution values are total), or carry the mask alongside. The failure mode is silent and numerically plausible, which is the worst kind. The assertion is one line and belongs in the generated adapter. |
| **`zero_copy_only=True` as the default in adapter code** | DM-37, DM-39 | **adopt.** It turns an unintended copy into an error rather than a performance mystery, and documents which columns are genuinely borrowable. Where a copy is required, passing `False` explicitly records that decision at the call site. |
| `combine_chunks()` before conversion | DM-36 | **note** — a chunked column has no single buffer; zero-copy requires one chunk `[probe]`. |

### Gaps and risks

- numpy is **unpinned in §3.1** and arrives transitively through both Pyomo and pyarrow. numpy 2.x changed several conversion and dtype behaviours relative to 1.x. For a parity harness that compares numbers, the numpy version is part of the reproducibility contract (**DM-48**) and should be pinned like everything else.

---

## 11. scipy — structural and numerical cross-checks (P10)

**Role:** scipy is not named in §3.3, but arrives with Pyomo and is the natural oracle for §15's native structural and numerical analyses.

### Capability inventory `[probe]`, `[api:scipy@1.18.1]`

| Capability | Surface |
|---|---|
| Sparse formats | `csr_matrix`, `csc_matrix`, `coo_matrix`, `bsr_matrix`, `dia_matrix`, `dok_matrix`, `lil_matrix` |
| Graph algorithms | `scipy.sparse.csgraph.{connected_components, structural_rank, maximum_bipartite_matching, reverse_cuthill_mckee, depth_first_order}` |
| Dense SVD | `scipy.linalg.{svd, svdvals}` |
| Sparse SVD | `scipy.sparse.linalg.svds` |

### Pass 2 — the cross-check opportunity

| Item | DM / gate | Recommendation |
|---|---|---|
| **`maximum_bipartite_matching` as an oracle for §15.3's Hopcroft–Karp** | **DM-53** (verify invariants and equivalence across representations), DM-54 | **adopt as a test-only oracle.** The supporting-library map established that `petgraph` ships only general-graph matching (Gabow, O(\|V\|³)) and that §15.3 therefore implements Hopcroft–Karp natively. Hand-written matching on the equation×variable incidence graph is exactly the kind of code that is subtly wrong on degenerate inputs — and scipy ships a bipartite implementation that can be run over the same golden snapshots in CI. Matching *cardinality* must agree; the matching itself need not (it is not unique). |
| **`structural_rank` as a DOF cross-check** | DM-53 | **adopt as a test-only oracle** for §15.2's structural checks and `problems.degrees_of_freedom`. |
| **`svds` / `svdvals` against §15.5's SVD toolbox** | DM-53, DM-39 | **evaluate.** §15.5 specifies dense `faer` SVD for small problems and "iterative for large" — which the Arrow-side map flagged as having no faer implementation behind it. scipy's `svds` is a working iterative sparse SVD and is a legitimate oracle for the small-problem path, and a reference for what the large path must eventually do. |
| scipy in *production* paths | **DM-38**, D11 | **reject.** D11 gives the native numerics ownership of execution layouts. scipy's role here is exclusively as a **test-time oracle** running through the existing Python/parity boundary — never inside a pass, a solve, or a diagnostic that the platform ships. |

### Gaps and risks

- scipy, like numpy, is unpinned and transitive. If it is going to serve as a correctness oracle in CI, it is part of the test contract and must be pinned (**DM-48**).
- An oracle that disagrees is only useful if the disagreement is attributable. These cross-checks should compare *invariants* (matching cardinality, structural rank, singular-value count below tolerance), not object identity.

---

## 12. IDAES 2.12 — the parity surface and its dependency boundary (P11)

**Role:** §3.1 "IDAES 2.10 for parity tests only"; §24's parity harness; the source of the behaviours §9–§17 reproduce.

### Availability `[probe]`, `[api:idaes@2.12.0]`

IDAES **2.12.0** imports cleanly on CPython 3.13.12. 191 modules, 2762 public items extracted. The modules the parity harness needs all resolve:

| Module | Parity role |
|---|---|
| `idaes.core` | flowsheet, control-volume and property-package base classes (§9–§13) |
| `idaes.core.util.model_statistics` | DOF and variable/constraint counts (§15.2) |
| `idaes.core.util.scaling` | scaling factors and the Jacobian conditioning checks (§15.4, §16) |
| `idaes.core.util.model_diagnostics` | the diagnostics toolbox §15 reproduces |
| `idaes.models.properties.modular_properties` | the `GenericParameterBlock` configuration §9.1 maps to relations |

### Version finding

§3.1 names **2.10**; current is **2.12.0** `[pypi:idaes-pse]`, and this map is built against 2.12.0 per the owner's instruction to use latest throughout. **Parity is only meaningful against a stated version** — §24's acceptance criteria compare platform behaviour to IDAES behaviour, and IDAES changed between 2.10 and 2.12. This must be pinned exactly, not floored.

### The dependency boundary

IDAES 2.12.0 requires `pyomo>=6.10.1` — exactly this map's Pyomo pin, so the two are compatible `[pypi:idaes-pse]`. It also pulls a substantial tail: **`pint` 0.26.1**, `pydantic` 2.13.5, `sympy` 1.14.0, `networkx` 3.6.1, plus pandas, matplotlib and click `[probe]`.

| Item | DM / gate | Recommendation |
|---|---|---|
| **pint enters through IDAES *and* through Pyomo's own units** | **DM-02** (one authority per fact), **G1** | **adopt the boundary rule** stated in §5: no Python-side units library is an *authority*; §8's dynamic registry is. pint may validate; it may never define. The §3.3 exclusion should be reworded from presence to authority, because presence is not achievable — Pyomo's units machinery *is* pint. |
| **`pydantic` and `sympy` arrive in the parity environment** | **DM-57**, DM-58 | **note, and fence.** Neither is a platform dependency; both are capable enough to tempt use. The parity environment should be a separate dependency set from the platform's Python package, so that what IDAES drags in cannot quietly become something the platform relies on. |
| Record the IDAES version in every parity result | **DM-31**, DM-48 | **adopt.** §20.3 records solver and package versions for runs; parity comparisons need the same treatment or a parity failure cannot be attributed to a platform change versus an IDAES change. |

### Gaps and risks

- **IDAES caps the interpreter at 3.13** (classifiers 3.10–3.13; `requires_python` unset) `[pypi:idaes-pse]`. §3.1's "Python 3.11+" is therefore only true for the platform package; the parity environment has an upper bound as well as a lower one. Two dependency sets, two interpreter ranges.
- The vendored `idaes-pse/` tree in this repository is at the version the blueprint named (2.10 line), not 2.12.0. Reading the tree and testing against the pinned package are two different sources; the pinned package is the one parity claims are made against.

---

## 13. Survey — pandas, polars, datafusion-python (P12)

| Library | Status | Reason |
|---|---|---|
| **pandas** (3.0.5 observed) | **not a platform dependency**; arrives via IDAES | §21.1's bundle is Arrow streams; a pandas conversion loses the null/NaN distinction the same way numpy does (§10) and adds an index concept the relational model does not have. Acceptable inside parity/notebook code, never in the adapter. |
| **polars** | **not used; supported by construction** | §21.1 names it as a possible consumer. Because the boundary is the PyCapsule protocol rather than a pyarrow type, polars can consume the bundle with no platform change — which is the point of §2's choice and worth stating as a benefit already earned. |
| **datafusion-python** | **not used; consumer only** | Observed at 54.0.0 against the Rust engine's 55.1.0. Version skew is tolerable precisely because it consumes the capsule protocol, not our engine. Do not let it become a dependency: two DataFusion versions in one process would be a genuine problem, and D10 already fixes DataFusion's four roles on the Rust side. |

---

## 14. Principle-alignment register (Python boundary)

Capabilities the blueprint does not claim, which would improve charter alignment. Disciplined by §G and **DM-58**: every row names a principle and the hand-written work or risk it removes. Rejections are recorded too.

### Adopt

| # | Capability | Where | DM / gate | What it removes or prevents |
|---|---|---|---|---|
| 1 | Generate the Python contract classes from `RelationSpec`, as §4.2 already does for Rust | §7 | **DM-52**, DM-23, DM-56 | Hand-writing the column list a second time — which §4.2 itself calls a governance failure — and the drift that follows. Largest structural item in the Python surface. |
| 2 | Forbid extra keys at every boundary (prefer the global hook factory) | §7 | **DM-42**, **G2** | Measured: the default **silently drops unknown keys**, converting detectable schema drift into silent data loss. |
| 3 | Consume cattrs sub-errors via `transform_error()`; never set `detailed_validation=False` | §7 | DM-47, DM-30 | Detailed validation is already on by default; the real gap is that nothing consumes the per-field sub-errors §23.2 requires be preserved. |
| 4 | Governance lint for `Any`/bare `dict`/bare `list` via `attrs.fields()` | §7 | **DM-06**, G3 | §21.5's "no `Any` fields" is currently a convention with nothing enforcing it. |
| 5 | Detect extension-type degradation via retained `ARROW:extension:name` metadata | §3 | **DM-42**, **G2**, DM-59 | Measured: loss is silent at the type level but the metadata key survives — so it is detectable at the cost of one boundary check. |
| 6 | Ship and register the ten `pse.*` `ExtensionType` classes from the platform package | §3 | **DM-44** | Each consumer writing its own forks the serialization contract. |
| 7 | Per-column loss profile in the §21.1 manifest | §3 | **G7**, DM-42 | Lets a consumer verify extension coverage rather than discover absence. |
| 8 | Never convert a nullable Arrow column to a bare `ndarray`; assert `null_count == 0` | §10 | **DM-08**, **G2** | Measured: null becomes **NaN**, collapsing the distinction §4.4 exists to protect. Silent and numerically plausible. |
| 9 | `zero_copy_only=True` as the adapter default | §10 | DM-37, DM-39 | Turns an unintended copy into an error instead of a performance mystery. |
| 10 | Verify the external-function library resolves before building the model | §4 | **G7**, DM-43, DM-30 | Measured: a missing library is a *warning*; failure surfaces later, from the solver, far from the cause. |
| 11 | `identify_inconsistent_units` instead of a bare `assert_units_consistent` | §5 | **DM-47**, §23.2 | Returns the offending components; `compile.math` exists to name the culprit. |
| 12 | Units as cross-check only — registry is the source | §5, §12 | **DM-02**, **G1** | Keeps pint from becoming a second quantity authority. |
| 13 | `SolverFactory(name).available(False)` pre-flight | §5 | DM-43, G7 | A missing solver becomes a declared capability failure, not a late crash. |
| 14 | msgspec owns the §20.2 manifest; strict, constrained, located errors | §8, §9 | DM-42, DM-47 | Gives §20.5's "unknown registry fingerprint is rejected" a real mechanism. |
| 15 | Write the attrs/cattrs ÷ msgspec division into §21.5 | §9 | **DM-56, DM-57** | Three libraries with overlapping capability and no stated division is how the same meaning gets expressed twice. |
| 16 | `IncidenceGraphInterface` and `scipy` cross-checks as standing CI parity tests | §6, §11 | **DM-53**, DM-54 | Turns available techniques into regression controls over the golden snapshots. |
| 17 | Record IDAES, `pyomo.contrib.*` and numpy/scipy versions in run and parity records | §6, §12 | **DM-31**, DM-48 | A parity failure cannot otherwise be attributed to us versus them. |
| 18 | Pin every Python library `==`, both interpreters | §1 | DM-48, DM-51 | §3.1 currently floors two and omits three entirely. |

### Evaluate

| # | Capability | Where | DM | Why not yet an adopt |
|---|---|---|---|---|
| 19 | `msgspec.json.schema()` for Python-internal wire types only | §8 | **DM-23** | Real value, real risk: §4.2 already generates authoring JSON Schema from the registry. Two generators for one contract will drift. Confine to types with no registry counterpart. |
| 20 | `pyomo.contrib.iis` as an oracle for §15.5's MIS | §6 | DM-47, DM-53 | Free cross-check via the adapter that already exists; not a replacement for the native implementation. |
| 21 | `scipy.sparse.linalg.svds` as an oracle for §15.5's SVD toolbox | §11 | DM-53, DM-39 | Also a reference for the "iterative for large" path the Arrow map found unimplemented. |
| 22 | `polynomial_degree()` assertions after lowering | §4 | DM-24, DM-53 | Cheap generated equivalence check between IR classification and Pyomo's. |
| 23 | `msgspec.Meta` constraints mirroring registry invariants | §8 | DM-07 | Only where the constraint guards the wire format; otherwise it re-expresses a rule P2 already enforces (DM-56). |
| 24 | `__arrow_ext_scalar_class__` for `pse.quantity_value` | §3 | DM-06 | Worth it only for types a human inspects interactively. |

### Reject, with reason

| # | Capability | Where | Why |
|---|---|---|---|
| 25 | scipy or numpy in production paths | §10, §11 | D11 gives native numerics ownership of execution layouts. Test-time oracles only. |
| 26 | pandas anywhere in the adapter | §13 | Loses the null/NaN distinction and adds an index concept the relational model does not have. |
| 27 | datafusion-python as a dependency | §13 | Two DataFusion versions in one process; D10 already fixes the engine's roles on the Rust side. |
| 28 | `pa.UnknownExtensionType` as the degradation mechanism | §3 | Measured not to occur on the IPC/capsule path. |
| 29 | Pyomo `Block`s mirroring relation structure | §4 | D12: the adapter builds no hierarchy; engineering names live in the source map. |
| 30 | pint as a quantity authority | §5, §12 | §8's registry is sole authority. Presence is unavoidable; authority is not. |

---

## 15. Acceptance-gate review (Python boundary)

The survey's revision-2 gate results are historical. Blueprint §21 now specifies
the following **Proposed** mechanisms. The [revision-5 review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md)
assesses the changed contracts; no current end-to-end Python/backend certification
is supplied by this map.

| Gates | Current mechanism | Remaining acceptance |
|---|---|---|
| G1 | Registry-generated contracts; pint validates the adapter's units and never defines model relations | Generated-schema/units disagreement fixtures |
| G2–G3 | Recursive ingress/egress admission, extension loss profile, mask-preserving nullable transfer and strict contract structuring | Missing registrations, nested metadata, unknown fields, null/NaN and wrong-unit negative cases |
| G4 | Solver/external-library preflight and supported numerical-policy/guarded-route checks | An actual selected solver route; constructing `Expr_if` is insufficient |
| G5 | Idempotent extension registration; immutable stream/buffer owners retain reservations until final release | Repeated import, delayed drain, cancellation and partial-ingestion fixtures |
| G6–G7 | `LinearExpression` only for proved affine-in-active-variable bodies; declared kernel/derivative bindings and versioned backend conformance | Nonlinear children in `Affine`, constant degree zero, guarded failures and genuine solver round trips |

## 16. Leverage matrix

| Blueprint requirement | API | Status |
|---|---|---|
| §21.1 tables cross as `__arrow_c_stream__` | `Table`/`RecordBatchReader.__arrow_c_stream__`; `RecordBatchReader.from_stream` | confirmed |
| §21.1 consumers: pyarrow / polars / datafusion-python | the capsule protocol is implementation-neutral | confirmed — and version skew with datafusion-python is tolerable by design |
| §4.4 Python consumers register `pyarrow.ExtensionType`; unknown consumers see storage types | `register_extension_type`; measured both paths | **confirmed exactly** `[probe]` |
| §4.3 metadata preserved into Python | schema + field metadata across IPC and capsule | **confirmed** `[probe]` |
| §21.2 step 1 units so `assert_units_consistent` passes | `pyomo.util.check_units` — **not** `pyomo.environ`; requires **pint** | **erratum ×2** — wrong module implied; contradicts §3.3 |
| §21.2 affine-in-variables lowering | `LinearExpression` | **Qualified probe:** constructor works for the measured linear example; nonlinear children take ordinary expression lowering (ADR-0047) |
| §21.2 runtime Conditional | `Expr_ifExpression` | construction observed; actual guarded writer/solver route remains a conformance requirement (ADR-0047) |
| §21.2 step 2 `KernelCall` → `ExternalFunction` | `AMPLExternalFunction` | confirmed — **but a missing library only warns** |
| §21.2 step 3 `scaling_factor` suffix | `Suffix(direction=EXPORT)` | confirmed `[probe]` |
| §21.2 step 4 `dual`/`ipopt_zL_out`/`ipopt_zU_out` | `Suffix(direction=IMPORT)` | declaration confirmed; **round-trip `[UNVERIFIED]`** — no solver installed |
| §21.3 parmest / PyROS / GDPopt / pyomo.dae / IncidenceGraphInterface | all present at 6.10.1 | confirmed `[probe]` — all but `pyomo.dae` in `pyomo.contrib` |
| §21.5 typed attrs classes, cattrs structuring | `attrs.define`, `cattrs.Converter` | confirmed available |
| §21.5 "no `Any` fields" | nothing enforces it | **erratum — unenforced** `[probe]` |
| §21.5 msgspec's role | unstated in the blueprint | **refined** — §9 adjudicates it |
| §21.5 `contract_substrate_discipline.md` | file deleted with `docs/library_ref/` | **erratum E1** |
| §3.1 Python pins | two floors, three omissions, IDAES at 2.10 | **erratum** — §1 |
| D11 borrow buffers where layouts permit | `zero_copy_only=True` succeeds only for null-free single-chunk primitives | confirmed `[probe]` |
| §3.3 "no Python-side units library" | Pyomo units *are* pint | **erratum** — reword to authority |

---

## 17. Evidence ledger

| # | Cluster | Instrument | Source | Target | Outcome |
|---|---|---|---|---|---|
| 1 | setup | PyPI | `pypi.org/pypi/{pkg}/json` | latest version + `requires_python` for all eight | all eight pinned at current release; idaes-pse classifiers 3.10–3.13 only |
| 2 | setup | uv / venv | `py314` (CPython 3.14.7) | `==`-pinned install of the 7-library main set | receipt in §1.3 |
| 3 | setup | uv / venv | `py313` (CPython 3.13.12) | main set + `idaes-pse==2.12.0` | IDAES imports cleanly; pulls pint 0.26.1, pydantic 2.13.5, sympy 1.14.0, networkx 3.6.1 |
| 4 | all | **api dump** | `apidump.py` on both environments | public API of 8 libraries → JSON | 8 dumps, 982 modules, 15 K items — receipt in §1.3 |
| 5 | P1, P2 | api | `pyarrow@25.0.1` | `ExtensionType`, PyCapsule dunders, registry functions | `__arrow_ext_serialize__`/`__arrow_ext_deserialize__`, `register_extension_type`, `UnknownExtensionType` |
| 6 | **P2** | **probe** | `probe_arrow.py` PROBE 1 | extension type through IPC, registered vs not | registered → `SemanticId`; unregistered → `FixedSizeBinaryType`, **`ARROW:extension:name` metadata retained** |
| 7 | **P1, P2** | **probe** | `probe_arrow.py` PROBE 2 | schema/field metadata across IPC and `__arrow_c_stream__` | preserved on both paths; extension type survives the capsule path |
| 8 | **P9** | **probe** | `probe_arrow.py` PROBE 8 | numpy zero-copy and nulls | zero-copy **refused** with nulls; `zero_copy_only=False` turns **null → NaN**; `-0.0` preserved |
| 9 | **P3** | **probe** | `probe_pyomo.py` PROBE 3 | `LinearExpression`, `Expr_if`, `ExternalFunction` | all confirmed; **missing external library only warns** |
| 10 | **P4** | **probe** | `probe_pyomo.py` PROBE 4 | units consistency, with and without pint | **`pyo.units` raises `DeferredImportError` without pint**; with pint, `InconsistentUnitsError` + `identify_inconsistent_units` names the culprit |
| 11 | P4 | **probe** | `probe_pyomo.py` PROBE 5 | suffixes and solver availability | suffixes declare OK; **no solver available** → round-trip `[UNVERIFIED]` |
| 12 | **P6** | **probe** | `probe_contracts.py` PROBE 6 | cattrs strictness and `Any` | **unknown keys silently dropped by default**; `forbid_extra_keys` rejects; **`Any` accepted** |
| 13 | **P7** | **probe** | `probe_contracts.py` PROBE 7 | msgspec constraints and schema | unknown field + type + `Meta(gt=0)` all rejected **with JSON-path locations**; `json.schema()` emits `additionalProperties: False` |
| 14 | P5 | probe | import checks on `py314` | parmest, PyROS, GDPopt, pyomo.dae, incidence_analysis, iis | all present; `IncidenceGraphInterface` at `pyomo.contrib.incidence_analysis.interface` |
| 15 | P10 | probe | `scipy@1.18.1` | sparse formats, csgraph, SVD | `maximum_bipartite_matching`, `structural_rank`, `svds`, `svdvals` all present |
| 16 | P11 | probe | `idaes@2.12.0` on `py313` | parity-relevant modules | `idaes.core`, `model_statistics`, `scaling`, `model_diagnostics`, `modular_properties` all import |
| 17 | P11 | PyPI | `pypi.org/pypi/idaes-pse/json` | dependency requirements | `pyomo>=6.10.1`, `pint>=0.24.1`, pydantic, sympy, networkx, matplotlib, click |
| 18 | P6 | c7 resolve | `cattrs` | strict structuring, converter configuration | `/python-attrs/cattrs` (205 snippets); `/python-attrs/attrs` (475) |
| 19 | **P6** | c7 query-docs | `/python-attrs/cattrs` | `forbid_extra_keys` / `detailed_validation` and when to enable them | **corrected this document**: detailed validation is **on by default since 22.1.0**; supplied `transform_error()` and the global `register_structure_hook_factory` idiom |

---

## 18. Open items and recommended blueprint amendments

This is a **Proposed** design disposition, not a rerun of the Python corpus. The
original API/probe sections retain their conditions. Current dependency and
interpreter authority is pyproject.toml, uv.lock and blueprint §3.1.

| Earlier item / revision-5 finding | Current disposition | Remaining acceptance |
|---|---|---|
| Unit authority and generated contracts | Blueprint §3.3/§4.2/§21.5: pint validates, registry generates contract classes; attrs/cattrs handles structured material and msgspec handles wire records | Generator/runtime contract checks; no duplicated independently editable schema |
| Exact pins, interpreter split and parity version | Blueprint §3.1/§6.14 and ADR-0003/0018/0024; IDAES 2.12.0 is the parity reference | Actual named parity gate for the relevant slice; historical absence of a solver is not a claim about the current setup |
| Deleted discipline reference | Normative Python contracts are stated in blueprint §21.5 | No dependency on the deleted library-guide file |
| Extension registration and nullable arrays | Blueprint §4.4/§21.1/§21.6, ADR-0039; platform admission invokes checks, unaware consumers have an explicit loss profile | Real bundle/query ingress, masks, mismatched extension versions and round trips |
| R4-09: `Affine` versus a linear problem | Blueprint §21.2, ADR-0047: verify degree 0/1 only for the qualified `LinearExpression` route; nonlinear children use ordinary expressions | Constant and nonlinear-child fixtures, declared numerical policy |
| R4-02/10: guarded and opaque kernels | Blueprint §18.5/§18.9/§21.2/§21.4, ADR-0043/0047; actual writer/solver support, units and derivatives are required | Excluded-branch failure tests, native/Pyomo outcome comparisons and external-function solves |
| Result suffixes and ecosystem tools | Blueprint §21.2/§21.3: record actual versions and cross-check native structures | Real dual/bound-multiplier round trips and supported `pyomo.contrib` operations; old import/construction probes do not close these |
| L7: bounded coalescing | Blueprint §21.1, ADR-0048, R-24; stream ownership remains mandatory | Representative transfer/memory measurement and independent semantic equality |

The implementation sequence and boundary fixtures are recorded in
[plan 02](../plans/02-blueprint-revision-5-contracts.md). Performance and solver
support remain unmeasured/unverified until those specific paths are exercised.
