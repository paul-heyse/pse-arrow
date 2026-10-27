# cattrs

Realizations considered: installed `cattrs` ==26.2.0 (`.venv`); upgrade route 26.2.1 (PyPI latest).
Research date 2026-10-27. Sources: installed `cattrs/preconf/msgspec.py` (26.2.0).

## Types

### cattrs/msgspec-preconf-passthrough
- **Need it serves:** keep attrs classes as the model while delegating encoding to msgspec with
  minimal Python-level work.
- **Recognizable code shapes:** a cattrs `Converter` unstructuring attrs rows to dicts that are then
  fed to msgspec.
- **Mechanisms and APIs:** `cattrs.preconf.msgspec.make_converter()` → `MsgspecJsonConverter`
  (`dumps`, `loads`, `get_dumps_hook`); `configure_converter` registers `Struct` ↔ `to_builtins` /
  `convert`; `configure_passthroughs` returns `msgspec.to_builtins` (i.e. lets msgspec encode attrs
  instances natively) when every field's hook is identity — but falls back to cattrs generation when
  attrs fields are private (`_x`, which msgspec skips).
- **Obligations discharged:** routing attrs→JSON through msgspec's native attrs encoding when
  possible.
- **Residual caller obligations:** custom unstructure hooks (identity hex/prefixed forms) disable the
  passthrough for those fields; strictness on structure is cattrs' (`forbid_extra_keys`) not msgspec's.
- **Semantic conditions:** passthrough encodes non-finite floats as `null` (msgspec behavior).
- **Version / build conditions:** passthrough for dataclasses with private attributes since 25.1.0.
- **Limits and negative results:** does not unify the two class systems; it is a codec bridge.
- **Evidence:** Interface-checked: installed `cattrs/preconf/msgspec.py` 26.2.0.

## Current applicability

### [CBO-043](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-043) — Two Python model systems and a custom schema-to-msgspec emitter
- **Types:** cattrs/msgspec-preconf-passthrough.
- **Realization:** installed 26.2.0.
- **Conclusion:** fits with conditions but only narrows the codec — identity types need hooks, so the
  passthrough would not engage for rows carrying `SemanticId`/`ContentHash` unless those types gain
  msgspec-native encodings (`enc_hook`); it does not address the two-model-systems question.
- **What it would discharge / what remains local:** some unstructure code; local: identity hooks,
  finite check, header injection.
- **Evidence:** Interface-checked. Application Proposed.

### [CBO-041](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-041) — Boundary documents transcoded through JSON, with checks in both languages
- **Types:** cattrs/msgspec-preconf-passthrough.
- **Conclusion:** subsumed by `msgspec.convert(row, Doc, from_attributes=True)` (msgspec.md), which
  removes the cattrs hop entirely for the two edit documents.
- **Evidence:** Interface-checked. Application Proposed.
