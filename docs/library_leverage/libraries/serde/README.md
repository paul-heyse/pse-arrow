# serde (derive attributes)

Realizations considered: installed `serde` / `serde_derive` =1.0.229 (latest stable 1.0.229,
2026-07-18), `serde_json` =1.0.151.
Research date 2026-10-27. Sources: serde.rs `attr-flatten` (serde-rs.github.io master);
serde issue #745 (state via GitHub API 2026-10-27); repository `boundary_document_wire.rs`,
`pse-ids/src/document.rs`, `pse-model/src/lib.rs` at `c2f8b485a`.

## Types

### serde/version-header-envelope
- **Need it serves:** a required, checked-first version header on a JSON document whose body is an
  existing type, without colliding with body fields and while refusing unknown body fields.
- **Recognizable code shapes:** `struct Versioned<T> { document_version: Version<N>, #[serde(flatten)]
  body: T }` plus a hand `DeserializeSeed`/`MapAccess` adapter; a separate raw-bytes header scan.
- **Mechanisms and APIs:** `#[serde(flatten)]`; `#[serde(tag = "...")]` (internally tagged enum);
  `#[serde(deny_unknown_fields)]`; `#[serde(deserialize_with)]`; `DeserializeSeed`.
- **Obligations discharged:** field merging on serialize; tag dispatch for string-valued tags.
- **Residual caller obligations:**
  - `flatten` and `deny_unknown_fields` cannot be combined ("Neither the outer nor inner flattened
    struct should use deny_unknown_fields", serde.rs attr-flatten) — strictness needs a custom
    deserializer.
  - Internally tagged enums accept only string-like tag values; integer tags are an open request
    (serde #745 "Allow integer tags for internally tagged enums", open, updated 2026-02-25). A numeric
    `document_version: 1` cannot be the serde tag.
  - Header-first ordering and duplicate-key refusal are not serde features.
- **Semantic conditions:** flatten and internal tagging buffer into private `Content`, which changes
  number handling and error paths compared with direct deserialization.
- **Version / build conditions:** unchanged across 1.0.2xx.
- **Limits and negative results:** no derive-only route to "integer header + strict flattened body".
- **Evidence:** Interface-checked: serde.rs attr-flatten text; GitHub issue state.

### serde/selective-ordered-identity-framing
- **Need it serves:** derive a canonical byte framing (for content identity) from a type's
  declaration so that adding a field cannot silently escape identity, while allowing exclusions,
  canonical spellings and ordering.
- **Recognizable code shapes:** a custom `serde::Serializer` that hashes the data model beside many
  hand-written `fn frame(&self, hasher)` impls enumerating fields.
- **Mechanisms and APIs:** `#[serde(skip)]`, `#[serde(skip_serializing_if)]`, `#[serde(rename)]`,
  `#[serde(serialize_with)]`, `#[serde(getter)]` (remote), `#[serde(into = "...")]` /
  `#[serde(remote = "...")]` shadow types; field order = declaration order.
- **Obligations discharged:** exhaustive field enumeration from the declaration; ordering by
  declaration.
- **Residual caller obligations:** one `Serialize` impl per type is shared by **every** serializer:
  identity-only skips or spellings change the JSON wire too, unless the framing goes through a
  shadow type (`into`/`remote`) or a separate trait. Canonicalization (sorted sets, graph
  canonical forms, first-use ordinals) remains hand code.
- **Semantic conditions:** a serde-model framer frames field names and type tags if it chooses to;
  a hand framer that frames bare values produces different bytes for the same value, so switching
  mechanisms changes identities unless the new path emulates the old byte layout exactly.
- **Version / build conditions:** n/a.
- **Limits and negative results:** serde has no "identity-only" attribute namespace; no derive emits a
  non-serde trait.
- **Evidence:** Interface-checked: serde attribute docs (well-known stable surface); repository
  framers at `c2f8b485a`.

## Current applicability

### [CBO-042](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-042) — Document wire envelopes and directional schema surgery
- **Types:** serde/version-header-envelope.
- **Realization:** installed 1.0.229.
- **Conclusion:** ruled out here — the two serde routes that would replace the hand header
  deserializer (internal tag; flatten + `deny_unknown_fields`) are respectively unsupported for
  integer tags and documented as incompatible; the streaming `DeserializeSeed` adapter is the
  serde-sanctioned extension point for this contract.
- **What it would discharge / what remains local:** nothing beyond what is used today.
- **Semantic gaps against the CBO's preservation constraints:** a string tag would change the
  existing flat wires (compatibility constraint).
- **Unsettled premises and what would settle them:** if serde #745 lands, an integer internal tag
  would still not provide header-first checking or duplicate refusal.
- **Evidence:** Interface-checked. Application Proposed (negative).

### [CBO-039](../../../library_leverage/codebase/authoring-and-identity.md#cbo-039) — Hand-written identity framing beside the serde-derived framer
- **Types:** serde/selective-ordered-identity-framing.
- **Realization:** installed 1.0.229.
- **Conclusion:** ruled out here for the hand-framed types as they stand; unsettled for a derive.
  **Premise correction:** the hand impls implement a different trait and byte layout —
  `pse_model::SemanticFrame` ("stable typed value framing, independent of serde"), which frames bare
  values (`Option` as bool + value, `Vec` as u64 length + items, strings without type tags) — while
  the serde `Framer` (`pse-ids/src/document.rs`) frames type tags (`"bool"`, `"i64"`, …) and serde
  field/variant names. Moving a `SemanticFrame` type to the serde framer would change every identity
  it contributes to (golden vectors). Several framed types (for example `pse-quantity` `Scheme`) do
  not derive serde at all, and their variant tags (`"named"`, `"difference"`) differ from Rust names.
- **What it would discharge / what remains local:** with serde: nothing without an identity
  migration. A local `#[derive(SemanticFrame)]` proc-macro (field-order framing, `#[frame(skip)]`,
  `#[frame(tag = "...")]`) could reproduce today's bytes for full-field frames and keep selective and
  canonicalized ones hand-written — no library supplies that derive.
- **Prerequisites, configuration or extension:** classification of each impl as full-field vs
  selective vs canonical (still open, as the CBO notes).
- **Semantic gaps against the CBO's preservation constraints:** frozen bytes would break on the serde
  route.
- **Unsettled premises and what would settle them:** a reflection library (for example `facet`, not
  researched in this pass) could drive a framer from a declaration independent of serde; settle with
  a focused assessment if the derive route is considered.
- **Evidence:** Interface-checked (repository sources). Application Proposed.
