# datamodel-code-generator

Realizations considered: upgrade route `datamodel-code-generator` 0.86.0 (released 2026-10-18;
0.83.0 2026-09-24, 0.84.0 2026-10-15, 0.85.0 2026-10-16 — a fast release cadence). Not installed.
Research date 2026-10-27. Sources: Context7 `/datamodel-code-generator/datamodel-code-generator`
(docs: `cli-reference/model-customization.md`, `template-customization.md`, `faq.md`,
`output-model-types.md`; source `model/msgspec.py`); PyPI JSON API.

## Types

### datamodel-code-generator/jsonschema-to-msgspec-structs
- **Need it serves:** generate typed Python msgspec Structs from published JSON Schema instead of a
  bespoke schema-to-Python emitter.
- **Recognizable code shapes:** a hand-written JSON-Schema walker that string-builds Python classes,
  enums, unions and constraints.
- **Mechanisms and APIs:** `--output-model-type msgspec.Struct`; `--extra-fields forbid` →
  `forbid_unknown_fields=True`; `allow_mutation: False` → `frozen=True`; model-level `kw_only=True`;
  `--use-msgspec-tagged-unions` (every discriminated-union variant gets `tag_field`/`tag`); default
  0.83.0 form tags only single-literal discriminators and keeps others as `Literal` fields;
  `--use-subclass-enum`, `--use-msgspec-nullable-enum-alias`; `--use-annotated` for `Meta`
  constraints; `--disable-timestamp`, `--keep-model-order` (stable order respecting dependencies),
  `--formatters` (explicit or none); `--custom-template-dir` + `--extra-template-data` (Jinja2
  templates) for class keywords and extra members.
- **Obligations discharged:** schema walking, `$ref`/`$defs` resolution, union/enum/constraint
  mapping, ordering.
- **Residual caller obligations:** byte determinism across tool versions (output form is versioned —
  e.g. tagged-union behavior changed around 0.83.0; formatters must be pinned or disabled); custom
  type mappings for identity strings; any repository-specific class attributes or helper hooks
  (templates).
- **Semantic conditions:** with tagged unions the discriminator is neither a field nor a constructor
  argument, and a variant decoded alone accepts a payload without the tag (documented); `Literal`
  form instead keeps the field but breaks union decoding of untagged variants.
- **Version / build conditions:** Python-side tool; output pinned only by pinning the tool and its
  formatter; frequent minor releases change output.
- **Limits and negative results:** no built-in semantic naming of serialize vs deserialize variants;
  integer tags only if the schema discriminator is an integer const (not checked).
- **Evidence:** Interface-checked: Context7 docs and `model/msgspec.py` `CONFIG_MAPPING`.

## Current applicability

### [CBO-043](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-043) — Two Python model systems and a custom schema-to-msgspec emitter
- **Types:** datamodel-code-generator/jsonschema-to-msgspec-structs.
- **Realization:** upgrade route 0.86.0 (new dev/codegen dependency).
- **Conclusion:** unsettled — the generic mapping (forbid unknown fields, frozen, kw_only, tagged
  unions, stable model order, no timestamp) now exists, but whether it can reproduce the repository's
  contract (version-header metadata `_pse_required_version*`, identity value types, the
  `Output` directional names, imports from `pse.contracts.values`, and byte determinism under
  `codegen-check`) through templates and type mappings is not established; ADR-0116's
  byte-determinism reason must be re-tested against `--disable-timestamp --keep-model-order` with
  formatters disabled.
- **What it would discharge / what remains local:** the ~800-line `Emitter` schema walk; local:
  templates carrying repository hooks, directional naming (schema post-pass), header metadata.
- **Prerequisites, configuration or extension:** pinned tool + template directory; a codegen step
  invoking a Python tool from the Rust generator pipeline (adds a toolchain dependency to codegen).
- **Semantic gaps against the CBO's preservation constraints:** byte determinism across tool bumps
  (every bump is a regeneration); tagged-union semantics differ from the current required-header
  check (see msgspec.md).
- **Unsettled premises and what would settle them:** probe P3 — run 0.86.0 over the published
  `docs/generated` (or equivalent) schemas with the flags above twice and diff against the current
  emitter output.
- **Evidence:** Interface-checked (docs). Application Proposed.
