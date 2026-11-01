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