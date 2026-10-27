# pythonize

Realizations considered: upgrade route `pythonize` 0.29.0 (released 2026-06-12; depends on
`pyo3 ^0.29`, so compatible with the 0.29.2 pin; not in the lockfile). 0.28.0 source present in the
local registry (pyo3 0.28) was used for API inspection; 0.29.0 dependency list from crates.io.
Research date 2026-10-27. Sources: pythonize 0.28.0 source (`src/ser.rs`, `src/de.rs`); crates.io API
(0.29.0 dependencies and release dates).

## Types

### pythonize/serde-to-python-objects
- **Need it serves:** move serde-modelled Rust values to and from Python builtins (dict/list/str/
  int/float/None) without serializing to JSON text in between.
- **Recognizable code shapes:** Rust `serde_json::to_vec` → Python `json/msgspec decode`; Python
  `encode_json` → Rust `serde_json::from_slice` for in-process calls.
- **Mechanisms and APIs:** `pythonize(py, &value) -> Bound<PyAny>`; `pythonize_custom::<P, _>` with
  `PythonizeTypes` (choose mapping, named-mapping and list types: `PythonizeMappingType`,
  `PythonizeNamedMappingType`, `PythonizeListType`); `depythonize::<T>(&Bound<PyAny>)` drives serde
  `Deserialize` from Python objects (mapping/sequence protocols), so `deny_unknown_fields` and serde
  validation apply on the Rust side.
- **Obligations discharged:** the text encode/parse hop and its allocation; serde attribute
  semantics on both directions.
- **Residual caller obligations:** typed Python objects still need a typed conversion
  (`msgspec.convert(obj, Type)`) after `pythonize` — a second object traversal instead of a JSON
  decode; inputs from msgspec Structs need `msgspec.to_builtins` first (Structs are not mappings);
  byte-extent limits must become object/size limits; non-finite floats cross as `float('nan')` /
  `f64::NAN` instead of JSON `null`, so finite refusal must be explicit on both sides.
- **Semantic conditions:** integer width and float exactness follow Python objects (no decimal text
  round trip); dict key order is insertion order; duplicate keys cannot occur (a JSON-level duplicate
  check has nothing to check); serde `is_human_readable` is `true`.
- **Version / build conditions:** pythonize releases track PyO3 minors (0.29 ↔ pyo3 0.29); a PyO3
  bump to 0.30 needs the matching pythonize.
- **Limits and negative results:** produces builtins only; constructing msgspec Structs directly
  via `PythonizeNamedMappingType` is possible in principle but bypasses validation.
- **Evidence:** Interface-checked: pythonize 0.28.0 source (API names and traits); crates.io 0.29.0
  dependency metadata (`pyo3 ^0.29`).

## Current applicability

### [CBO-041](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-041) — Boundary documents transcoded through JSON, with checks in both languages
- **Types:** pythonize/serde-to-python-objects.
- **Realization:** upgrade route 0.29.0 (new dependency; moves with the PyO3 family pin).
- **Conclusion:** fits with conditions for in-process paths only — it removes the text hop but not
  the typed conversion, and it removes the JSON wire that §21.5 names as the language-neutral
  contract and that the byte-extent allowance and NaN→null refusal currently rely on.
- **What it would discharge / what remains local:** discharges `serde_json::to_vec` + Python JSON
  decode (Rust→Python) and `encode_json` + `from_slice` (Python→Rust) where no wire is persisted;
  local: finite-value refusal, size allowances, version header (no raw bytes to scan), identity
  string forms.
- **Prerequisites, configuration or extension:** explicit finite checks (Rust newtype or validator);
  allowance by object count/size; deciding which crossings are "in-process only".
- **Semantic gaps against the CBO's preservation constraints:** non-finite refusal changes mechanism
  (null-by-encoding → explicit check); "exact wire form of identities" still holds if identities
  serialize as strings; streaming allowance limits must be redefined.
- **Unsettled premises and what would settle them:** whether the JSON hop is a measurable cost at all
  (CBO: payloads modest, nothing measured) — a micro-measurement would settle priority.
- **Evidence:** Interface-checked. Application Proposed.
