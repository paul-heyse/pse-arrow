# rkyv

Realizations considered: upgrade route `rkyv` 0.8.18 (latest stable, crates.io updated 2026-08-05;
not in the workspace lockfile; 0.7.46 and 0.8.18 sources present in the local cargo registry from
other projects). No installed realization.
Research date 2026-10-27. Sources: Context7 `/websites/rs_rkyv` (docs.rs latest: `rkyv::access`,
`api::low::access`, `#[derive(Archive)]`, `util::AlignedVec`); crates.io API.

## Types

### rkyv/validated-zero-copy-archive
- **Need it serves:** read a large build-time data product directly from embedded bytes without a
  parse/decode phase, optionally keeping the archived form as the runtime representation.
- **Recognizable code shapes:** startup hydration of big read-only tables; generated data-as-code;
  repeated decode of the same immutable product at process start.
- **Mechanisms and APIs:** `#[derive(Archive, Serialize, Deserialize)]` produces an `ArchivedT`
  mirror type; `rkyv::to_bytes::<E>(&value)` at generation; `rkyv::access::<ArchivedT, E>(&bytes)`
  validates the buffer (feature `bytecheck`) and returns `&ArchivedT` with no copying;
  `rkyv::deserialize::<T, E>(archived)` rebuilds owned values when needed. `#[rkyv(remote = ...)]`
  and `with` wrappers (`with::InlineAsBox`, custom `ArchiveWith`) adapt foreign types;
  `#[rkyv(compare(PartialEq), derive(Debug))]` passes traits to archived types.
- **Obligations discharged:** a structurally validated, zero-copy view of the data; optional owned
  reconstruction without text parsing.
- **Residual caller obligations:** derive/remote definitions for every reachable type, including
  foreign ones; alignment of the input buffer (an `include_bytes!` array is byte-aligned — copy into
  `AlignedVec` at startup or wrap the include in a `#[repr(align(N))]` struct); deciding whether
  consumers use `ArchivedT` (API change) or deserialize (losing most of the zero-copy benefit).
- **Semantic conditions:** archived types differ from native types (`ArchivedString`,
  `ArchivedVec`, `ArchivedHashMap`, `ArchivedBTreeMap`), so APIs returning `&RelationSpec`-like
  references need either deserialization or a parallel archived API; `&'static str` cannot be
  produced by safe deserialization (it yields owned `String`); byte determinism of hash-map archives
  across differing insertion orders is not documented (not established here).
- **Version / build conditions:** 0.8 is a breaking redesign from 0.7; feature flags select
  pointer width, endianness and alignment (`unaligned`, `big_endian`, `pointer_width_*`), and these
  are part of the format.
- **Limits and negative results:** no serde interoperability — serde derives do not help; foreign
  types without rkyv support (for example Arrow `DataType`, `Field`, `Schema`) need full remote
  shadow definitions or encoding into opaque bytes; no schema evolution.
- **Evidence:** Interface-checked: docs.rs (latest) `access`, `access_pos`, `Archive` derive example,
  `AlignedVec` alignment example via Context7. Not checked: hash-map archive determinism, 0.8.18
  remote-derive limits for recursive enums.

## Current applicability

### [CBO-040](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-040) — Registry products baked as ~87k lines of Rust plus JSONL plus IPC
- **Types:** rkyv/validated-zero-copy-archive.
- **Realization:** upgrade route (new dependency `rkyv` 0.8.18).
- **Conclusion:** ruled out here for the full-product route; unsettled for a narrow sub-product —
  the runtime `Registry` owns native types (`Vec<RelationSpec>`, `Arc<ContractArena>`, Arrow
  `SchemaRef`), so zero-copy would require an archived-type API across all consumers or a full
  deserialize that forfeits the distinctive benefit while adding rkyv derives and Arrow remote
  shadows that postcard/JSON do not need.
- **What it would discharge / what remains local:** the parse step only; all type-mirroring of Arrow
  types and `&'static str` handling would become local.
- **Prerequisites, configuration or extension:** Archive derives on ~15 product types plus members;
  remote definitions for Arrow `DataType`/`Field`/`Schema`/`Arc` graphs; aligned embedding.
- **Semantic gaps against the CBO's preservation constraints:** deterministic bytes not established
  for hash-map members; `&'static str` fields would change to owned strings.
- **Unsettled premises and what would settle them:** only if a measured first-use decode cost of the
  postcard/JSON route proved material would an archived-API redesign be worth comparing.
- **Evidence:** Interface-checked (docs as above; product types at `c2f8b485a`). Application Proposed.
