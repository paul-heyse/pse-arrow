# schemars

Realizations considered: installed `schemars` =1.2.2 / `schemars_derive` 1.2.2; upgrade route 1.2.3
(released 2026-10-12: fixes for arbitrary-precision numbers and `unused_qualifications`; puts the tag
of internally-tagged enums first in `properties`).
Research date 2026-10-27. Sources: schemars 1.2.2 source (`src/generate.rs`, `src/_private/mod.rs`);
upstream `CHANGELOG.md` (master, 2026-10-27); Context7 `/gresau/schemars` (generating docs).

## Types

### schemars/directional-contract-schemas
- **Need it serves:** publish separate JSON Schemas for what a type accepts (deserialize) and what it
  emits (serialize), where serde attributes (`skip_serializing_if`, `default`, `rename`
  `serialize`/`deserialize` split, `alias`, `try_from`/`into`) make the two differ.
- **Recognizable code shapes:** a generator producing input and output catalogues; a post-pass that
  compares definitions by name across the two and renames the differing output ones
  (`FooOutput`), propagating the rename to referrers.
- **Mechanisms and APIs:** `SchemaSettings::{draft2020_12, ...}` then `.for_deserialize()` /
  `.for_serialize()` (field `contract: Contract::{Deserialize, Serialize}`, `#[non_exhaustive]`);
  `into_generator()`, `into_root_schema_for::<T>()`, `root_schema_for`, `subschema_for`,
  `take_definitions`; `SchemaGenerator::contract()`. Internally each definition is keyed by
  `SchemaUid(schema_id, contract)`; a name collision within one generator gets a numeric suffix
  (`Foo2`, `Foo3`).
- **Obligations discharged:** the per-direction schema shape of every serde-derived type.
- **Residual caller obligations:** naming across directions. A generator has one contract at a time;
  there is no public setter that switches contract mid-generator, and even within one generator
  identical opposite-contract schemas are **not** deduplicated (source TODO in
  `insert_new_subschema_for`: "If we've already added a schema for T with the 'opposite' contract,
  then check whether the new schema is identical. If so, re-use the original"). Merging two
  directional catalogues into one namespace — sharing identical definitions and giving differing
  ones stable, meaningful names — remains caller work.
- **Semantic conditions:** suffix names are positional (`2`, `3`), not semantic, and depend on
  traversal order.
- **Version / build conditions:** behavior unchanged in the 1.2.3 changelog.
- **Limits and negative results:** no built-in "shared vs directional" naming; no cross-generator
  dedupe.
- **Evidence:** Interface-checked: schemars 1.2.2 `src/generate.rs` (`Contract`, `SchemaUid`,
  `find_ref` suffixing, TODO at `insert_new_subschema_for`); Context7 generating docs.

### schemars/flatten-and-closed-object-policy
- **Need it serves:** describe a wrapper whose own field is merged into an inner object
  (`#[serde(flatten)]`) while keeping a closed (unknown-field-refusing) object contract.
- **Recognizable code shapes:** hand `impl JsonSchema` for an envelope that injects a header property
  into the inner schema, recurses into `oneOf`/`anyOf`/`allOf` alternatives, and rewrites `examples`.
- **Mechanisms and APIs:** derive with `#[serde(flatten)]` uses `_private::json_schema_for_flatten`,
  which deliberately removes `additionalProperties: false` / `unevaluatedProperties: false` from the
  flattened schema (so the merged object can carry the outer fields); container attributes
  `#[schemars(transform = path)]` (applied after all other attributes since 1.2.1) and
  `#[schemars(extend("key" = value))]` can post-process the merged schema;
  `schemars::transform::{Transform, transform_subschemas, RecursiveTransform}`.
- **Obligations discharged:** property merging for flattened structs and (since 1.1.0) more accurate
  flattened enums.
- **Residual caller obligations:** re-imposing a closed policy after flatten (for 2020-12,
  `unevaluatedProperties: false` at the merged level works through composition), propagating a header
  into each enum alternative, and fixing examples — expressible as a `transform` function, but still
  local logic.
- **Semantic conditions:** the schema must agree with the actual deserializer: serde's derived
  flatten does not support `deny_unknown_fields`, so a closed schema is only truthful when a custom
  deserializer enforces it.
- **Version / build conditions:** transform ordering changed in 1.2.1 (now last).
- **Limits and negative results:** no attribute that preserves an inner type's unknown-field policy
  across flatten.
- **Evidence:** Interface-checked: schemars 1.2.2 `src/_private/mod.rs` (`json_schema_for_flatten`,
  `AllowUnknownProperties`, `flatten`); CHANGELOG 1.1.0 and 1.2.1 entries.

## Current applicability

### [CBO-042](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-042) — Document wire envelopes and directional schema surgery
- **Types:** schemars/directional-contract-schemas; schemars/flatten-and-closed-object-policy.
- **Realization:** installed 1.2.2 (1.2.3 changes nothing relevant; its tag-first ordering is
  neutralized by the repository's `sort_all_objects`, though a bump is still a regeneration).
- **Conclusion:** ruled out as a remover of the renaming pass — **premise correction:** the boundary
  catalogue already uses contract modes (`boundary_documents.rs` `schema()` selects
  `for_deserialize`/`for_serialize` per direction); the renaming pass exists because schemars has no
  cross-contract dedupe or semantic naming, which is exactly what `directional_names`/`rename_output`
  supply. Fits with conditions only as a host for the envelope surgery: the `Versioned` hand
  `JsonSchema` could become a derive with `#[serde(flatten)]` plus a `#[schemars(transform = ...)]`
  that re-closes the object, adds the header to alternatives and fixes examples — moving, not
  deleting, the logic.
- **What it would discharge / what remains local:** discharges the hand `schema_name`/`schema_id`
  and property merge; local: closed-policy reimposition, alternative header injection, examples,
  output naming.
- **Prerequisites, configuration or extension:** a transform function; the streaming header
  deserializer stays (see serde.md).
- **Semantic gaps against the CBO's preservation constraints:** valid examples and domain `version`
  fields are preserved either way; a derive-based schema must still match the custom deserializer.
- **Unsettled premises and what would settle them:** whether an upstream contribution (dedupe of
  identical opposite-contract schemas, the TODO above) would cover the "equal definitions stay
  shared" half; the semantic `Output` naming would still be local.
- **Evidence:** Interface-checked (sources above; repository `boundary_documents.rs` and
  `codegen/documents.rs` at `c2f8b485a`). Application Proposed.

### [CBO-041](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-041) — Boundary documents transcoded through JSON, with checks in both languages
- **Types:** schemars/directional-contract-schemas (schema is the cross-language contract).
- **Realization:** installed 1.2.2.
- **Conclusion:** not a mechanism for the runtime header check; it is relevant only in that the
  published schema already declares `document_version` as a required `const`, which a schema-driven
  Python emitter can turn into a typed field (see msgspec.md).
- **Evidence:** Interface-checked (repository schema generation). Application Proposed.
