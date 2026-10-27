# msgspec

Realizations considered: installed `msgspec` ==0.21.1 (2026-04-12); upgrade route 0.22.0
(2026-09-29: `Literal[True/False]`, `convert` int→float `ValidationError` fix, `Raw` error naming,
3.15/free-threading wheels; breaking: `gc=False` with weakref slot raises).
Research date 2026-10-27. Tested: probe `docs/design_review/evidence/library-leverage-msgspec-tagged-decode-2026-10-27/README.md` (msgspec 0.21.1, attrs 26.1.0, CPython 3.14.8,
synthetic structs, one run). Sources: msgspec 0.21.1 `_core.c` (tag `0.21.1` on GitHub; functions
`json_ensure_tag_matches`, `json_decode_struct_map_inner`, `json_decode_struct_union`, `convert`
docstring); Context7 `/websites/jcristharif_msgspec` (structs, supported-types, converters, api);
GitHub releases page (0.21.x, 0.22.0).

## Types

### msgspec/tagged-struct-header
- **Need it serves:** let the decoder itself check a discriminating header field (string or integer)
  instead of a separate pre-scan.
- **Recognizable code shapes:** Python code that walks `msgspec.inspect.type_info` to find
  version-carrying structs and re-checks the header on `msgspec.Raw` slices before decoding.
- **Mechanisms and APIs:** `class Doc(msgspec.Struct, tag_field="document_version", tag=1)`;
  tags may be `str` or `int` (not mixed in one union); unions of tagged structs dispatch on the tag.
- **Obligations discharged:**
  - In a **union**, the JSON decoder scans ahead for the tag field (it need not be first), rewinds,
    and refuses a missing tag (`ms_missing_required_field(tag_field)`), then decodes the selected
    struct.
  - Decoding a tagged struct **directly**, the tag value is checked when encountered
    (`json_ensure_tag_matches`, field index −2), int or str.
- **Residual caller obligations:**
  - Direct (non-union) decode does **not** require the tag: a payload without it decodes (Tested,
    probe A2/S3; datamodel-code-generator docs state the same). A union of a tagged struct with `None`
    behaves the same (O1). Only a union of at least two tagged structs makes the tag required (U2:
    `Object missing required field `document_version``).
  - Header-first ordering holds only on the union path: an unknown tag is reported before an earlier
    body error (U5); on direct decode the first error in document order wins (A7 body error, A9
    unknown field).
  - Duplicates: a duplicated tag with a *differing* value is refused through the value check in
    either order (A10/A11/U6); an *equal* duplicate tag and duplicated ordinary fields are accepted,
    last value winning (A12/A13/H1). Duplicate refusal in general stays caller work.
  - Error text is msgspec's (`Invalid value 2 - at `$.document_version``; `Expected `int` - at ...`
    for `"1"`, `true` or `1.0`), not a domain message such as "Unsupported document version (current: N)".
  - The tag field cannot also be a declared field: constructing with it raises
    `TypeError: Unexpected keyword argument 'document_version'` (E2); encoding always writes it (E1).
- **Semantic conditions:** the tag participates in encoding (always written).
- **Version / build conditions:** unchanged in 0.22.0 release notes.
- **Limits and negative results:** no "required tag on direct decode" option found in 0.21.1 source
  (search scope: `_core.c` tag paths for JSON and msgpack).
- **Evidence:** Interface-checked: `_core.c`@0.21.1 (`json_ensure_tag_matches`, union scan with
  rewind and missing-tag error); Context7 tagged-union docs. Tested within the probe's conditions
  (cases A1–A16, S1–S3, U1–U8, O1, H1, E1–E2).

### msgspec/typed-convert-and-attrs-interop
- **Need it serves:** validate already-built Python objects (dicts, attrs/dataclass instances) into
  typed Structs without a JSON round trip.
- **Recognizable code shapes:** `cattrs.unstructure(row)` → manual checks → `msgspec.convert(...)` →
  `msgspec.json.encode`.
- **Mechanisms and APIs:** `msgspec.convert(obj, type, *, strict=True, from_attributes=False,
  dec_hook=None, str_keys=False, builtin_types=None)`; `from_attributes=True` reads attributes from
  arbitrary objects (attrs/ORM rows) into Struct/dataclass/attrs targets; native attrs support for
  encode/decode (validators run on decode; attrs converters unsupported; private `_x` attributes
  skipped); `msgspec.to_builtins`; `msgspec.Raw` for delayed decoding; `Meta` constraints;
  `__post_init__` runs on init, decode and convert.
- **Obligations discharged:** strict structural typing on decode/convert (bool is distinct from int;
  strict mode refuses str→number); `forbid_unknown_fields=True` per Struct; `__post_init__`
  validation surfaced as `ValidationError`.
- **Residual caller obligations:**
  - **Struct `__init__` performs no type validation** (documented intentional design; only decode
    and convert validate). Exact-type checks at construction (attrs `instance_of` validators) have no
    msgspec equivalent except a hand `__post_init__`.
  - Non-finite floats (Tested): JSON decode refuses `NaN`/`Infinity` literals as malformed JSON
    (`DecodeError`), refuses `null` for a non-optional `float` but accepts it as `None` for
    `float | None`, and refuses `1e999` (`Number out of range`); encode writes `nan`/`inf` as `null`.
    `convert` from dicts or attrs (with or without `from_attributes`) **accepts** `nan` and `inf` into
    `float` and `float | None`; `Meta(ge=0)` refuses `nan` but accepts `inf`. Finite-value refusal on
    in-memory paths is caller work.
  - `from_attributes=True` (Tested): attrs row → tagged Struct succeeds without a tag attribute (C1);
    without the flag it fails `Expected `object`, got `Row`` (C2); into a tagged *union* it fails
    `Object missing required field `document_version`` (C3) — the tag must exist as an attribute.
    bool is refused for `int` and `float` on convert (C8/C9/N16).
  - `msgspec.json.encode` of a frozen attrs instance emitted keys out of declaration order (C10,
    observed, unexplained).
- **Semantic conditions:** `from_attributes` maps by attribute name; `dec_hook` handles custom
  identity types; equality/hash on frozen Structs are field-wise (no custom equality key).
- **Version / build conditions:** 0.22.0 fixes an int→float `convert` error class.
- **Limits and negative results:** no construction-time validation mode.
- **Evidence:** Interface-checked: Context7 structs ("Type Validation", "Post-Init Processing"),
  supported-types (attrs, floats, Raw), converters; `_core.c`@0.21.1 `convert` docstring. Tested
  within the probe's conditions (E3 no init validation; N1–N16; C1–C10).

## Current applicability

### [CBO-041](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-041) — Boundary documents transcoded through JSON, with checks in both languages
- **Types:** msgspec/tagged-struct-header; msgspec/typed-convert-and-attrs-interop.
- **Realization:** installed 0.21.1 (0.22.0 changes nothing material here).
- **Conclusion:** fits with conditions — tagging versioned roots (`tag_field="document_version",
  tag=N`) moves the version *value* check into the decoder (including refusal of `"1"`, `true`, `1.0`
  and of differing duplicate tags), but the probe confirms the current contract is stronger than
  msgspec's direct-decode semantics: the tag is not required, a body or unknown-field error earlier in
  the object is reported first, equal duplicate tags and other duplicate keys pass, and messages differ.
  `_check_version_headers` can therefore shrink to "header present, unique, first" (or be dropped where
  Rust remains the authoritative checker) but cannot disappear while those guarantees hold. For the
  attrs edit rows, `msgspec.convert(row, Doc, from_attributes=True)` works without a tag attribute
  (direct, non-union target), replacing the cattrs hop, but the explicit finite check must stay
  because `convert` accepts `nan`/`inf`.
- **What it would discharge / what remains local:** discharges the type-info walk for version value
  matching and the cattrs hop; local: required-ness/ordering/duplicate guarantees, domain error text,
  finite check on in-memory paths, identity `dec_hook`.
- **Prerequisites, configuration or extension:** generated document Structs emitted as tagged (the
  emitter owns this); Native-wire documents whose header is a domain `version` field can also be
  tagged only if that field is not otherwise declared.
- **Semantic gaps against the CBO's preservation constraints:** "identical header diagnostics on both
  sides" breaks unless the Python message is wrapped; unknown-field refusal is preserved
  (`forbid_unknown_fields`, decode and convert); non-finite refusal is preserved on JSON decode only.
- **Unsettled premises and what would settle them:** whether the Python-side check is needed at all
  for Rust→Python documents (Rust produced them under the same contract) — a contract decision, not a
  library question.
- **Evidence:** Interface-checked; library behavior Tested in the probe (synthetic types, msgspec
  0.21.1). Application Proposed.

### [CBO-043](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-043) — Two Python model systems and a custom schema-to-msgspec emitter
- **Types:** msgspec/typed-convert-and-attrs-interop.
- **Realization:** installed 0.21.1.
- **Conclusion:** fits with conditions for carrying rows as Structs only if construction-time exact
  validation is dropped or re-implemented in `__post_init__`; the CBO's counterevidence is sharpened:
  the gap is not "no exact-type validator" but "no validation at all on `__init__`" (Tested: E3
  `DocV1(name=5, count='x')` succeeds); decode/convert do validate strictly, including bool≠int
  (C8/C9/N16), but `convert` accepts non-finite floats (N10–N13).
- **What it would discharge / what remains local:** one class system and one codec (cattrs
  forbid-extra-keys hooks); local: per-field `__post_init__` checks (or acceptance that rows are only
  validated when converted), `_pse_equality_key` semantics (Struct equality is field-wise), Arrow/null
  extension-type handling.
- **Prerequisites, configuration or extension:** generator emits Structs with `frozen=True,
  kw_only=True, forbid_unknown_fields=True` and `__post_init__` validators where construction checks
  are kept.
- **Semantic gaps against the CBO's preservation constraints:** exact-type bool/finite-float
  validation at construction is lost unless hand-emitted; frozen values preserved; `typing.Any` ban
  unaffected.
- **Unsettled premises and what would settle them:** whether rows are constructed by user code
  (needing construction validation) or only decoded/converted — consumer inventory (the CBO says rows
  are used only by `codec` and `_modeling` outside tests).
- **Evidence:** Interface-checked. Application Proposed.
