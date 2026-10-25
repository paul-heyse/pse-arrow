# Serde, serde_json, and Schemars — Comprehensive Rust Capability Reference

Research date: October 24, 2026 | Current stable releases verified against docs.rs and upstream documentation

| Library    | Target version | Primary responsibility                                                        |
| ---------- | -------------- | ----------------------------------------------------------------------------- |
| Serde      | 1.0.229        | Generic, strongly typed serialization and deserialization framework           |
| serde_json | 1.0.151        | JSON parsing, serialization, transformation, streaming, and raw JSON handling |
| Schemars   | 1.2.3          | JSON Schema generation from Rust types, including Serde-aware wire contracts  |

Versions:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+2



Primary documentation:

- Serde: `https://serde.rs/`
- Serde API: `https://docs.rs/serde/1.0.229/serde/`
- serde_json API: `https://docs.rs/serde_json/1.0.151/serde_json/`
- Schemars guide: `https://graham.cool/schemars/`
- Schemars API: `https://docs.rs/schemars/1.2.3/schemars/`

## 1. Conceptual architecture

The three libraries form a complementary stack for defining, interpreting, producing, and describing structured data.

Rust types and domain models

Structs, enums, generics, newtypes, collections

Serde

Serialize / Deserialize traits

Schemars

JsonSchema trait

serde_json

Typed JSON, Value DOM, streaming, raw fragments

JSON Schema

Properties, constraints, variants, references, metadata

JSON bytes / text

Validators, documentation, APIs, code generators, external consumers

The central architectural distinction is that Serde defines the data-conversion protocol, serde_json implements a particular data format, and Schemars describes the externally visible data contract.

They do not duplicate one another.

### 1.1 Serde: a format-independent data model

Serde connects two independently extensible sides:

- Data structures implement `Serialize` and/or `Deserialize<'de>`.
- Formats implement `Serializer` and `Deserializer<'de>`.

The corresponding traits communicate through Serde's abstract data model rather than requiring every Rust type to understand JSON, CBOR, MessagePack, or another particular encoding.

Derive macros generate much of the required code at compile time. Serde's trait-based design ordinarily avoids runtime reflection and supports specialization through Rust's compiler optimizations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 1.2 serde_json: a concrete JSON execution engine

serde_json implements Serde's format-side interfaces and supplies functionality specific to JSON:

- Direct conversions between Rust types and JSON.
- A dynamically typed JSON value tree.
- Stream-oriented parsing and serialization.
- Borrowed raw JSON fragments and preserved JSON subdocuments.
- Configurable representation and parsing behavior.

It therefore supports several distinct processing architectures, ranging from direct typed decoding to selective processing of partially understood documents.

### 1.3 Schemars: a structural contract generator

Schemars introduces an independent `JsonSchema` trait and derive macro.

It can inspect the declared structure of Rust types at compile time and subsequently construct JSON Schema documents describing their expected JSON representations.

Importantly, Schemars understands many Serde attributes. Consequently, its output can describe the serialized or deserialized representation of a type rather than simply the Rust field names and types.

Schemars is not itself a runtime JSON Schema validation engine. It generates schema descriptions that other validators and consumers can interpret. Schema generation also does not automatically prove that custom runtime serialization behavior and emitted schemas are equivalent.

### 1.4 Capability boundaries

| Concern                                               | Library responsible                                |
| ----------------------------------------------------- | -------------------------------------------------- |
| Rust-to-format abstraction                            | Serde                                              |
| Derive-based conversion implementations               | `serde_derive`, exposed through Serde              |
| JSON parsing and output generation                    | serde_json                                         |
| JSON numeric and object representation                | serde_json                                         |
| Low-allocation borrowing                              | Serde's borrowing model, subject to format support |
| Arbitrary JSON document manipulation                  | serde_json                                         |
| JSON Schema generation                                | Schemars                                           |
| Serialization-aware schema generation                 | Schemars interpreting Serde annotations            |
| Structural JSON Schema validation                     | Separate validator                                 |
| Domain-specific validation and cross-field invariants | Application logic or validation libraries          |
| Human-readable API contracts and schema-based tooling | Schemars output plus consuming tools               |

The combined architectural opportunity: Rust types can provide a common foundation for compile-time type safety, runtime serialization, and machine-readable schema generation. This can substantially reduce duplicated definitions and conversion logic.

However, the three representations are not automatically identical in meaning. The most consequential gaps involve defaults, nullable values, custom transformations, enum matching, numerical precision, and validation semantics.

These distinctions are developed below.

Version correction: A newly indexed upstream release listing confirms that serde_json 1.0.154, published October 11, 2026, supersedes 1.0.151. The remaining analysis targets 1.0.154. Its release record is `https://docs.rs/crate/serde_json/1.0.154`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



## 2. Serde: the serialization and deserialization framework

### 2.1 The abstract data model

Serde's most important capability is the separation of a Rust type's structure from the format used to represent it.

```
pub trait Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}

pub trait Deserialize<'de>: Sized {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;
}
```

Illustrative signatures, simplified from the public traits.

The generic interfaces operate over a fixed Serde data model, including primitive values, strings, byte sequences, options, sequences, maps, structs, and several forms of enums.

A serializer can distinguish a tuple from a sequence or a struct from a map, even if the eventual format maps them to the same JSON representation. That distinction is particularly relevant to formats with positional encoding, tagged objects, or schema-dependent decoding.

This gives Serde two independent extension directions:

| Extension direction                | Mechanism                                           | What it enables                                                              |
| ---------------------------------- | --------------------------------------------------- | ---------------------------------------------------------------------------- |
| New Rust type                      | Implement `Serialize` / `Deserialize`               | Existing formats can process the type                                        |
| New data format                    | Implement `Serializer` / `Deserializer`             | Existing serializable types can use the format                               |
| New representation of a known type | Derive attributes, adapters, custom implementations | Alternate wire contracts without changing the underlying Rust representation |

Serde also handles conversion without requiring an intermediate `serde_json::Value`. This avoids obligatory construction of a dynamic value tree and permits a format implementation to process a sequence or object incrementally.

Limit: Format independence is not complete interchangeability. A Serde type using JSON-specific assumptions—particularly `deserialize_any`, dynamically shaped maps, or special representations—may not work in a non-self-describing binary format without adaptation.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde

+2



`https://serde.rs/data-model.html`

`https://serde.rs/impl-serialize.html`

`https://serde.rs/impl-deserialize.html`

### 2.2 Derive macros as declarative wire-contract definitions

`#[derive(Serialize, Deserialize)]` is substantially more than automatic struct-to-object conversion.

Serde's container, field, and enum-variant attributes express a configurable mapping between Rust types and serialized representations.

| Capability family             | Important attributes                                                               | Semantics                                                                                |
| ----------------------------- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Names and compatibility       | `rename`, `rename_all`, `rename_all_fields`, `alias`                               | External names, including independent read/write names and legacy aliases                |
| Presence and defaults         | `default`, `skip`, `skip_serializing`, `skip_deserializing`, `skip_serializing_if` | Controls which data is emitted, accepted, or synthesized                                 |
| Object structure              | `flatten`, `deny_unknown_fields`, `transparent`                                    | Flatten nested fields, reject surplus input, remove newtype wrapping                     |
| Representation conversion     | `with`, `serialize_with`, `deserialize_with`, `from`, `try_from`, `into`           | Delegate conversions or use intermediary types                                           |
| Generic and lifetime handling | `bound`, `borrow`                                                                  | Override generated trait bounds or allow borrowed deserialization                        |
| External types                | `remote`, `getter`                                                                 | Model the representation of types whose trait implementations cannot be derived directly |

Several attributes have architectural implications beyond simple formatting.

Asymmetric read/write contracts. `rename(serialize = "...", deserialize = "...")`, separate `skip_*` annotations, and intermediary `from`/`into` types allow the accepted input representation to differ from the emitted output representation. This is useful for migration, compatibility, and specialized interfaces, but means a single symmetric schema cannot always describe both directions accurately.

Compatibility aliases. `alias` accepts additional names during deserialization without changing the canonical serialized field name. It supports backward-compatible input evolution without proliferating duplicate Rust fields.

Defaults versus omissions. `#[serde(default)]` changes deserialization when a field is absent; `#[serde(skip_serializing_if = "...")]` changes serialization only. A field omitted on output does not automatically become optional on input. `Option<T>` is a special case: missing and explicit JSON `null` ordinarily both deserialize to `None`.

Transparent wrappers. `#[serde(transparent)]` serializes a suitable single-field struct as the wrapped field rather than as an object containing that field. This allows Rust-level newtypes to preserve stronger distinctions without requiring additional JSON structure.

Flattening. `#[serde(flatten)]` projects another struct's fields or a map's entries into the containing object. A flattened `Map<String, Value>` can also retain otherwise unmodeled fields. However, `flatten` is explicitly unsupported in combination with `deny_unknown_fields` on either the outer or flattened struct.

Unknown fields. Normal struct deserialization typically ignores unrecognized fields. `deny_unknown_fields` opts into rejecting them, providing stricter input shape checking, but is not a complete domain validator.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde

+2



`https://serde.rs/container-attrs.html`

`https://serde.rs/field-attrs.html`

`https://serde.rs/variant-attrs.html`

### 2.3 Enum representations: consequential wire-model choices

Serde supports four principal enum representations.

| Representation              | Example                              | Properties                                                                         |
| --------------------------- | ------------------------------------ | ---------------------------------------------------------------------------------- |
| Externally tagged (default) | `{"Created":{"id":7}}`               | Preserves explicit variant identity; supports all enum variant forms               |
| Internally tagged           | `{"type":"Created","id":7}`          | Discriminator is inside the object; suitable for object-oriented message protocols |
| Adjacently tagged           | `{"type":"Created","data":{"id":7}}` | Discriminator and payload are separate object fields                               |
| Untagged                    | `{"id":7}`                           | Variant inferred by attempting deserialization against each possible shape         |

The declarations are:

```
#[serde(tag = "type")]                 // internally tagged
#[serde(tag = "type", content = "data")] // adjacently tagged
#[serde(untagged)]                     // untagged
```

Tagged representations make variant identity explicit. They are generally easier to reason about when schemas, external clients, compatibility rules, or diagnostic tools need to distinguish message families.

Untagged representations can be useful for unions whose JSON shapes are externally determined or already standardized. However, Serde attempts variants in declaration order and accepts the first successful match. Overlapping variants can therefore create subtle, order-dependent semantics. Failure reporting can also be less informative, and repeated attempts can increase deserialization work.

Representation restrictions matter:

- Internally tagged enums do not support tuple variants.
- Internally tagged, adjacently tagged, and untagged enum deserialization require allocation support.
- Externally tagged enums are compatible with Serde's no-allocation environment.
- A discriminated Rust enum is not necessarily represented the same way in JSON and a binary Serde format.

In Schemars, these choices influence whether the generated schema uses explicit discriminator-like properties, alternatives such as `oneOf`/`anyOf`, or another structural encoding. The schema describes allowed shapes but does not necessarily reproduce Serde's first-success matching algorithm.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde



`https://serde.rs/enum-representations.html`

### 2.4 Custom serialization without bespoke format parsers

There are three useful levels of customization.

Field-level conversion delegates representation of an individual field through `serialize_with`, `deserialize_with`, or `with`. This is appropriate when a domain type should remain unchanged internally but must appear as a string, number, byte encoding, or another externally prescribed representation.

Container-level conversion uses `from`, `try_from`, or `into` to route an entire type through a wire-facing intermediary. `try_from` is particularly useful for enforcing structural invariants as part of conversion.

Trait-level implementations implement `Serialize` or `Deserialize` directly, using low-level visitor and serializer protocols. This supports shapes that derive attributes cannot express cleanly, including specialized positional forms, unusual union formats, incremental extraction, and stateful construction.

On serialization, a custom implementation can call `serialize_seq`, `serialize_map`, `serialize_struct`, or related serializer methods and emit items without first collecting them into a separate JSON tree.

On deserialization, `Visitor<'de>` receives typed events from `Deserializer<'de>`. `SeqAccess`, `MapAccess`, and `EnumAccess` supply incremental access to composite values.

The main tradeoff is that handwritten implementations become responsible for maintaining consistency between parsing behavior, emitted values, error handling, and any schemas generated elsewhere.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde

+1



### 2.5 Borrowing and ownership-aware deserialization

Serde's deserialization lifetimes are an important performance and API design capability.

`Deserialize<'de>` allows returned values to borrow from the input if the deserializer can expose suitable borrowed data. In contrast, `DeserializeOwned` requires the deserialized result to be independent of the input lifetime.

```
#[derive(serde::Deserialize)]
struct Event<'a> {
    kind: &'a str,
    #[serde(borrow)]
    comment: std::borrow::Cow<'a, str>,
}
```

With suitable input, a string can be referenced directly rather than copied into a fresh allocation.

This is conditional, not universal:

- `serde_json::from_str` and `from_slice` can support borrowed data.
- `serde_json::from_reader` requires an owned result because its I/O buffer cannot safely be borrowed by the returned value.
- JSON strings containing escape sequences require decoding; a borrowed `&str` is not always possible.
- `Cow<'de, str>` allows borrow-or-own behavior, with allocations when decoding or other transformations require ownership.
- Borrowed results cannot outlive the input buffer.

This distinction makes it possible to construct low-allocation parsing pipelines when input buffers have controlled lifetimes, while retaining ordinary owned DTOs at boundaries requiring independent ownership.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde



`https://serde.rs/lifetimes.html`

### 2.6 Stateful, selective, and format-level deserialization

Serde offers mechanisms that are less prominent than ordinary derive but useful in specialized parsers.

`DeserializeSeed` extends deserialization with caller-provided state. It can deserialize into existing containers, select portions of a stream, or use contextual information that a conventional stateless `Deserialize` implementation does not possess.

`IgnoredAny` consumes a value of arbitrary type without storing its contents. A custom visitor can combine `IgnoredAny` with sequence or map access to skip irrelevant substructures efficiently. This is distinct from first parsing an entire document into `Value` and then discarding unwanted portions.

`deserialize_any` allows a type to ask a self-describing format to determine the input's representation dynamically. It is useful for flexible JSON input but creates compatibility problems with formats that require the expected type in advance.

`IntoDeserializer` and value deserializers enable reuse of existing `Deserialize` implementations when the input is already available as Rust values rather than raw JSON or another byte format.

Together these allow parsers to be specialized around how much data is needed, how it is consumed, and whether external state is involved, without building an entirely separate deserialization framework.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



`https://docs.rs/serde/1.0.229/serde/de/trait.DeserializeSeed.html`

`https://docs.rs/serde/1.0.229/serde/de/struct.IgnoredAny.html`

### 2.7 External types, no_std, and crate boundaries

Serde's `remote` derive can define a local representation for a type in another crate even where Rust's orphan rules prevent implementing the Serde traits directly.

The remote definition is checked against the real type at compile time, subject to accessibility and construction restrictions. A consuming field can then use `#[serde(with = "RemoteTypeDef")]`.

This capability complements wrapper types and custom adapters: the former can model an external struct's structure; the latter can adapt a representation even when the structure cannot be mirrored.

Serde also supports constrained environments:

- With `std`, ordinary standard-library integrations are available.
- With `alloc` but not `std`, heap-backed collections remain available.
- Without either, implementations are restricted to functionality that does not require heap allocation.
- The `rc` feature supports `Rc<T>`/`Arc<T>` representations, but serialization does not preserve shared pointer identity across deserialization.

The crate structure is also relevant: `serde_core` contains the traits without derive support, while `serde_derive` implements the procedural macros. `serde` reexports the familiar public interface. For some dependency structures, using `serde_core` where derives are not needed can permit more parallel compilation rather than requiring all users to wait on procedural macro compilation.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde

+2



`https://serde.rs/remote-derive.html`

`https://serde.rs/feature-flags.html`

`https://serde.rs/no-std.html`

`https://docs.rs/serde_core/1.0.229/serde_core/`

## 3. serde_json: typed, dynamic, streaming, and raw JSON processing

`serde_json` provides several processing modes with materially different allocation, ownership, and extensibility characteristics.

### 3.1 Four processing modes

| Mode                         | Key interfaces                                         | Appropriate problem shape                                 |
| ---------------------------- | ------------------------------------------------------ | --------------------------------------------------------- |
| Direct typed decoding        | `from_str::<T>`, `from_slice::<T>`, `from_reader::<T>` | The expected structure is largely known                   |
| Dynamic document model       | `Value`, `Map`, `Number`, `json!`                      | Heterogeneous, exploratory, or structurally mutable JSON  |
| Incremental processing       | `Deserializer`, `StreamDeserializer`, `Serializer`     | Sequences of messages, controlled I/O, custom traversal   |
| Raw subdocument preservation | `RawValue`, `to_raw_value`                             | Opaque payloads, pass-through content, selective decoding |

These are not mutually exclusive. A typed struct can contain a `Value` for a flexible section or a `RawValue` for an opaque section.

This composition is often more efficient and semantically clearer than converting every payload into a fully dynamic document.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/serde_json/1.0.154/serde_json/`

### 3.2 Direct typed conversion

The main interfaces are:

```
serde_json::from_str::<T>(text)
serde_json::from_slice::<T>(bytes)
serde_json::from_reader::<R, T>(reader)

serde_json::to_string(&value)
serde_json::to_vec(&value)
serde_json::to_writer(writer, &value)
```

Pretty-printing variants and value conversion functions are also provided.

A key distinction is that `from_str`/`from_slice` can produce borrowed types, while `from_reader` requires `DeserializeOwned`.

Direct typed decoding can avoid the allocations, conversions, and dynamic dispatch-like branching associated with building and subsequently traversing a `Value` tree.

It also delegates field interpretation, required-field handling, enum matching, and custom conversion to the relevant `Deserialize` implementations.

That is not equivalent to arbitrary JSON Schema validation: the accepted language of a type is defined by its Serde implementation, not by an automatically enforced schema document.

### 3.3 The dynamic `Value` model

`serde_json::Value` represents JSON as:

```
enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

The dynamic model offers:

- `get`, `get_mut`, `as_object`, `as_array`, and typed `as_*` accessors.
- Object/array traversal and mutation.
- `json!` for constructing JSON values with Rust expressions.
- `Value::pointer` and `pointer_mut`, using JSON Pointer paths.
- `take`, object sorting, and conversion via `to_value` / `from_value`.

For example, a JSON Pointer such as `/components/0/name` addresses a nested array element; `~1` and `~0` encode `/` and `~` within pointer segments.

An important access distinction:

```
value.get("missing")  // Option<&Value>
value["missing"]      // &Value, yields JSON Null if missing
```

Indexing is convenient for exploratory code, but it can conflate an absent property with an explicitly present JSON `null`. When preserving that distinction matters, `get` or direct map access is more precise.

Also, `Value` is not an arbitrary Rust object graph: JSON objects have string keys, JSON arrays are ordered, and JSON values do not preserve Rust pointer identity.

Conversion to or from `Value` is useful for constructing dynamic schemas, merging documents, or dealing with extension attributes. It does introduce an intermediate representation and can lose details such as original whitespace and some lexical number information.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



`https://docs.rs/serde_json/1.0.154/serde_json/enum.Value.html`

`https://docs.rs/serde_json/1.0.154/serde_json/macro.json.html`

### 3.4 Streaming and incremental deserialization

`serde_json::Deserializer` can be constructed from a string, slice, or I/O reader. It can be used directly through Serde's deserialization trait or converted into a stream iterator.

```
let stream = serde_json::Deserializer::from_str(input)
    .into_iter::<Event>();

for event in stream {
    let event = event?;
    // Process this completed event.
}
```

Illustrative fragment; `Event` is an application-defined deserializable type.

`StreamDeserializer` processes consecutive JSON values. This supports newline-delimited JSON and other streams of individually delimited JSON values; newline framing itself is not mandatory.

Its `byte_offset()` method helps locate the successfully consumed prefix of an input buffer. On an EOF error, a caller can retain the remaining suffix, append incoming bytes, and retry. It is not a general-purpose incremental token-parser checkpoint; its documented contract concerns completed values and must be respected around errors.

Important difference: Streaming consecutive JSON values is not the same as iterating over the elements of a single enormous top-level JSON array without materializing the array. The latter calls for a suitable custom Serde visitor, another incremental parser, or a different framing convention.

`from_reader` also deserves care:

- It parses from a `Read` source without internally buffering the whole document.
- A `BufReader` is appropriate for sources where many small reads are inefficient.
- It expects the input stream to end after the JSON value.
- On a persistent socket, it can continue waiting for EOF; using a managed `Deserializer` avoids that requirement.
- Its upstream documentation explicitly notes that parsing from an already-loaded string or slice is often faster than direct reader parsing.

Therefore, direct reader parsing provides a memory/I/O architectural option, not an unconditional speed improvement.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



`https://docs.rs/serde_json/1.0.154/serde_json/de/struct.StreamDeserializer.html`

`https://docs.rs/serde_json/1.0.154/serde_json/de/fn.from_reader.html`

### 3.5 `RawValue`: selectively preserving JSON without materializing it

`serde_json::value::RawValue` is one of the crate's most valuable advanced capabilities.

When the `raw_value` feature is enabled, it represents a complete, syntactically valid JSON value as its original JSON text.

For example:

```
#[derive(serde::Deserialize)]
struct Envelope<'a> {
    id: u64,

    #[serde(borrow)]
    payload: &'a serde_json::value::RawValue,
}
```

A caller can deserialize the known `id` field while retaining `payload` without constructing its nested Rust representation.

Key properties:

- `&RawValue` borrows the JSON fragment from a string or byte-slice input.
- `Box<RawValue>` owns a raw fragment and supports input sources such as `from_reader`.
- Serialization emits the raw fragment directly rather than serializing it as an escaped string.
- Formatting inside the preserved fragment is retained, even when the surrounding document is serialized differently.
- `RawValue::from_string` validates an owned JSON fragment.
- `RawValue::from_string_unchecked` avoids validation but is `unsafe` and requires a single well-formed value with no surrounding whitespace.

This enables efficient message envelopes, selective inspection, opaque extension payloads, and pass-through transformations. It also preserves lexical details that `Value` normally normalizes.

The limitation is intentional: a raw fragment is not a parsed object. Querying nested fields, enforcing detailed constraints, or modifying its structure requires subsequent parsing or specialized processing.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/serde_json/1.0.154/serde_json/value/struct.RawValue.html`

### 3.6 Numeric representation and fidelity

JSON has a single syntactic number category, while Rust has distinct signed, unsigned, and floating-point types.

`serde_json::Number` bridges those representations, but the distinction affects correctness.

| Mode                  | Representation characteristics                                                                      | Important limitation                                                                   |
| --------------------- | --------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Default               | Integer values generally represented within `i64`/`u64`; other numeric input can use floating point | Large integers and high-precision decimals may not retain exact values through `Value` |
| `float_roundtrip`     | More precise floating-point parsing for round-trips                                                 | Adds parsing cost; does not create arbitrary-precision numeric arithmetic              |
| `arbitrary_precision` | Preserves JSON number literals within `Number`                                                      | Exact text preservation is not itself mathematical decimal semantics                   |

With `arbitrary_precision`, `Number` supports representations outside the default integer range, including conversion to `i128`/`u128` where the value fits.

`Number::from_f64` rejects nonfinite inputs such as NaN and infinity because they are not JSON numbers.

The engineering distinction is between:

1. Preserving numeric value within a chosen Rust type.
2. Preserving the original JSON number literal.
3. Performing exact arithmetic on arbitrary-precision decimal values.

These are separate requirements. `arbitrary_precision` helps with the second, but does not replace a decimal or arbitrary-precision arithmetic library.

For financial, scientific, and other precision-sensitive exchange, an explicitly chosen decimal, integer-scaled, or string-based wire representation may be more suitable than unrestricted conversion through `f64`.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



`https://docs.rs/serde_json/1.0.154/serde_json/value/struct.Number.html`

### 3.7 Object ordering and output customization

By default, `serde_json::Map` uses a `BTreeMap`, giving sorted-key iteration rather than original insertion order.

Enabling `preserve_order` changes that representation to an `IndexMap`-backed map.

This matters for:

- Keeping object members in their original order during parse/emit cycles.
- Producing more recognizable human-facing configuration output.
- Maintaining externally meaningful presentation order where consumers care about it.

However, JSON object order is not generally a semantic contract. `preserve_order` is not a JSON canonicalization mechanism, and some map removal operations can themselves affect order. The map API includes order-aware variants for operations where relative order matters.

On the output side, `Serializer::with_formatter` accepts an implementation of `serde_json::ser::Formatter`, allowing custom control over JSON formatting beyond compact or pretty output.

This is appropriate for specialized layout and emission requirements, but should not be confused with cryptographic canonical JSON. Canonicalization requires more rules than formatting or key sorting alone, including number and string normalization.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



`https://docs.rs/serde_json/1.0.154/serde_json/map/struct.Map.html`

`https://docs.rs/serde_json/1.0.154/serde_json/ser/trait.Formatter.html`

### 3.8 Error classification, limits, and resource behavior

`serde_json::Error` distinguishes:

- `Io`: underlying I/O failure.
- `Syntax`: invalid JSON syntax.
- `Data`: input inconsistent with the expected Rust representation.
- `Eof`: incomplete input.

It also exposes line and column information. For nested typed structures, the separate `serde_path_to_error` crate can add a field path to the failure, which often gives substantially better diagnostics than line/column alone.

The parser has a default nesting limit of 128 levels. The optional `unbounded_depth` feature permits disabling that limit, but doing so does not make recursive processing safe: deeply nested parsing, formatting, debugging, or dropping can still overflow the stack.

The upstream documentation identifies `serde_stacker` as a possible deserialization adapter and explicitly notes that subsequent operations on deeply nested values require separate care.

Finally, neither a successful parse nor conversion into `Value` guarantees application-level constraints. Payload sizes, array lengths, numeric ranges, accepted domain values, and structural consistency may still require explicit limits or validation.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



`https://docs.rs/serde_json/1.0.154/serde_json/struct.Error.html`

`https://docs.rs/serde_json/1.0.154/serde_json/de/struct.Deserializer.html`

### 3.9 Feature-specific capabilities

| Feature               | Primary effect                                        |
| --------------------- | ----------------------------------------------------- |
| `std` (default)       | Standard-library and I/O integrations                 |
| `alloc`               | Heap-backed operations without requiring all of `std` |
| `raw_value`           | Borrowed and owned raw JSON fragments                 |
| `preserve_order`      | Insertion-order JSON objects through `IndexMap`       |
| `arbitrary_precision` | Preservation of JSON number literals                  |
| `float_roundtrip`     | Higher-precision floating-point parsing behavior      |
| `unbounded_depth`     | Optional removal of parser recursion limit            |

`serde_json` supports `no_std` with an allocator, but not a general allocator-free configuration. `serde-json-core` is a separate crate for more constrained embedded use.

Cargo feature flags are additive across a dependency graph: enabling a feature in any dependency can change the effective compiled feature set.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+1



`https://docs.rs/crate/serde_json/1.0.154/features`

## 4. Schemars: JSON Schema as a derived Rust type contract

Schemars provides machine-readable descriptions of JSON representations using Rust types, Serde metadata, and optional schema-specific annotations.

It supports both ordinary JSON Schema generation and custom generation strategies for consumers with different schema dialects or compatibility requirements.

### 4.1 `JsonSchema`, derive, and root schema generation

The principal trait is:

```
pub trait JsonSchema {
    fn schema_name() -> std::borrow::Cow<'static, str>;
    fn json_schema(generator: &mut schemars::SchemaGenerator)
        -> schemars::Schema;

    // Additional provided methods:
    // inline_schema()
    // schema_id()
}
```

Simplified illustrative signature.

`#[derive(JsonSchema)]` supplies implementations for structs, enums, and generic types whose relevant fields also implement `JsonSchema`.

`schema_for!(T)` produces a root schema, including referenced definitions for nested reusable types.

For example:

```
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Command {
    request_id: String,

    #[schemars(length(min = 1, max = 64))]
    operation: String,

    #[serde(default)]
    payload: Option<serde_json::Value>,
}

let schema = schema_for!(Command);
let document = serde_json::to_string_pretty(&schema)?;
```

Illustrative code fragment; not executed.

This produces a schema describing the external camelCase fields, optional payload, closed object shape, and length annotation.

Crucial distinction: The `#[schemars(length(...))]` annotation affects the generated schema; it does not by itself cause `serde_json::from_str::<Command>` to reject an overlong string. Runtime validation requires another mechanism.

The default output currently targets JSON Schema Draft 2020-12. Nested types ordinarily become definitions under `$defs`, referenced using `$ref`.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



`https://docs.rs/schemars/1.2.3/schemars/trait.JsonSchema.html`

`https://graham.cool/schemars/`

### 4.2 Input versus output contracts

This is one of Schemars' most consequential and easily overlooked capabilities.

In general, a type's deserialization language is not identical to the language of values produced by its serialization implementation.

Schemars makes the distinction explicit using `SchemaSettings::contract`:

- `Contract::Deserialize`: describe accepted input. This is the default.
- `Contract::Serialize`: describe emitted output.

The convenience methods are `for_deserialize()` and `for_serialize()`.

```
use schemars::generate::SchemaSettings;

let input_schema = SchemaSettings::draft2020_12()
    .for_deserialize()
    .into_generator()
    .into_root_schema_for::<Command>();

let output_schema = SchemaSettings::draft2020_12()
    .for_serialize()
    .into_generator()
    .into_root_schema_for::<Command>();
```

The difference is observable with several attributes:

| Serde behavior                               | Input contract                              | Output contract                             |
| -------------------------------------------- | ------------------------------------------- | ------------------------------------------- |
| `rename(deserialize = ..., serialize = ...)` | Input property name                         | Output property name                        |
| `skip_deserializing`                         | Excluded from accepted input representation | May be present on output                    |
| `skip_serializing`                           | May be accepted on input                    | Excluded from output                        |
| `default`                                    | Missing value may be accepted               | Does not necessarily make output optional   |
| `skip_serializing_if`                        | Does not make input optional by itself      | Field may be omitted                        |
| `from` / `try_from`                          | Based on intermediary input representation  | Not necessarily affected                    |
| `into`                                       | Not necessarily affected                    | Based on intermediary output representation |

In some generated schemas, `readOnly` and `writeOnly` metadata are also used to describe direction-specific fields.

This distinction is especially important for generated clients, request/response APIs, configuration readers/writers, and data interchange systems with asymmetric compatibility guarantees.

A single “schema for a Rust type” is not always sufficient to represent both directions.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://graham.cool\&sz=32)

Schemars

+2



`https://graham.cool/schemars/generating/`

### 4.3 Serde-aware structural modeling

Schemars recognizes a substantial set of Serde attributes and incorporates their wire-format implications.

| Serde feature                                    | Schemars behavior                                                                   |
| ------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `rename`, `rename_all`, `rename_all_fields`      | Adjusts property and variant names                                                  |
| Enum tagging attributes                          | Describes externally, internally, adjacently tagged, or untagged shapes             |
| `default`                                        | Can remove a property from `required` and emit a JSON Schema default                |
| `skip`, `skip_serializing`, `skip_deserializing` | Modifies field/variant inclusion and direction-specific metadata                    |
| `flatten`                                        | Merges nested field schema structure                                                |
| `deny_unknown_fields`                            | Generates `additionalProperties: false`                                             |
| `transparent`                                    | Uses the schema of the wrapped field                                                |
| `from`, `try_from`, `into`                       | Reflects the appropriate conversion representation                                  |
| `with`                                           | Can substitute the declared JSON representation, subject to Schemars-specific rules |

This allows a Rust type's external naming, requiredness, and variant structure to be reflected in schema artifacts with relatively little duplication.

However, Schemars understands documented attributes, not arbitrary application behavior. A handwritten `Serialize` implementation is executable Rust logic and cannot generally be inferred from the type definition.

That becomes especially important when a field uses custom conversion modules.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://graham.cool\&sz=32)

Schemars

+1



`https://graham.cool/schemars/deriving/attributes/`

### 4.4 Schema-specific attributes and contract enrichment

Schemars introduces `#[schemars(...)]` annotations for metadata and constraints not represented directly by Serde.

| Family                 | Key attributes                            | Output capabilities                                |
| ---------------------- | ----------------------------------------- | -------------------------------------------------- |
| Documentation          | `title`, `description`, Rust doc comments | Human-readable schema annotations                  |
| Examples and lifecycle | `example`, `deprecated`                   | Examples, deprecation metadata                     |
| String constraints     | `length`, `pattern`, `regex`, `contains`  | Length constraints and patterns                    |
| Numeric constraints    | `range`                                   | Minimum and maximum constraints                    |
| Semantic formats       | `email`, `url`, `ip`, `ipv4`, `ipv6`      | JSON Schema `format` annotations                   |
| Nested collections     | `inner(...)`                              | Constraints on collection items                    |
| Presence               | `required`                                | Overrides optionality for supported fields         |
| Representation         | `with`, `schema_with`, `inline`           | Custom representation and reference behavior       |
| Schema extension       | `extend`, `transform`                     | Arbitrary keywords and post-generation adjustments |

The derive macro also recognizes supported `validator` and `garde` attributes, enabling some validation constraints declared for those libraries to be reflected in generated schemas.

This creates an opportunity to reuse constraint declarations across runtime validation and external schema generation.

But support is attribute-specific rather than a general translation of all validation logic. Cross-field relationships, custom functions, arbitrary predicates, and complex invariants may not have a direct JSON Schema counterpart.

Additionally, JSON Schema's `format` keyword may be an annotation rather than an enforced assertion depending on the validator and its configuration. Publishing a schema with `format: "email"` does not guarantee equivalent behavior in every consumer.

Schemars-specific attributes can override corresponding Serde attributes; an attribute can also be explicitly unset for schema generation using syntax such as `#[schemars(!from)]`.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://graham.cool\&sz=32)

Schemars



### 4.5 Custom representations: `with`, `schema_with`, and `JsonSchema`

Consider a field with a numeric Rust type that is emitted as a JSON string:

```
#[serde(with = "number_as_text")]
amount: u64,
```

Serde can serialize this according to the functions in `number_as_text`, but Schemars does not automatically know that the on-wire representation is a string.

The corresponding schema override can be:

```
#[serde(with = "number_as_text")]
#[schemars(with = "String")]
amount: u64,
```

There are several extension depths:

`#[schemars(with = "Type")]` tells Schemars to use another type's schema. This is suitable where the external representation is already expressible as another Rust type.

`#[schemars(schema_with = "...")]` delegates to a custom schema-generation function. It is appropriate for representation-specific definitions that simple type substitution cannot express.

A handwritten `JsonSchema` implementation provides full control over schema identity, structure, and how nested type definitions are generated.

This third level is particularly useful for domain newtypes whose valid value spaces are narrower than their underlying Rust representation, or for custom format mappings that need explicit schema keywords.

The decisive requirement is semantic consistency: a schema should describe values that the relevant serializer actually emits, or the deserializer actually accepts, for the chosen contract direction.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://graham.cool\&sz=32)

Schemars

+1



`https://graham.cool/schemars/examples/7-custom_serialization/`

### 4.6 Reusable schema definitions and recursive types

`SchemaGenerator` manages named definitions, references, and root schema construction.

Its significant methods include:

- `subschema_for::<T>()`: generate a schema or reference for a nested type.
- `root_schema_for::<T>()`: generate a root schema including referenced definitions.
- `into_root_schema_for::<T>()`: consume the generator to produce the root.
- `definitions()` / `definitions_mut()`: inspect or modify accumulated definitions.
- `take_definitions()`: extract accumulated definitions.
- `contract()` / `settings()`: inspect active generation configuration.

The `JsonSchema` trait separates human-facing names and structural identity:

- `schema_name()` determines a friendly schema name.
- `schema_id()` identifies a schema implementation for deduplication and reference management.
- `inline_schema()` controls whether a type is normally expanded inline.

For generic types, distinct instantiations that generate different schemas must have distinct identities. Poorly chosen IDs in manual implementations can otherwise cause definition reuse or collisions.

Named definitions permit compact schemas for types reused in many locations and are important for recursive models. Inlining may improve compatibility with consumers that have limited `$ref` support, but recursive relationships still require references.

This makes Schemars a reusable schema graph builder rather than simply a macro that prints independent field descriptions.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



`https://docs.rs/schemars/1.2.3/schemars/generate/struct.SchemaGenerator.html`

### 4.7 Schema dialects and generation settings

`SchemaSettings` supports several output targets:

| Constructor      | Target                                 |
| ---------------- | -------------------------------------- |
| `draft07()`      | JSON Schema Draft 7                    |
| `draft2019_09()` | JSON Schema Draft 2019-09              |
| `draft2020_12()` | JSON Schema Draft 2020-12              |
| `openapi3()`     | OpenAPI 3.0-compatible schema behavior |

The distinction between OpenAPI 3.0 and newer OpenAPI versions matters. OpenAPI 3.0's Schema Object is not fully interchangeable with JSON Schema 2020-12; the `openapi3()` setting specifically targets 3.0 semantics.

Other settings include:

- `definitions_path`: where reusable definitions and corresponding references are located.
- `meta_schema`: the declared meta-schema URI.
- `inline_subschemas`: whether schemas should normally be expanded rather than referenced.
- `untagged_enum_variant_titles`: whether generated untagged enum alternatives include variant titles.
- `contract`: input or output representation.
- `transforms`: modifications applied to generated schema documents.

For long-lived machine-readable contracts, explicitly selecting the intended dialect is more precise than relying on the default, which the upstream documentation states may change in a future release.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/schemars/1.2.3/schemars/generate/struct.SchemaSettings.html`

### 4.8 Schema transformations

Schemars 1.x provides a `Transform` trait for modifying generated schemas after their initial construction.

A transform can operate on a particular schema or recursively over nested schemas using `transform_subschemas` or `RecursiveTransform`.

The crate includes several built-in transformations:

| Transform                      | Purpose                                                 |
| ------------------------------ | ------------------------------------------------------- |
| `RemoveRefSiblings`            | Avoid schemas containing other keywords beside `$ref`   |
| `ReplaceBoolSchemas`           | Replace boolean schemas with equivalent object forms    |
| `ReplaceConstValue`            | Replace `const` with an equivalent single-value `enum`  |
| `ReplacePrefixItems`           | Adapt tuple-related keywords                            |
| `ReplaceUnevaluatedProperties` | Rewrite unsupported `unevaluatedProperties` constructs  |
| `RestrictFormats`              | Restrict or remove format values                        |
| `SetSingleExample`             | Convert multiple-example metadata into a single example |
| `AddNullable`                  | Add nullable metadata for schemas allowing `null`       |

These allow a canonical Rust-derived model to produce schema artifacts for consumers with different levels of dialect support.

For example, a client code generator that cannot understand particular draft keywords might be accommodated by a targeted schema transform instead of requiring a second independent set of Rust type declarations.

However, not all transforms are guaranteed to preserve every subtle validation meaning under every dialect. Transforming a schema for compatibility is different from proving semantic equivalence.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/schemars/1.2.3/schemars/transform/`

### 4.9 Additional schema-generation mechanisms

Value-based generation. `schema_for_value!` and `SchemaGenerator::root_schema_for_value` infer a schema from an existing serializable value. This can help when a type does not implement `JsonSchema`, but is less complete than type-derived generation. One observed enum variant cannot describe all variants, and one sample cannot establish the complete input domain.

Remote schema derivation. Schemars provides a remote-derive approach for external types, complementing Serde's remote-derive capability. This addresses orphan-rule restrictions without requiring the external crate to implement `JsonSchema` itself.

Numeric enum representations. For enums serialized as numeric discriminants through `serde_repr`, Schemars offers `JsonSchema_repr`. This can produce an integer enum schema instead of a string-valued enum schema.

Metadata from documentation. Rust doc comments can populate schema descriptions, providing a connection between source-level explanations and generated external contracts.

Arbitrary extensions. `#[schemars(extend(...))]` permits custom schema properties, including vendor extensions used by downstream documentation or code-generation tooling.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+3



`https://graham.cool/schemars/examples/5-remote_derive/`

`https://graham.cool/schemars/examples/8-enum_repr/`

### 4.10 Version 1.x architecture and compatibility considerations

Schemars 1.x introduced consequential changes relative to the older 0.8 generation model.

| Earlier Schemars 0.8 model                                    | Schemars 1.x                                                                             |
| ------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Structured types in `schemars::schema` such as `SchemaObject` | `schemars::Schema`, wrapping a JSON Schema-compatible `serde_json::Value` representation |
| Separate `RootSchema` type                                    | Root generation returns `Schema`                                                         |
| `visit::Visitor`                                              | `transform::Transform`                                                                   |
| `SchemaSettings::visitors`                                    | `SchemaSettings::transforms`                                                             |
| Older default Draft 7 behavior                                | Current default Draft 2020-12                                                            |
| Some unsuffixed dependency feature names                      | Version-qualified names such as `chrono04`, `uuid1`, `indexmap2`                         |

This distinction matters because many Rust examples on the internet still target Schemars 0.8. Code referencing `schemars::schema::SchemaObject` or `schemars::visit::Visitor` is not current 1.2.3 API.

Schemars also explicitly states that the exact emitted schema structure may change between library releases without being treated as a SemVer-breaking change. Consequently, schema text should not be assumed byte-stable merely because a library dependency remains on a compatible version range.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://graham.cool\&sz=32)

Schemars

+1



`https://graham.cool/schemars/migrating/`

### 4.11 Feature flags and external type support

Schemars' default features include `derive` and `std`. It supports reduced standard-library configurations with appropriate core/alloc types.

Important optional integrations include:

| Feature                                | External type family               |
| -------------------------------------- | ---------------------------------- |
| `chrono04`, `jiff02`                   | Date and time types                |
| `uuid1`                                | UUIDs                              |
| `url2`                                 | URL types                          |
| `bytes1`                               | Byte buffers                       |
| `indexmap2`, `smallvec1`, `arrayvec07` | Specialized collections            |
| `bigdecimal04`, `rust_decimal1`        | Decimal numeric types              |
| `semver1`                              | Versions                           |
| `either1`                              | Alternative/union types            |
| `raw_value`                            | `serde_json::value::RawValue`      |
| `preserve_order`                       | Retention of schema property order |

The version-number suffixes are significant: they identify the supported major versions of external crates.

The decimal integrations, in particular, allow Schemars to describe types with semantics beyond ordinary JSON floating-point numbers, although the exact wire representation still depends on the type's Serde implementation and enabled features.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



`https://docs.rs/crate/schemars/1.2.3/features`

## 5. Cross-library composition: advanced architectural patterns

The greatest value of these libraries comes from combining their capabilities without conflating their responsibilities.

### 5.1 A shared type model across execution and documentation

A Rust type can participate in three separate activities:

1. `Deserialize` defines how external data becomes a Rust value.
2. `Serialize` defines how a Rust value becomes external data.
3. `JsonSchema` defines a machine-readable description of one of those representations.

This makes it possible to use Rust type declarations as a common source for runtime conversion, generated documentation, external tooling contracts, and schema-driven integrations.

The key architectural benefit is reduced duplication of structural knowledge. Field names, enum variants, nesting, and much of the representation mapping need not be independently specified in each consumer.

The limitation is that type declarations are not a complete specification of arbitrary executable behavior. Custom validation and conversions may require explicit schema representations, and generated schemas may not enforce all invariants.

### 5.2 Missing, null, default, and required are different concepts

These distinctions are especially consequential for configuration files, partial updates, patches, and generated interfaces.

| Concept                 | JSON example           | Meaning                              |
| ----------------------- | ---------------------- | ------------------------------------ |
| Missing                 | `{}`                   | No property exists                   |
| Explicit null           | `{"value":null}`       | Property exists and contains null    |
| Present value           | `{"value":5}`          | Property exists and contains a value |
| Deserialization default | `#[serde(default)]`    | Rust supplies a value when missing   |
| Schema required         | `"required":["value"]` | Property must be present             |
| Schema default          | `"default":5`          | Annotation documenting a default     |

Serde's ordinary `Option<T>` handling conflates missing and explicit null.

Where all three input states matter, `serde_with::rust::double_option` supports `Option<Option<T>>` with:

- `None`: missing.
- `Some(None)`: explicit null.
- `Some(Some(value))`: present value.

The corresponding `default`, `skip_serializing_if`, and `with` attributes complete the field-level behavior.

This is a specialized capability of the companion `serde_with` library, not a default behavior of Serde's nested `Option` implementation.

Another important boundary: JSON Schema's `default` keyword does not itself populate missing values during validation. Serde deserialization defaults and JSON Schema default annotations are related but operationally separate mechanisms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



Sources:

`https://docs.rs/serde_with/latest/serde_with/rust/double_option/`

`https://json-schema.org/understanding-json-schema/reference/annotations`

### 5.3 Extensible records and unknown fields

There are several distinct strategies for dealing with evolving external objects.

| Strategy                    | Mechanism                     | Behavior                                                  |
| --------------------------- | ----------------------------- | --------------------------------------------------------- |
| Ignore unknown fields       | Serde default struct behavior | Accept unrecognized fields without retaining them         |
| Reject unknown fields       | `deny_unknown_fields`         | Fail when unexpected properties appear                    |
| Capture extra properties    | `#[serde(flatten)]` on a map  | Retain extras in a dedicated Rust field                   |
| Report ignored properties   | `serde_ignored`               | Track field paths that were discarded                     |
| Preserve ignored properties | `serde-ignored-fields`        | Retain ignored data through a wrapper and reserialization |

The difference between observing ignored fields and preserving them is important.

`serde_ignored` provides a callback when Serde ignores a field, useful for diagnostics or warnings.

`serde-ignored-fields` provides `PreserveIgnoredFields<T, U>`, which can retain ignored fields. It requires a suitable self-describing format and provides optional Schemars integration that forwards the modeled type's schema.

The flatten-map alternative is simpler for ordinary extensible objects, but is incompatible with `deny_unknown_fields` and may not match every nested representation or shape.

These provide multiple ways to support evolving documents without creating a parallel dynamic model for every operation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



Sources:

`https://docs.rs/serde_ignored/latest/serde_ignored/`

`https://docs.rs/serde-ignored-fields/latest/serde_ignored_fields/`

### 5.4 Complex representation adapters with `serde_with`

Serde's `with` attribute is useful for individual transformations, but more complicated nested representations can become awkward.

`serde_with` provides `serde_as`, introducing composable `SerializeAs` and `DeserializeAs` adapters.

For example, a representation can apply a `DisplayFromStr` conversion to the elements of a vector nested inside an optional map, without defining a new conversion function for the entire composite type.

Its functionality includes adapters for:

- String-based numeric and custom-type representations.
- Hexadecimal, Base64, and related encodings.
- Maps represented as sequences and vice versa.
- Duplicate-key and duplicate-element policies.
- Alternate null/default behavior.
- Nested container transformations.
- Dates, durations, timestamps, and selected external types.

An especially useful interaction is `serde_with`'s `schemars_1` feature.

With that feature enabled, the `serde_as` procedural macro can detect `JsonSchema` derives and generate corresponding `#[schemars(with = ...)]` annotations for adapted fields.

This reduces a common source of schema drift: changing the runtime wire representation through an adapter while forgetting to update the emitted schema.

This support is scoped to adapters and representations supported by the integration; it is not a proof of semantic equivalence for arbitrary custom conversions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



Sources:

`https://docs.rs/serde_with/latest/serde_with/guide/serde_as/`

`https://docs.rs/serde_with/latest/serde_with/attr.serde_as.html`

`https://docs.rs/serde_with/latest/serde_with/guide/feature_flags/`

### 5.5 Schema generation and runtime schema validation

Schemars produces schema documents; separate libraries interpret and enforce them.

A relevant companion is the Rust `jsonschema` crate.

It can:

- Compile and reuse validators.
- Validate arbitrary JSON instances.
- Report structured validation errors.
- Support multiple JSON Schema drafts.
- Register custom formats and keywords.
- Validate schema documents against their meta-schemas.
- Resolve external references, with configurable retrieval behavior.

This creates two distinct validation pathways.

| Validation pathway           | Strength                                                                   | Boundary                                                                |
| ---------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Direct Serde deserialization | Enforces Rust type structure and conversion logic                          | Does not automatically enforce all Schemars-only annotations            |
| JSON Schema validation       | Enforces supported schema constraints independently of Rust DTO conversion | May not exactly reproduce custom Serde conversions or domain invariants |

These approaches can complement one another when independent schema enforcement is valuable.

However, validating a JSON document against a generated schema and then deserializing it is not inherently necessary for every typed JSON operation. The extra validation may duplicate work, and the resulting guarantees depend on whether the generated schema precisely matches the intended runtime behavior.

A `jsonschema` validator also has its own draft and format-assertion settings. The library documents that some draft keywords are not fully implemented, so validator conformance and the intended schema dialect must be considered independently.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source:

`https://docs.rs/jsonschema/latest/jsonschema/`

### 5.6 Format transcoding without intermediate document models

The `serde-transcode` crate can connect a Serde deserializer directly to a Serde serializer, avoiding construction of a complete intermediate data model.

This supports operations such as:

- JSON to CBOR.
- JSON to another self-describing data format.
- JSON reformatting through a different serializer.
- Format conversion where application-level Rust DTOs are unnecessary.

The mechanism is distinct from `serde_json::Value` conversion. It forwards Serde data-model events rather than constructing a persistent tree.

It is generally suited to self-describing source formats, and conversion still depends on compatibility between the source's values and the destination format's representation constraints.

This can replace considerable bespoke parsing and serialization orchestration when the operation is fundamentally representational rather than semantic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://serde.rs\&sz=32)

Serde



Source:

`https://serde.rs/transcode.html`

## 6. Capability combinations and selection boundaries

The following table connects common operation shapes to native mechanisms, without prescribing a particular application architecture.

| Operation shape                            | Relevant combination                                             | Main distinction                                              |
| ------------------------------------------ | ---------------------------------------------------------------- | ------------------------------------------------------------- |
| Strict typed JSON ingestion                | Serde derive + serde_json direct parsing                         | Rust type conversion and accepted field policy                |
| Version-tolerant typed ingestion           | `alias`, `default`, tagged enums                                 | Compatibility without duplicate DTO structures                |
| Typed envelopes with unknown content       | Typed fields + `Value` or `RawValue`                             | Parsed flexibility versus opaque preservation                 |
| Low-allocation reads                       | `from_slice` + borrowed fields + `Cow`                           | Borrows when possible; input lifetime remains significant     |
| High-volume event streams                  | `StreamDeserializer` + typed messages                            | Multiple values, not automatic streaming of all nested arrays |
| Precise numeric exchange                   | `arbitrary_precision`, explicit wire types, decimal integrations | Literal fidelity and arithmetic exactness are separate        |
| Machine-readable contracts                 | Serde derives + Schemars `JsonSchema`                            | Representation-aware schema generation                        |
| Different request/response representations | Schemars input/output contracts                                  | A type can have asymmetric external behavior                  |
| Interoperable schema targets               | Schema settings + transforms                                     | Dialect adaptation without separate type models               |
| Composable custom wire formats             | `serde_with::serde_as` + `schemars_1`                            | Representation adapters with associated schema metadata       |
| Runtime schema enforcement                 | Schemars + external JSON Schema validator                        | Generation and validation are separate                        |
| Cross-format pass-through conversion       | Serde deserializer + `serde-transcode` + serializer              | No mandatory intermediate JSON DOM                            |

## 7. Cross-cutting limitations and guarantees

### 7.1 Schema fidelity is a relationship, not an automatic guarantee

There are three distinct questions:

1. Can a JSON value be deserialized into a particular Rust type?
2. Can that Rust type serialize into a particular JSON value?
3. Does a particular JSON Schema describe exactly the desired collection of values?

For straightforward derives, the three can align closely.

For customized conversions, they may differ substantially.

Examples include unordered or first-matching untagged enum alternatives, ignored unknown fields, schema-only range constraints, deserialization defaults, custom `try_from` validation, and serialized numeric representations.

Schemars provides hooks for expressing these distinctions, but cannot infer arbitrary application logic.

### 7.2 A type's validity is not necessarily fully described by JSON Schema

Rust types can encode invariants through constructors, visibility, conversions, and custom deserialization.

JSON Schema can express many structural constraints, but not every possible predicate or cross-field computational relationship.

Conversely, Schemars can describe constraints that a direct Serde deserializer does not enforce.

Consequently, the generated schema and the runtime type should be understood as connected but separate contracts.

### 7.3 Representation preservation is not semantic preservation

Converting JSON through `Value`, deserializing into typed objects, or transcoding between formats can change representation details even when ordinary data values appear equivalent.

Potential differences include key order, whitespace, numeric literals, variant representation, and discarded unknown fields.

`RawValue`, `preserve_order`, and `arbitrary_precision` address different subsets of these problems. None is a universal preserve-every-byte mechanism for an entire transformed document.

### 7.4 Performance follows the chosen representation

The most important qualitative differences are:

- Direct typed decoding avoids an obligatory intermediate dynamic tree.
- Borrowed deserialization can avoid copying strings and other borrowable data.
- `RawValue` avoids parsing and allocating nested opaque content.
- Streaming can bound peak memory relative to the size of a multi-message input.
- Value trees provide flexible random access and mutation but must materialize their contents.
- Schema generation is normally a separate activity from processing each JSON message and can be reused across many messages.
- Schema validation adds independent structural checking, with corresponding processing cost.

The fastest approach cannot be determined from these mechanisms alone; the relevant tradeoffs depend on the shape of data, amount of access needed, allocation behavior, and format guarantees.

### 7.5 Build and version boundaries

At the versions researched:

- Serde's core framework and derive procedural macros are separable; its upstream README documents a lower MSRV for the core crate than for `serde_derive`.
- serde_json requires allocation for its `no_std` mode and has feature-dependent numeric, ordering, and raw-value behavior.
- Schemars has version-qualified feature integrations for external crates.
- Schemars 1.x schemas must not be treated as API-equivalent to 0.8 schema types and visitors.
- Schemars does not guarantee that the exact JSON structure of its generated schemas stays unchanged across compatible crate releases.

These conditions are particularly relevant where reproducible generated artifacts, constrained targets, or compilation behavior are important.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+2



## 8. Additional ecosystem reference map

These are companion packages, not functionality built directly into the three requested crates.

| Library                | Distinct capability                                                       | Primary documentation                                               |
| ---------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `serde_with`           | Composable representation adapters and Schemars integration               | `https://docs.rs/serde_with/latest/serde_with/`                     |
| `serde_path_to_error`  | Nested paths for serialization/deserialization errors                     | `https://docs.rs/serde_path_to_error/latest/serde_path_to_error/`   |
| `serde_ignored`        | Reports silently ignored fields                                           | `https://docs.rs/serde_ignored/latest/serde_ignored/`               |
| `serde-ignored-fields` | Preserves unmodeled fields through deserialization                        | `https://docs.rs/serde-ignored-fields/latest/serde_ignored_fields/` |
| `serde_repr`           | Numeric enum discriminant serialization                                   | `https://docs.rs/serde_repr/latest/serde_repr/`                     |
| `serde-transcode`      | Format-to-format conversion without a full intermediate tree              | `https://docs.rs/serde-transcode/latest/serde_transcode/`           |
| `serde_stacker`        | Dynamically growing stack adaptation for deep deserialization             | `https://docs.rs/serde_stacker/latest/serde_stacker/`               |
| `jsonschema`           | Runtime JSON Schema validation and reference handling                     | `https://docs.rs/jsonschema/latest/jsonschema/`                     |
| `validator`            | Rust-level field validation with supported Schemars attribute translation | `https://docs.rs/validator/latest/validator/`                       |
| `garde`                | Rust-level validation with supported Schemars attribute translation       | `https://docs.rs/garde/latest/garde/`                               |
| `serde-json-core`      | JSON serialization/deserialization for constrained embedded environments  | `https://docs.rs/serde-json-core/latest/serde_json_core/`           |

## 9. Final architectural synthesis

The three libraries are best understood as distinct but composable layers of a structured-data system.

Serde is the type-to-representation abstraction. Its deepest capabilities are declarative wire modeling, independent serializer/deserializer implementations, borrowed decoding, custom visitor protocols, external-type adapters, and format-neutral extensibility.

serde_json is the concrete JSON processing engine. Its deepest capabilities are the choice between direct typed parsing and dynamic values, stream-level processing, preservation of opaque fragments, customizable numerical and ordering behavior, and low-level serializer/deserializer control.

Schemars is the schema-description layer. Its deepest capabilities are Serde-aware type-to-schema generation, explicitly different serialization/deserialization contracts, reusable recursive schema definitions, configurable dialect targets, and transformation hooks for downstream consumers.

The most valuable distinction is that these libraries allow runtime conversion, external representation, and machine-readable contract generation to share substantially the same Rust type definitions without forcing them to be the same mechanism.

That can eliminate considerable bespoke serialization infrastructure and duplicated structural definitions, while still leaving room for intentionally separate runtime validation, advanced transformations, and consumer-specific schema requirements.

The principal engineering responsibility that remains is ensuring semantic alignment where the abstractions do not guarantee it automatically—particularly custom serialization, validation, missing/null handling, numerical fidelity, unknown fields, and union matching.

Research note: This reference is based on upstream documentation and API references, not local execution or benchmarking. Companion crates are identified by capability, without assuming particular versions or availability in any reader's environment.