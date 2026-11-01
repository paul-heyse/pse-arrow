# Figment — Rust Configuration Composition Library

Version: 0.10.19 (latest stable verified) | Research date: October 29, 2026

Figment is a Serde-integrated, semi-hierarchical configuration library for Rust. It combines configuration from files, environment variables, in-memory objects, defaults, and custom sources, resolves precedence through explicit composition strategies, and deserializes the result into strongly typed Rust structures.

Its distinctive strength is provenance-preserving configuration composition: individual configuration values retain source metadata through merges, profile resolution, and extraction, enabling detailed diagnostics and source-aware interpretation.

Official references:

- Documentation: [https://docs.rs/figment/0.10.19/figment/](https://docs.rs/figment/0.10.19/figment/?utm_source=chatgpt.com)
- Repository: [https://github.com/SergioBenitez/Figment](https://github.com/SergioBenitez/Figment?utm_source=chatgpt.com)
- Crate: [https://crates.io/crates/figment](https://crates.io/crates/figment?utm_source=chatgpt.com)

The documentation and registry currently identify 0.10.19 as the latest stable release. The discussion below targets that version rather than unreleased repository changes.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



## 1. Architectural orientation

Figment separates configuration acquisition, composition, and interpretation.

Configuration providers

TOML / JSON / YAML · Environment · Serde objects · Custom providers

Provider::data() + Metadata

Figment composition

Recursive merge/join · Array policies · Profiles · Provenance tags

Typed extraction and inspection

Serde models · Subtree extraction · Raw values · Metadata · Diagnostics

| Component              | Responsibility                                                               |
| ---------------------- | ---------------------------------------------------------------------------- |
| `Provider`             | Acquires and normalizes configuration from a source                          |
| `Figment`              | Combines providers, selects profiles, resolves conflicts, preserves metadata |
| `Profile`              | Defines an additional configuration namespace with default/global overlays   |
| `Value`, `Dict`, `Tag` | Represents untyped configuration data and its provenance                     |
| Serde `Deserialize`    | Converts composed values into application-defined types                      |
| `Metadata`, `Error`    | Supports source attribution, inspection, and structured diagnostics          |
| `Magic`, `Jail`        | Supports source-aware values and isolated configuration testing              |

### Why the architecture matters

Figment performs composition before application-specific deserialization. This creates several useful possibilities:

- Configuration inputs can be combined independently of the Rust structures that eventually consume them.
- Nested settings can be overridden individually without replacing entire configuration objects.
- Multiple consumers can extract different typed views from the same composed configuration.
- Configuration provenance remains inspectable after composition, rather than disappearing when values are converted into application types.
- Library and framework configuration can be composed through the same `Provider` interface as application-level configuration.

A `Figment` itself implements `Provider`, allowing independently constructed configuration graphs to be nested or composed without inventing another configuration representation. Source acquisition is eager when providers are added; typed extraction happens separately.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



## 2. Merge strategies and precedence

Figment provides four explicit conflict-resolution strategies. Their differences are more consequential than the order in which configuration sources are conventionally listed.

| Operation | Dictionary conflicts | Array conflicts            | Scalar/type conflicts |
| --------- | -------------------- | -------------------------- | --------------------- |
| `merge`   | Recursive union      | Incoming replaces existing | Incoming wins         |
| `join`    | Recursive union      | Existing retained          | Existing wins         |
| `admerge` | Recursive union      | Concatenate                | Incoming wins         |
| `adjoin`  | Recursive union      | Concatenate                | Existing wins         |

Source: [https://docs.rs/figment/0.10.19/figment/struct.Figment.html](https://docs.rs/figment/0.10.19/figment/struct.Figment.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.1 Recursive dictionary composition

When both conflicting values are dictionaries, Figment combines their keys and recursively resolves overlapping entries.

For example, consider the following configuration:

Existing

```
[server]
host = "localhost"
port = 8000

[logging]
level = "info"
```

Incoming

```
[server]
port = 9000
tls = true
```

Result under `merge`

```
[server]
host = "localhost"
port = 9000
tls = true

[logging]
level = "info"
```

The existing `server.host` and `logging.level` values survive, while `server.port` is overwritten.

This supports configuration overlays without serializing and replacing complete nested objects.

A consequential distinction: if one provider supplies a dictionary and another supplies a scalar or array at the same key, the types conflict. Figment does not preserve the dictionary's children through that replacement. The winning value is selected according to the operation.

### 2.2 Array semantics

Arrays are normally treated as atomic values, rather than collections whose elements can be individually merged.

- `merge` replaces an existing array with the incoming array.
- `join` retains the existing array.
- `admerge` and `adjoin` concatenate both arrays, preserving their existing-then-incoming order.

Array concatenation is valuable for configuration that naturally accumulates entries, such as plugin lists or search paths. It is not a keyed merge: Figment does not automatically identify matching objects within arrays, deduplicate entries, or merge array elements by an `id` field.

An important implication is that repeated composition with `admerge` or `adjoin` can accumulate duplicate entries. These operations express append semantics, not set semantics.

### 2.3 Precedence is a composition policy

`merge` and `join` allow precedence to be expressed without assuming that the last source must always win.

Typical structural possibilities include:

- Defaults followed by overrides: `merge` application-specific values over a default provider.
- Authoritative values followed by fallback: `join` lower-priority sources into an already populated configuration.
- Accumulating configuration: use `admerge` for arrays while applying incoming-wins semantics to scalars.
- Combining independently constructed sub-configurations: compose one `Figment` into another, retaining recursive resolution and provenance.

There is no need for a separate general-purpose deep-merge implementation when these four policies match the intended semantics.

However, composition is not equivalent to schema-aware patching. Figment does not natively implement deletion markers, per-field merge policies, arbitrary merge functions, or semantic array reconciliation.

## 3. Profiles: semi-hierarchical configuration

Profiles provide an orthogonal configuration dimension alongside nested keys.

Each provider exposes a map of:

`Profile → Dict<String, Value>`

A `Figment` combines these profile-specific dictionaries and resolves the selected profile during extraction.

### 3.1 Three profile roles

| Profile                                               | Meaning                                        | Precedence                                  |
| ----------------------------------------------------- | ---------------------------------------------- | ------------------------------------------- |
| `Profile::Default`                                    | Common fallback configuration                  | Lowest across profile overlays              |
| Custom profile, e.g. `debug`, `staging`, `production` | Profile-specific configuration                 | Overrides defaults                          |
| `Profile::Global`                                     | Configuration applied to all selected profiles | Overrides both default and selected profile |

The important distinction is that Global is a high-priority overlay, not a synonym for defaults.

For example, if `default` declares `port = 8000`, `staging` declares `port = 9000`, and `global` declares `port = 7000`, selecting `staging` produces `port = 7000`.

This global precedence holds even when a normal profile source is subsequently merged. Within the same profile, the chosen composition operation still determines conflicts.

### 3.2 Profile selection and discovery

- `Figment::select(profile)` returns a Figment with a different selected profile.
- `Figment::profile()` reports the selected profile.
- `Figment::profiles()` enumerates profiles with stored configuration; selection alone does not imply that a profile contains data.
- `Profile::new()` and `Profile::const_new()` construct dynamic and static profiles.
- `Profile::from_env()` and `Profile::from_env_or()` support environment-driven profile selection.
- Profile names are case-insensitive, so `STAGING` and `staging` identify the same profile.

Profile selection changes which already-collected values are considered during extraction. It is not a request to re-read configuration providers.

### 3.3 Profile nesting versus ordinary object nesting

File-based providers distinguish between `.profile("staging")` and `.nested()`.

- `.profile("staging")` interprets the whole file as settings belonging to the staging profile.
- `.nested()` interprets the file's top-level keys as names of different profiles.
- Without either, a file's configuration belongs to the default profile.

Consequently, a TOML file containing `[database]` means a nested `database` object by default, but `.nested()` would instead interpret `database` as a profile.

This distinction matters for both existing configuration formats and the design of reusable configuration files. Profiles are not automatically inferred from section names.

### 3.4 Provider-level profile selection

The `Provider` trait has an optional `profile()` method with semantics different from emitting data into a particular profile:

- `Provider::data()` controls which profiles receive the provider's values.
- `Provider::profile()` can select a profile on the receiving `Figment`.
- The selection hint applies when the provider is merged, not joined.

Thus, a custom provider can both supply configuration and announce a preferred active profile. This makes configuration-driven profile selection possible without hardcoding selection in the caller, but means merging the provider can change subsequent extraction behavior.

The profile model is particularly useful when a system shares defaults while supporting multiple environment-specific or independently configurable execution contexts. It is not an automatic inheritance hierarchy among arbitrary profiles: a custom profile inherits from `Default` and is overridden by `Global`, not from another arbitrary custom profile.

Sources:

[https://docs.rs/figment/0.10.19/figment/struct.Profile.html](https://docs.rs/figment/0.10.19/figment/struct.Profile.html?utm_source=chatgpt.com)

[https://docs.rs/figment/0.10.19/figment/trait.Provider.html](https://docs.rs/figment/0.10.19/figment/trait.Provider.html?utm_source=chatgpt.com)

[https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3

## 4. Built-in provider families

### 4.1 Provider capability map

| Provider          | Source                          | Principal capabilities                                         |
| ----------------- | ------------------------------- | -------------------------------------------------------------- |
| `Toml`            | File or string                  | Structured TOML, profile nesting, file discovery               |
| `Json`            | File or string                  | Structured JSON, profile nesting                               |
| `Yaml`            | File or string                  | YAML configuration and Serde conversion                        |
| `YamlExtended`    | File or string                  | YAML plus merge-key expansion                                  |
| `Env`             | Process environment             | Filtering, remapping, nesting, structured value parsing        |
| `Serialized<T>`   | Any `T: Serialize`              | Typed defaults, computed settings, arbitrary keyed overrides   |
| `(key, value)`    | Serde-serializable value        | Direct keyed global configuration                              |
| `Figment`         | Existing composed configuration | Composition of independently constructed configuration graphs  |
| Custom `Provider` | Arbitrary source                | User-defined loading, transformation, and metadata attribution |

Source: [https://docs.rs/figment/0.10.19/figment/providers/](https://docs.rs/figment/0.10.19/figment/providers/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 4.2 File and string providers: `Format` and `Data<F>`

Figment separates the parsing format from the configuration source.

`Format` defines how to deserialize textual input. `Data<F>` implements the actual provider, including file/string selection, file discovery, profile placement, and source metadata.

`Data<F>` construction and behavior

- `F::file(path)` — reads a file with upward directory search for relative paths.
- `F::file_exact(path)` — uses the specified path without searching parent directories.
- `F::string(contents)` — parses configuration already available in memory.
- `.profile(profile)` — places the parsed dictionary into a specified profile.
- `.nested()` — interprets top-level dictionary entries as separate profiles.

The `Format` trait must be in scope for `Toml::file()`, `Json::string()`, and equivalent calls.

Important file-discovery semantics

`file()` searches the current directory and successive parent directories for a relative file path, using the first match. This can support project-root configuration discovery without a bespoke path-search routine.

A missing optional file contributes no configuration, while parsing errors from a found file are retained as errors. `file_exact()` does not search parent directories, so its relative-path behavior is more predictable for applications with controlled working directories.

The provider's metadata records the file source when a file is loaded. This makes later diagnostics and source-relative paths possible.

There are meaningful boundaries:

- Parent-directory search is not the same as a configurable search-path registry.
- Figment does not automatically discover every matching file or load multiple files in a directory.
- File inclusion, recursive import directives, and file watching are not general built-in `Data` capabilities.
- Parsing is performed when the provider is composed, rather than on each subsequent extraction.

Documentation: [https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.3 YAML merge-key expansion

`YamlExtended` exposes an easily overlooked feature distinct from Figment's own configuration merging.

It supports YAML's historical `<<` merge-key syntax through `serde_yaml::Value::apply_merge()`. This permits YAML anchors and aliases to supply reusable mappings, with local keys overriding values obtained through the YAML merge key.

There are therefore two different operations:

1. YAML merge-key expansion: Happens inside the YAML parser while producing a provider's data.
2. Figment provider composition: Happens when the resulting provider is merged or joined with other sources.

This distinction matters because YAML merge-key expansion resolves mappings within a YAML document, whereas Figment's `merge` policies control precedence across providers.

`YamlExtended` supports a draft YAML 1.1 extension that is not part of YAML 1.2. The ordinary `Yaml` provider does not apply these merge keys.

Documentation: [https://docs.rs/figment/0.10.19/figment/providers/struct.YamlExtended.html](https://docs.rs/figment/0.10.19/figment/providers/struct.YamlExtended.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.4 Serialized configuration: `Serialized<T>`

`Serialized<T>` converts arbitrary Serde-serializable Rust data into configuration, avoiding the need to write temporary TOML/JSON or manually construct Figment values.

Its significant construction options are:

| API                                | Behavior                                                     |
| ---------------------------------- | ------------------------------------------------------------ |
| `Serialized::defaults(value)`      | Serialize a dictionary-shaped value into the default profile |
| `Serialized::globals(value)`       | Serialize a dictionary-shaped value into the global profile  |
| `Serialized::from(value, profile)` | Serialize a dictionary-shaped value into a chosen profile    |
| `Serialized::default(key, value)`  | Emit a single value at a key path in the default profile     |
| `Serialized::global(key, value)`   | Emit a single value at a key path in the global profile      |
| `.key(path)`                       | Assign a nested destination key                              |
| `.profile(profile)`                | Change the emitted profile                                   |

Keyed versus unkeyed values: Unkeyed serialization expects a dictionary-shaped root. Keyed serialization can accept scalars, arrays, or any other supported `Value`, because the key supplies the containing dictionary structure.

This is particularly useful for:

- Supplying typed defaults from `Default` implementations.
- Injecting configuration computed during program initialization.
- Combining CLI parsing results with file and environment settings.
- Exposing separately constructed component configurations.
- Overriding one deep key without constructing a complete configuration object.

The compact `(key, value)` provider is a shortcut for a global keyed value, not a default-profile value. It is useful for authoritative programmatic overrides, but its Global placement can be surprising when mixed with profile-specific settings.

Documentation: [https://docs.rs/figment/0.10.19/figment/providers/struct.Serialized.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Serialized.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 5. Environment variables: transformation and structured parsing

`Env` is more sophisticated than a simple string-valued environment-variable reader. It combines a configurable key-transformation pipeline with value parsing and nested dictionary construction.

### 5.1 Selecting and transforming keys

| API                       | Functionality                                            |
| ------------------------- | -------------------------------------------------------- |
| `Env::raw()`              | Select the process environment without initial filtering |
| `Env::prefixed("APP_")`   | Select prefixed variables and strip the prefix           |
| `.filter(predicate)`      | Retain selected keys                                     |
| `.map(transform)`         | Rewrite keys                                             |
| `.filter_map(transform)`  | Filter and rewrite simultaneously                        |
| `.only(&[...])`           | Whitelist keys                                           |
| `.ignore(&[...])`         | Exclude keys                                             |
| `.lowercase(bool)`        | Control automatic lowercasing                            |
| `.split(pattern)`         | Convert key separators into nested paths                 |
| `.profile(profile)`       | Place variables into a designated profile                |
| `.global()`               | Place variables into the global profile                  |
| `.iter()`                 | Inspect selected, transformed key/value pairs            |
| `Env::var`, `Env::var_or` | Convenient case-insensitive environment-variable lookup  |

Transformations can be chained, permitting policies such as multiple prefixes, namespace renaming, allowlists, exclusions, and nonstandard environment naming conventions without implementing a new provider.

Keys are treated case-insensitively during environment-variable selection and comparisons, with lowercase output by default.

### 5.2 Nested environment configuration

By default, `APP_DATABASE_HOST` becomes the key `database_host` after prefix removal and lowercasing.

With `.split("__")`, `APP_DATABASE__HOST` becomes the nested path `database.host`.

The distinction between `_` and `__` is conventional, not inherent: `.split(pattern)` splits on whatever pattern is specified. Using a double separator can preserve underscores within individual field names.

Custom `.map()` transformations are available when simple splitting is insufficient.

Collision consideration: `Env::iter()` documents no guaranteed iteration order. If distinct variables normalize to the same destination key, the provider can encounter collisions. Key transformations should therefore avoid ambiguous mappings rather than depending on which variable happens to be processed last.

### 5.3 Structured value parsing

In the published 0.10.19 environment provider, values can be parsed into Figment's typed value representation using a TOML-like syntax.

| Environment value              | Interpreted type      |
| ------------------------------ | --------------------- |
| `true`, `false`                | Boolean               |
| `42`, `-5`                     | Integer               |
| `1.25`                         | Floating-point number |
| `[1, 2, 3]`                    | Array                 |
| `{host="localhost",port=5432}` | Dictionary            |
| `"hello"`                      | Quoted string         |
| `hello`                        | Unquoted string       |

The parser also supports quoted-string escapes, including common escaped characters and Unicode escape sequences.

This has two useful consequences:

- Entire lists or nested objects can be supplied through one environment variable.
- Structured values can be merged with values from TOML, JSON, or other sources without requiring the application to parse their contents separately.

It also creates a semantic distinction: an environment variable is not necessarily retained as a string. For example, an unquoted value that resembles a number or boolean may be interpreted as such.

The parser is not an unrestricted TOML document parser. Invalid structured syntax may be interpreted as a literal string rather than producing the structured value a caller expected.

Another limitation is that non-Unicode environment values are converted using replacement characters when iterating, so byte-exact non-Unicode environment values are not preserved by the normal `Env` interface.

Documentation: [https://docs.rs/figment/0.10.19/figment/providers/struct.Env.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Env.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 6. Typed extraction, inspection, and subconfiguration reuse

The extraction layer is deliberately separate from provider composition. One composed `Figment` can be interpreted as multiple Rust types, examined as raw values, or narrowed to a subconfiguration.

| API                               | Capability                                                          | Important distinction                         |
| --------------------------------- | ------------------------------------------------------------------- | --------------------------------------------- |
| `extract::<T>()`                  | Deserialize the entire selected configuration into `T`              | Strict Serde interpretation                   |
| `extract_lossy::<T>()`            | Deserialize with relaxed numeric/boolean conversions                | Does not relax every type constraint          |
| `extract_inner::<T>("a.b")`       | Deserialize a value at a dotted key path                            | No need to define a root configuration struct |
| `extract_inner_lossy::<T>("a.b")` | Combine key-path extraction with relaxed conversion                 | Useful for heterogeneous textual inputs       |
| `focus("a.b")`                    | Produce another `Figment` rooted at a nested dictionary             | Preserves Figment-level operations            |
| `find_value("a.b")`               | Retrieve the underlying `Value`                                     | Supports untyped inspection and diagnostics   |
| `contains("a.b")`                 | Test whether a key exists in a successfully evaluated configuration | Returns `false` on configuration errors too   |
| `find_metadata("a.b")`            | Look up the source metadata for a value                             | Does not require typed deserialization        |

Source: [https://docs.rs/figment/0.10.19/figment/struct.Figment.html](https://docs.rs/figment/0.10.19/figment/struct.Figment.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.1 Strict versus lossy conversion

Strict extraction delegates interpretation to Serde's typed deserialization machinery.

Lossy extraction extends numeric and boolean interpretation to accept common textual representations. In particular:

- Strings such as `"true"`, `"yes"`, `"on"`, and `"1"` can deserialize as `true`.
- Strings such as `"false"`, `"no"`, `"off"`, and `"0"` can deserialize as `false`.
- The integers `0` and `1` can also be interpreted as booleans.
- Numeric strings can be parsed as integers or floating-point values.

These conversions are not unrestricted coercion. Arbitrary strings still fail numeric conversion, and lossy extraction does not automatically repair missing fields, invalid enum variants, incompatible object shapes, or semantic validation failures.

This is useful when a configuration combines inherently typed sources with textual interfaces, but strict extraction provides a narrower contract when implicit conversions are undesirable.

Source: [https://docs.rs/figment/0.10.19/figment/value/enum.Value.html](https://docs.rs/figment/0.10.19/figment/value/enum.Value.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.2 `focus()` versus `extract_inner()`

These mechanisms are superficially similar but support different architectural uses.

`extract_inner::<T>("service.database")` immediately interprets a subtree as one type.

`focus("service.database")` produces another `Figment` whose root is that subtree. The resulting Figment can be inspected, selected, further focused, extracted, or composed as a provider.

This permits a parent configuration to expose multiple reusable configuration components without forcing every component to depend on one application-wide struct.

A constraint: `focus()` is designed for dictionary-shaped subconfiguration. Focusing a scalar or array does not turn that value into an equivalent root dictionary.

### 6.3 Serde remains the schema and validation boundary

Figment supports Serde-compatible typed models, including nested structs, enums, optional fields, generic containers, and custom deserializers.

Serde attributes such as `default`, `rename`, `alias`, `deny_unknown_fields`, and `deserialize_with` can therefore shape the interpretation of configuration data. Validation implemented in a custom `Deserialize` implementation can participate in extraction.

This is a useful division of responsibility:

- Figment decides which source values survive composition.
- Serde decides how those values map into Rust types.
- Application-defined type constructors or validation logic enforce constraints that ordinary deserialization cannot express.

Figment does not provide a separate general schema language, cross-field constraint engine, or automatic schema migration facility. Those concerns can be integrated through Serde or complementary libraries.

One documented subtlety is that Serde's `flatten` attribute can complicate accurate error attribution. The effective diagnostic fidelity depends partly on how the target type deserializes its fields.

## 7. Internal value model and provenance

Figment's source-aware intermediate representation is an important distinction from libraries that simply deserialize each file into a temporary struct and discard source context.

### 7.1 `Value`, `Dict`, and `Map`

`figment::value::Value` represents a configuration value through the following variants:

| Variant  | Representation                                     |
| -------- | -------------------------------------------------- |
| `String` | String                                             |
| `Char`   | Character                                          |
| `Bool`   | Boolean                                            |
| `Num`    | Integer or floating-point number                   |
| `Empty`  | No-value representation, including `None` and unit |
| `Dict`   | Nested map of string keys to values                |
| `Array`  | Sequence of values                                 |

Every `Value` variant carries a `Tag`. The public `Map<K, V>` alias uses `BTreeMap`, while `Dict` specializes it to string keys and `Value` entries.

`Value` also provides `serialize()`, `deserialize()`, `find()`, `find_ref()`, type-specific accessors, conversion helpers, and tag inspection. Thus, custom providers can operate at the intermediate-representation level without requiring a typed configuration schema in advance.

The distinction between missing and present-but-empty is material. `Empty::None` and `Empty::Unit` are represented values, whereas an absent dictionary key has no `Value` at all. Composition operates over the values that exist; optionality and default interpretation are resolved by deserialization.

Source: [https://docs.rs/figment/0.10.19/figment/value/enum.Value.html](https://docs.rs/figment/0.10.19/figment/value/enum.Value.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.2 Metadata and tags

Each source contributes a `Metadata` record identifying its origin. The record can include:

- A human-readable provider name.
- A `Source`, such as a file path or a custom source identifier.
- The Rust call location at which the provider was added, when available.
- A custom key-path interpolater.

A `Tag` connects each value to its producing metadata. As values survive merges or replacements, the provenance associated with the surviving values remains available.

This supports inspecting not only a final setting's value, but also where that effective value came from.

`Metadata::interpolate()` is particularly useful for human-readable diagnostics: it can render a logical key such as `database.host` in the naming convention of its originating provider. For example, an environment provider can render uppercase environment-style names instead of the internal lowercase key path.

The core accessors are `Figment::metadata()`, `find_metadata(path)`, and `get_metadata(tag)`.

Source: [https://docs.rs/figment/0.10.19/figment/struct.Metadata.html](https://docs.rs/figment/0.10.19/figment/struct.Metadata.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.3 `Tagged<T>`: retain provenance in typed values

`figment::value::magic::Tagged<T>` is a less obvious advanced feature.

Ordinary extraction turns a configuration value into an application-defined Rust type, normally dropping the provenance information from the extracted field.

By declaring a field as `Tagged<T>`, a consumer can retain the value's provenance tag together with its typed contents. It can then obtain the corresponding metadata from the original Figment.

This is useful for source-aware configuration inspection, diagnostics, migration tooling, or cases where a field's interpretation depends on its origin.

The important lifetime distinction is that the tag is an identifier, not a self-contained source record. Accessing associated provider metadata requires the relevant Figment; a tag alone does not reproduce the entire provenance context.

Documentation: [https://docs.rs/figment/0.10.19/figment/value/magic/struct.Tagged.html](https://docs.rs/figment/0.10.19/figment/value/magic/struct.Tagged.html?utm_source=chatgpt.com)

### 7.4 `RelativePathBuf`: configuration-file-relative paths

`figment::value::magic::RelativePathBuf` uses the same provenance infrastructure for a practical semantic problem: relative paths in configuration files.

Suppose a configuration file at `/etc/example/config.toml` specifies `certs/server.pem`.

With an ordinary `PathBuf`, the path is simply relative to the process working directory unless application code resolves it differently.

With `RelativePathBuf`:

- `.original()` returns the path as written.
- `.relative()` resolves it relative to the file that supplied the value, if that file origin is known.
- `.metadata_path()` exposes the underlying source-file path information.
- `.serialize_original()` and `.serialize_relative()` control serialized representation.

Absolute paths remain absolute. Values originating from an environment variable or an in-memory provider do not automatically gain a configuration-file base path.

A notable capability is preservation of path-origin semantics when a `RelativePathBuf` is serialized into another Figment and subsequently extracted again.

There is also a serialization distinction: its default structured serialization preserves the special path semantics and is not interchangeable with ordinary `PathBuf` serialization. The explicit serialization helpers are relevant when producing more conventional output.

Source: [https://docs.rs/figment/0.10.19/figment/value/magic/struct.RelativePathBuf.html](https://docs.rs/figment/0.10.19/figment/value/magic/struct.RelativePathBuf.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.5 `Magic` and `Either`

`RelativePathBuf` and `Tagged<T>` rely on Figment's special source-aware deserialization facilities, exposed through the `Magic` mechanism.

`figment::value::magic::Either<A, B>` supports deserializing one of two alternatives while accommodating these special value types. It is relevant when a field must accept alternative representations, one of which requires Figment-specific metadata-aware decoding.

This is distinct from general-purpose Serde `untagged` enums: normal Serde representation matching does not necessarily preserve the special deserialization behavior required by Figment's magic types.

Documentation: [https://docs.rs/figment/0.10.19/figment/value/magic/enum.Either.html](https://docs.rs/figment/0.10.19/figment/value/magic/enum.Either.html?utm_source=chatgpt.com)

## 8. Errors and diagnostic behavior

Figment retains provider and deserialization failures in a structured `Error` rather than reducing them to plain strings.

`Error` exposes:

| Field or method | Diagnostic purpose                         |
| --------------- | ------------------------------------------ |
| `kind`          | Structural error classification            |
| `profile`       | Active profile associated with the failure |
| `path`          | Key path where the problem occurred        |
| `metadata`      | Source attribution when known              |
| `missing()`     | Recognize a missing-field error            |
| `with_path()`   | Add key-path context in custom errors      |
| `chain()`       | Combine errors                             |
| `count()`       | Count errors represented                   |
| `IntoIterator`  | Iterate through multiple collected errors  |

The underlying `Kind` representation distinguishes failures such as missing fields, invalid types, invalid values, unknown variants, unknown fields, duplicate fields, and provider-originated messages.

Deferred failure behavior: When a provider is added, Figment attempts to obtain its configuration and stores any failure. The error is surfaced when an extraction or another fallible evaluation is performed. Adding another provider does not silently erase an earlier source failure.

Multiple failing providers can produce an error chain. However, this should not be interpreted as a guarantee that every possible schema violation in one deserialization pass is discovered and aggregated; normal deserialization can stop at an encountered error.

An important inspection trap is `contains(path)`: it returns `false` if the configuration cannot be successfully evaluated, as well as when the key is absent. `find_value(path)` is the better API when callers must distinguish missing values from configuration errors.

Source: [https://docs.rs/figment/0.10.19/figment/struct.Error.html](https://docs.rs/figment/0.10.19/figment/struct.Error.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 9. Extensibility: custom providers and formats

Figment exposes two different extension boundaries: custom configuration sources and custom textual formats.

### 9.1 Implementing `Provider`

The central interface is intentionally small:

| Trait member                                       | Contract                                    |
| -------------------------------------------------- | ------------------------------------------- |
| `metadata(&self) -> Metadata`                      | Identify and describe the provider          |
| `data(&self) -> Result<Map<Profile, Dict>, Error>` | Supply profile-indexed configuration values |
| `profile(&self) -> Option<Profile>`                | Optionally select a profile when merged     |

A custom provider is not required to produce TOML, JSON, or any other text representation. It can construct `Value` trees directly or use Serde serialization.

This allows integration with configuration held in databases, remote services, application state, CLI arguments, or domain-specific sources.

The architectural opportunity is to centralize source-specific behavior inside providers while leaving precedence, profile resolution, extraction, and error reporting to Figment.

Provider contracts and limitations:

- The public `Provider` interface is synchronous. Integrating an asynchronous source requires obtaining or preparing its values outside the trait call, or synchronously adapting the operation.
- Providers should return meaningful `Metadata`, rather than treating source attribution as optional boilerplate.
- Providers choose how their data maps into profiles. Merely returning a dictionary does not give the receiving Figment a selected profile.
- The special `Provider::profile()` selection hint applies to `merge` operations, not `join`.
- Figment provides no general source-specific caching, authentication, polling, or retry protocol; these remain responsibilities of the source integration.

A further useful capability is that `&T` implements `Provider` when `T` does. A caller can therefore reuse a provider by reference rather than transferring ownership, where the underlying provider's implementation supports the desired reuse.

Documentation: [https://docs.rs/figment/0.10.19/figment/trait.Provider.html](https://docs.rs/figment/0.10.19/figment/trait.Provider.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.2 Implementing `Format`

For a new text-based format, implementing `Format` can be simpler than implementing `Provider` directly.

`Format` requires:

- `const NAME: &'static str`
- An associated `Error` type implementing `std::error::Error`
- `from_str<T: DeserializeOwned>()`, which parses a source string into a Serde-compatible type.

It supplies the common source-handling machinery through `Data<F>`, including `file()`, `file_exact()`, `string()`, and `from_path()`.

This separates how text is parsed from where it was obtained. A custom text format automatically benefits from the same general file, string, metadata, and profile facilities as built-in formats.

It is most useful when a source is fundamentally a textual data format that already has, or can provide, a Serde deserializer.

A source that fetches records dynamically or needs specialized acquisition behavior is more naturally implemented as a `Provider`.

`Format` is generic rather than dynamically dispatched; the published trait is not `dyn` compatible.

Documentation: [https://docs.rs/figment/0.10.19/figment/providers/trait.Format.html](https://docs.rs/figment/0.10.19/figment/providers/trait.Format.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.3 Reusable configuration for libraries and frameworks

The upstream documentation describes a powerful integration pattern for library authors: make a library's configuration type itself implement `Provider`.

A reusable library can expose:

- A typed configuration structure.
- A `Default` implementation describing baseline settings.
- A `Config::figment()` constructor representing its default configuration sources.
- A conversion or constructor that extracts the final configuration from a caller-supplied provider.

This enables consumers to extend or replace library defaults using normal Figment composition rather than requiring a separate override API for every configuration field.

It also permits application-wide configuration and embedded library configuration to share a common intermediate representation while retaining independent typed extraction boundaries.

An important distinction is that using Figment as a library-facing configuration abstraction does not require the consumer to adopt the library author's file format or precedence assumptions. The provider boundary allows those choices to remain flexible.

Source: [https://docs.rs/figment/0.10.19/figment/](https://docs.rs/figment/0.10.19/figment/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.4 Supporting utilities

The `figment::util` module contains several small capabilities useful when implementing providers or adapters.

| Utility                  | Purpose                                                     |
| ------------------------ | ----------------------------------------------------------- |
| `map!`                   | Construct generic maps concisely                            |
| `nest(path, value)`      | Build nested dictionaries from dotted key paths             |
| `diff_paths(path, base)` | Compute relative paths                                      |
| `bool_from_str_or_int`   | Serde helper for flexible boolean interpretation            |
| `vec_tuple_map`          | Serialize/deserialize maps as sequences of key/value tuples |

`nest()` is especially relevant for providers that translate flat source keys into structured configuration dictionaries.

`vec_tuple_map` helps where a data representation needs sequence-of-pairs semantics rather than an ordinary mapping representation; it does not change Figment's normal `Dict` key model.

Documentation: [https://docs.rs/figment/0.10.19/figment/util/](https://docs.rs/figment/0.10.19/figment/util/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 10. Integrations and ecosystem boundaries

### 10.1 Rocket web framework

Figment is the configuration foundation of the Rust Rocket framework.

Rocket uses Figment to compose default configuration, configuration files, environment variables, and user-supplied providers. It exposes configuration facilities such as `Config::figment()` and `rocket::custom()`, and reexports Figment types under `rocket::figment`.

This is a particularly strong example of Figment's ability to serve as a framework-level configuration extension contract, not merely as an application startup helper.

The distinction is important: Rocket defines its own configuration defaults, recognized settings, and framework initialization behavior. Those are Rocket capabilities built on Figment, not automatically provided by Figment to arbitrary applications.

Documentation: [https://rocket.rs/guide/v0.5/configuration/](https://rocket.rs/guide/v0.5/configuration/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rocket.rs\&sz=32)

Rocket Web Framework



### 10.2 CLI parsers, including `clap`

Figment does not parse command-line arguments itself. A CLI parser such as `clap` can produce a Rust structure whose values are then exposed through `Serialized<T>`.

This permits CLI values to participate in the same composition graph as files, environment variables, and defaults.

The main semantic consideration is whether a CLI option was actually supplied. A parser may populate absent options with defaults; blindly merging the fully populated parser result can unintentionally override values from other sources.

Nullable options, explicit occurrence detection, or a separate overrides structure can preserve the distinction between an unspecified argument and an intentional override.

That distinction belongs to the CLI integration layer, while Figment handles the subsequent composition.

### 10.3 Secret-file adapters

The companion crate `figment_file_provider_adapter` implements a reusable provider decorator.

It allows configuration such as `API_KEY_FILE=/run/secrets/api_key` to be interpreted as the contents of the indicated file and exposed under the key `API_KEY`.

Its capabilities include:

- Wrapping an existing Figment provider.
- Recognizing the `_FILE` suffix.
- Customizing that suffix.
- Restricting file-value interpretation to selected keys.
- Giving ordinary values priority when both `KEY` and `KEY_FILE` are present.

This supports file-mounted secrets and container deployment conventions without requiring a new source parser.

It is not part of Figment's core crate. Nor does the adapter itself provide an encrypted secret store, access-control service, or secret rotation mechanism.

Documentation: [https://docs.rs/figment_file_provider_adapter/latest/figment_file_provider_adapter/](https://docs.rs/figment_file_provider_adapter/latest/figment_file_provider_adapter/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 11. Test infrastructure: `Jail`

Figment provides a dedicated testing facility, `Jail`, enabled by the `test` feature.

It creates a temporary working environment suitable for testing configuration files, environment values, directory discovery, and precedence behavior.

| Capability                                      | API                                                |
| ----------------------------------------------- | -------------------------------------------------- |
| Run an isolated test closure returning `Result` | `Jail::try_with()`                                 |
| Run a test closure and panic on failure         | `Jail::expect_with()`                              |
| Create files and directories                    | `create_file()`, `create_binary()`, `create_dir()` |
| Modify the test working directory               | `change_dir()`                                     |
| Set or clear environment variables              | `set_env()`, `clear_env()`                         |
| Inspect the temporary directory                 | `directory()`                                      |

`Jail` handles cleanup and synchronizes its own invocations to avoid competing changes to process-global environment variables and the current working directory.

This makes it valuable for testing behavior that is otherwise awkward to exercise deterministically: parent-directory file search, profile-specific configurations, environment-variable collision behavior, error provenance, and source-relative paths.

It is not a security sandbox or a process-isolated environment. Environment variables and the current directory remain process-global state. The synchronization guarantees are for code operating through `Jail`; unrelated code mutating those globals is not inherently coordinated.

Documentation: [https://docs.rs/figment/0.10.19/figment/struct.Jail.html](https://docs.rs/figment/0.10.19/figment/struct.Jail.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 12. Features, runtime characteristics, and limitations

### 12.1 Cargo feature flags

In the published 0.10.19 crate, no feature flags are enabled by default. Serde-backed core functionality is available without optional provider features.

| Feature       | Enables                                                            |
| ------------- | ------------------------------------------------------------------ |
| `env`         | Environment provider and its parsing support                       |
| `toml`        | TOML provider                                                      |
| `json`        | JSON provider                                                      |
| `yaml`        | YAML and YAML Extended providers                                   |
| `test`        | `Jail` testing facilities                                          |
| `parse-value` | Environment-value parsing support as an internal/optional facility |

The published dependency graph uses `toml` 0.8, `serde_json` 1.x, `serde_yaml` 0.9, and `pear` 0.2 for relevant optional functionality.

A configuration needing environment and TOML providers would explicitly enable `features = ["env", "toml"]`.

Version-sensitive caution: the repository's current `master` manifest differs from the published 0.10.19 package, including its default feature set and TOML parsing dependency. Those repository changes should not be assumed to be released functionality merely because the manifest still identifies itself as 0.10.19.

The published crate manifest and version-specific documentation are the appropriate authority for this reference.

Published feature metadata: [https://docs.rs/crate/figment/0.10.19/features](https://docs.rs/crate/figment/0.10.19/features?utm_source=chatgpt.com)

Published manifest: [https://docs.rs/crate/figment/0.10.19/source/Cargo.toml](https://docs.rs/crate/figment/0.10.19/source/Cargo.toml?utm_source=chatgpt.com)

Development manifest: [https://github.com/SergioBenitez/Figment/blob/master/Cargo.toml](https://github.com/SergioBenitez/Figment/blob/master/Cargo.toml?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2

### 12.2 Lifecycle, concurrency, and performance implications

Figment's execution model has three significant stages:

1. Providers read and normalize their inputs when composed.
2. Configuration values and metadata are retained in Figment's internal representation.
3. Typed extraction resolves the relevant profile overlays and performs deserialization.

This has several implications.

Snapshot-like acquisition. Figment does not automatically watch files or environment variables and update already-collected values. A caller requiring updated external configuration must arrange to read and compose a new configuration, or otherwise manage rebuilding the relevant Figment.

Repeated extraction. The same composed Figment can be used to obtain multiple typed views. This avoids repeatedly invoking external source providers simply to deserialize different portions of the configuration.

Composition costs. Recursive dictionary operations and retained provenance have processing and memory costs. Figment is not a zero-allocation view over a collection of source files, nor does it claim to eliminate the need for application-level configuration validation.

Concurrency. The resulting `Figment` implements `Send` and `Sync` in the published API. This permits a composed configuration to be shared across threads. It does not make process-environment mutation or file acquisition automatically atomic, and it does not imply that all provider implementations have identical thread-safety properties.

I found no sufficiently relevant official benchmark establishing a quantitative advantage of Figment against another configuration library under comparable workloads. Accordingly, performance expectations should be based on the operations and source sizes involved rather than an assumed speed ranking.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 12.3 Key limitations and consequential distinctions

| Area                  | What Figment provides                                | What remains outside its native scope                              |
| --------------------- | ---------------------------------------------------- | ------------------------------------------------------------------ |
| Configuration sources | Files, environment, Serde values, custom providers   | Built-in remote configuration protocols and authentication         |
| Precedence            | Four recursive conflict-resolution strategies        | Arbitrary field-specific merge functions or deletion patches       |
| Profiles              | Default, selected custom, and global overlay         | General multi-parent profile inheritance graphs                    |
| Typed interpretation  | Serde deserialization, strict/lossy conversions      | Independent schema evolution and cross-field rule engines          |
| Arrays                | Atomic replacement/retention or concatenation        | Deduplication and keyed element reconciliation                     |
| Errors                | Source-aware, structured, chainable failures         | Guaranteed exhaustive collection of all possible validation errors |
| Lifecycle             | Eager loading and reusable extraction                | Automatic live reload and change notification                      |
| Security              | Provider abstraction usable with secret integrations | Native credential vault, encryption, or secret redaction policy    |
| Testing               | Controlled temporary environment via `Jail`          | True OS-level process isolation                                    |

There are also two subtle cross-cutting distinctions worth preserving.

First, value precedence and profile precedence are independent concepts. A source may win within its profile yet still be superseded by the Global profile during extraction.

Second, provenance survives value composition, but it is not a complete history of every overwritten value. Tags identify the producing sources of the values retained in the resulting configuration. Figment is not an audit-log system recording every previous value and conflict decision.

## 13. Overall capability assessment

Figment is especially differentiated by the combination of four mechanisms:

- Recursive source composition: explicit precedence policies, with special array behavior.
- Semi-hierarchical profiles: configurable overlays without introducing separate configuration schemas for every environment.
- Source-aware typed interpretation: Serde extraction, raw value access, diagnostics, and provenance-preserving magic types.
- A reusable provider contract: composition that can be shared among applications, embedded libraries, frameworks, and custom sources.

These mechanisms collectively make Figment more than a file-format loader. It can act as a durable configuration integration layer between independently managed sources and strongly typed application interfaces.

Its strongest architectural opportunity is avoiding duplicated configuration plumbing: bespoke deep-merge logic, environment-to-struct mapping, profile overlays, source-relative path resolution, provider-source diagnostics, and library-specific override layers can often be represented using native Figment capabilities.

Its boundaries are equally clear. It is not a live configuration service, generalized validation engine, secret-management platform, or declarative schema migration system. Where those capabilities are needed, Figment can provide the compositional and typed extraction layer beneath them, while the additional responsibilities remain with specialized components.

### Primary reference map

| Investigation topic                               | Direct source                                                                                                                                                                             |
| ------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Conceptual overview and library integration       | [https://docs.rs/figment/0.10.19/figment/](https://docs.rs/figment/0.10.19/figment/?utm_source=chatgpt.com)                                                                               |
| Merge, join, extraction, focus, profile selection | [https://docs.rs/figment/0.10.19/figment/struct.Figment.html](https://docs.rs/figment/0.10.19/figment/struct.Figment.html?utm_source=chatgpt.com)                                         |
| Custom providers                                  | [https://docs.rs/figment/0.10.19/figment/trait.Provider.html](https://docs.rs/figment/0.10.19/figment/trait.Provider.html?utm_source=chatgpt.com)                                         |
| Text-format extension                             | [https://docs.rs/figment/0.10.19/figment/providers/trait.Format.html](https://docs.rs/figment/0.10.19/figment/providers/trait.Format.html?utm_source=chatgpt.com)                         |
| Environment transformation and parsing            | [https://docs.rs/figment/0.10.19/figment/providers/struct.Env.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Env.html?utm_source=chatgpt.com)                             |
| File-based providers                              | [https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html?utm_source=chatgpt.com)                           |
| Typed and programmatic configuration providers    | [https://docs.rs/figment/0.10.19/figment/providers/struct.Serialized.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Serialized.html?utm_source=chatgpt.com)               |
| Profiles                                          | [https://docs.rs/figment/0.10.19/figment/struct.Profile.html](https://docs.rs/figment/0.10.19/figment/struct.Profile.html?utm_source=chatgpt.com)                                         |
| Internal value representation                     | [https://docs.rs/figment/0.10.19/figment/value/enum.Value.html](https://docs.rs/figment/0.10.19/figment/value/enum.Value.html?utm_source=chatgpt.com)                                     |
| Provenance metadata                               | [https://docs.rs/figment/0.10.19/figment/struct.Metadata.html](https://docs.rs/figment/0.10.19/figment/struct.Metadata.html?utm_source=chatgpt.com)                                       |
| Source-aware path handling                        | [https://docs.rs/figment/0.10.19/figment/value/magic/struct.RelativePathBuf.html](https://docs.rs/figment/0.10.19/figment/value/magic/struct.RelativePathBuf.html?utm_source=chatgpt.com) |
| Error model                                       | [https://docs.rs/figment/0.10.19/figment/struct.Error.html](https://docs.rs/figment/0.10.19/figment/struct.Error.html?utm_source=chatgpt.com)                                             |
| Test facilities                                   | [https://docs.rs/figment/0.10.19/figment/struct.Jail.html](https://docs.rs/figment/0.10.19/figment/struct.Jail.html?utm_source=chatgpt.com)                                               |
| Published source and manifest                     | [https://docs.rs/crate/figment/0.10.19/source/](https://docs.rs/crate/figment/0.10.19/source/?utm_source=chatgpt.com)                                                                     |

All capability discussions above refer to the published stable 0.10.19 API, with development-branch differences called out separately.