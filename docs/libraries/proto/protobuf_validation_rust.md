# Protobuf Validation in Rust: prost-protovalidate, Build-Time Validation, and Prost/Buffa Integration

Technical capability reference · October 31, 2026

Primary version scope: `prost-protovalidate` 0.6.0, `prost-protovalidate-build` 0.6.0, and `prost-protovalidate-types` 0.6.0. These are the latest published releases verified for the three-crate family, dated July 10, 2026.

Related implementation: `protovalidate-buffa` 0.7.2 (September 7, 2026), a separate validation ecosystem providing native Buffa code generation, including CEL compilation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+3



## 1. Executive orientation

The `prost-protovalidate` ecosystem implements the Protovalidate constraint model for Rust Protocol Buffers, using the standard `buf.validate` schema annotations.

It supports two fundamentally different validation execution models:

1. Descriptor-driven runtime validation: Interpret schema constraints using `prost-reflect`, including Common Expression Language (CEL) expressions, with compiled-rule caching and structured validation results.
2. Generated native Rust validation: Analyze protobuf descriptors at build time and generate `Validate` implementations that inspect message fields directly, eliminating runtime reflection and CEL interpretation for supported rules.

A third, hybrid mechanism enables Buffa-generated messages to use the same validation framework: supported constraints execute as generated Rust, while unsupported constraints—including CEL—can be delegated to a runtime validation bridge.

The distinction matters because protobuf validation need not require dynamic schema interpretation on every request. Schema-aware code generation can turn ordinary constraints into static Rust operations, while runtime interpretation remains available when constraints require substantially greater expressiveness.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 1.1 Crate architecture

| Component                                  | Principal responsibility                                                                   | Architectural role                                  |
| ------------------------------------------ | ------------------------------------------------------------------------------------------ | --------------------------------------------------- |
| `prost-protovalidate`                      | Runtime constraint engine, CEL evaluation, `Validate` trait, errors, validation helpers    | Execution and public validation interface           |
| `prost-protovalidate-build`                | Descriptor analysis and generation of native validation implementations                    | Build-time specialization                           |
| `prost-protovalidate-types`                | Generated `buf.validate` types, descriptor extension extraction, canonical rule metadata   | Shared schema and semantic definitions              |
| `prost` + `prost-reflect`                  | Message representations, encoding, runtime descriptors, dynamic messages                   | Underlying protobuf infrastructure                  |
| `buffa` + `buffa-build`                    | Alternative protobuf generation with different Rust representations and presence semantics | Alternative generated-message backend               |
| `protovalidate-buffa` + its codegen plugin | Dedicated Buffa validation implementation, including CEL-to-Rust compilation               | Separate, more extensive static-generation approach |

The first three crates belong to one project:

[https://github.com/zs-dima/prost-protovalidate](https://github.com/zs-dima/prost-protovalidate?utm_source=chatgpt.com)

The independent Buffa-native project is:

[https://github.com/mathematic-inc/protovalidate-buffa](https://github.com/mathematic-inc/protovalidate-buffa?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2

### 1.2 The central dataflow

`.proto` schema + `buf.validate` annotations

Canonical constraints, including standard rules and CEL

Protobuf descriptors / FileDescriptorSet

Field metadata, custom options, message relationships

Build-time compiler

`prost-protovalidate-build`

Generated Rust `Validate` implementations

Direct field access; Prost or Buffa

Runtime interpreter

`prost-protovalidate`

Descriptor-driven rules and CEL evaluation

Cached compiled constraints

Structured validation results

Field paths · Rule identifiers · Violation messages · Evaluation failures

The generated and dynamic paths are alternative implementations of the same schema constraints, but they are not interchangeable for every rule or execution requirement. Code generation has a deliberately restricted rule surface; runtime validation offers CEL and dynamic descriptor interpretation.

The shared `rules_meta` definitions help align rule identifiers and messages across both engines. This is useful for consistent validation diagnostics regardless of execution strategy.

## 2. The validation language: `buf.validate`

The three-crate family does not invent its own Rust validation annotations. It consumes the language-independent `buf.validate` protobuf options maintained by Buf.

That makes the `.proto` schema the source of validation semantics across different language implementations. Validation rules are attached to descriptors rather than embedded into a particular generated Rust struct layout.

### 2.1 Standard rule families

| Rule family         | Notable capabilities                                                                                         | Important semantics                                                                                 |
| ------------------- | ------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| Numeric and boolean | Equality, exclusions, comparisons, ranges, membership, finite-value checks                                   | Protobuf scalar defaults and presence determine when rules execute                                  |
| String              | Length, patterns, prefix/suffix, substring, membership, email, URI, hostname, IP, UUID, ULID, protobuf names | Format validators validate syntax, not domain existence or ownership                                |
| Bytes               | Size, patterns, prefix/suffix, inclusion/exclusion                                                           | Byte sequences are not necessarily valid UTF-8; pattern evaluation has special error considerations |
| Enum                | Specific values, allowed/forbidden sets, `defined_only`                                                      | Unknown numeric values can matter for open enums                                                    |
| Repeated            | Cardinality, `unique`, `items` constraints                                                                   | Individual items can be validated with recursively nested rules                                     |
| Map                 | Cardinality, `keys` and `values` constraints                                                                 | Diagnostics must identify whether the key or value failed                                           |
| Message and oneof   | Required fields, nested validation, oneof presence, group-level constraints                                  | Presence and branch selection have different meanings                                               |
| Well-known types    | `Any`, `Duration`, `Timestamp`, `FieldMask`, wrapper types                                                   | Specialized semantics exceed ordinary scalar checks                                                 |

These constraints can be composed. For example, a repeated string field can have a collection cardinality constraint and separate string rules applied to each item. A map can constrain its keys and values independently.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate+1



`https://protovalidate.com/schemas/standard-rules/`

### 2.2 Presence, defaults, and ignored fields

Presence is among the most consequential aspects of protobuf validation.

- Explicit-presence fields, such as proto3 `optional`, message fields, and oneof members, distinguish unset from set-to-default. Ordinary field constraints generally skip an unset field unless the rule requires its presence.
- Implicit-presence scalars cannot distinguish absent from a default value in the generated message representation. Validation operates on the represented value, including `0`, `false`, and `""`.
- `required` checks presence when presence exists. With implicit-presence scalars, it instead requires a non-default value.
- `IGNORE_ALWAYS` explicitly disables relevant field validation; `IGNORE_IF_ZERO_VALUE` skips rules when a field has its zero value.

These differences explain why `required: true`, `string.min_len: 1`, and an explicitly optional string are not interchangeable. They can accept or reject different states even when the Rust field appears to contain the same text.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate



Two oneof mechanisms also require distinction. A real protobuf `oneof` enforces at most one selected alternative in the data model; `(buf.validate.oneof).required = true` requires that an alternative be selected. The virtual oneof facility under message rules can impose one-of constraints over otherwise independent fields, including fields that are not members of a protobuf oneof.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate



`https://protovalidate.com/schemas/standard-rules/`

### 2.3 CEL: expression-based semantic constraints

Common Expression Language extends standard constraints to relationships that cannot be expressed as isolated field checks.

A field-level CEL rule evaluates `this` as the field value. A message-level rule evaluates `this` as the containing message and can inspect several fields.

CEL supports boolean logic, arithmetic, comparisons, collection operations, field presence tests, pattern matching, and Protobuf-aware values. In Protovalidate, a rule may return a boolean or a string: `true` or an empty string means success; `false` or a nonempty string means failure, with the nonempty string usable as a dynamic diagnostic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Protovalidate+1



A compact example illustrates both kinds of rule:

```

syntax = "proto3";

import "buf/validate/validate.proto";

message TransferRequest {
  uint64 amount = 1 [
    (buf.validate.field).uint64.gt = 0
  ];

  uint64 available = 2;

  option (buf.validate.message).cel = {
    id: "transfer.sufficient_funds"
    message: "amount exceeds available funds"
    expression: "this.amount <= this.available"
  };
}

```

The numeric constraint can be implemented as a direct Rust comparison. The message-level CEL expression introduces a dependency between fields, requiring either CEL evaluation or a compiler capable of translating that expression into Rust.

CEL has its own type system. In particular, protobuf unsigned integers become CEL `uint`, so literal `1u` and literal `1` are not equivalent in every comparison. Field presence tests such as `has(this.optional_field)` also depend on the original protobuf descriptor semantics rather than merely the generated Rust type.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Protovalidate



`https://buf.build/docs/protovalidate/cel/`

`https://protovalidate.com/schemas/custom-rules/`

### 2.4 Reusable predefined constraints

Protovalidate supports defining reusable, parameterized rules by extending standard rule messages and attaching `(buf.validate.predefined)` expressions.

The practical distinction from simply copying CEL across many fields is that the constraint acquires its own schema-level definition and parameters. A reusable rule can express a domain-specific convention while still being encoded as protobuf options and consumed through descriptors.

For Rust, this capability has two implications:

1. The runtime validator must be able to resolve the custom extension descriptors and compile their associated expressions.
2. The native `prost-protovalidate-build` generator cannot compile predefined CEL into direct Rust checks; such messages require a runtime path or another generator with CEL compilation support.

`ValidatorOption::AdditionalDescriptorSetBytes` is relevant when runtime resolution requires descriptors not already present in the validator's normal descriptor context.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate+2



`https://protovalidate.com/schemas/predefined-rules/`

A further architectural boundary is worth emphasizing: valid protobuf wire encoding does not imply that the represented message satisfies its validation rules. The validation options describe additional semantics; neither Prost nor Buffa automatically enforces them merely by decoding a message.

## 3. `prost-protovalidate`: descriptor-driven runtime validation

The runtime library is centered on `Validator`, which uses reflection metadata to discover constraints, compile their execution representations, and evaluate concrete messages.

### 3.1 Reflection and evaluator lifecycle

The primary public entry points are:

| API                             | Role                                                                            |
| ------------------------------- | ------------------------------------------------------------------------------- |
| `Validator::new()`              | Construct a reusable validator                                                  |
| `Validator::with_options(...)`  | Configure compilation, timing, descriptor registration, and validation behavior |
| `Validator::validate(&message)` | Validate a message implementing `prost_reflect::ReflectMessage`                 |
| `Validator::validate_with(...)` | Validate with per-call overrides                                                |
| `validate(&message)`            | Convenience validation using the library's shared validator                     |

The runtime engine converts message content into a representation suitable for `prost-reflect`, resolves rules from descriptors, and caches their compiled form. The cache is important because compiling expressions and preparing constraints can be much more expensive than evaluating an already-compiled rule set.

The default strategy is lazy compilation: a message's rules are prepared as needed. When descriptor registration is complete and validation must reject malformed schemas before accepting requests, eager compilation becomes relevant.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/struct.Validator.html`

### 3.2 Validator configuration

`ValidatorOption` and `ValidationOption` operate at different scopes: validator construction and individual validation calls.

| Configuration                       | Effect                                                                                                |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `FailFast`                          | Stop after the first violation instead of collecting all detectable violations                        |
| `DisableLazy`                       | Disable lazy preparation; useful with registered message descriptors for early constraint compilation |
| `MessageDescriptors(...)`           | Register message descriptors for precompilation                                                       |
| `AdditionalDescriptorSetBytes(...)` | Supply additional descriptor definitions, notably for custom extensions                               |
| `NowFn(...)`                        | Inject a clock source instead of depending on the wall clock                                          |
| `AllowUnknownFields`                | Permit unknown fields in constraint definitions, weakening strict schema validation for compatibility |
| `ValidationOption::Filter(...)`     | Decide which message, field, or oneof branches should be evaluated for a particular call              |

The clock injection facility is particularly useful for rules involving current time, such as `timestamp.lt_now`, `timestamp.gt_now`, and `timestamp.within`. It makes validation reproducible without modifying the schema or application message.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/enum.ValidatorOption.html`

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/enum.ValidationOption.html`

### 3.3 Selective validation through `Filter`

The runtime `Filter` trait provides granular control over which constraints are traversed. Its callbacks cover messages and, optionally, fields and oneofs:

- `should_validate`
- `should_validate_field`
- `should_validate_oneof`

This is more expressive than a global validation enabled/disabled flag. A caller can apply a common schema to different operations while selectively excluding portions of its constraints.

However, a filtered validation result means the selected constraints passed, not that the entire message satisfies its declared schema. The distinction is important when treating validation as a trust boundary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/trait.Filter.html`

### 3.4 Constraint compilation versus evaluation errors

The runtime separates three categories:

| Error              | Interpretation                                                                                 |
| ------------------ | ---------------------------------------------------------------------------------------------- |
| `ValidationError`  | The message was evaluated and one or more constraints failed                                   |
| `CompilationError` | Constraint definitions or CEL expressions could not be prepared successfully                   |
| `RuntimeError`     | Evaluation encountered a failure that cannot be represented as an ordinary violated constraint |

`Error` combines these categories for the descriptor-driven validator.

This distinction is essential for correct service behavior. An invalid client-provided value is not the same failure as an invalid server-owned validation schema. Exposing every error as `INVALID_ARGUMENT` would incorrectly attribute schema and evaluator defects to the caller.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.5 Structured violations and diagnostics

A `ValidationError` contains `Violation` records that expose:

- `field_path`: the location of the offending field, including nested structures;
- `rule_id`: the identifier of the failed validation rule;
- `rule_path`: the protobuf option location for that rule;
- `message`: the associated human-readable explanation;
- optional descriptor/value enrichment for the offending field and rule.

Generated validation implementations also create `Violation` instances, but they do not provide the same runtime reflection enrichment. Their field and rule descriptors or dynamic values are absent.

The library includes helpers for composing paths through nested messages, repeated indexes, and map keys, so generated checks can preserve the same structural location model as runtime validation. `Violation::to_proto()` supports conversion into the canonical protobuf violation representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/struct.Violation.html`

The structured error representation can serve as an interchange layer for diagnostics, API error responses, and tooling that needs to associate a violation with a specific schema constraint rather than parsing free-form messages.

## 4. `prost-protovalidate-build`: specializing constraints into Rust

`prost-protovalidate-build` is a build-time compiler that translates supported `buf.validate` constraints into generated implementations of:

```
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationError>;
}
```

Generated implementations use the concrete Rust message representation rather than inspecting a `DynamicMessage` during validation.

This is a significant architectural option: rule interpretation can move from request processing into the compilation pipeline. The resulting executable still validates input at runtime, but the validation logic is already expressed as Rust.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 4.1 Descriptor-based generation pipeline

The generator consumes a `FileDescriptorSet` produced by the protobuf compilation step. A typical Prost configuration is:

```

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path =
        std::path::PathBuf::from(std::env::var("OUT_DIR")?)
            .join("file_descriptor_set.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(&["proto/service.proto"], &["proto"])?;

    prost_protovalidate_build::Builder::new()
        .file_descriptor_set_path(&descriptor_path)?
        .compile()?;

    Ok(())
}

```

The generated validation implementations are then included alongside the generated message types:

```
include!(concat!(env!("OUT_DIR"), "/validate_impl.rs"));
```

This illustrates the essential dependency: message code generation and validation code generation must agree on descriptor identities and Rust type locations. The validation generator does not independently define the message types.

A straightforward dependency arrangement is:

```

[dependencies]
prost = "0.14"
prost-protovalidate = { version = "0.6", default-features = false }

[build-dependencies]
prost-build = "0.14"
prost-protovalidate-build = "0.6"

```

The example assumes the schema is eligible for native code generation. Where dynamic validation is also required, the runtime feature selection must change accordingly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`https://docs.rs/crate/prost-protovalidate-build/0.6.0`

### 4.2 Static rule coverage and advanced edge cases

The native generator handles substantially more than simple numeric or string field comparisons. Its published coverage includes:

- recursive validation of nested messages, repeated items, and map keys/values;
- oneof requirements and protobuf presence-aware behavior;
- wrapper-type unwrapping and specialized well-known-type checks;
- `Any.type_url` restrictions and `FieldMask` path-coverage semantics;
- time-relative timestamp rules, including `lt_now`, `gt_now`, and `within`;
- floating-point `repeated.unique` with canonicalized signed-zero handling and NaN-aware semantics;
- public reusable format helpers such as `validators::is_email`, `is_uuid`, and `is_ip_prefix`.

The generator's handling of `repeated.unique` is a useful illustration of why schema-aware generation is more than a mechanical translation of annotations. Ordinary Rust hashing does not directly capture Protovalidate's desired floating-point equality semantics. The implementation normalizes `+0.0` and `-0.0` and permits repeated NaNs according to the specified comparison behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



`https://github.com/zs-dima/prost-protovalidate/blob/main/CHANGELOG.md`

Time-dependent rules have a specific tradeoff. Generated validation samples the system clock once per invocation, whereas the runtime validator permits an injected `NowFn`. The generated `Validate::validate(&self)` method does not accept a clock context, so deterministic testing or alternate time semantics may favor the runtime engine.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 4.3 What happens when the generator cannot cover a message?

An especially important capability is the generator's analysis of whole-message eligibility, including nested dependencies.

A message with a CEL rule is not treated as if its ordinary constraints can be generated and the CEL rule silently ignored. Instead, the generator classifies the message as requiring runtime execution. If a containing message depends on a nested message that requires runtime validation, that dependency can also disqualify the container from entirely static validation.

The builder offers three policies:

| Policy                       | Behavior                                                                                                       | Availability    |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------- | --------------- |
| Default                      | Skip generating the affected implementation and emit a `cargo:warning` explaining the runtime-only requirement | Prost and Buffa |
| `fail_on_runtime_only(true)` | Fail the build with `Error::RuntimeOnly`                                                                       | Prost and Buffa |
| `runtime_bridge(true)`       | Generate a `Validate` implementation that delegates the affected message to the runtime validator              | Buffa only      |

The default policy is diagnostic, not automatically fail-closed: an application must actually invoke the runtime validator for messages that lack a generated implementation.

`fail_on_runtime_only(true)` is therefore an important build-time completeness control. It makes unsupported rule coverage a compilation failure rather than something dependent on developers noticing Cargo warnings.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



### 4.4 The Buffa runtime bridge

Version 0.6.0 introduces `Builder::runtime_bridge(true)` for the Buffa backend.

```

prost_protovalidate_build::Builder::new()
    .file_descriptor_set_path(&descriptor_path)?
    .backend(prost_protovalidate_build::Backend::Buffa)
    .runtime_bridge(true)
    .compile()?;

```

When a message needs CEL or another runtime-only mechanism, the generated `Validate` implementation:

1. Serializes the Buffa message into protobuf wire bytes.
2. Uses an embedded `FileDescriptorSet` to resolve the message type.
3. Decodes the bytes into the runtime reflection representation.
4. Applies a shared, cached `Validator`.
5. Converts the result into the generated `Validate` interface.

The bridge exposes `RuntimeBridge::from_fds` and `RuntimeBridge::validate_wire`. Its descriptor pool and evaluator are constructed once and reused; generated code normally keeps the bridge in a lazy static.

This approach provides broad coverage without implementing an independent CEL evaluator for Buffa. The cost is that bridged messages incur serialization, dynamic decoding, and interpretation. Messages eligible for direct generation continue to use their native checks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/bridge/struct.RuntimeBridge.html`

Two constraints are material:

- `runtime_bridge(true)` and `fail_on_runtime_only(true)` are mutually exclusive.
- The generated bridge embeds descriptor bytes using `include_bytes!` relative to `OUT_DIR`. An incompatible output-directory override can break the generated inclusion.

The bridge also introduces an error-category boundary: `validate_wire` returns the runtime's full `Error`, distinguishing ordinary violations, compilation failures, and evaluation failures. Generated `Validate` implementations collapse that result to `ValidationError` because it is the trait's return type. Callers requiring the original runtime error categories should use the lower-level bridge API rather than assuming the trait preserves all distinctions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.5 Feature tiers and dependency footprint

`prost-protovalidate` 0.6.0 supports three useful compilation profiles.

| Runtime dependency configuration                   | Available capabilities                                                                                        |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `prost-protovalidate = "0.6"`                      | Full descriptor-driven runtime, CEL, generated validation support                                             |
| `default-features = false, features = ["reflect"]` | Runtime reflection and standard rules, but no CEL interpreter                                                 |
| `default-features = false`                         | Slim library with generated `Validate` support, errors, and reusable validation helpers; no reflection engine |

The middle tier is more than a dependency optimization. If a message references CEL but the `cel` feature is disabled, the validator returns `CompilationError`; it does not treat the rule as satisfied.

The slim tier retains the common `Validate` trait and violation structures while omitting the dynamic machinery. Optional `tonic` and `tonic-types` features add transport-specific integration rather than changing the underlying constraint language.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`https://docs.rs/crate/prost-protovalidate/0.6.0/features`

## 5. `prost-protovalidate-types`: schema and descriptor infrastructure

The `types` crate is a shared semantic layer, not a second validation engine.

It contains Rust representations of the `buf.validate` schema itself, plus utilities for retrieving constraint annotations from protobuf descriptors.

### 5.1 Generated constraint model

Notable public types include:

- `FieldRules`, with its type-specific rule variants;
- `MessageRules`, `OneofRules`, and `PredefinedRules`;
- numeric, string, bytes, enum, repeated, map, and well-known-type rule messages;
- `Rule`, describing a CEL constraint;
- `Ignore`, representing skip semantics;
- `FieldPath`, `FieldPathElement`, `Violation`, and `Violations`, representing canonical protobuf diagnostics.

These are the protobuf schema types, not application-domain message types. This distinction matters because their purpose is to describe and transport validation metadata independently of any particular application schema.

The package is re-exported through `prost_protovalidate::types`, so a separate dependency is often unnecessary when the runtime library is already present.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



`https://docs.rs/crate/prost-protovalidate-types/0.6.0`

### 5.2 Typed descriptor extension extraction

With the `reflect` feature enabled, the crate exposes `DESCRIPTOR_POOL` and extension-resolution APIs for discovering annotations.

Important interfaces include:

| Interface                                             | Purpose                                                           |
| ----------------------------------------------------- | ----------------------------------------------------------------- |
| `FieldConstraintsExt`                                 | Read `buf.validate.field` constraints from a field descriptor     |
| `MessageConstraintsExt`                               | Read `buf.validate.message` constraints from a message descriptor |
| `OneofConstraintsExt`                                 | Read oneof validation constraints                                 |
| `PredefinedConstraintsExt`                            | Resolve predefined rule metadata                                  |
| `FieldConstraintsDynExt` / `MessageConstraintsDynExt` | Access raw dynamic constraint representations                     |
| `field_constraints_typed` and corresponding helpers   | Extract typed rule structures with explicit decode errors         |

The static `DESCRIPTOR_POOL` supplies reflection metadata for the validation schema and its extensions. It is not, by itself, a complete descriptor registry for every application message a program might validate.

This separation makes the crate useful for descriptor-processing tools as well as validators: a schema analyzer can inspect declared constraints without needing to execute them.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.3 `rules_meta`: shared rule identity and diagnostic semantics

A less obvious part of the crate is `rules_meta`, introduced before version 0.6.0.

It centralizes canonical rule identifiers, violation-message templates, and combined-range metadata. Both the dynamic engine and native generator consume these definitions.

This reduces a typical problem in systems with multiple validation backends: independent implementations returning different rule identifiers or error messages for the same schema constraint.

The separation also makes it possible for validation tooling to reuse common rule metadata rather than recreating a private mapping of rule identifiers.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 6. Prost and Buffa: two message representation models

### 6.1 Prost support

The principal runtime path is built around:

- `prost::Message` for protobuf binary encoding and decoding;
- `prost-reflect::ReflectMessage` for associating a concrete Rust message with its descriptor;
- `prost_reflect::DynamicMessage` and descriptor metadata for interpreting constraints;
- `prost-build` for generating application Rust types and the descriptor set.

These are distinct traits and capabilities. Implementing `prost::Message` alone is not equivalent to providing reflection metadata.

The native build-time generator eliminates reflection from the normal validation hot path for eligible types. In contrast, descriptor-driven validation can operate over schemas and constraint expressions without a separate generated validator for each message.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 6.2 Buffa support in `prost-protovalidate-build`

Buffa is a separate Rust protobuf implementation with a different generated representation and message trait. The 0.5.0 release of the Prost-Protovalidate family added `Backend::Buffa` to accommodate those differences explicitly.

The generator handles several representation-level distinctions:

| Concern                  | Prost target                        | Buffa target                                                       |
| ------------------------ | ----------------------------------- | ------------------------------------------------------------------ |
| Message field presence   | Commonly represented by `Option<T>` | Uses `MessageField<T>` and presence methods                        |
| Enum values              | Prost-generated enum representation | Buffa's `EnumValue` representation, normalized through `to_i32`    |
| Generated type naming    | Prost naming conventions            | Preserves Buffa conventions, including names such as `UUID`        |
| Module resolution        | Prost generated module paths        | Descriptor-derived Buffa paths, including nested and package cases |
| Runtime reflective trait | `prost_reflect::ReflectMessage`     | Buffa's separate reflection model; not a direct substitute         |

These accommodations are necessary because protobuf wire compatibility does not imply Rust source-level or trait-level compatibility.

The Buffa generator can consume descriptor data from `buffa-build` or a wrapper such as `connectrpc-build`. What matters is matching the schema, code-generation naming rules, and generated type paths.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`https://github.com/anthropics/buffa/blob/main/docs/guide.md`

### 6.3 The separate `protovalidate-buffa` ecosystem

There is another Rust validation implementation that merits explicit comparison:

`protovalidate-buffa` 0.7.2, paired with `protoc-gen-protovalidate-buffa`.

It is not a renamed or dependent form of `prost-protovalidate-build`. It uses a different compilation architecture: CEL expressions are translated into native Rust at code-generation time, not sent to a runtime CEL interpreter.

Its components are:

| Crate/tool                       | Function                                                                               |
| -------------------------------- | -------------------------------------------------------------------------------------- |
| `protovalidate-buffa`            | `Validate` trait, structured errors, CEL-generated-code helpers, standard rule helpers |
| `protoc-gen-protovalidate-buffa` | Protobuf compiler plugin that emits validation implementations                         |
| `protovalidate-buffa-macros`     | ConnectRPC request-validation attribute macro                                          |
| `protovalidate-buffa-protos`     | Buffa-generated representations of `buf.validate` protobuf types                       |

The plugin reads custom options using Buffa's extension machinery. It can compile both standard constraints and custom CEL into code referencing the generated Buffa representation.

An additional distinction is borrowed-message validation. Buffa exposes generated zero-copy views, and this generator emits validation for views as well as owned messages. That can avoid converting a borrowed incoming request into an owned Rust message solely to run validation.

With ConnectRPC, the `#[protovalidate_buffa::connect_impl]` macro can insert validation at service-handler entry, using the request view. It validates incoming requests, not automatically outgoing responses.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://github.com/mathematic-inc/protovalidate-buffa`

`https://docs.rs/protovalidate-buffa/0.7.2/protovalidate_buffa/`

### 6.4 Full native CEL compilation versus hybrid bridging

| Dimension                      | `prost-protovalidate-build` with Buffa                                                           | `protovalidate-buffa`                                              |
| ------------------------------ | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| Standard constraints           | Generated direct Rust                                                                            | Generated direct Rust                                              |
| Custom CEL                     | Runtime interpreter through bridge                                                               | Compiled into native Rust                                          |
| Unsupported static rule shapes | Runtime bridge or build failure                                                                  | Dedicated compilation/conformance implementation                   |
| Generated `Validate` trait     | From `prost-protovalidate`                                                                       | From `protovalidate-buffa`                                         |
| Borrowed Buffa views           | No equivalent generated-view support established by the reviewed sources                         | Explicitly supported                                               |
| Runtime descriptors            | Required for bridged messages                                                                    | Not required for ordinary generated validation                     |
| Transport integration          | Tonic helpers in Prost-Protovalidate; Buffa bridge can be used with other transport integrations | Native ConnectRPC adapter and handler macro                        |
| Error categories               | Full dynamic error types; generated trait has narrower error type                                | Explicit `violations`, `compile_error`, and `runtime_error` fields |

The dedicated Buffa implementation reports passing all 2,872 cases in its pinned Protovalidate conformance harness, including proto2, proto3, and editions 2023. That is a useful compatibility claim, although it is not a benchmark or a guarantee concerning every possible future CEL or protobuf feature.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



The two projects' `Validate` traits are distinct and should not be treated as interchangeable merely because they expose the same method name.

Compatibility note: the dedicated Buffa project's published documentation targets Buffa 0.9.1, ConnectRPC 0.9, and Rust 1.88 or newer. Because Buffa remains pre-1.0, its generated type layout can change between minor versions; the validation plugin, message generator, and ConnectRPC dependency need to remain aligned.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 7. RPC integration and structured error transport

Validation can execute at a service boundary, but neither the protobuf schema nor the validation crates automatically guarantee that every incoming request is checked.

### 7.1 Tonic integration

The `tonic` feature of `prost-protovalidate` provides a `ValidateRequest` extension:

```
use prost_protovalidate::tonic::ValidateRequest;

// Within a Tonic handler:
request.validate_inner()?;
```

This supports handlers receiving an ordinary `tonic::Request<T>`, including unary requests and the initial request for a server-streaming response.

For client-streaming and bidirectional RPCs, the individual messages within `tonic::Streaming<T>` must be validated as they are received. Validating the stream wrapper is not equivalent to validating its contents.

The transport error mapping distinguishes:

- `ValidationError` → `InvalidArgument`.
- `CompilationError` and `RuntimeError` → `Internal`, with fixed messages that avoid exposing evaluator internals.

The optional `tonic-types` feature adds `google.rpc.BadRequest` details, providing individual field violations in a conventional gRPC error representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`https://github.com/zs-dima/prost-protovalidate/blob/main/README.md`

### 7.2 Canonical Protovalidate errors versus `BadRequest`

`google.rpc.BadRequest` and `buf.validate.Violations` are different structured error representations.

The former emphasizes a textual field identifier and description. The latter can retain richer protobuf structure: rule identifiers, field paths, rule paths, repeated indexes, map-key selectors, and whether a failed rule applies to a map key.

This distinction becomes consequential when errors are consumed programmatically rather than merely displayed to a person.

The separate `protovalidate-buffa` implementation uses canonical `buf.validate.Violations` details in its ConnectRPC integration. It also implements explicit protections against leaking rejected input into public diagnostics:

- Free-form violation messages and map-key values are omitted from public details by default.
- The encoded detail payload is limited to 4 KiB.
- Only whole violations fitting within that budget are included.
- Clients must therefore regard the returned violation collection as potentially incomplete.

Its `decode_violations` helper decodes those details from Connect or gRPC errors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`https://github.com/mathematic-inc/protovalidate-buffa`

The two error formats can be converted or interpreted by application code, but doing so may lose information. Neither Tonic nor ConnectRPC inherently dictates that all requests and responses be validated.

## 8. Performance model and evidence

The relevant performance distinction is not simply “reflection is slow, generated code is fast.” There are multiple costs with different lifecycles.

| Cost                   | Runtime validator                                | Generated Prost/Buffa validation                            | Buffa runtime bridge                             |
| ---------------------- | ------------------------------------------------ | ----------------------------------------------------------- | ------------------------------------------------ |
| Schema analysis        | Lazy or eagerly cached                           | Build time                                                  | Build time plus cached runtime preparation       |
| CEL compilation        | Runtime, cached                                  | Not supported by `prost-protovalidate-build`                | Runtime, cached                                  |
| CEL evaluation         | Runtime interpreter                              | Not applicable for unsupported CEL                          | Runtime interpreter                              |
| Field access           | Reflection-oriented                              | Concrete generated Rust access                              | Dynamic representation after wire conversion     |
| Message conversion     | May involve conversion to dynamic representation | No reflection conversion                                    | Buffa serialization followed by dynamic decoding |
| Rule metadata          | Retained for reflection                          | Encoded into generated checks                               | Embedded descriptors for dynamic branch          |
| Binary/build footprint | Reflection and evaluator dependencies            | More generated code; potentially fewer runtime dependencies | Generated code plus dynamic runtime machinery    |

### 8.1 Published runtime benchmarks

The project publishes a Criterion baseline from May 16, 2026 using:

`cargo bench -p prost-protovalidate --all-features`

Selected results are:

| Benchmark                                         | Reported range |
| ------------------------------------------------- | -------------- |
| Transcode small message to dynamic representation | 214–219 ns     |
| Transcode medium message                          | 1.81–1.89 µs   |
| Cached end-to-end validation, small fixture       | 367–370 ns     |
| Cached end-to-end validation, medium fixture      | 444–464 ns     |
| Cold end-to-end validation                        | 6.38–6.70 ms   |

The cold result emphasizes why evaluator reuse matters. The source explicitly characterizes these measurements as a regression baseline, dependent on the tested hardware, compiler, OS, fixtures, and benchmark setup.

These figures are not a direct runtime-versus-generated-code comparison. The published table does not establish a universal speedup factor for native generation or the Buffa bridge.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



`https://github.com/zs-dima/prost-protovalidate/blob/main/BENCHMARKS.md`

### 8.2 Consequential performance tradeoffs

Native validation can avoid reflection dispatch, repeated rule interpretation, and message conversion. Its effectiveness will depend on rule complexity, the size of nested collections, regex execution, allocation patterns, and the behavior of the generated code after compiler optimization.

Runtime validation has a different advantage: compiled evaluators can be shared across repeated calls, and descriptors permit dynamically selected message types. The upfront cost is amortized when a schema is validated frequently.

The Buffa bridge exchanges additional wire conversion for full runtime rule coverage without a second CEL implementation. Conversely, `protoc-gen-protovalidate-buffa` moves CEL compilation into the build pipeline, potentially reducing per-request evaluator overhead at the expense of generated-code complexity and tighter dependence on Buffa's generated representation.

These are mechanism-based expectations, not quantified performance guarantees.

## 9. Versioning, conformance, and important limitations

### 9.1 Version boundaries

| Component                                      | Verified release or compatibility       |
| ---------------------------------------------- | --------------------------------------- |
| `prost-protovalidate` family                   | 0.6.0                                   |
| `buf.validate` schema embedded in this release | Protovalidate v1.2.2                    |
| Prost dependency                               | 0.14                                    |
| `prost-reflect` dependency                     | 0.16                                    |
| Prost-Protovalidate MSRV                       | Rust 1.86                               |
| Independent `protovalidate-buffa`              | 0.7.2                                   |
| Independent Buffa project's documented targets | Buffa 0.9.1, ConnectRPC 0.9, Rust 1.88+ |

The primary project's 0.6.0 changelog reports passing 2,872 out of 2,872 tests from its pinned upstream Protovalidate conformance harness. The independent Buffa project reports the same count against its corresponding generated implementation. These are two separate project-reported conformance results, not evidence that their public Rust APIs or performance are equivalent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



### 9.2 Material caveats

Version-sensitive feature gating. Starting with 0.5.0, `default-features = false` no longer means “runtime validator without CEL.” It disables reflection as well. A CEL-free runtime requires the explicit `reflect` feature. This is a meaningful change for code relying on earlier 0.4.x behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Edition 2023 compatibility uses normalization. The runtime supplies `normalize_edition_descriptor_set` and handles Edition 2023 `DELIMITED` group encoding so descriptors can be consumed by `prost-reflect` 0.16. This is an adaptation around reflection-engine capabilities, not evidence that all protobuf edition semantics are implemented natively by that version of `prost-reflect`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Generated and runtime errors are not perfectly equivalent. An upstream known limitation describes `bytes.pattern` applied to invalid UTF-8: generated validation emits an ordinary `bytes.pattern` violation, whereas runtime validation reports a `RuntimeError`. This was documented in the 0.4.1 changelog; I found no explicit removal of the limitation in the subsequent release notes. It illustrates the information loss imposed by the generated `Validate` trait's narrower return type.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



CEL type semantics matter. Protobuf values are mapped into CEL types; numeric coercion, presence tests, field access, and short-circuit/error behavior must follow CEL rather than ordinary Rust semantics. A replacement expression that looks mathematically similar may not have identical behavior for missing fields, unsigned values, or evaluation errors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Protovalidate+1



Protobuf conformance is not complete application validation. Constraints can establish local value properties and message-level relationships. They do not, by themselves, verify external database state, authorization, consistency across concurrent operations, or the current existence of referenced resources. Those requirements remain outside the scope of schema-local validation.

Validation placement remains explicit. Generated `Validate` methods, a runtime `Validator`, or request macros supply the checking operation; the transport integration determines when it occurs. A successful protobuf decode must not be mistaken for having performed that check.

## 10. Architectural distinctions that guide further investigation

The ecosystem exposes several different ways to achieve schema-based validation without building a bespoke constraint engine.

| General requirement                                                         | Capability worth investigating                                         |
| --------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| Dynamic protobuf message types or schemas selected at runtime               | `prost-protovalidate::Validator` and `prost-reflect`                   |
| Cross-field invariants and expressive CEL                                   | Full runtime validation or the independent Buffa CEL compiler          |
| Low-overhead validation of static standard-rule schemas                     | `prost-protovalidate-build` with its native `Validate` implementations |
| Assurance that native generation has not omitted unsupported constraints    | `fail_on_runtime_only(true)`                                           |
| Uniform validation of Buffa messages with both standard and CEL constraints | Buffa `runtime_bridge(true)`                                           |
| CEL without runtime interpretation, including borrowed Buffa views          | `protoc-gen-protovalidate-buffa`                                       |
| Deterministic time-sensitive validation                                     | Runtime `Validator` with injected `NowFn`                              |
| Annotation discovery or schema-aware validation tooling                     | `prost-protovalidate-types` descriptor extraction                      |
| Consistent programmatic field/rule diagnostics                              | Structured `Violation` or canonical `buf.validate.Violations`          |
| Automatic checking of ConnectRPC request views                              | Independent Buffa `#[connect_impl]` macro                              |

The broader architectural opportunity is to move constraint meaning into protobuf schemas while choosing execution independently.

That can reduce duplicated field checks, handwritten request validation, bespoke path construction, inconsistent errors, and divergent validation logic between services. The central tradeoff is between dynamic expressiveness and metadata preservation on one side, and generated specialization and reduced runtime coordination on the other.

## 11. Primary-source reference map

For further research, these are the most useful direct sources, grouped by responsibility.

Prost-Protovalidate implementation and APIs

- Repository and integration guide: `https://github.com/zs-dima/prost-protovalidate`
- Runtime API: `https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/`
- Build-time generator: `https://docs.rs/crate/prost-protovalidate-build/0.6.0`
- Constraint types: `https://docs.rs/crate/prost-protovalidate-types/0.6.0`
- Runtime bridge: `https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/bridge/struct.RuntimeBridge.html`
- Releases and behavior changes: `https://github.com/zs-dima/prost-protovalidate/blob/main/CHANGELOG.md`
- Published performance baseline: `https://github.com/zs-dima/prost-protovalidate/blob/main/BENCHMARKS.md`

Canonical Protovalidate language

- Standard validation rules: `https://protovalidate.com/schemas/standard-rules/`
- Custom CEL rules: `https://protovalidate.com/schemas/custom-rules/`
- Predefined reusable rules: `https://protovalidate.com/schemas/predefined-rules/`
- CEL semantics: `https://buf.build/docs/protovalidate/cel/`
- Canonical `buf.validate` protobuf schema: `https://github.com/bufbuild/protovalidate/blob/main/proto/protovalidate/buf/validate/validate.proto`
- Buf's Protovalidate rules and lint semantics: `https://buf.build/docs/lint/rules/`

Buffa-specific implementation

- Dedicated Buffa-native validator and compiler: `https://github.com/mathematic-inc/protovalidate-buffa`
- Runtime API: `https://docs.rs/protovalidate-buffa/0.7.2/protovalidate_buffa/`
- Buffa implementation and code-generation guide: `https://github.com/anthropics/buffa/blob/main/docs/guide.md`

The most consequential finding is that Rust now has both a hybrid Prost/Buffa validation architecture and a separate Buffa-native CEL compilation architecture. Both can enforce the same `buf.validate` schema vocabulary, but their execution models, reflection dependencies, error types, borrowed-message support, and code-generation boundaries are materially different.

# Addendum: Advanced Protobuf Validation Capabilities in Rust

Companion to: Protobuf Validation in Rust: prost-protovalidate, Build-Time Validation, and Prost/Buffa Integration

Research date: November 3, 2026\
Version scope: `prost-protovalidate`, `prost-protovalidate-build`, and `prost-protovalidate-types` 0.6.0; `protovalidate-buffa` 0.7.2; Buf Protovalidate schema v1.2.2 where supported by these Rust releases.

This addendum extends the original reference with capabilities and semantic distinctions not developed there. It focuses on schema verification, generation-time integration, advanced validation semantics, diagnostic interoperability, and compatibility boundaries. Existing explanations of the core validation engines, standard rule families, runtime bridging, and general performance model are not repeated.

## 1. Schema-level verification and validation completeness

### 1.1 Buf's `PROTOVALIDATE` lint rule

Protovalidate constraints are executable schema metadata. Consequently, correctness has two distinct dimensions: whether the constraints themselves are valid and whether a particular message satisfies them.

Buf's `PROTOVALIDATE` lint rule performs static analysis of the first dimension, independently of the Rust runtime validator or generated validator implementations. It is included in Buf's `STANDARD` lint category.

The rule checks considerably more than protobuf option syntax.

| Constraint category       | Static verification capabilities                                                                                       |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| CEL expressions           | Compilation and type checking; permitted `bool` and `string` results; valid identifiers and error-message declarations |
| Type-specific rules       | Compatibility between the annotated field type and selected constraint family                                          |
| Contradictory rules       | Detection of certain unsatisfiable combinations, such as simultaneous `contains` and `not_contains` for the same value |
| Redundant rules           | Detection of certain constraints rendered unnecessary by stronger constraints                                          |
| Presence and ignore       | Invalid combinations of `required`, `IGNORE_IF_ZERO_VALUE`, `IGNORE_ALWAYS`, oneof membership, and extensions          |
| Repeated values           | Valid cardinality bounds, compatible item constraints, and supported `unique` element types                            |
| Maps                      | Valid key/value rules, supported key types, and compatible `min_pairs`/`max_pairs`                                     |
| Message-level constraints | Valid CEL and mutually exclusive message-level options                                                                 |

CEL rules receive additional structural checks:

- Expressions must compile to an acceptable return type.
- Boolean-returning rules require a failure message; string-returning rules must not provide a redundant static failure message.
- Full `Rule` identifiers must be nonempty, syntactically valid, and unique within the relevant field or message rule collection.
- Shorthand `cel_expression` entries are compiled and type-checked without requiring explicit identifiers.

These checks substantially reduce the need to discover malformed constraints through failed runtime validator initialization or generated-code compilation. They do not establish that arbitrary sets of cross-field CEL conditions are jointly satisfiable; the documented contradiction analysis is narrower than general-purpose constraint solving.

Composition with Rust validation. Schema linting, native code generation, and runtime validation establish different properties:

| Mechanism                                               | Property established                                                                          | Important limitation                                                                            |
| ------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `buf lint` / `PROTOVALIDATE`                            | Constraint definitions pass the linter's static checks                                        | Does not prove that a particular Rust backend implements every constraint                       |
| `prost-protovalidate-build::fail_on_runtime_only(true)` | A native validator is not accepted when its message requires unsupported execution mechanisms | Does not replace general schema linting                                                         |
| Runtime `Validator` with prepared descriptors           | Referenced evaluator definitions can be compiled and interpreted by the configured engine     | Compilation alone does not establish that actual message values are valid                       |
| `Validate::validate()` or `Validator::validate()`       | An individual message satisfies the constraints actually executed                             | Does not establish compatibility with earlier validation policies or external application state |

The resulting distinction is between schema correctness, backend coverage, and data validity. A system can establish each separately without implementing a private validation-language compiler.

Primary sources:

`https://buf.build/docs/lint/rules/`

`https://buf.build/docs/lint/`

`https://docs.rs/prost-protovalidate-build/0.6.0/prost_protovalidate_build/struct.Builder.html`

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+2

### 1.2 Validation-policy evolution is separate from protobuf compatibility

Protobuf schema compatibility does not imply validation-policy compatibility.

For example, changing a `string.min_len` constraint from 3 to 8 leaves the protobuf wire encoding unchanged, yet changes which messages are accepted. Similarly, introducing `required`, restricting an enum's allowed values, or changing a cross-field CEL expression can reject previously valid data.

Buf's built-in breaking-change rules intentionally exclude custom options because their semantics are application-defined. This includes changes to validation constraints encoded as custom protobuf options.

Three compatibility dimensions therefore remain separate:

| Dimension                                      | Typical verification                                                                     |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Binary/JSON and generated-source compatibility | `buf breaking` with `WIRE`, `WIRE_JSON`, `PACKAGE`, or `FILE` rules                      |
| Internal constraint validity                   | `PROTOVALIDATE` linting                                                                  |
| Compatibility of accepted message values       | Semantic rule comparison, representative-message testing, or custom compatibility checks |

Validation-policy compatibility is also directional. A stronger rule can preserve the validity of all newly accepted messages under an older validator while rejecting values that the older validator accepted. A relaxed rule has the opposite compatibility characteristics.

This matters when producers and consumers evolve independently. A producer using a weaker policy may generate messages that a stricter consumer rejects, even though the schema remains wire-compatible.

An additional mechanism is available through Buf check plugins, which can implement custom lint or breaking-change rules that inspect protobuf options. They enable domain-specific compatibility analysis without modifying the standard protobuf compiler or implementing an independent schema parser.

However, general CEL implication is not readily decidable for arbitrary expressions under all environments. A custom plugin can reliably enforce selected syntactic or policy restrictions without necessarily proving semantic equivalence between arbitrary old and new CEL programs.

Primary sources:

`https://buf.build/docs/breaking/rules/`

`https://buf.build/docs/breaking/`

`https://buf.build/docs/lint/`

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs+2

## 2. Advanced validation-schema semantics

### 2.1 Shorthand CEL through `cel_expression`

Protovalidate supports both full CEL rule definitions and the shorter `cel_expression` annotation on fields and messages.

```
message Operation {
  uint64 requested = 1;
  uint64 available = 2;

  option (buf.validate.message).cel_expression =
    "this.requested <= this.available";
}
```

Unlike a full `Rule`, `cel_expression` does not require an explicitly authored `id` or `message`. Protovalidate derives its rule identity from the expression itself.

This distinction affects more than schema brevity:

- Full `Rule` definitions allow stable, domain-specific rule identifiers independent of the underlying expression.
- Shorthand expressions simplify local constraints but couple diagnostic identity to expression text.
- Changing an expression can therefore change its diagnostic identifier even when the conceptual business rule remains the same.
- Both forms require CEL support in the Rust execution backend.

The Prost-Protovalidate 0.6.0 runtime recognizes both forms. With CEL disabled, attempting to validate a message containing either form returns a `CompilationError` instead of silently ignoring the expression.

Full rules consequently remain useful where stable diagnostic IDs, localization, or downstream rule identification matter. Shorthand rules are useful when concise schema-local constraints are more valuable than separately maintained identities.

Sources:

`https://protovalidate.com/schemas/custom-rules/`

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/`

[image](https://www.google.com/s2/favicons?domain=https://javadoc.io\&sz=32)

javadoc.io+2

### 2.2 Virtual oneofs and decoded-message information

Protovalidate's message-level `oneof` constraints offer capabilities beyond protobuf's native `oneof` representation.

A virtual oneof can include independently declared fields, including repeated and map fields. Unlike a native oneof, it does not change the protobuf message's storage representation.

Two semantics are especially important.

Implicit presence. For fields that do not track presence, the zero value counts as unset when evaluating virtual-oneof membership. For repeated fields and maps, the relevant zero values are empty collections.

Multiple values on the wire. Native protobuf oneof decoding generally retains the last encountered alternative when multiple alternatives appear in serialized input. A validator operating on the decoded native oneof cannot reconstruct those earlier assignments.

With independent fields, a virtual oneof can observe multiple populated members after decoding and reject the combination. This provides stronger decoded-message validation for such cases, although it is not a general raw-wire auditing mechanism.

Virtual-oneof membership also implicitly establishes `IGNORE_IF_ZERO_VALUE` for its member fields unless explicitly overridden. Unpopulated alternatives therefore do not ordinarily fail their own field constraints simply because another alternative was selected.

The distinction is useful whenever alternatives must remain independently addressable in the protobuf representation but still satisfy a one-of constraint.

Source:

`https://protovalidate.com/schemas/standard-rules/`

`https://github.com/bufbuild/protovalidate/blob/main/proto/protovalidate/buf/validate/validate.proto`

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate+1

### 2.3 Presence-aware ignore behavior

The original guide distinguishes `IGNORE_UNSPECIFIED`, `IGNORE_IF_ZERO_VALUE`, and `IGNORE_ALWAYS`. One qualification is particularly important:

`IGNORE_IF_ZERO_VALUE` does not cause an explicitly present optional scalar with a zero value to bypass its validation rules.

The canonical schema defines this option as a no-op for fields that track presence, where its behavior is equivalent to `IGNORE_UNSPECIFIED`.

| Field representation                | Value/state       | Effect of `IGNORE_IF_ZERO_VALUE` |
| ----------------------------------- | ----------------- | -------------------------------- |
| Implicit-presence `string`          | `""`              | Skip field validation            |
| Explicit-presence `optional string` | Absent            | Skip ordinary field validation   |
| Explicit-presence `optional string` | Present with `""` | Validate the present value       |
| Implicit-presence numeric           | `0`               | Skip field validation            |
| Explicit-presence numeric           | Present with `0`  | Validate the present value       |
| Repeated/map                        | Empty collection  | Skip its field-level constraints |

The distinction persists even when two values appear identical in a generated Rust API. Validation semantics depend on protobuf presence information, not simply on whether a scalar equals its default.

`IGNORE_ALWAYS` is stronger: it suppresses the field's validation rules, including `required`, and can suppress traversal into the validation rules of a referenced message.

Source:

`https://github.com/bufbuild/protovalidate/blob/main/proto/protovalidate/buf/validate/validate.proto`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 2.4 Reversed numeric ranges as exclusion constraints

Protovalidate deliberately supports reversed lower/upper numeric bounds. They are interpreted as an exclusion interval rather than an ordinary conjunction with no possible satisfying value.

For example:

```
int32 allowed = 1 [
  (buf.validate.field).int32 = {
    gt: 10
    lt: 5
  }
];
```

This accepts values strictly below 5 or strictly above 10. Values from 5 through 10 fail.

The capability can replace a simple CEL disjunction when an allowed domain consists of two unbounded ranges separated by a forbidden interval.

The exact boundary behavior depends on the chosen inclusive or exclusive operators (`gt`, `gte`, `lt`, `lte`). The result should not be simplified to an ordinary Rust expression that combines both comparisons with `&&`.

Importantly, Buf's static linter recognizes this standard-rule convention; reversed bounds are not automatically classified as an invalid contradiction.

Source:

`https://protovalidate.com/reference/rules/`

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate

### 2.5 Additional standard-rule functionality

Several further rule capabilities have architectural significance.

| Capability                                   | Semantics and implications                                                                                               |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `cel_expression`                             | Expression-only CEL annotations, with derived rule identity                                                              |
| `example` fields                             | Schema-embedded example values that do not enforce validity; potentially useful for documentation and fixture generation |
| `string.len`, `min_len`, `max_len`           | Measure Unicode code points rather than UTF-8 bytes                                                                      |
| `string.len_bytes`, `min_bytes`, `max_bytes` | Measure encoded UTF-8 byte length                                                                                        |
| `protobuf_fqn`                               | Validates a fully qualified protobuf name without a leading dot                                                          |
| `protobuf_dot_fqn`                           | Validates a fully qualified protobuf name requiring the historical leading-dot form                                      |
| `well_known_regex`                           | Includes predefined HTTP-header-name and HTTP-header-value validation                                                    |
| `string.strict`                              | Controls strict versus permissive behavior for supported HTTP-header regular expressions                                 |

Character counts are not the same as byte counts, and neither necessarily equals the number of user-perceived grapheme clusters. For example, Unicode combining marks can make a visually single character consist of multiple code points.

The string-length distinctions therefore affect what is being constrained: Unicode text length, encoded storage size, or a higher-level display property not directly implemented by these standard rules.

`example` annotations can make validation schemas more informative to code generators and documentation tools, but they should not be confused with test cases or guarantees. Their values are advisory, and validators do not use them to determine validity.

The protobuf-FQN rules also reflect a historical versus modern protobuf naming distinction. A descriptor-processing system that uses leading-dot absolute names has a different syntactic requirement from one that uses names such as `google.protobuf.Timestamp` without the leading dot.

Sources:

`https://protovalidate.com/reference/rules/`

`https://github.com/bufbuild/protovalidate/releases`

[image](https://www.google.com/s2/favicons?domain=https://protovalidate.com\&sz=32)

Protovalidate+2

## 3. Runtime validator lifecycle and configuration

### 3.1 Preloaded descriptors and closed-world validation

`prost-protovalidate::Validator` normally prepares evaluators lazily and caches compiled rules. Version 0.6.0 additionally supports a more restrictive configuration combining:

- `ValidatorOption::MessageDescriptors`
- `ValidatorOption::DisableLazy`

`MessageDescriptors` supplies descriptors whose evaluators should be prepared during validator construction. `DisableLazy` prevents the validator from accepting a previously unregistered message type by compiling a new evaluator on demand.

An unknown message type then causes a compilation error.

This establishes a meaningful distinction between two execution models:

| Model                      | Behavior                                                       |
| -------------------------- | -------------------------------------------------------------- |
| Lazy descriptor discovery  | Prepare constraints as message types are encountered           |
| Closed descriptor registry | Restrict validation to a known, preloaded set of message types |

The second model is useful where an unexpected message type should be interpreted as a configuration mismatch rather than as an invitation to compile additional validation logic.

It can also move some compilation work out of steady-state request processing.

However, closed-world compilation is not an authorization mechanism. It establishes membership in a configured validation-schema set, not whether a caller is permitted to submit a message of that type.

`AdditionalDescriptorSetBytes` provides a related but distinct capability: additional encoded descriptors for resolving custom or proprietary validation-rule extensions. The presence of those descriptors does not automatically select every message type for validation or authorize unknown constraints.

Source:

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/enum.ValidatorOption.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 3.2 Per-call validation context

The `ValidationOption` enum supports three per-invocation controls, independently of the validator's construction-time defaults.

| Option                          | Per-call behavior                                                  |
| ------------------------------- | ------------------------------------------------------------------ |
| `ValidationOption::FailFast`    | Stops collecting violations after the first encountered failure    |
| `ValidationOption::Filter(...)` | Overrides which message and field branches are validated           |
| `ValidationOption::NowFn(...)`  | Supplies the current-time function for time-relative rules and CEL |

Per-call `NowFn` is particularly relevant when the same compiled validator is reused across requests with different logical evaluation times.

It permits deterministic testing, replay, and evaluation relative to an externally supplied clock context without recompiling all rule expressions or constructing separate validators for each time context.

The clock function is a Rust `Arc<dyn Fn() -> Timestamp + Send + Sync>`. Its exact output is still the caller's responsibility.

A filtered invocation similarly changes the scope of the assertion: a successful result means that the selected constraints passed. It does not imply that the entire message satisfies the unfiltered schema.

Source:

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/enum.ValidationOption.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 3.3 Unknown constraint options and schema-version forward compatibility

`ValidatorOption::AllowUnknownFields` permits constraint messages to contain unknown fields without rejecting them solely for that reason.

This mechanism addresses a common version-skew scenario: the `.proto` compiler and schema registry may recognize a newer `buf.validate` option that the Rust validator does not yet implement.

Allowing unknown fields can preserve compatibility with older constraint consumers. It is important, however, not to interpret this as semantic compatibility.

If a new rule affects validity but the older validator does not recognize or execute it, successful validation cannot establish compliance with that newer rule.

Consequently, three conditions must be kept distinct:

1. A constraint can be represented in a descriptor.
2. A validator can parse or tolerate that descriptor.
3. A validator can enforce the constraint's actual semantics.

Only the third establishes validation coverage.

This distinction becomes particularly important when interoperating with evolving shared protobuf schemas or independently versioned language implementations.

Source:

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/enum.ValidatorOption.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 4. Native validation generation: additional interfaces and constraints

### 4.1 Descriptor input, type-location mapping, and output control

`prost-protovalidate-build::Builder` exposes integration interfaces beyond its conventional file-based `build.rs` workflow.

| Interface                            | Role                                                                     | Architectural significance                                                                      |
| ------------------------------------ | ------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| `file_descriptor_set_bytes(Vec<u8>)` | Supplies encoded descriptors directly in memory                          | Allows generation from externally prepared descriptor sets without requiring another input file |
| `file_descriptor_set_path(...)`      | Loads a descriptor set from a filesystem path                            | Integrates with standard `prost-build` and other descriptor-producing tools                     |
| `extern_path(proto_path, rust_path)` | Maps protobuf package paths to external Rust modules                     | Keeps validation implementations compatible with message types generated or defined elsewhere   |
| `out_dir(...)`                       | Overrides the output directory                                           | Supports alternate generated-file placement                                                     |
| `backend(Backend)`                   | Selects the Prost or Buffa Rust representation                           | Separates descriptor-based validation analysis from generated-message representation            |
| `fail_on_runtime_only(true)`         | Rejects unsupported native-generation requirements                       | Makes native coverage a build-time requirement                                                  |
| `runtime_bridge(true)`               | Generates runtime-delegating implementations for unsupported constraints | Provides a uniform `Validate` interface when using the Buffa backend                            |

The important distinction is between protobuf schema identity and Rust type identity.

A descriptor identifies a message by its protobuf fully qualified name. Generated Rust code must additionally know where that message's Rust type resides.

For Prost, `extern_path` can supply this mapping:

```
prost_protovalidate_build::Builder::new()
    .file_descriptor_set_bytes(descriptor_bytes)
    .extern_path(".example.shared", "::shared_proto::example::shared")
    .compile()?;
```

This is an illustrative configuration, not an executed example. The `extern_path` mapping must agree with the corresponding `prost-build` configuration and the actual Rust module structure.

Without such alignment, a descriptor can be perfectly valid while the generated validator references a nonexistent or incorrect Rust type.

This mechanism is particularly useful when protobuf definitions and their validation implementations have separate generation or publication boundaries.

Source:

`https://docs.rs/prost-protovalidate-build/0.6.0/prost_protovalidate_build/struct.Builder.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 4.2 Native-generation completeness versus implicit runtime fallback

The builder's default handling of unsupported rules is deliberately diagnostic: it skips the affected implementation and emits a Cargo warning.

That behavior must not be confused with a runtime fallback automatically installed into the application.

Three distinct outcomes exist:

| Builder configuration        | Outcome for a runtime-only message                                  |
| ---------------------------- | ------------------------------------------------------------------- |
| Default                      | No generated `Validate` implementation; diagnostic warning          |
| `fail_on_runtime_only(true)` | Build-time failure                                                  |
| Buffa `runtime_bridge(true)` | Generated `Validate` implementation delegates to runtime validation |

For the default Prost backend, the dynamic validator remains a separate API and execution path. It does not automatically implement `Validate` for messages that native generation skipped.

The generator also accounts for nested dependencies. If native validation of a parent message would require traversing a child whose constraints cannot be fully generated, the parent can also become runtime-only.

This prevents an especially dangerous partial-generation behavior: producing a successful validation result from generated checks that omitted constraints reachable through nested messages.

Completeness here is relative to the generator's supported rule set. It does not establish that validation is actually invoked on every incoming message or that a service has no alternative unchecked input path.

Source:

`https://docs.rs/prost-protovalidate-build/0.6.0/prost_protovalidate_build/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 4.3 Runtime-bridge output constraints

The Buffa runtime bridge introduces a dependency between generated source placement and descriptor embedding.

When `runtime_bridge(true)` is enabled, generated code embeds the descriptor set through `include_bytes!` using a path derived from Cargo's `OUT_DIR`.

Consequently:

- The generated bridge requires the `prost-protovalidate` reflection feature, plus CEL support when CEL rules are present.
- The generated validation source and embedded descriptor set must remain compatible with the expected `OUT_DIR` layout.
- Combining the runtime bridge with a custom `out_dir` for compiled generated code is explicitly unsupported.
- `runtime_bridge(true)` and `fail_on_runtime_only(true)` are mutually exclusive. The builder reports `Error::ConflictingOptions` if both are enabled.

These are compilation and packaging constraints, not simply runtime performance considerations.

There is also a semantic distinction in errors: the lower-level runtime bridge can return compilation or evaluation errors separately from ordinary validation violations. The generated `Validate` trait has a narrower `Result<(), ValidationError>` contract.

Where preserving the original error categories matters, the lower-level bridge interface remains relevant.

Source:

`https://docs.rs/prost-protovalidate-build/0.6.0/prost_protovalidate_build/struct.Builder.html`

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/bridge/struct.RuntimeBridge.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 5. Independent Buffa validator generation: output architecture and extension points

The dedicated `protoc-gen-protovalidate-buffa` generator exposes a richer code-generation interface than the original guide describes.

These capabilities belong to the independent `protovalidate-buffa` ecosystem, not to `prost-protovalidate-build::Backend::Buffa`.

### 5.1 Field-name compatibility through `idiomatic_field_names`

Buffa supports optional idiomatic Rust field naming. The validation generator must apply corresponding conventions so its generated field accesses match those of the message generator.

The option is:

`idiomatic_field_names=true`

It can be specified on both `protoc-gen-buffa` and `protoc-gen-protovalidate-buffa`.

Buffa's naming logic accounts for collisions across the descriptor set, including imported definitions. Because generated validation code accesses concrete fields, a mismatch between these naming conventions can produce uncompilable implementations.

The validator generator distinguishes two naming domains:

- Rust field and accessor names: Follow the Buffa representation and its naming options.
- Protobuf schema names: Remain authoritative for CEL expressions and structured validation field paths.

This separation is valuable: Rust-specific naming normalization need not alter schema-authored CEL semantics or external diagnostic field identifiers.

Oneof enum and variant naming is unaffected by this option.

The project documents using `strategy: all` for both generators when consistent global naming and collision analysis are required.

Source:

`https://github.com/mathematic-inc/protovalidate-buffa#quick-start`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 5.2 Packaging and output-layout controls

The generator provides two independent output controls:

- `packaging`: Whether generated module/package scaffolding is emitted.
- `file_per_package`: Whether validators are consolidated by protobuf package or emitted separately for each source file.

Their combinations produce four layouts.

| `packaging` | `file_per_package` | Validator files                         | Generated module files |
| ----------- | ------------------ | --------------------------------------- | ---------------------- |
| `true`      | `false`            | One `.rs` per source proto              | Yes                    |
| `false`     | `false`            | One `.validate.rs` per source proto     | No                     |
| `true`      | `true`             | One `.rs` per protobuf package          | Yes                    |
| `false`     | `true`             | One `.validate.rs` per protobuf package | No                     |

The defaults are `packaging=true` and `file_per_package=false`.

The distinction affects the shape of the generated Rust module tree, not the underlying validation language.

Packaging enabled delegates more Rust module construction to the plugin.

Packaging disabled avoids generating `mod.rs` scaffolding and allows validation implementations to be mounted alongside separately generated messages, including inside modules whose structure is managed by other tooling.

Per-package generation consolidates validator implementations for multiple source files that share a protobuf package. This can reduce generated-module fragmentation, provided both the message generator and validator generator use compatible layouts.

A representative Buf configuration is:

```
version: v2
plugins:
  - local: protoc-gen-buffa
    out: gen/proto
    strategy: all
    opt:
      - file_per_package=true

  - local: protoc-gen-protovalidate-buffa
    out: gen/proto
    strategy: all
    opt:
      - packaging=false
      - file_per_package=true
```

With this arrangement, a package such as `example.v1` can produce both `example.v1.rs` and `example.v1.validate.rs` for inclusion within the same Rust module.

Additional documented behaviors matter:

- The generator emits validators for message types even when they contain no validation annotations.
- Packages containing only enums or services do not require validator includes.
- The package hierarchy must remain consistent for cross-package references.
- `proto_module` has no effect when `packaging=false`.
- Switching layouts does not automatically delete generated files left by an earlier configuration.

These capabilities allow validation generation to be composed with different Rust module-packaging strategies without modifying the protobuf schema.

Source:

`https://github.com/mathematic-inc/protovalidate-buffa#include-validators-beside-message-types`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 5.3 In-process generator integration

The independent Buffa generator also exposes programmatic generation mechanisms.

Its documented internal pipeline separates descriptor scanning from Rust rendering:

- `scan::gather` analyzes a `CodeGeneratorRequest` and collects validation information.
- `emit::render` renders generated validator code.
- `emit::render_with_options` supports explicit output-layout configuration.
- `emit::Options` carries settings such as `packaging`, `file_per_package`, and `proto_module`.

Generator parameters, including field-naming options, can be supplied through `CodeGeneratorRequest.parameter`.

For example, the project's documented rendering options include:

```
emit::Options {
    packaging: false,
    file_per_package: true,
    ..Default::default()
}
```

This decomposition makes the generator useful beyond a standalone `protoc` executable. It potentially allows a larger schema-processing system to coordinate descriptor analysis, code generation, and source packaging in one process.

The interface is also a coupling point: these APIs are part of the generator implementation, not an independent language-neutral validation specification. Consumers using them directly depend on the generator's Rust API stability.

Source:

`https://github.com/mathematic-inc/protovalidate-buffa#quick-start`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 5.4 Generated CEL execution, borrowed views, and error categories

Buffa's dedicated validation generator translates CEL rules into Rust rather than relying on a runtime interpreter.

That introduces several consequential characteristics beyond the basic static-versus-dynamic comparison:

Compilation-time analysis. The generator can identify certain malformed rule/type combinations, unknown CEL field references, and invalid virtual-oneof specifications during generation.

Owned and borrowed execution. Generated validators can operate on Buffa's owned messages and borrowed views, avoiding an ownership conversion solely for validation.

Structured error-category preservation. `protovalidate-buffa::ValidationError` contains distinct fields for ordinary violations, compilation diagnostics, and evaluation diagnostics:

```
pub struct ValidationError {
    pub violations: Vec<Violation>,
    pub compile_error: Option<String>,
    pub runtime_error: Option<String>,
}
```

The latter two categories permit failures that cannot correctly be represented as a violated user-authored constraint.

For example, an operation that attempts regex matching against invalid UTF-8 may encounter an evaluation precondition failure rather than a normal pattern mismatch.

This also demonstrates a limitation of generated validation: eliminating a runtime CEL interpreter does not eliminate every possible evaluation error. Generated operations still have runtime preconditions.

The ConnectRPC adapter treats compilation and evaluation diagnostics as internal errors, even when ordinary violations are also present. It does not classify all unsuccessful validations as client-side invalid arguments.

Source:

`https://docs.rs/protovalidate-buffa/0.7.2/protovalidate_buffa/`

`https://github.com/mathematic-inc/protovalidate-buffa#error-model`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1

## 6. RPC validation contracts and diagnostic interoperability

### 6.1 Tonic's `ValidateRequest` applies to generated `Validate` implementations

The `prost-protovalidate` Tonic extension has a narrower contract than a general-purpose descriptor-driven validation middleware.

Its implementation is:

```
impl<T: Validate> ValidateRequest for tonic::Request<T>
```

Calling `request.validate_inner()` invokes the `Validate` implementation of the contained message. It does not automatically discover a descriptor, construct a dynamic validator, or interpret CEL rules independently.

This creates an important execution distinction:

| Validation mechanism                   | Tonic integration behavior                                                                                        |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Native Prost `Validate` implementation | `request.validate_inner()` invokes the generated checks                                                           |
| Prost message requiring runtime CEL    | Requires an explicit `Validator::validate()` call unless another compatible `Validate` implementation is supplied |
| Buffa-generated bridge implementation  | Exposes `Validate` through the bridge, subject to the bridge's runtime dependencies                               |
| Client-streaming or bidirectional RPC  | Individual decoded stream messages require validation                                                             |

The dynamic `Validator::validate()` API operates on `prost_reflect::ReflectMessage`; the generated `Validate` trait is a separate interface.

A message that supports reflection does not thereby implement `Validate`.

This is a consequential distinction when a protobuf schema mixes messages eligible for native generation with messages requiring runtime-only constraints.

The Tonic helper guarantees that the particular `Validate` implementation is invoked. It does not guarantee that every protobuf message is covered by one.

Source:

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/tonic/trait.ValidateRequest.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 6.2 Internal validation diagnostics versus public RPC errors

Validation results can contain three different classes of information:

1. User-value violations: A constraint was evaluated successfully but rejected a message value.
2. Constraint compilation failures: A rule definition, descriptor, or CEL expression could not be prepared.
3. Evaluation failures: A rule encountered an invalid runtime precondition or another failure that is not an ordinary value violation.

The distinction matters at an RPC boundary.

The first category ordinarily corresponds to an invalid request argument. The other two can indicate server-owned schema or evaluator defects and should not automatically be attributed to the client.

The Prost-Protovalidate Tonic integration maps ordinary validation failures to `INVALID_ARGUMENT`, while compilation and runtime errors map to `INTERNAL` with generic messages.

However, ordinary violation messages are not necessarily safe for unrestricted public disclosure.

Protovalidate allows string-valued CEL expressions to construct dynamic failure messages. Such expressions can include or derive information from rejected input.

The Tonic adapter uses the formatted `ValidationError` as the public status message. When `tonic-types` is enabled, it additionally emits individual `google.rpc.BadRequest` details containing descriptions.

The safety of those public messages therefore depends partly on the validation rules that produced them.

This qualification is important even though the library already protects compilation and evaluation errors from direct exposure.

Source:

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/tonic/`

`https://protovalidate.com/schemas/custom-rules/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 6.3 Buffa's bounded public error representation

The independent `protovalidate-buffa` implementation provides a more restrictive default error-publication policy.

Its `ValidationError::into_connect_error()` conversion distinguishes ordinary violations from validator defects.

| Diagnostic condition                   | RPC status         | Public detail                     |
| -------------------------------------- | ------------------ | --------------------------------- |
| Ordinary violations only               | `invalid_argument` | Bounded `buf.validate.Violations` |
| Compilation or evaluation error        | `internal`         | No validation detail              |
| Mixed violations and evaluator defects | `internal`         | No validation detail              |
| Empty `ValidationError`                | `internal`         | No validation detail              |

The presence of an evaluator defect takes precedence over ordinary violations, even when both are recorded.

Public violation details intentionally omit free-form rule messages and map-key values, including numeric and boolean map keys.

The remaining fields can preserve:

- Schema-authored rule identifiers.
- Protobuf field names, field numbers, and available field types.
- Repeated-field indexes.
- Structural field and rule paths.
- The `for_key` indicator distinguishing map-key failures from other failures.

An omitted map-key selector can therefore preserve the general schema location while no longer identifying the exact offending map entry.

This is an intentional loss of diagnostic specificity in exchange for reducing rejected-data exposure.

Bounded serialization. The implementation limits an individual public `buf.validate.Violations` detail to 4,096 encoded protobuf bytes.

The serializer retains the longest prefix of complete violations that fits within the limit. It does not shorten individual rule identifiers or paths to squeeze more violations into the available budget.

If a violation cannot fit, that violation and all following violations are omitted. The RPC status remains `invalid_argument`.

The canonical `buf.validate.Violations` schema does not carry a truncation flag. Consequently, clients must treat the received collection as potentially incomplete even when decoding succeeds.

The 4-KiB budget provides headroom against typical gRPC metadata limits, but it is not a guarantee that the entire status envelope, metadata, and additional details fit within a transport-specific limit.

Source:

`https://github.com/mathematic-inc/protovalidate-buffa#wire-defaults-and-diagnostic-fidelity`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 6.4 Feature tiers for canonical diagnostics and JSON

`protovalidate-buffa` exposes several optional Cargo features that separate validation execution from diagnostic transport.

| Configuration              | Capability                                                                                                         |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Default features           | Includes ConnectRPC integration                                                                                    |
| `default-features = false` | Core validation runtime without the optional ConnectRPC, base64, and generated diagnostic-schema dependencies      |
| `features = ["protos"]`    | Enables canonical `buf.validate` diagnostic representations and conversion without requiring transport integration |
| `features = ["json"]`      | Enables `protos` and protobuf JSON support for generated diagnostic types                                          |
| `connect`                  | Provides ConnectRPC integration and enables the `protos` functionality it requires                                 |

The `protos` feature exposes the generated canonical validation schema through `protovalidate_buffa::proto`.

`ValidationError::to_proto()` then provides a protobuf diagnostic representation without requiring the validation result to be converted into an RPC status.

This conversion is deliberately more complete than the default public error representation. It can preserve violation messages and map-key values, making it suitable for trusted internal inspection but potentially sensitive.

It also cannot encode every error category. Compilation and evaluation diagnostics remain on the Rust `ValidationError`, because the canonical `buf.validate.Violations` schema has no corresponding fields.

The `json` feature adds protobuf JSON support through Buffa's JSON helpers. This should not be confused with enabling ConnectRPC's JSON transport: the two are independent capabilities.

The generated diagnostic-schema Rust types are checked into the upstream project. Ordinary downstream builds therefore do not require recompiling those protobuf definitions with `protoc`.

Source:

`https://github.com/mathematic-inc/protovalidate-buffa#typed-client-decoding`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 6.5 Typed error-detail decoding across Connect and gRPC

The independent Buffa runtime provides:

```
decode_violations(&ErrorDetail)
```

This is a client-side decoder for canonical `buf.validate.Violations` details.

It accepts recognized representations of the message's type identity, including the bare type name and URLs ending in `/buf.validate.Violations`.

Its behavior distinguishes unrelated details from malformed matching details:

| Input condition                                | Result                  |
| ---------------------------------------------- | ----------------------- |
| Unrelated error-detail type                    | `Ok(None)`              |
| Recognized and valid violations detail         | `Ok(Some(...))`         |
| Recognized type with malformed protobuf/base64 | `DecodeViolationsError` |
| Recognized type exceeding resource limits      | `DecodeViolationsError` |
| Recognized detail with no violations           | `DecodeViolationsError` |

The decoder accepts padded and unpadded standard base64, tolerates unknown protobuf fields, and does not depend on the optional human-readable `debug` field.

Its documented resource bounds are:

| Resource                    | Limit       |
| --------------------------- | ----------- |
| Base64 input                | 5,464 bytes |
| Decoded protobuf payload    | 4,096 bytes |
| Repeated/map element memory | 1 MiB       |

Buffa's normal recursion and unknown-field limits also apply.

The decoder preserves messages and map keys received from peers, even though the corresponding server-side implementation omits them in its default public output. This is important: a decoder's input trust model is independent of the sender's redaction policy.

The same canonical protobuf diagnostic type supports Connect's JSON error details and gRPC status details. This reduces the need for custom, transport-specific validation-error parsers.

Source:

`https://github.com/mathematic-inc/protovalidate-buffa#typed-client-decoding`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 6.6 Diagnostic identity and information loss without reflection

`prost-protovalidate` preserves the principal textual fields of generated violations:

- `field_path()`
- `rule_path()`
- `rule_id()`
- `message()`

However, their presence does not imply that the generated result contains the complete descriptor-backed metadata available during runtime reflection.

Without the `reflect` feature, `Violation::to_proto()` emits rule-path elements with names but without descriptor-derived `field_number` and `field_type` metadata.

Similarly, generated validation results do not automatically provide the reflection-enriched field and rule descriptors or dynamic field/rule values.

This creates a distinction between:

Textual diagnostic compatibility: Consumers can identify a failed field or rule by a string path or ID.

Descriptor-enriched diagnostic compatibility: Consumers can use protobuf field numbers, descriptor types, and dynamic values to connect a violation to its schema definition.

The second can be more useful for tooling that processes evolving schemas, traverses nested structures, or needs to interpret violations independently of a particular generated Rust representation.

A consumer that receives only the textual representation should not infer descriptor identities that were never included.

Source:

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/struct.Violation.html`

`https://docs.rs/prost-protovalidate/0.6.0/prost_protovalidate/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 7. Legacy validation migration and schema dependency management

### 7.1 `protoc-gen-validate` migration

Buf provides `protovalidate-migrate` to convert schemas using the older `protoc-gen-validate` (PGV) annotations into the modern `buf.validate` model.

The migration tool is maintained alongside the upstream Protovalidate project and supports incremental conversion rather than requiring an immediate replacement of every existing annotation.

| Option                    | Function                                                               |
| ------------------------- | ---------------------------------------------------------------------- |
| Default migration         | Adds equivalent Protovalidate annotations alongside PGV annotations    |
| `--replace-protovalidate` | Regenerates existing Protovalidate annotations from legacy definitions |
| `--remove-legacy`         | Removes old PGV annotations                                            |
| `--legacy-import`         | Overrides the legacy validation import path                            |
| `--protovalidate-import`  | Overrides the new validation-schema import path                        |
| `--write` / `-w`          | Writes changes to the source files                                     |
| `--output` / `-o`         | Writes transformed output to another location                          |

The principal architectural concern is semantic equivalence.

Several legacy constructs changed meaning or representation:

| Legacy PGV functionality           | Protovalidate equivalent or distinction                         |
| ---------------------------------- | --------------------------------------------------------------- |
| `validate.rules` field annotations | Consolidated into `buf.validate.field`                          |
| Legacy oneof `required`            | `buf.validate.oneof.required`                                   |
| Message-skipping rules             | `IGNORE_ALWAYS` on the relevant message field                   |
| Legacy `ignore_empty`              | `IGNORE_IF_ZERO_VALUE`, subject to the new presence semantics   |
| Legacy field `required`            | Modern `required`, with explicitly clarified presence semantics |
| `map.no_sparse`                    | No direct equivalent; removed                                   |

The migration guide also distinguishes PGV's code-generation model from Protovalidate's normal runtime-reflection model.

Rust's `prost-protovalidate-build` and `protoc-gen-protovalidate-buffa` provide additional native-generation implementations, but those are separate projects from the official language migration tooling.

The important implication is that an annotation migration is not simply a mechanical renaming exercise. A converted schema may require semantic validation against representative message values, particularly where legacy field-presence and skipping behavior mattered.

Primary source:

`https://buf.build/docs/migration-guides/migrate-from-protoc-gen-validate/`

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Protovalidate

### 7.2 Reproducible schema dependencies through the Buf Schema Registry

The protobuf validation schema, Rust runtime validator, message generator, and validation generator can all have independent version lifecycles.

Buf publishes the canonical Protovalidate definitions as a schema-registry module:

`buf.build/bufbuild/protovalidate`

A Buf module can declare that dependency in `buf.yaml`; `buf.lock` records the resolved commit and content digest, including transitive dependencies.

This offers several useful mechanisms:

| Mechanism               | Capability                                                              |
| ----------------------- | ----------------------------------------------------------------------- |
| Module dependency       | Resolve the shared `buf.validate` definitions without private vendoring |
| Label reference         | Select a named schema release or moving label                           |
| Commit pin              | Select an immutable schema snapshot                                     |
| `buf.lock`              | Record exact resolved dependencies and cryptographic digests            |
| Dependency verification | Reject downloaded dependencies that do not match their recorded digests |
| Dependency cache        | Reuse already fetched schema modules for offline operations             |

A reproducible descriptor compilation does not, by itself, establish runtime semantic compatibility.

For example, `prost-protovalidate` 0.6.0 uses a schema vocabulary aligned with Protovalidate v1.2.2. A separately resolved newer schema could introduce options that its Rust evaluator does not implement.

This introduces a multi-dimensional compatibility surface:

1. Schema definitions and option field numbers.
2. Rust types representing those options.
3. Evaluator support for their semantics.
4. Validation generator support for the selected message representation.
5. Runtime feature flags enabling required execution machinery.

A successful protobuf compilation establishes the first part of this chain, not all five.

The Buf module lock and the Rust dependency lock consequently protect different parts of the system. Neither can substitute for the other's coverage.

Source:

`https://buf.build/docs/bsr/module/dependency-management/`

[image](https://www.google.com/s2/favicons?domain=https://buf.build\&sz=32)

Buf Docs

### 7.3 Version alignment with Buffa and ConnectRPC

The independent Buffa validation ecosystem is sensitive to the precise representation generated by Buffa.

For the published `protovalidate-buffa` 0.7.2 generation, the upstream project documents compatibility with Buffa 0.9.1, ConnectRPC 0.9, and Rust 1.88 or newer.

Generated validator code directly references Buffa's owned-message and view representations. A change to the message runtime's generated field layout, enum representation, or borrowed-view interface can therefore break source-level compatibility without changing protobuf wire compatibility.

This is especially relevant because Buffa remains pre-1.0, so minor-version updates can introduce breaking changes.

The `connect` feature also introduces a transitive Buffa dependency through ConnectRPC. If the validation runtime and RPC runtime depend on incompatible Buffa versions, multiple incompatible generated-message runtime types can coexist in the dependency graph.

Accordingly, binary protobuf compatibility must not be mistaken for compatibility among:

- `buffa` message-generation output;
- `protoc-gen-protovalidate-buffa` generated code;
- `protovalidate-buffa` validation runtime;
- `connectrpc` request/view representations.

Source:

`https://github.com/mathematic-inc/protovalidate-buffa#compatibility`

`https://docs.rs/crate/protovalidate-buffa/0.7.2`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1

## 8. Conformance, verification, and evidence boundaries

### 8.1 Upstream conformance is distinct from continuous integration

Both Rust validation projects report passing all 2,872 cases in their respective pinned upstream Protovalidate conformance suites.

The reported suites cover proto2, proto3, and Editions 2023.

These results are meaningful because they test complex combinations of protobuf value semantics, constraint types, and diagnostic behavior against a common upstream contract.

However, the independent Buffa project's documentation makes an important distinction:

- Its routine CI executes workspace tests and Clippy.
- Full upstream conformance is documented as a local or pre-release verification procedure rather than a test automatically executed on every commit.

The published conformance result therefore establishes reported behavior for the tested implementation and pinned harness, not continuous verification of every subsequent development revision.

The Prost-Protovalidate project likewise identifies a pinned schema and conformance harness, with its own update and regression procedures.

The two reported results should not be interpreted as evidence that the libraries share Rust APIs, execution overhead, or identical behavior on every future schema revision.

Sources:

`https://github.com/mathematic-inc/protovalidate-buffa#conformance-testing`

`https://github.com/zs-dima/prost-protovalidate`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1

### 8.2 Generated/runtime parity is an additional verification dimension

A single validation language can be implemented through multiple execution architectures:

- Runtime CEL interpretation and descriptor traversal.
- Native generation of standard-rule checks.
- Native generation of CEL expressions.
- Wire-serialization bridges from one protobuf runtime to another.

Each implementation introduces different opportunities for semantic divergence.

For example, byte-pattern rules applied to invalid UTF-8 can produce a runtime evaluation error in one engine and an ordinary constraint violation in another.

Such a difference may not affect whether the input is accepted, but it does affect error classification, diagnostic transport, and the information available to callers.

The Prost-Protovalidate project documents generated-versus-runtime parity checks covering cases such as numeric boundaries, nested messages, presence, map paths, well-known types, and virtual oneofs.

This is a separate concern from both schema validity and language conformance.

A useful conceptual separation is:

| Verification class          | Question answered                                                                                 |
| --------------------------- | ------------------------------------------------------------------------------------------------- |
| Schema lint                 | Are the declared constraints structurally and semantically well-formed under the linter's checks? |
| Upstream conformance        | Does an implementation satisfy the tested Protovalidate reference cases?                          |
| Backend parity              | Do alternative execution backends classify the same values and errors equivalently?               |
| Cross-version compatibility | Does the implementation remain compatible with a newer schema or runtime?                         |
| Integration correctness     | Is validation invoked at the intended boundary, with errors mapped appropriately?                 |

These verification classes establish different guarantees. Passing one cannot automatically substitute for the others.

Source:

`https://github.com/zs-dima/prost-protovalidate/blob/main/CHANGELOG.md`

`https://github.com/mathematic-inc/protovalidate-buffa#conformance-testing`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1

## 9. Architectural synthesis: validation as a schema-governed compilation pipeline

The additional functionality reveals an architectural capability beyond choosing between reflection and generated checks.

Protobuf validation can operate as a schema-governed compilation pipeline whose rule definitions, execution specialization, diagnostics, and transport integration are independently controlled.

The relevant native mechanisms compose as follows:

| Architectural concern                     | Existing mechanism                                                          | Bespoke functionality potentially avoided                                     |
| ----------------------------------------- | --------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| Constraint definition                     | `buf.validate` annotations, standard rules, CEL, reusable predefined rules  | Duplicated validation logic encoded independently in each consumer            |
| Constraint verification                   | Buf `PROTOVALIDATE` lint                                                    | Custom rule-definition syntax and type checkers                               |
| Schema identity and dependency management | Descriptor sets, BSR modules, `buf.lock`                                    | Private descriptor registries and dependency bookkeeping                      |
| Native execution specialization           | Prost-Protovalidate generator or Buffa CEL compiler                         | Handwritten static field validators and custom CEL-to-Rust translation        |
| Runtime interpretation                    | `Validator`, descriptor registration, compiled-rule caching                 | Custom dynamic rule dispatch and evaluator lifecycle management               |
| Execution completeness                    | Runtime-only classification, build failure policies, Buffa runtime bridging | Ad hoc unsupported-rule fallback mechanisms                                   |
| Generated-code integration                | `extern_path`, backend selection, generator output-layout controls          | Manual reconciliation of package paths and generated implementation locations |
| Diagnostic representation                 | `Violation`, `FieldPath`, `buf.validate.Violations`                         | Custom hierarchical error-path schemas                                        |
| Error transport                           | Tonic `BadRequest`, Buffa canonical Connect/gRPC details                    | Transport-specific diagnostic encoding and decoding                           |
| Confidentiality and resource bounds       | Buffa's bounded/redacted public diagnostics                                 | Some bespoke error-detail redaction and payload-size enforcement              |
| Semantic verification                     | Upstream conformance and backend parity tests                               | Entirely private validation-equivalence test frameworks                       |

There are also important limits to this composition.

A schema linter cannot establish every possible semantic invariant. Generated code does not guarantee that validation is invoked. Schema compatibility does not imply validation-policy compatibility. Reflection-free execution can sacrifice descriptor-enriched diagnostic information. Native CEL generation can eliminate interpreter overhead but remains dependent on the compiler's support for the CEL semantic model.

Likewise, neither standard rules nor CEL automatically enforce invariants dependent on external database state, transactional isolation, authorization, or concurrent operations.

The most significant design opportunity is therefore not necessarily replacing runtime validation with generated validation. It is the ability to preserve a single schema-level definition of constraints while independently choosing verification, execution, representation, and diagnostic policies according to the relevant requirements.

That separation can reduce custom machinery without conflating properties that are fundamentally different: protobuf encoding validity, schema correctness, constraint enforcement, diagnostic completeness, transport safety, and application-level consistency.