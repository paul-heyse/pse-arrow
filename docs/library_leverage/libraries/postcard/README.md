# postcard

Realizations considered: upgrade route `postcard` 1.1.3 (released 2025-07-24; latest stable on
crates.io at 2026-10-27; not in the workspace lockfile, source present in the local cargo registry
from another project). No installed realization.
Research date 2026-10-27. Sources: postcard 1.1.3 source (`src/de/mod.rs`, `src/de/deserializer.rs`);
arrow-schema 59.3.0 `src/field.rs` tests (`assert_binary_serde_round_trip` uses postcard);
crates.io API.

## Types

### postcard/embedded-serde-blob
- **Need it serves:** ship a large, build-time-resolved data product inside a binary as one compact
  byte blob (`include_bytes!`) and rebuild owned runtime values from it at first use, instead of
  emitting the data as Rust source.
- **Recognizable code shapes:** generated Rust files of tens of thousands of lines consisting of
  struct literals and `vec.push(...)` sequences; generator comments about compiler memory; a
  generated `hydrate()` that rebuilds owned values; the same products also emitted as JSON lines.
- **Mechanisms and APIs:** serde `Serialize`/`Deserialize` derives on the product types;
  `postcard::to_stdvec` / `to_allocvec` at generation time; `postcard::from_bytes::<T>(&'a [u8])`
  at runtime, where `T: Deserialize<'a>`. With a `'static` input (from `include_bytes!`), `&str` and
  `&[u8]` fields can borrow from the blob (serde implicit borrowing), so no leak or copy is needed for
  string-literal-like fields. Varint integers, length-prefixed sequences/strings, enum variants as
  varint indices; the 1.x wire format is specified and stable.
- **Obligations discharged:** a compact, specified binary encoding and decoding of any serde data
  model value that uses only compile-time-known shapes; zero-copy borrowing of strings/bytes.
- **Residual caller obligations:** serde derives on every product type (and its transitive members);
  deterministic iteration order of every serialized map/set (postcard writes in iteration order);
  validation of reconstructed invariants (postcard checks structure, not domain rules); version
  binding between blob and the types that read it (a type change silently changes the layout —
  regenerate in the same change).
- **Semantic conditions:**
  - **Not self-describing.** `deserialize_any` and `deserialize_ignored_any` are refused ("We wont
    ever support this", `src/de/deserializer.rs`). Therefore `serde_json::Value`, `#[serde(flatten)]`,
    `#[serde(untagged)]`, internally/adjacently tagged enums and any `Deserialize` impl that calls
    `deserialize_any` cannot be decoded.
  - `is_human_readable()` is `false`: types whose serde impl branches on it encode differently than
    in JSON.
  - Determinism is exactly the determinism of the serde event stream: `BTreeMap`/`Vec` are stable,
    `HashMap`/`HashSet` are not (random iteration order per process).
  - Field order is declaration order; no field names are written (renaming a field does not change
    bytes, reordering does).
  - **Stale blobs can decode into the wrong meaning (added 2026-10-27).** Decoding is positional, so
    if two fields of the same type are swapped in the reading type but the blob was not
    regenerated, `from_bytes` succeeds and returns the values in each other's fields. Mitigate with
    a shape digest in a blob header that the generator writes and the reader recomputes from the
    same type description (both sides derived, not hand-maintained), or accept that regeneration
    and equivalence tests own staleness.
  - A JSON blob is not a drop-in alternative for borrowed `&'static str` fields: serde can borrow a
    `&str` only from an unescaped JSON string, so escaped strings need `Cow` or leaking (well-known
    serde behaviour; not re-verified here).
- **Version / build conditions:** pure Rust, `alloc`/`use-std` features; no build script. Serde 1.x.
- **Limits and negative results:** no schema evolution (no optional/unknown-field tolerance);
  no partial access — the whole value is decoded; does not avoid allocation of owned containers.
- **Evidence:** Interface-checked: postcard 1.1.3 source (`from_bytes` signature borrowing `'a`;
  `deserialize_any` refusal; `is_human_readable` false). Interface-checked: arrow-schema 59.3.0
  unit tests round-trip `Field` through postcard with and without metadata. Not probed: borrowing of
  `&'static str` fields under serde derive (expected from serde's implicit `&str` borrowing; settle
  with a probe).

## Current applicability

### [CBO-040](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-040) — Registry products baked as ~87k lines of Rust plus JSONL plus IPC
- **Types:** postcard/embedded-serde-blob.
- **Realization:** upgrade route (new dependency `postcard` 1.1.3); no pinned family moves.
- **Conclusion:** fits with conditions — a single postcard blob can carry the structural products,
  but only after serde derives exist on the product types and three non-postcard-safe members are
  re-encoded.
- **What it would discharge / what remains local:** discharges the source-literal emission of the
  products (`resolved_product_N` functions, `vector()` push sequences, `NativeJsonPool` offsets) and
  the JSONL file; the generator becomes "serialize `ResolvedRegistryProducts`" and hydration becomes
  "decode then `ResolvedRegistryProducts::hydrate`". Remains local: `hydrate()` itself (index
  construction, `install_products`), the fifteen self-description `RecordBatch` IPC files (no serde
  for `RecordBatch`), and the deterministic encoding of the members below.
- **Prerequisites, configuration or extension:**
  - Serde derives on `ResolvedRegistryProducts` and every member type. **Observed:** at
    `c2f8b485a` none of `RelationSpec`, `EnumSpec`, `IdentitySpec`, `InvariantSpec`, `MigrationSpec`,
    `AlgorithmSpec`, `DocumentSpec`, `FieldTypeRow`, `RelationObligations`,
    `ResolvedContractProducts`, `ResolvedSemanticProducts` derive serde (they derive
    `Clone, Debug, PartialEq, Eq` only); only `FieldContract`'s inner arrow `Field` is serialized
    today. This is the dominant cost of the route.
  - `ResolvedSemanticProducts::descriptions` is `BTreeMap<SemanticId, serde_json::Value>`:
    postcard cannot decode `serde_json::Value`. Carry each as canonical JSON text (a `String`) or
    use a self-describing format for that member.
  - Arrow `Field`/`Schema` serde (arrow-schema `serde` feature, already enabled at the 59.3.0 pin)
    serializes `metadata: HashMap<String, String>` in hash order → non-deterministic bytes. Keep the
    current sorted canonical JSON for fields, encode schemas as Arrow IPC schema messages (which sort
    metadata keys), or serialize through a local sorted shadow type.
  - `&'static str` fields (`primary_key: Vec<&'static str>`, `doc: &'static str`) need borrowing
    from a `'static` blob — supported by `from_bytes` on `include_bytes!` data.
- **Semantic gaps against the CBO's preservation constraints:** fingerprints and closure identities
  are stored values, so they survive if decoded bit-exactly (they are byte arrays); deterministic
  bytes for `codegen-check` hold only after the HashMap members are handled; "hydration imports no
  runtime proofs" is unaffected (the products already exclude them).
- **Unsettled premises and what would settle them:** whether compile memory/time actually drop and
  whether first-use decode is acceptable — neither is measured (see probe request P1 in the
  cluster README); whether every transitive member's serde impl avoids `deserialize_any`.
- **Evidence:** Interface-checked (postcard 1.1.3 source; arrow-schema 59.3.0 serde derive and
  postcard tests; product struct definitions at `c2f8b485a`). Application Proposed.

**Reviewer correction (2026-10-27).** The products are not "serialized three ways": each
sub-product is encoded once, by what its type supports. The JSONL is a deduplicated canonical-JSON
pool of serde-capable leaves (Arrow `Field`, references, description values; 4,576 lines and 6,548
decodes), and its `HashMap` ordering is already solved by a canonical-JSON writer, so the
Field-metadata prerequisite above applies only to members serialized through postcard natively
(for example nested `Field` metadata inside `DataType` values). Probe P1 has run: a model field
added together with its emitter breaks the generator's own build before regeneration (the
generator re-exports the schema crate, which compiles the generated module unconditionally), and
the generated module is about 72% of that crate's full build, recompiled whenever its line
numbers shift (*Tested* and *Measured*, [evidence](../../../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md)). The comparative review
therefore recommends this route, with `Field` and description values carried as canonical-JSON
text, IPC batches unchanged and a derived shape-digest guard, amending the proposed ADR-0210 (which
records 304 ms versus 30 ms, fixture-bounded). See the [comparative review](../../reviews/declarations-registry-and-python-boundary.md#cbo-040).
