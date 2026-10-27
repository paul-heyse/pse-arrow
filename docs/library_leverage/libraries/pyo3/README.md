# PyO3 (and pyo3-introspection)

Realizations considered: installed `pyo3` / `pyo3-macros` =0.29.2 with `experimental-inspect`,
`pyo3-introspection` =0.29.2; upgrade routes 0.29.3 (2026-09-30, stub fixes) and 0.30.0/0.30.1
(2026-10-13/14: `#[pyclass]` no longer emits `FromPyObject` by default and
`skip_from_py_object` is deprecated; stubs emit `__all__`, re-export submodules, PEP 800
`disjoint_base`).
Research date 2026-10-27. Sources: Context7 `/websites/pyo3_rs` (class guide v0.29.3 parameter table,
`#[pyo3(get)]` requirements, FAQ "Ctrl-C", `Python::check_signals` docs); pyo3-introspection 0.29.2
source (`src/stubs.rs`, `src/model.rs`); pyo3-macros-backend 0.29.2 `src/pyclass.rs`; upstream
`CHANGELOG.md` (main, 0.29.2→0.30.1).

## Types

### pyo3/pyclass-as-public-type
- **Need it serves:** expose a native object directly as the public Python type (correct module,
  name, attributes, docs, stubs) rather than wrapping it in a Python class that forwards each
  accessor.
- **Recognizable code shapes:** Python classes holding `_handle` and defining one property/method per
  native accessor (`return self._handle.x`); a stub file for the native module plus hand-maintained
  wrapper signatures and docstrings.
- **Mechanisms and APIs:** `#[pyclass(frozen, module = "pkg.public", name = "Public")]` (the class
  reports that module/name; it still lives in the extension module's namespace and must be
  re-exported from the public module); `get_all` / `#[pyo3(get, name = ...)]` for plain fields
  (`&T: IntoPyObject` or `T: IntoPyObject + Clone`; non-`Py<T>` fields are cloned per access);
  `#[getter]` methods for computed values; `subclass` / `extends` for inheritance; `eq`, `ord`,
  `hash`, `str`; Rust doc comments become `__doc__`; `experimental-inspect` +
  `pyo3_introspection::{introspect_cdylib, module_stub_files}` produce stubs that **include
  docstrings** (`stubs.rs` `push_docstring`).
- **Obligations discharged:** attribute exposure, naming, `__module__`, runtime docs, stub signatures
  (and docstrings if kept).
- **Residual caller obligations:** re-export into the public module; behavior that is genuinely
  Python-side (iteration helpers, construction of other public types, tuple conversion) needs Rust
  `#[pymethods]` or remains in Python; a Python-side subclass of a `subclass` pyclass gets every
  accessor for free, but objects created by Rust are instances of the base class, not the Python
  subclass.
- **Semantic conditions:** getters that return pyclass values clone unless the field is `Py<T>`;
  frozen + `Sync` classes allow `Py::get` without attaching.
- **Version / build conditions:** `experimental-inspect` is feature-gated and evolving (0.29.3 and
  0.30 change stub output); 0.30 changes `FromPyObject` defaults.
- **Limits and negative results:** `module=` does not move the class into another module or alter
  stub placement (stubs follow `#[pymodule]` structure; not established beyond that).
- **Evidence:** Interface-checked: pyclass parameter table (pyo3.rs v0.29.3), `get` requirements,
  introspection 0.29.2 `stubs.rs`.

### pyo3/bulk-return-crossing
- **Need it serves:** return a whole collection in one boundary crossing.
- **Recognizable code shapes:** `tuple(handle.item(i) for i in range(handle.count))`.
- **Mechanisms and APIs:** return `Vec<T>` (→ `list`) or build `PyTuple::new(py, iter)`; return
  `Vec<Py<T>>` to preserve identity of shared objects.
- **Obligations discharged:** one call, one allocation of the container.
- **Residual caller obligations:** none beyond choosing tuple vs list.
- **Evidence:** Interface-checked: PyO3 conversion tables (well-known stable API).

### pyo3/interruptible-detached-wait
- **Need it serves:** keep a blocking native call interruptible by Ctrl-C while the GIL is released.
- **Recognizable code shapes:** loops of `py.detach(|| recv_timeout(Δ))` followed by
  `py.check_signals()`.
- **Mechanisms and APIs:** `Python::detach` (formerly `allow_threads`); `Python::check_signals`
  (calls `PyErr_CheckSignals`, runs Python handlers, **does nothing on non-main threads or
  non-main interpreters**).
- **Obligations discharged:** releasing the GIL; running pending Python signal handlers.
- **Residual caller obligations:** the wait/poll loop, its interval, cancel-and-drain ordering, cause
  attachment. PyO3 offers no signal-woken wait primitive (FAQ recommends calling `check_signals`
  regularly).
- **Semantic conditions:** detection latency is the poll interval; a signal arriving while attached
  is handled at the next check.
- **Limits and negative results:** no wakeable form of `check_signals` (search scope: PyO3 guide
  FAQ/class pages via Context7, `Python` API docs, CHANGELOG 0.29.2→0.30.1).
- **Evidence:** Interface-checked: PyO3 FAQ and `check_signals` docs; re-confirmed through
  Context7 `/pyo3/pyo3` on 2026-10-27 (the FAQ and guide recommend calling `check_signals`
  regularly; there is still no wakeable form).

### pyo3/per-type-python-handle-cache
- **Need it serves:** look up a Python class or function (for example a generated model type)
  once per interpreter and reuse it on every boundary crossing, instead of importing it or
  resolving it through a callback on each call.
- **Recognizable code shapes:** `py.import("pkg.mod")?.getattr("Type")?` inside a conversion that
  runs per value; a Python-side registry dict consulted from Rust; a generic helper that resolves
  the target class from a type parameter at every call.
- **Mechanisms and APIs:** `pyo3::sync::PyOnceLock::import(py, module, attr)` initializes and
  caches an imported attribute once (`src/sync/once_lock.rs`); `PyOnceLock<Py<PyType>>` with
  `get_or_try_init` for computed handles.
- **Obligations discharged:** one-time, thread-safe import under the interpreter; no repeated
  module lookup.
- **Residual caller obligations:** one cache per target type; error mapping when the import fails.
- **Semantic conditions:** a `static` inside a generic function is shared by every
  monomorphization, so a per-type cache cannot be written once in a generic helper; it must be
  emitted per type (for example by a code generator) or keyed explicitly.
- **Version / build conditions:** present in the installed 0.29.2.
- **Evidence:** Interface-checked: installed pyo3 0.29.2 `src/sync/once_lock.rs`.

## Current applicability

### [CBO-044](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-044) — Python wrapper layer forwarding native handles accessor by accessor
- **Types:** pyo3/pyclass-as-public-type; pyo3/bulk-return-crossing.
- **Realization:** installed 0.29.2 (all listed attributes exist at the pin).
- **Conclusion:** fits with conditions — making the pyclass the public type (with `module=`/`name=`,
  runtime docs from Rust doc comments, re-export through the existing gate module) removes the
  forwarding properties; the conditions are that the gate rule is satisfied by re-export rather than
  wrapping, and the genuinely behavioral members (`TableStream` iteration, `RunResult` construction,
  identity tuple conversion, wrapping returned handles into public classes) move into `#[pymethods]`
  or stay as small Python functions.
- **What it would discharge / what remains local:** ~405 `self._handle.` forwards across 13 modules
  (count confirmed: `_modeling` 136, `_runs` 88, `_workflow` 48, `_project_operations_glue` 42,
  `diagnostics` 26, `_study_workbench` 18, `_artifacts` 11, `_strategies` 10, `_research` 8,
  `_inspection` 6, `_physical` 5, `_analyses` 4, `_project_bundle` 3) and their hand docstrings;
  per-index loops (e.g. `ModelingNativeAnalysis.attempts`) become one tuple-returning method.
  Remains: public naming policy, behavioral helpers.
- **Prerequisites, configuration or extension:** **premise sharpening:** the stub generator
  deliberately strips docstrings (`xtask/src/codegen/python_stubs/validate.rs` "Stub files contain
  signatures only, as required by the repository's Python stub contract"), so "docstrings kept in
  step by hand" refers to Python wrapper docs vs Rust docs, and introspection could carry them if
  that stub contract changed. 45 classes already use `module = "pse._native"`; the change is the
  module/name values plus re-export.
- **Semantic gaps against the CBO's preservation constraints:** the import gate (`_build.py` sole
  importer + ast-grep rule) survives if public modules import from `_build`; a public surface
  separate from native names is expressible with `name=`; `isinstance` checks against wrapper
  classes change.
- **Unsettled premises and what would settle them:** whether wrappers carry policy beyond
  forwarding in the other 12 modules (only samples read); whether PyO3 0.30 stub changes
  (`__all__`, submodule re-export) are needed for a clean public-module stub.
- **Evidence:** Interface-checked. Application Proposed.

**Reviewer correction (2026-10-27).** 41 wrapper classes hold `_handle`, 6 of them
Python-constructed (for example `Runtime`, which composes generated host-operation mixins), and
there are 22 cross-wrapper `._handle` unwraps. `#[pyclass(subclass)]` therefore serves the
user-constructed owners, while Rust-returned objects stay base-class instances. The comparative
review recommends the route but requires an ADR and design review; adopting the PyO3 0.30 stub
features would be a family-major ADR. See the [comparative review](../../reviews/declarations-registry-and-python-boundary.md#cbo-044).

### [CBO-041](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-041) — Boundary documents transcoded through JSON, with checks in both languages
- **Types:** pyo3/per-type-python-handle-cache.
- **Realization:** installed 0.29.2; `PyOnceLock` is already used in `pse-py/src/{enums,identities}.rs`.
- **Conclusion:** fits as part of a trusted Rust→Python decode: generated per-type `PyOnceLock`
  handles resolve each document class once.
- **Semantic gaps:** must be emitted per type by the generator, not as one generic helper.
- **Evidence:** Interface-checked; Proposed. Recorded from the comparative review, which found that
  Rust→Python decode re-runs the untrusted-input traversal, with a callback into Rust, on bytes
  Rust just wrote. See the [comparative review](../../reviews/declarations-registry-and-python-boundary.md#cbo-041).

### [CBO-017](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-017) — Signal-aware blocking waits at the Python boundary
- **Types:** pyo3/interruptible-detached-wait.
- **Realization:** installed 0.29.2.
- **Conclusion:** fits as the primitive already used; no PyO3 capability removes polling — the
  library route is consolidation onto one helper, optionally combined with a CPython wakeup fd (see
  cpython-signal-wakeup-fd.md).
- **What it would discharge / what remains local:** nothing new from PyO3; one local helper would
  own interval, cancel→drain→raise ordering and cause attachment. **Premise sharpening:** 10
  `check_signals` sites in `pse-py` (workflow.rs 7, diagnostics.rs 2, inspection/stream.rs 1) with
  intervals of 100 ms (`blocking_native_call`, task-dump), 20 ms (`thread::sleep` capture join,
  `wait_ready` command wait) and a single pre-check (stream `next_batch`); orderings differ
  (pre-check vs post-join check; `request_stop` vs `cancel`).
- **Evidence:** Interface-checked. Application Proposed.
