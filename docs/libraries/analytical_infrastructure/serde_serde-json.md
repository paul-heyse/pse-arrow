# Serde and serde_json — Comprehensive Rust Library Reference

Research date: October 29, 2026 | Stable versions: serde 1.0.229 · serde_json 1.0.154

Primary repositories

- Serde: `https://github.com/serde-rs/serde`
- serde_json: `https://github.com/serde-rs/json`
- Serde documentation: `https://serde.rs/`
- serde API: `https://docs.rs/serde/1.0.229/serde/`
- serde_json API: `https://docs.rs/serde_json/1.0.154/serde_json/`

Versions verified against current published crates and documentation. The discussion targets released functionality unless explicitly qualified.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



## 1. Architectural orientation

Serde is a serialization framework and abstraction layer; serde_json is a concrete JSON implementation of that framework. Their most important architectural property is the separation of the Rust data model from the external representation, allowing the same Rust types to interact with different serialization formats without format-specific implementations.

The relationship is:

Rust types and domain structures

Structs, enums, collections, wrappers, borrowed data, custom types

Serialize / Deserialize

Serde data model and trait protocols

Serializer · Deserializer · Visitor · access traits

Format implementations

serde_json

CBOR / MessagePack

Postcard / other formats

Serde uses generic traits and compile-time code generation rather than runtime reflection. For many combinations of data type and format, Rust can optimize away substantial abstraction overhead; this is an architectural mechanism, not a guarantee of equivalent performance for every serializer or workload.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 1.1 Capability map

| Capability family                | Principal mechanisms                                                       | Architectural significance                                                                             |
| -------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Type-driven serialization        | `Serialize`, `Deserialize<'de>`, derive macros                             | External representations can be generated from Rust types with minimal handwritten conversion code     |
| Schema and wire-shape adaptation | Rename, aliases, defaults, flattening, enum tagging, conversion attributes | Separate Rust type design from external field names and representation conventions                     |
| Custom representation            | Manual trait implementations, `Visitor`, `Serializer`, `Deserializer`      | Support specialized wire formats, validation during decoding, and nonstandard data representations     |
| Borrowed and streaming decoding  | `'de`, `DeserializeSeed`, `SeqAccess`, `MapAccess`                         | Reduce allocations and process large inputs without constructing complete intermediate data structures |
| JSON text processing             | `from_*`, `to_*`, JSON serializer/deserializer                             | Move between JSON text and typed Rust data through strings, slices, and I/O                            |
| Dynamic JSON structures          | `Value`, `Map`, `Number`, `json!`                                          | Manipulate partially known, heterogeneous or evolving JSON structures                                  |
| JSON parsing and forwarding      | `RawValue`, streaming deserialization                                      | Preserve or defer selected JSON subtrees without materializing them as ordinary values                 |
| Representation control           | Formatters, feature flags, precision and ordering options                  | Control output layout, number handling, and JSON object ordering                                       |
| Ecosystem composition            | `serde_with`, `serde_bytes`, `serde_path_to_error`, other companion crates | Extend representation and diagnostic behavior without implementing a new serialization framework       |

### 1.2 Fundamental responsibility boundaries

| Component        | Responsibility                                                          | Does not inherently provide                                                      |
| ---------------- | ----------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `serde`          | Generic traits, data model, derives, type-level serialization mechanics | A JSON parser or writer, external schema language, automatic semantic validation |
| `serde_core`     | Serde trait definitions without derive support                          | Derive macros                                                                    |
| `serde_derive`   | Procedural macro implementation for `Serialize` and `Deserialize`       | A runtime serialization engine                                                   |
| `serde_json`     | JSON parsing, writing, dynamic values, format-specific behavior         | General format independence, full JSON Schema validation, canonical JSON         |
| Companion crates | Additional adapters, validation, diagnostics, or other data formats     | Functionality should not be assumed to belong to core Serde                      |

A particularly relevant distinction in current versions is `serde_core`. The `serde` crate re-exports its core traits and integrates derive support. Packages that only implement traits manually or use them as bounds can depend directly on `serde_core`, which can improve compilation parallelism with procedural macro dependencies. Packages using Serde derives must depend on `serde`, not merely `serde_core`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: `https://docs.rs/serde_core/1.0.229/serde_core/`

## 2. Serde's data model and trait architecture

### 2.1 The 29-category data model

Serde defines a common representation protocol encompassing 29 categories of values. These are not runtime `Value` objects or a universal intermediate AST: they are operations through which types and format implementations communicate.

| Data-model category | Representation and significance                                                              |
| ------------------- | -------------------------------------------------------------------------------------------- |
| Primitive scalars   | Boolean, signed/unsigned integers through 128 bits, `f32`, `f64`, `char`                     |
| Strings             | UTF-8 strings, with separate deserialization paths for transient, borrowed and owned storage |
| Bytes               | Byte sequences, independently represented from strings                                       |
| Option              | Presence/absence through `serialize_some` / `serialize_none` and `deserialize_option`        |
| Unit                | The Rust unit value `()`                                                                     |
| Unit struct         | A named zero-field type                                                                      |
| Newtype struct      | A named wrapper around one value                                                             |
| Sequence            | Dynamically sized element sequence, potentially unknown length                               |
| Tuple               | Fixed-length, positionally structured sequence                                               |
| Tuple struct        | Named fixed-length positional structure                                                      |
| Map                 | Arbitrary key/value pairs                                                                    |
| Struct              | Named fields known to the serializer/deserializer                                            |
| Enum variants       | Unit, newtype, tuple and struct variants                                                     |

Important consequences:

- Struct versus map: Both can become JSON objects, but a struct communicates statically known fields, allowing some binary formats to avoid encoding field names.
- Tuple versus sequence: A tuple's length is known from its type; a sequence may require explicit length encoding.
- Newtype versus wrapper object: A format may encode a newtype identically to its contained value. `serde_json` does so by default.
- Byte sequence versus ordinary sequence: A format supporting native bytes can encode them more efficiently than an array of integers.
- Enum representation: Serde distinguishes variant identity from variant content, allowing formats to represent variants through names, numeric identifiers or specialized encodings.

The distinction between struct, map and sequence categories is particularly important when making a type portable between JSON and schema-dependent binary formats.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

`https://serde.rs/data-model.html`

`https://docs.rs/serde/1.0.229/serde/trait.Serializer.html`

### 2.2 Four foundational traits

| Trait               | Implemented by               | Core behavior                                                               |
| ------------------- | ---------------------------- | --------------------------------------------------------------------------- |
| `Serialize`         | Rust data type               | Converts itself into a format-independent sequence of serializer operations |
| `Deserialize<'de>`  | Rust data type               | Constructs itself from a format-independent deserializer                    |
| `Serializer`        | Output format implementation | Converts Serde data-model operations into external output                   |
| `Deserializer<'de>` | Input format implementation  | Interprets external input and drives type-specific visitor logic            |

Serialization is driven by a type invoking serializer methods. Deserialization is driven by a deserializer invoking methods on a `Visitor` provided by the destination type.

This difference is consequential: the serializer generally starts with an available Rust value, while the deserializer must construct a value incrementally, account for borrowing and detect mismatches.

### 2.3 Self-describing versus non-self-describing formats

Serde supports two distinct styles of deserialization.

Self-describing formats, including JSON, can inspect input tokens and identify whether the next value is a map, string, sequence or number. They can support `Deserializer::deserialize_any`, which lets the format select the visitor operation.

Non-self-describing formats, such as Postcard, generally require the receiving Rust type to specify which representation to decode. They depend on type-directed `deserialize_*` calls.

Consequences:

- A custom `Deserialize` implementation relying exclusively on `deserialize_any` is not universally format-portable.
- Generic dynamic values such as `serde_json::Value` are unsuitable as universal decoding targets for all binary formats.
- A strongly typed representation can often be more portable and more efficiently encoded than a generic dynamic tree.
- A custom serializer may legitimately reject data-model categories unsupported by its underlying format.

Serde also exposes `Serializer::is_human_readable()` and `Deserializer::is_human_readable()`. Custom types can use these signals to represent, for example, an identifier as a readable string in JSON and a compact numeric or binary representation in another format.

This allows one Rust type to support multiple appropriate wire representations without forcing every format to adopt identical byte-level semantics.

Sources:

`https://docs.rs/serde/1.0.229/serde/trait.Deserializer.html`

`https://serde.rs/impl-deserialize.html`

`https://serde.rs/data-format.html`

## 3. Derive-based representation control

Serde derive is more than boilerplate elimination. Its attribute system is a declarative representation layer that can accommodate naming conventions, partial schemas, evolving fields, alternate wire representations and selected validation rules.

### 3.1 Container and field transformation

| Mechanism                                    | Functionality                                                   | Important distinction                                                                 |
| -------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `rename`                                     | Override serialized/deserialized names                          | Can provide independent names for the two directions                                  |
| `rename_all`                                 | Apply casing rules to struct fields or enum variants            | Supports camelCase, PascalCase, snake_case, kebab-case and other built-in conventions |
| `rename_all_fields`                          | Apply field renaming across an enum's struct variants           | Distinct from renaming the variants themselves                                        |
| `alias`                                      | Accept additional field or variant names during deserialization | Supports backward compatibility without changing the preferred serialized name        |
| `default`, `default = "path"`                | Supply values when fields are absent                            | Field-level and container-level defaults have different scope                         |
| `skip`                                       | Omit a field in both directions                                 | Deserialization uses a default value                                                  |
| `skip_serializing`                           | Omit only during encoding                                       | Does not make the field optional during decoding                                      |
| `skip_deserializing`                         | Omit only during decoding                                       | Uses a default for the omitted field                                                  |
| `skip_serializing_if`                        | Conditional field omission                                      | Evaluates a user-supplied predicate on the field                                      |
| `flatten`                                    | Merge a nested struct or map into the surrounding object        | Supports shared field groups and extension fields                                     |
| `deny_unknown_fields`                        | Reject unrecognized input fields                                | Opt-in strictness; incompatible with `flatten`                                        |
| `transparent`                                | Serialize/deserialize through a single contained field          | Allows domain-specific wrapper types without additional wire nesting                  |
| `serialize_with`, `deserialize_with`, `with` | Supply field-specific adapters                                  | Changes representation without changing the Rust field type                           |
| `bound`                                      | Override generated generic trait bounds                         | Important for generic wrappers and custom adapters                                    |
| `borrow`                                     | Enable borrowing through annotated nested types                 | Introduces deserializer lifetime constraints                                          |

Attributes are applicable at container, variant or field scope according to their definitions. Not all attributes can be used at all levels.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde+2



Sources:

`https://serde.rs/container-attrs.html`

`https://serde.rs/variant-attrs.html`

`https://serde.rs/field-attrs.html`

### 3.2 Flattening as extensible schema composition

`#[serde(flatten)]` is a particularly valuable mechanism for partially fixed schemas.

A struct can combine:

- Explicit typed fields carrying guaranteed domain structure.
- One or more flattened nested structs supplying reusable groups of fields.
- A flattened `BTreeMap<String, serde_json::Value>` or similar map that captures otherwise unrecognized fields.

This permits an external object to evolve without changing the main Rust type for every newly introduced key.

Architectural advantage: It provides a hybrid typed/dynamic representation directly within Serde, avoiding separate merging and field-dispatch machinery.

Limitations matter:

- Only named-field structs can contain flattened fields.
- The flattened type must be struct-like or map-like.
- `deny_unknown_fields` cannot be combined with this mechanism.
- Flattening affects key ownership and namespace collision behavior; the attribute does not establish a general schema-composition or conflict-resolution policy.
- Deserializing flattened or internally tagged structures can require buffering intermediate content; zero-allocation assumptions should not be transferred automatically to these constructs.

Source: `https://serde.rs/attr-flatten.html`

### 3.3 Enum representation strategies

Serde supports four principal enum layouts, with substantial differences in compatibility, performance and ambiguity.

| Strategy          | Representative JSON                  | Characteristics                                                     |
| ----------------- | ------------------------------------ | ------------------------------------------------------------------- |
| Externally tagged | `{"Created":{"id":7}}`               | Default; variant known before decoding its contents                 |
| Internally tagged | `{"type":"Created","id":7}`          | Tag mixed with named fields; cannot support tuple variants          |
| Adjacently tagged | `{"type":"Created","data":{"id":7}}` | Explicit separation between variant identity and payload            |
| Untagged          | `{"id":7}`                           | No discriminator; candidate variants attempted in declaration order |

Externally tagged: Supports all variant shapes and is the only option among these four whose deserialization works without an allocator. It is especially suitable for formats that need to decode variant identity before payload contents.

Internally tagged: Useful for object-shaped events or commands where the discriminator is one of the object's fields. It supports struct variants, appropriate newtype variants and unit variants, but not tuple variants.

Adjacently tagged: Works with a stable discriminator/content envelope, including payloads that are not themselves maps.

Untagged: Can accept heterogeneous input representations without an explicit discriminator. The first matching variant wins. It is valuable for compatibility, but overlapping variant shapes can produce unexpected matches. Decoding may also be expensive because multiple alternatives are attempted, and failures often have weak diagnostic specificity.

Additional mechanisms include:

- `#[serde(other)]` on a unit variant of an internally or adjacently tagged enum to accept unknown tags.
- `#[serde(alias = "...")]` to accept historical variant names.
- Variant-level `#[serde(untagged)]` to handle selected variants without tagging; such variants must occur last.
- `#[serde(field_identifier)]` and `#[serde(variant_identifier)]` for custom deserializers handling named or numeric identifiers.
- Variant-specific custom serializer/deserializer functions.

Internally tagged, adjacently tagged and untagged enum deserialization require allocation support.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde+2



Sources:

`https://serde.rs/enum-representations.html`

`https://serde.rs/variant-attrs.html`

### 3.4 Type adaptation and conversion

Serde includes several mechanisms that can eliminate manual intermediate-DTO processing.

`#[serde(from = "T")]`: Deserialize a different type and convert using `From<T>`. The conversion is infallible from Serde's perspective.

`#[serde(try_from = "T")]`: Deserialize into an intermediate type and apply `TryFrom<T>`. Domain invariants, parsed identifiers and constrained values can therefore be validated during decoding, with conversion errors propagated.

`#[serde(into = "T")]`: Convert into a serialization representation and serialize that type. The derived implementation requires the source type to implement `Clone` and `Into<T>`.

`#[serde(transparent)]`: Preserve the representation of a single field while retaining a distinct Rust type. This is useful for domain newtypes that should not introduce another object or array layer.

`#[serde(remote = "Type")]`: Declare a mirror definition of an external type and derive conversion logic despite Rust's orphan rules. Supports public-field structures, enums, and private-field structures with appropriate getters and reconstruction conversions.

Remote derive is especially useful when adapting third-party library types without introducing persistent wrapper types throughout a system. Its mirror definition is checked against the remote structure at compile time, but an inaccessible or otherwise unconstructible remote type can still require a different approach.

`#[serde(bound = "...")]`: Override inferred trait bounds when a generic field uses conversion logic that does not actually require its generic parameters to implement Serde traits themselves.

Sources:

`https://serde.rs/remote-derive.html`

`https://serde.rs/attr-bound.html`

`https://serde.rs/container-attrs.html`

### 3.5 Missing, null, unknown and duplicate fields

These are distinct semantic conditions.

| Condition                                      | Typical Serde/JSON behavior                    | Important implication                                                             |
| ---------------------------------------------- | ---------------------------------------------- | --------------------------------------------------------------------------------- |
| Missing required field                         | Deserialization error                          | A field is required unless optionality or a default is established                |
| Missing `Option<T>`                            | Usually `None`                                 | An option need not carry `#[serde(default)]` for ordinary derived struct decoding |
| Explicit JSON `null` for `Option<T>`           | `None`                                         | Missing and explicit null are normally conflated                                  |
| Missing field with `default`                   | Default value supplied                         | Does not itself cause the field to be omitted during serialization                |
| Unknown field                                  | Ignored by default in ordinary derived structs | Forward compatibility is the default, not strict schema enforcement               |
| Unknown field with `deny_unknown_fields`       | Error                                          | Useful for strict input contracts                                                 |
| Duplicate recognized field in a derived struct | Typically a duplicate-field error              | Not the same as map insertion behavior                                            |
| Duplicate key in a map or `Value::Object`      | Later values normally replace earlier ones     | Potentially significant when interpreting untrusted or security-sensitive JSON    |

In particular, `Option<Option<T>>` does not automatically provide a reliable three-way distinction between absent, null and actual values. Doing that requires customized deserialization semantics or an appropriate adapter.

Likewise, `skip_serializing_if = "Option::is_none"` controls the output representation; it does not by itself change deserialization rules.

Sources:

`https://serde.rs/field-attrs.html`

`https://serde.rs/container-attrs.html`

`https://github.com/serde-rs/serde/issues/2897`

`https://github.com/serde-rs/json/issues/652`

## 4. Advanced serialization and deserialization mechanisms

### 4.1 Handwritten `Serialize`

For specialized representations, `Serialize::serialize` directly invokes one of the methods supplied by the format's `Serializer`.

Composite values use stateful serializers:

| Serializer family | Representative operations                                                     |
| ----------------- | ----------------------------------------------------------------------------- |
| Sequences         | `serialize_seq`, `serialize_element`, `end`                                   |
| Maps              | `serialize_map`, `serialize_key`, `serialize_value`, `serialize_entry`, `end` |
| Structs           | `serialize_struct`, `serialize_field`, `end`                                  |
| Tuples            | `serialize_tuple`, `serialize_element`, `end`                                 |
| Struct variants   | `serialize_struct_variant`, `serialize_field`, `end`                          |
| Tuple variants    | `serialize_tuple_variant`, `serialize_field`, `end`                           |

This supports types that serialize as projections, filtered fields, transformed collections, computed properties or protocol-specific representations without first materializing an equivalent Rust structure.

The generic serializer still determines how those operations appear in its format.

### 4.2 Handwritten `Deserialize` and the Visitor protocol

`Deserialize<'de>` supplies a `Visitor<'de>` describing how external input should be interpreted as the destination type.

The visitor can receive primitive values, borrowed or owned strings, bytes, sequences, maps and enum variants.

More advanced interfaces include:

- `SeqAccess`: Incremental consumption of sequence elements through `next_element` and `next_element_seed`.
- `MapAccess`: Incremental consumption of object entries using key/value accessors, including seeded variants.
- `EnumAccess`: Identify the active enum variant.
- `VariantAccess`: Deserialize unit, newtype, tuple or struct variant payloads.
- `de::Error`: Generate structured type, length, value, variant and field mismatch errors.
- `Unexpected` / `Expected`: Describe mismatched input in error reporting.

A custom visitor can process elements as they arrive instead of building a complete collection. For example, it can compute aggregates, fill a specialized structure, deduplicate records or enforce domain invariants during decoding.

This capability is distinct from streaming multiple top-level JSON documents: a custom visitor can process a single very large JSON array or object incrementally, without retaining all its members.

Source: `https://serde.rs/stream-array.html`

### 4.3 Stateful deserialization with `DeserializeSeed`

`DeserializeSeed<'de>` is the stateful counterpart of `Deserialize`.

Ordinary `Deserialize` constructs a value without external mutable state. A seed can carry context into the deserialization operation, allowing decoding to be driven by pre-existing state.

Potential applications include:

- Appending data into an existing collection.
- Incrementally populating a registry, graph or index.
- Interning strings or identifiers while decoding.
- Resolving dictionary-encoded identifiers through an existing dictionary.
- Applying context-dependent construction rules.
- Building domain-specific structures rather than intermediate generic collections.

A seed can also be passed through `SeqAccess::next_element_seed` and `MapAccess::next_value_seed`, making stateful decoding composable within larger documents.

This can replace a two-stage operation of deserializing everything into temporary objects and then independently transforming those objects into final state.

Tradeoff: Seed-driven decoding makes deserialization stateful; partial mutation on failure, rollback requirements and borrowing must be considered explicitly.

Source: `https://docs.rs/serde/1.0.229/serde/de/trait.DeserializeSeed.html`

### 4.4 Generic deserializer adapters

The `serde::de::value` module contains reusable components for constructing deserializers from existing Rust values.

Important examples:

- `IntoDeserializer` converts supported Rust values into Serde deserializers.
- `SeqDeserializer` and `MapDeserializer` adapt iterators into deserialization sources.
- `SeqAccessDeserializer`, `MapAccessDeserializer` and `EnumAccessDeserializer` bridge the corresponding access traits into full deserializers.
- `StrDeserializer` / `BorrowedStrDeserializer` distinguish transient from borrowable string inputs.
- Corresponding primitive, byte and owned-string deserializers support direct value adaptation.

These are useful for implementing configuration overlays, in-memory translation layers and structured input adapters without manufacturing JSON text as an intermediate representation.

`IgnoredAny` is another useful primitive: it accepts and discards a value without constructing a dynamic `Value` tree. This can help selectively skip irrelevant fields in custom visitors.

Sources:

`https://docs.rs/serde/1.0.229/serde/de/value/`

`https://docs.rs/serde/1.0.229/serde/de/struct.IgnoredAny.html`

### 4.5 Borrowed deserialization and zero-copy behavior

Serde's `'de` lifetime represents how long data can safely be borrowed from the deserialization source.

Three input-storage categories matter:

| Category  | Visitor methods                              | Lifetime behavior                                            |
| --------- | -------------------------------------------- | ------------------------------------------------------------ |
| Transient | `visit_str`, `visit_bytes`                   | Reference is only valid during the visitor call              |
| Borrowed  | `visit_borrowed_str`, `visit_borrowed_bytes` | Reference can be retained for the appropriate `'de` lifetime |
| Owned     | `visit_string`, `visit_byte_buf`             | Ownership is transferred to the visitor                      |

Two important type bounds are:

- `T: Deserialize<'de>` — `T` may borrow from the source for the specified lifetime.
- `T: DeserializeOwned` — `T` can be deserialized without retaining references into the input.

With `serde_json`, borrowing is available from `&str` and `&[u8]` sources when the input token can be referenced directly. It is not available through ordinary `io::Read` sources.

The distinction is more subtle than choosing borrowed Rust field types:

- An unescaped JSON string may be borrowed directly.
- A string containing escape sequences may require decoding into separate storage.
- A borrowed target such as `&str` cannot always accept a JSON token that requires a newly allocated decoded string.
- `Cow<'a, str>` permits borrowing or owning, but derive behavior and `#[serde(borrow)]` determine whether borrowing is actually attempted.
- Nested borrowed types may need explicit `#[serde(borrow)]` attributes.

Consequently, borrowed deserialization is an important allocation-avoidance mechanism, not a guarantee that every string in a JSON document can be decoded without allocating.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde+1



Sources:

`https://serde.rs/lifetimes.html`

`https://docs.rs/serde/1.0.229/serde/de/trait.DeserializeOwned.html`

### 4.6 In-place deserialization

Serde's implementation also includes an in-place deserialization mechanism, `Deserialize::deserialize_in_place`, used in generated/internal deserialization paths.

Its potential benefit is reuse of existing destination storage rather than reconstructing and replacing every value.

However, the method is documentation-hidden rather than a prominently supported application-level extension API. An error may leave the destination in a partially updated but memory-safe state.

It is therefore worth recognizing as part of Serde's implementation machinery, but not treating it as a general transaction-safe update protocol.

### 4.7 Serialization as a projection

One powerful architectural consequence of the above traits is that a serializable representation need not correspond one-to-one with an owned struct.

A `Serialize` implementation can traverse an existing structure and expose a logical representation assembled from references, computed fields, iterators and nested serializers.

This allows serialization directly from existing state, without allocating an intermediate representation merely to match a desired wire shape. The opportunity is strongest when the target shape can be expressed as Serde data-model operations and the destination format supports the required representations.

Sources:

`https://serde.rs/impl-serialize.html`

`https://serde.rs/impl-deserialize.html`

## 5. serde_json: JSON processing capabilities

`serde_json` implements Serde's format traits while also providing a JSON-specific dynamic representation and supporting utilities.

Its most important architectural distinction is between direct typed decoding, dynamic value-tree construction, and preserving or streaming JSON without building the full tree.

### 5.1 Input and output APIs

| Operation                 | API                                 | Properties                                                                       |
| ------------------------- | ----------------------------------- | -------------------------------------------------------------------------------- |
| JSON text → typed value   | `from_str::<T>()`                   | Can deserialize types that borrow from the input                                 |
| JSON bytes → typed value  | `from_slice::<T>()`                 | Can borrow from the byte slice                                                   |
| I/O → typed value         | `from_reader::<R,T>()`              | Direct synchronous I/O; destination must implement `DeserializeOwned`            |
| Typed value → string      | `to_string()`, `to_string_pretty()` | Produces an owned UTF-8 JSON string                                              |
| Typed value → bytes       | `to_vec()`, `to_vec_pretty()`       | Produces an owned JSON byte buffer                                               |
| Typed value → writer      | `to_writer()`, `to_writer_pretty()` | Writes through `std::io::Write` without first producing a complete output string |
| Rust value → JSON value   | `to_value()`                        | Uses serialization into an in-memory JSON representation                         |
| JSON value → Rust type    | `from_value()`                      | Consumes a `Value`; requires `DeserializeOwned`                                  |
| Inline value construction | `json!`                             | Builds a `Value` from JSON-like syntax containing Rust expressions               |

Several operations are not guaranteed to succeed merely because a Rust type implements the corresponding Serde trait. For example, `to_value()` can fail when the representation contains map keys that cannot be expressed as JSON object names.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Source: `https://docs.rs/serde_json/1.0.154/serde_json/`

### 5.2 Typed decoding versus dynamic value construction

Three main representations are available.

| Representation                | Strengths                                                                         | Costs or limitations                                    |
| ----------------------------- | --------------------------------------------------------------------------------- | ------------------------------------------------------- |
| `T: Deserialize`              | Typed fields, compile-time access, direct construction, derived structural checks | Requires a receiving type and applicable decoding rules |
| `serde_json::Value`           | Arbitrary JSON trees, dynamic fields, mutation, exploration                       | Owned dynamic tree, runtime type checks, allocation     |
| `serde_json::value::RawValue` | Retain unprocessed JSON subtrees and preserve their text                          | JSON-specific, lacks structured access until parsed     |

A hybrid typed/dynamic representation is also possible using a typed struct with selected `Value` fields or a flattened extension map.

This is often more expressive than choosing between an entirely rigid struct and an entirely dynamic JSON document.

### 5.3 Dynamic `Value` and its manipulation APIs

`Value` is a recursive enum with six variants:

`Null`, `Bool`, `Number`, `String`, `Array(Vec<Value>)`, and `Object(Map<String, Value>)`.

It supports several operations beyond ordinary indexed access.

Inspection and conversion

- `is_*` and `as_*` methods inspect or access specific JSON representations.
- `as_object_mut()` and `as_array_mut()` permit mutable structural operations.
- `get()` / `get_mut()` perform checked index or key lookup.
- `Value::take()` moves a value out, replacing it with `Null`.
- `From` implementations and `json!` simplify construction; `to_value()` supports fallible construction from arbitrary serializable types.

JSON Pointer

- `pointer()` and `pointer_mut()` implement RFC 6901 pointer paths.
- Paths address nested objects and arrays using slash-separated tokens.
- Pointer escape rules distinguish `~1` for `/` and `~0` for `~`.
- This supplies a useful built-in mechanism for deterministic path lookup and mutation.

Index semantics

Reading with `value["field"]` is convenient but returns `Value::Null` for a missing key, unsuitable parent type or invalid array index.

Consequently, an existing JSON `null` and a failed lookup become indistinguishable through that syntax. `get()` and `pointer()` return `Option` and preserve that distinction.

Mutable indexing has additional behavior: indexing a null/object value with a string key can create missing fields, while incompatible mutations or invalid array indices can panic.

For strict transformations, the checked APIs provide more explicit semantics than chained indexing.

Map-level facilities

`Map` supports entry APIs, iteration, mutable iteration, removal, retention and append operations. With order-preservation enabled, it also supports insertion-position manipulation and removal strategies with distinct ordering effects.

Sources:

`https://docs.rs/serde_json/1.0.154/serde_json/value/enum.Value.html`

`https://docs.rs/serde_json/1.0.154/serde_json/map/struct.Map.html`

### 5.4 Stream deserialization

`serde_json::Deserializer` exposes constructors for slices, strings and readers, plus lower-level operations for managing parsing directly.

Important APIs include:

- `Deserializer::from_str`
- `Deserializer::from_slice`
- `Deserializer::from_reader`
- `Deserializer::into_iter::<T>`
- `Deserializer::end`
- `StreamDeserializer::byte_offset`

`StreamDeserializer` is an iterator over consecutive JSON values. It can process whitespace-separated or newline-separated records without requiring an enclosing JSON array.

It is suited to record streams and formats such as NDJSON, provided the stream contains valid, properly delimited JSON values.

`byte_offset()` reports progress through successfully deserialized input and supports resuming after incomplete input. A documented recovery pattern is to retain the remaining bytes after an EOF error, append newly received data and resume parsing.

The offset contract is important: it is not a general guarantee that every syntactically traversed byte after an arbitrary type error has been committed.

`Deserializer::end()` verifies that only trailing whitespace remains after a single value has been decoded. This distinguishes decoding one value from accepting a sequence of values.

Reader behavior: `from_reader()` does not internally provide `BufReader`-style buffering. Upstream documentation recommends caller-supplied buffering for sources such as files where frequent small reads are expensive.

Additionally, the convenience `from_reader()` API expects the input stream to reach EOF after the JSON value. It is not an appropriate record-framing mechanism for a persistent connection that remains open indefinitely.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

`https://docs.rs/serde_json/1.0.154/serde_json/struct.Deserializer.html`

`https://docs.rs/serde_json/1.0.154/serde_json/de/struct.StreamDeserializer.html`

`https://docs.rs/serde_json/1.0.154/serde_json/fn.from_reader.html`

### 5.5 Two different kinds of streaming

This distinction is essential for large-document processing.

Top-level record streaming: `StreamDeserializer<T>` produces multiple consecutive values, one at a time.

Intra-document streaming: A custom Serde visitor processes elements or fields inside one large JSON value, without accumulating the entire array/object as a `Vec` or `Value`.

These can be combined: a stream of records can be decoded into types whose individual fields use specialized visitors.

Neither mechanism eliminates the memory consumed by whatever data the application intentionally retains. For example, streaming into a growing `Vec` still eventually materializes the collection.

### 5.6 Output customization through formatters

`serde_json::Serializer<W, F>` exposes the output representation through a writer and formatter.

Built-in formatters:

- `CompactFormatter`: Minimized whitespace.
- `PrettyFormatter`: Indented, human-readable JSON.
- `PrettyFormatter::with_indent()`: Custom indentation bytes.

The `Formatter` trait permits more detailed control, including:

- Beginning and ending arrays, objects and strings.
- Separators between object keys/values and array elements.
- Numeric and boolean formatting.
- Character escaping and string fragments.
- Raw JSON fragment writing.
- Byte-array representation.

A custom formatter can therefore implement specialized formatting or representation policies without replacing the full JSON serialization framework.

The limit is that the application must ensure its formatter actually emits the desired valid protocol representation. Arbitrary formatter customization does not automatically preserve standard JSON semantics.

`Serializer::into_inner()` returns the underlying writer.

Source: `https://docs.rs/serde_json/1.0.154/serde_json/ser/trait.Formatter.html`

## 6. Advanced JSON-specific capabilities

### 6.1 `RawValue`: preserving unprocessed JSON subtrees

The `raw_value` Cargo feature enables `serde_json::value::RawValue`.

Unlike `Value`, which parses and reconstructs a JSON subtree, `RawValue` retains the JSON text representing one valid value.

It supports two especially useful representations:

| Form            | Input requirements                                       | Behavior                                                                |
| --------------- | -------------------------------------------------------- | ----------------------------------------------------------------------- |
| `&'de RawValue` | Borrow-capable source such as `from_str` or `from_slice` | References the original JSON text without allocating a separate subtree |
| `Box<RawValue>` | Including I/O-backed sources                             | Owns buffered raw JSON text                                             |

Capabilities:

- Defer decoding a nested payload until its contents are actually needed.
- Preserve the lexical formatting of an opaque subtree.
- Transfer subtrees between JSON objects without parsing them into ordinary values.
- Preserve numeric literals or string escapes in a subtree that might otherwise be normalized by a parse/serialize cycle.
- Combine typed envelope fields with opaque nested JSON payloads.

The `RawValue` representation can also be used as a deserialization source to parse a retained subtree later.

Additional APIs:

- `RawValue::get()` accesses the underlying JSON text.
- `RawValue::from_string()` validates an owned JSON string and produces `Box<RawValue>`.
- `RawValue::from_string_unchecked()` avoids reparsing but is unsafe and requires the caller to uphold the documented JSON-validity and surrounding-whitespace invariants.
- Constants `RawValue::NULL`, `TRUE` and `FALSE` provide predefined valid values.

Formatting distinction: When a `RawValue` is serialized, its internal formatting is retained rather than minified or pretty-printed. Formatting of the surrounding structure remains controlled by the active serializer.

Architectural opportunity: Typed envelope decoding plus raw payload retention can avoid constructing and immediately reserializing large, irrelevant subtrees in message routing, selective transformation and JSON forwarding systems.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source:

`https://docs.rs/serde_json/1.0.154/serde_json/value/struct.RawValue.html`

### 6.2 JSON numeric representation and precision

JSON has a general number grammar, but Rust has multiple distinct numeric types with finite ranges and precision.

`serde_json::Number` provides:

- `is_i64`, `is_u64`, `is_f64`
- `as_i64`, `as_u64`, `as_f64`
- `as_i128`, `as_u128`
- `from_i128`, `from_u128`
- `from_f64`, returning `None` for non-finite values

Two optional features address different preservation requirements.

| Feature               | Preservation target                                       | Mechanism and implications                                                                                                                 |
| --------------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `float_roundtrip`     | Floating-point value across `f64 → JSON → f64`            | Higher-precision float parsing; upstream documentation notes an approximately 2× cost for float parsing compared with the default approach |
| `arbitrary_precision` | Numeric JSON representation across `JSON → Number → JSON` | Stores the original numeric literal instead of reducing it to the default machine-number representation                                    |

These are not equivalent.

`float_roundtrip` addresses the accuracy of parsing floating-point values. It does not create a general arbitrary-precision decimal type.

`arbitrary_precision` preserves numerical text in `serde_json::Number`, supports larger integer representations through the `Number` APIs, and provides `Number::as_str()`. It does not itself implement arbitrary-precision arithmetic.

Important qualifications:

- Deserializing into an ordinary Rust `f64`, `i64` or `u64` still entails that type's representational limits.
- Without `arbitrary_precision`, converting arbitrary large JSON numbers into a `Value` may lose numeric precision or fail when representation limits are exceeded.
- Serializing an `i128` directly and storing that value inside `serde_json::Number` are distinct operations; the latter has additional representation constraints.
- JSON has no standard `NaN`, `Infinity` or `-Infinity` literals. `serde_json` serializes non-finite floating-point values as `null`, which can be lossy if not explicitly accounted for.
- Preserving a number's original spelling is different from preserving a mathematically equivalent decimal value.

Sources:

`https://docs.rs/serde_json/1.0.154/serde_json/value/struct.Number.html`

`https://docs.rs/crate/serde_json/1.0.154/features`

`https://github.com/serde-rs/json/blob/master/Cargo.toml`

### 6.3 Object ordering and deterministic output

By default, `serde_json::Map` uses a `BTreeMap`-based representation, which maintains keys in sorted order.

The `preserve_order` feature replaces this with `IndexMap`-based storage, enabling insertion-order and parsed-input-order retention for JSON `Value` objects.

This is particularly useful when re-emitting documents where object member ordering is significant to readers or downstream systems.

Additional map operations include:

- `shift_insert`: Insert or reposition an entry at a specific index.
- `swap_remove`: Remove efficiently by moving another entry into its place, changing order.
- `shift_remove`: Remove while maintaining relative order of the remaining entries.
- `sort_keys`: Explicitly sort a map's keys.
- `Value::sort_all_objects`: Recursively sort keys in nested JSON objects.

Important distinctions:

- `preserve_order` applies to `serde_json::Map`; it does not guarantee deterministic traversal of arbitrary Rust `HashMap` values serialized directly.
- JSON object member ordering is not a general semantic ordering guarantee of the JSON data model.
- Sorted keys alone do not establish canonical JSON. Numeric normalization, string escaping, encoding rules and other canonicalization requirements remain separate.
- Source ordering is preserved through appropriate `Value` parsing and serialization, not every possible intermediate transformation.
- With `preserve_order` enabled, `Map::remove()` uses swap-style removal. `shift_remove()` is the operation intended to retain relative ordering.&#x20;

  [image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

  Rust



Sources:

`https://docs.rs/serde_json/1.0.154/serde_json/map/struct.Map.html`

`https://docs.rs/serde_json/1.0.154/serde_json/value/enum.Value.html`

### 6.4 Recursion controls and deeply nested JSON

`serde_json` normally enforces a maximum nesting depth of 128.

The `unbounded_depth` feature exposes `Deserializer::disable_recursion_limit()`, allowing this check to be removed.

This does not make arbitrary recursion safe.

For deeply nested inputs, the upstream documentation explicitly identifies `serde_stacker` as an adapter that can dynamically grow the stack during deserialization.

There are several separate concerns:

- JSON parser nesting depth.
- Stack consumption during deserialization.
- Memory allocated for constructed values.
- Later recursive traversal, serialization, formatting and destruction.
- Overall payload size and computational cost.

Even if deserialization succeeds with a growing stack, dropping a deeply nested owned `Value` may still overflow the stack. The official documentation demonstrates a non-recursive destruction strategy for nested arrays.

The appropriate conclusion is that removing a parser depth limit requires additional stack and resource-management controls; it is not a general solution for unrestricted untrusted input.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

`https://docs.rs/serde_json/1.0.154/serde_json/struct.Deserializer.html`

`https://docs.rs/serde_stacker/latest/serde_stacker/`

### 6.5 JSON errors and detailed diagnostics

`serde_json::Error` provides structured error information:

| API           | Function                        |
| ------------- | ------------------------------- |
| `line()`      | One-based input line number     |
| `column()`    | Input column                    |
| `classify()`  | Error category                  |
| `is_io()`     | I/O failure                     |
| `is_syntax()` | Malformed JSON syntax           |
| `is_data()`   | Semantically incompatible value |
| `is_eof()`    | Unexpected end of input         |

Error categories:

- `Io`: Read/write failure.
- `Syntax`: Invalid JSON.
- `Data`: Type or semantic mismatch encountered during decoding.
- `Eof`: Premature end of input.

These categories are useful for distinguishing malformed data, incomplete streams, invalid types and transport failures.

A meaningful limitation is that serde_json's ordinary errors do not always identify the complete nested field path of the failing value. `serde_path_to_error` provides a wrapper to capture this information.

`serde_ignored` provides a different diagnostic function: recording the paths of fields that are silently ignored during otherwise successful deserialization.

Together, these enable more complete input diagnostics without replacing Serde's decoding machinery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

`https://docs.rs/serde_json/1.0.154/serde_json/struct.Error.html`

`https://docs.rs/serde_path_to_error/latest/serde_path_to_error/`

`https://docs.rs/serde_ignored/latest/serde_ignored/`

## 7. Build configurations and feature availability

Serde and serde_json have different constraints for `std`, allocation and optional functionality.

### 7.1 Serde feature flags

| Feature    | Default                | Capabilities and implications                                                            |
| ---------- | ---------------------- | ---------------------------------------------------------------------------------------- |
| `std`      | Enabled                | Standard-library collection and type support                                             |
| `alloc`    | Disabled independently | Heap-allocated types including `String`, `Vec`, `Box` and `Cow`, without requiring `std` |
| `derive`   | Disabled               | Re-exports procedural derives from `serde_derive`                                        |
| `rc`       | Disabled               | Implementations for `Rc` and `Arc`                                                       |
| `unstable` | Disabled               | Implementations or functionality requiring unstable Rust features                        |

Without `std` or `alloc`, Serde supports a smaller set of core types. Some derive mechanisms, notably deserializing untagged and certain tagged enums, need allocation.

The `rc` feature is worth distinguishing from graph serialization: it serializes the pointed-to contents, not object identities. Multiple references to the same `Rc` or `Arc` do not automatically become shared references after deserialization.

Cargo feature unification: Disabling default features in one dependency declaration does not disable features enabled by other packages in the dependency graph. This is particularly relevant to `no_std` builds.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde+1



Sources:

`https://serde.rs/feature-flags.html`

`https://serde.rs/no-std.html`

### 7.2 serde_json feature flags

| Feature               | Default                | Functionality                                                        |
| --------------------- | ---------------------- | -------------------------------------------------------------------- |
| `std`                 | Enabled                | Standard-library I/O and serializer interfaces                       |
| `alloc`               | Disabled independently | JSON functionality without the standard library, but with allocation |
| `arbitrary_precision` | Disabled               | Original numeric-literal representation in `Number`                  |
| `float_roundtrip`     | Disabled               | More accurate floating-point parsing for round-tripping              |
| `preserve_order`      | Disabled               | Insertion-order object maps using `IndexMap`                         |
| `raw_value`           | Disabled               | Unprocessed JSON subtrees                                            |
| `unbounded_depth`     | Disabled               | Ability to remove the deserializer nesting limit                     |

There is also an `indexmap` dependency feature used by `preserve_order`; normally the higher-level `preserve_order` feature is the intended selection.

The current 1.0.154 feature metadata shows that `preserve_order` activates `alloc`, `indexmap` and `foldhash` dependencies. This is relevant when distinguishing current feature requirements from older descriptions that coupled order preservation directly to `std`.

`serde_json` requires an allocator even in `no_std` configurations. For JSON operation without an allocator, upstream documentation points to the separate `serde-json-core` package.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Sources:

`https://docs.rs/crate/serde_json/1.0.154/features`

`https://docs.rs/serde_json/1.0.154/serde_json/`

## 8. Important companion libraries and integration boundaries

Serde has a substantial ecosystem of adapters. Several advanced capabilities that might otherwise motivate bespoke implementations are supplied by these packages rather than by `serde` or `serde_json` directly.

### 8.1 Representation and type adapters

| Library          | Key capabilities                                                                                                                     | When the distinction matters                                                                      |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| `serde_with`     | Composable `serde_as` adapters, string conversion, alternative collection layouts, defaults, timestamp representations, large arrays | Complex field representations without handwritten Serde trait implementations                     |
| `serde_bytes`    | Specialized bytes representation through `Bytes`, `ByteBuf`, `ByteArray` and `#[serde(with = "...")]`                                | Efficient native byte handling in compatible binary formats                                       |
| `serde_repr`     | Serialize/deserialize C-like enum numeric discriminants                                                                              | Numeric protocol enums rather than variant names                                                  |
| `serde-untagged` | Visitor-based alternatives for decoding untagged representations                                                                     | Avoiding some buffering and repeated trial-deserialization associated with derived untagged enums |

#### `serde_with` in greater depth

`serde_with` is among the most consequential Serde companion libraries.

Its `#[serde_as]` system is built around the `SerializeAs` and `DeserializeAs` traits. Unlike ordinary `#[serde(with = "...")]`, it supports composing adapters through nested containers.

A representation such as `Option<BTreeMap<String, Vec<T>>>` can apply custom conversion to individual nested values while preserving the outer collection structure.

Notable facilities include:

- `DisplayFromStr` for text-based representation of types supporting `Display` and `FromStr`.
- `PickFirst` for trying multiple accepted deserialization representations in priority order, while selecting one representation for serialization.
- `DefaultOnError` and related fallback adapters for explicitly tolerant decoding.
- Map and sequence representation conversions.
- Specialized timestamp, duration, numeric and string transformations.
- Large-array and const-generic-array support, including nested arrays.
- `skip_serializing_none` for applying optional-field omission policies across a structure.
- Helpers for structured strings, including separated lists.
- Integration with `schemars` to align some representation adapters with generated JSON Schema.

A notable core Serde limitation is that its built-in array implementations extend through length 32. `serde_with` provides a practical path for larger and const-generic arrays without requiring new custom serialization code.

`serde_as` is not universally interchangeable with Serde's `with` attribute: its adapter traits have their own supported-type boundaries, and variant-level support differs.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

`https://docs.rs/serde_with/latest/serde_with/`

`https://docs.rs/serde_with/latest/serde_with/guide/serde_as/`

`https://docs.rs/serde_bytes/latest/serde_bytes/`

`https://docs.rs/serde_repr/latest/serde_repr/`

`https://docs.rs/serde-untagged/latest/serde_untagged/`

### 8.2 Diagnostics, schema and validation

| Library               | Functionality                                                     | Boundary                                                     |
| --------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------ |
| `serde_path_to_error` | Track nested paths for serialization and deserialization failures | Augments errors; does not change the input schema            |
| `serde_ignored`       | Report ignored fields during deserialization                      | Diagnostics without requiring strict unknown-field rejection |
| `schemars`            | Derive JSON Schema from Rust types                                | Schema generation, not itself runtime schema validation      |
| `validator` / `garde` | Domain-level validation through companion mechanisms              | Semantic constraints beyond ordinary type decoding           |

Schemars is particularly useful when serialized Rust types also need a machine-readable external contract.

It recognizes many Serde attributes, including naming, tagging, flattening, defaults, and conversion representations, helping align generated JSON Schema with expected wire forms.

However, a custom serializer or deserializer can introduce behavior that schema derivation cannot infer automatically. Schema generation should therefore not be assumed to establish perfect equivalence between an arbitrary custom implementation and its generated schema.

Sources:

`https://docs.rs/serde_path_to_error/latest/serde_path_to_error/`

`https://docs.rs/serde_ignored/latest/serde_ignored/`

`https://docs.rs/schemars/latest/schemars/`

### 8.3 Type erasure and polymorphic serialization

Serde's fundamental traits use generic methods and are not directly object-safe.

Consequently, `dyn Serialize`, `dyn Serializer` and `dyn Deserializer` do not work like ordinary object-safe trait objects.

`erased-serde` provides object-safe equivalents of these interfaces. This enables runtime selection of serializer implementations and heterogeneous serialized values through trait objects.

`typetag` builds a higher-level polymorphic serialization/deserialization mechanism over this foundation, including support for boxed trait objects through registered implementations and tagged representations.

The distinction matters:

- Compile-time Serde derive naturally supports statically known enums.
- `erased-serde` supports type-erased serialization interfaces.
- `typetag` supports dynamic trait-object deserialization through registered concrete implementations.

Polymorphic deserialization is not merely type erasure; it also requires a reliable mechanism for identifying and constructing the appropriate concrete type.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Sources:

`https://docs.rs/erased-serde/latest/erased_serde/`

`https://docs.rs/typetag/latest/typetag/`

### 8.4 Format-to-format transcoding

`serde-transcode` converts data from a Serde `Deserializer` to a Serde `Serializer` without collecting the entire input into an intermediate object tree.

This can support:

- JSON → CBOR or other formats.
- Binary format → JSON.
- Compact JSON → pretty JSON.
- Streaming conversion between supported representations.

The key architectural distinction is between transcoding the data model and preserving the original source representation.

Transcoding can eliminate a complete intermediate tree, but it does not automatically preserve whitespace, original number spellings, object ordering or any information lost when the source is mapped into Serde's data model.

`serde_json::RawValue`, by contrast, preserves selected JSON subtrees textually but is JSON-specific.

Source: `https://docs.rs/serde-transcode/latest/serde_transcode/`

### 8.5 Specialized data formats

Serde's abstraction allows data formats to vary independently of the Rust types.

Notable ecosystem examples include:

| Format family | Representative crate                        | Relevant characteristics                                                   |
| ------------- | ------------------------------------------- | -------------------------------------------------------------------------- |
| JSON          | `serde_json`                                | Self-describing text, broad interoperability                               |
| CBOR          | `ciborium`                                  | Binary, self-describing representation with richer value types             |
| MessagePack   | `rmp-serde`                                 | Compact binary encoding                                                    |
| Postcard      | `postcard`                                  | Compact non-self-describing format, embedded/`no_std` emphasis             |
| TOML          | `toml`                                      | Human-readable configuration representation                                |
| YAML          | YAML implementations in the Serde ecosystem | Human-readable structured data with format-specific semantics              |
| RON           | `ron`                                       | Rust-oriented structured notation                                          |
| CSV           | `csv`                                       | Row-oriented/tabular serialization rather than arbitrary nested structures |

Supporting Serde traits does not mean all formats can represent all Rust structures or preserve the same semantics.

In particular, dynamic values, maps with complex keys, enum variants, numeric widths, bytes and nested collections may encounter format-specific limitations.

Source: `https://serde.rs/#data-formats`

## 9. Architectural combinations and tradeoffs

Several native mechanisms become especially valuable when combined.

| Problem shape                               | Relevant capabilities                                                              | What the combination can avoid                                                 |
| ------------------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Evolving structured messages                | Derived typed structs, defaults, aliases, tagged enums, flattened extension maps   | Repeated manual field-dispatch and compatibility-conversion code               |
| Typed envelopes with heterogeneous payloads | Typed fields, `RawValue`, selective deserialization                                | Constructing dynamic trees for opaque payloads                                 |
| High-volume structured input                | `from_slice`, borrowed fields, custom visitors, streaming deserialization          | Unnecessary ownership, intermediate collections and repeated input conversions |
| Direct construction of complex state        | `DeserializeSeed`, custom `MapAccess` / `SeqAccess` processing                     | Temporary fully materialized input models followed by separate reconstruction  |
| Complex external representations            | `serde_as`, `with`, `try_from`, remote derive                                      | Bespoke per-field serialization implementations                                |
| Strict or diagnosable input boundaries      | `deny_unknown_fields`, `serde_path_to_error`, `serde_ignored`, semantic validation | Independent JSON traversal solely for diagnostics                              |
| Polymorphic structured values               | Serde enums, `erased-serde`, `typetag`                                             | Bespoke runtime dispatch where the companion mechanisms fit                    |
| Format conversion                           | Serde format implementations, `serde-transcode`                                    | Intermediate JSON trees and manually copied values                             |
| Deterministic JSON output                   | Ordered collections, `Map::sort_keys`, serialization conventions                   | Some application-level key-sorting machinery, though not full canonicalization |

These are general architectural possibilities. Their value depends on representation requirements, source constraints, error guarantees and how much of the input must eventually be retained.

### 9.1 Information preservation is a separate concern from type safety

Serde's generic model and serde_json's dynamic model address different dimensions of correctness.

A representation can be type-correct while losing information from its source.

Examples include:

- Ignored unknown fields.
- Missing versus explicit-null distinctions.
- Repeated object keys.
- Source object ordering.
- Whitespace and original string escapes.
- Original numeric spellings.
- Out-of-range numeric values.
- Shared pointer identities.
- Format-specific enum representations.

Typed deserialization makes intended fields convenient and reliable to access, but does not establish lossless round-tripping by itself.

The appropriate mechanism depends on which information needs preservation: typed fields, `Value`, arbitrary-precision numbers, ordered maps, `RawValue`, or an entirely different source-preserving parser.

### 9.2 Representation correctness versus semantic validation

Serde's type-directed deserialization can check structure, recognized variants, required fields and primitive type compatibility.

It is not a general domain-validation framework.

For example, decoding a field into `String` does not verify that it contains a valid URL, a registered external identifier or a domain-specific grammar.

Mechanisms for enforcing stronger invariants include:

- Domain newtypes with `TryFrom`.
- `deserialize_with` or handwritten visitors.
- Custom `Deserialize` implementations.
- Companion validation crates.
- JSON Schema validation, when a formal external schema is required.

A useful architectural separation is to distinguish syntactic validity, structural compatibility, and domain validity rather than treating all successful deserialization as fully validated data.

### 9.3 Streaming versus ownership

The relevant distinctions are:

| Approach                        | Memory characteristics                                 | Principal limitation                                              |
| ------------------------------- | ------------------------------------------------------ | ----------------------------------------------------------------- |
| `from_slice` → owned struct     | Allocates according to destination structure           | Retains owned fields and collections                              |
| `from_slice` → borrowing struct | Can reference original input                           | Input must remain alive; not every token is borrowable            |
| `from_reader` → owned struct    | Input is processed incrementally                       | Destination can still grow to the full data size                  |
| `StreamDeserializer<T>`         | Processes records individually                         | Does not automatically stream elements inside each record         |
| Custom visitor                  | Can aggregate or dispatch nested elements directly     | Requires specialized deserialization logic                        |
| `DeserializeSeed`               | Can construct or update external state during decoding | Introduces state and partial-update considerations                |
| `RawValue`                      | Avoids materializing the selected subtree              | Requires retaining or owning the raw JSON text                    |
| `serde-transcode`               | Avoids a complete intermediate representation          | Cannot preserve information unsupported by the format abstraction |

No single approach is universally superior. A fully owned typed structure may be more efficient overall when it is repeatedly accessed after parsing, whereas raw preservation or streaming can be advantageous when most data is forwarded, discarded or processed once.

## 10. Performance characteristics and engineering limits

Serde's primary performance advantage is architectural: compile-time type specialization avoids the runtime-reflection machinery used by many serialization frameworks.

Specific mechanisms include:

Direct typed decoding. A derived deserializer can construct the destination type without first creating `serde_json::Value`. This removes an intermediate representation and its associated allocations.

Borrowed input. References into existing input can eliminate string allocation and copying where the format and token allow borrowing.

Streaming. Reader and visitor interfaces can avoid materializing data that will immediately be reduced, filtered or forwarded.

Raw preservation. `RawValue` can bypass full subtree construction and regeneration.

Writer-based output. Direct serialization into an `io::Write` can avoid allocating a complete string or byte vector. Writer buffering remains the caller's responsibility where applicable.

Format specialization. Binary formats can encode static structural information, numbers and byte buffers more compactly than JSON, depending on the particular format.

Precision options. Higher-precision numeric parsing can add meaningful computation; it should be viewed as a semantic choice, not a free optimization.

Enum selection. Untagged enum deserialization may be substantially more expensive than tagged dispatch when it must try multiple candidate variants.

Performance-oriented limitations:

- Serialization and deserialization are synchronous interfaces; serde_json does not itself provide a native asynchronous I/O API.
- Parsing from an I/O reader does not guarantee allocation-free decoding.
- Buffering, destination ownership and application transformations can dominate parser costs.
- Dynamic `Value` construction has different memory and dispatch characteristics from typed decoding.
- Custom visitors can reduce memory consumption while increasing implementation complexity.
- Writer serialization may have already emitted partial output when serialization or I/O fails; it does not provide transactional output semantics.
- Excessively deep data remains a stack and resource-management concern even with optional recursion controls removed.

The upstream serde_json documentation includes throughput estimates and a separate benchmark repository, but those figures should not be generalized to an unspecified workload. Data shape, string escaping, number parsing, destination types, allocation behavior and I/O materially affect observed results.

Sources:

`https://docs.rs/serde_json/1.0.154/serde_json/`

`https://github.com/serde-rs/json-benchmark`

`https://serde.rs/`

### 10.1 Limits especially important for machine-generated or untrusted JSON

| Concern                       | Available controls                                                   | Remaining responsibility                                                         |
| ----------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Syntax correctness            | serde_json parser                                                    | Application-level framing and transport handling                                 |
| Unknown fields                | `deny_unknown_fields`, `serde_ignored`                               | Selecting strict versus extensible schema behavior                               |
| Required fields               | Derived deserialization                                              | Defining appropriate defaults and domain invariants                              |
| Numeric precision             | `arbitrary_precision`, `float_roundtrip`, explicit destination types | Choosing acceptable arithmetic and representation semantics                      |
| Deep nesting                  | Default recursion limit, `serde_stacker`                             | Overall resource limits and later recursive operations                           |
| Duplicate object keys         | Derived struct detection in relevant cases; custom parsing           | Establishing a consistent rejection or conflict policy across dynamic maps       |
| JSON string and map semantics | Type-directed decoding and custom adapters                           | Ensuring external format compatibility                                           |
| Source preservation           | `RawValue`, ordered maps, numeric preservation                       | Determining what must be retained exactly                                        |
| Output integrity              | Fallible serializers and writers                                     | Atomic file replacement, framed transport or transaction management where needed |

## 11. High-value distinctions to retain

For an advanced programming agent, these distinctions are especially useful when evaluating Serde functionality:

1\. Serde is a protocol between Rust types and format implementations, not a parser or intermediate JSON tree. `serde_json` provides the JSON parser and dynamic tree.

2\. `Serialize` and `Deserialize` need not be symmetric. They can use different names, representations, skipped fields, defaults and transformations.

3\. Borrowed decoding is not the same as allocation-free parsing. It depends on input ownership, escaped content, destination types and lifetime constraints.

4\. `DeserializeSeed` is an architectural extension point for stateful decoding. It can construct final structures directly rather than requiring an intermediate materialization phase.

5\. `serde_json::Value` and `RawValue` solve different problems. The former provides structured dynamic manipulation; the latter preserves unprocessed JSON.

6\. `arbitrary_precision` and `float_roundtrip` are fundamentally different. One preserves numeric JSON representation; the other addresses floating-point parsing fidelity.

7\. Untagged enums trade explicit dispatch for representation flexibility. They introduce precedence, ambiguity and possible repeated parsing.

8\. `flatten` provides native schema extensibility but is not a conflict-resolution or schema-validation mechanism.

9\. Format independence has limits. JSON-specific map keys, raw literals, self-describing decoding and custom enum representations do not automatically transfer to arbitrary formats.

10\. Companion crates are an important part of the effective capability surface. In particular, `serde_with`, `serde_path_to_error`, `serde_ignored`, `serde-transcode`, `erased-serde` and `schemars` can eliminate substantial custom infrastructure.

## 12. Further primary-source reference map

The following sources provide the most useful starting points for detailed implementation investigation.

| Topic                          | Direct source                                                              |
| ------------------------------ | -------------------------------------------------------------------------- |
| Serde data model               | `https://serde.rs/data-model.html`                                         |
| Serialization implementation   | `https://serde.rs/impl-serialize.html`                                     |
| Deserialization implementation | `https://serde.rs/impl-deserialize.html`                                   |
| Derive attributes              | `https://serde.rs/attributes.html`                                         |
| Container attributes           | `https://serde.rs/container-attrs.html`                                    |
| Field attributes               | `https://serde.rs/field-attrs.html`                                        |
| Variant attributes             | `https://serde.rs/variant-attrs.html`                                      |
| Enum representations           | `https://serde.rs/enum-representations.html`                               |
| Borrowing and lifetimes        | `https://serde.rs/lifetimes.html`                                          |
| Remote derive                  | `https://serde.rs/remote-derive.html`                                      |
| Writing new data formats       | `https://serde.rs/data-format.html`                                        |
| Serde API                      | `https://docs.rs/serde/1.0.229/serde/`                                     |
| serde_json API                 | `https://docs.rs/serde_json/1.0.154/serde_json/`                           |
| JSON deserializer              | `https://docs.rs/serde_json/1.0.154/serde_json/struct.Deserializer.html`   |
| JSON dynamic values            | `https://docs.rs/serde_json/1.0.154/serde_json/enum.Value.html`            |
| JSON raw values                | `https://docs.rs/serde_json/1.0.154/serde_json/value/struct.RawValue.html` |
| JSON numbers                   | `https://docs.rs/serde_json/1.0.154/serde_json/value/struct.Number.html`   |
| JSON formatters                | `https://docs.rs/serde_json/1.0.154/serde_json/ser/trait.Formatter.html`   |
| Serde source                   | `https://github.com/serde-rs/serde`                                        |
| serde_json source              | `https://github.com/serde-rs/json`                                         |

### Overall architectural significance

Serde's value extends beyond converting structs to JSON. It provides a shared, compile-time-specialized representation protocol through which type systems, data formats, domain adapters, streaming processors and external-schema tooling can interact.

The most consequential advanced capabilities are its visitor and seed protocols, borrowed deserialization, declarative representation transformations, custom format implementations, and companion adapter ecosystem.

`serde_json` complements this with direct typed JSON parsing, an owned dynamic representation, input streaming, preserved raw subtrees, configurable numeric fidelity, object ordering and customizable output formatting.

Together, these enable a broad range of structured-data processing without requiring a bespoke JSON AST, serialization dispatcher, reflection system or general-purpose intermediate conversion pipeline—provided the desired semantics fit the distinctions and limits documented above.