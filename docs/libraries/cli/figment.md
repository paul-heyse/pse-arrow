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

# Figment — Technical Capability Addendum

Target version: 0.10.19 (published stable) | Research date: November 7, 2026

This addendum supplements the existing Figment — Rust Configuration Composition Library reference. It develops additional capabilities and consequential implementation details concerning file acquisition, provenance semantics, metadata extensibility, profile identity, configuration addressing, environment providers, numeric representation, and testing utilities.

All functionality described as available refers to the published Figment 0.10.19 crate. Differences in upstream development sources are identified separately.

## 1. File provider discovery, acquisition, and required-file semantics

### 1.1 File discovery and file loading occur at different stages

The published `Data<F>` implementation distinguishes between resolving a file's location and reading its contents. This distinction is especially important because `Data::file()` and `Data::file_exact()` do not merely apply different search policies: they also retain different information when constructed.

`Data::file(path)` performs file discovery immediately, when the provider is constructed. For a relative path, it searches the current working directory and successive parent directories. The first matching regular file is retained as a `PathBuf`. If no file is found, the provider retains the absence of a path.

`Data::file_exact(path)`, by contrast, stores the supplied path without checking its existence or searching parent directories.

The subsequent `Provider::data()` call performs file acquisition and parsing, normally when the provider is added to a `Figment`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



| Behavior                            | `Data::file()`                    | `Data::file_exact()`        |
| ----------------------------------- | --------------------------------- | --------------------------- |
| Parent-directory discovery          | Yes, for relative paths           | No                          |
| Path resolution during construction | Yes                               | No existence check          |
| Stored file reference               | Resolved path or absence          | Supplied path               |
| Missing file at construction        | Retains absence                   | Retains supplied path       |
| Missing file at composition         | Emits no data if discovery failed | Returns a provider error    |
| File content acquisition            | During provider composition       | During provider composition |

This produces several consequences that are not immediately apparent from the public constructor signatures.

Discovery is effectively snapshotted, but file contents are not. A `file()` provider constructed before a configuration file exists will continue to contribute no data even if the file is subsequently created before composition. Conversely, when discovery succeeds, the file's contents are not necessarily read until later. Changes between discovery and composition can therefore affect the configuration that is ultimately loaded.

For `file_exact()`, a relative path is retained rather than converted to a discovered absolute path. Consequently, the working directory at the time of file acquisition can determine which file is opened, potentially differing from the working directory when the provider was constructed.

A missing file handled by `file()` is therefore fundamentally different from a read failure encountered by `file_exact()`. The former contributes an empty provider; the latter produces an error that is retained by Figment and surfaced through fallible evaluation.

### 1.2 Required-file behavior without an explicit existence check

The strict behavior of `file_exact()` provides a useful configuration-loading contract when a particular file must exist.

An optional configuration source can use `file()`, accepting its discovery semantics. A required configuration source can use `file_exact()`, causing missing-file and I/O failures to enter Figment's normal provider-error machinery rather than requiring an independent preflight existence check.

This is not equivalent to schema-level required-field validation. A missing `file()` provider may still lead to a typed extraction error if mandatory fields cannot be obtained from other sources. The distinction is whether the missing source itself constitutes an error, independently of the final configuration schema.

The published 0.10.19 API does not expose configurable `required()` or `search()` builder methods. Those appear in newer upstream development code and must not be substituted for the published constructors.

Primary source: [https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html?utm_source=chatgpt.com)

Published implementation: [https://docs.rs/figment/0.10.19/src/figment/providers/data.rs.html](https://docs.rs/figment/0.10.19/src/figment/providers/data.rs.html?utm_source=chatgpt.com)

## 2. Custom metadata construction and source-native diagnostics

### 2.1 Metadata is an extensible provider contract

The original reference discusses how provenance metadata is retrieved and retained. A complementary capability is that custom providers can define the metadata associated with their configuration sources, including how logical configuration paths are represented in the source's own naming system.

The relevant public construction APIs are:

| API                                    | Capability                                                                    |
| -------------------------------------- | ----------------------------------------------------------------------------- |
| `Metadata::named(name)`                | Construct metadata identifying a provider without an explicit source location |
| `Metadata::from(name, source)`         | Construct metadata with a source identifier                                   |
| `Metadata::source(source)`             | Assign a source to existing metadata                                          |
| `Metadata::interpolater(f)`            | Install custom source-native key-path formatting                              |
| `Metadata::interpolate(profile, keys)` | Apply that formatting to a profile and key path                               |

The interpolater receives a `&Profile` and a slice of key components, returning a `String`. Its closure must implement `Fn`, `Clone`, `Send`, and `Sync`, and have a `'static` lifetime.

For example, a custom provider can identify a remote configuration source and represent errors using paths corresponding to that service's namespace.

```
use figment::Metadata;

let metadata = Metadata::named("Configuration Service")
    .source("config://services")
    .interpolater(|profile, keys| {
        if profile.is_custom() {
            format!("{}/{}", profile, keys.join("/"))
        } else {
            keys.join("/")
        }
    });
```

This example illustrates metadata construction; the returned `Metadata` would ordinarily be exposed by a custom `Provider::metadata()` implementation.

Rather than rendering an error at an internal path such as `database.primary.timeout`, the provider could represent the offending setting as `production/database/primary/timeout`, or another source-native identifier.

Figment then uses the interpolater in its existing diagnostic machinery. This permits source-specific diagnostics without replacing the core extraction or error model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.2 Source classification and inspection

The published `Source` enum distinguishes three source categories:

| Variant                                    | Meaning                                 | Useful accessor   |
| ------------------------------------------ | --------------------------------------- | ----------------- |
| `Source::File(PathBuf)`                    | Filesystem-backed configuration         | `file_path()`     |
| `Source::Code(&'static Location<'static>)` | Programmatically supplied configuration | `code_location()` |
| `Source::Custom(String)`                   | Arbitrary source identifier             | `custom()`        |

`Source` is marked `#[non_exhaustive]`, so exhaustive matching in downstream code must accommodate additional variants.

The distinction between a provider's source and its registration location is important. `Metadata::source` describes where the configuration came from, whereas `Metadata::provide_location` identifies the source-code location where the provider was added to Figment, when known.

These represent two independent forms of attribution: the configuration's origin and the code path responsible for incorporating that origin.

File-backed sources also have different semantics from custom sources. In particular, `RelativePathBuf` relies on file-origin metadata to establish a configuration-file-relative base. Attaching a remote URI through `Source::Custom` does not automatically give relative paths remote-resolution semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.3 Limits of source-native interpolation

Custom interpolation should not be confused with reversible source mapping.

An interpolater can render a logical key using a source's conventions, but it is not required to preserve or reconstruct the original source identifier exactly. When a provider performs arbitrary key rewriting, multiple original names may map to the same normalized key.

Likewise, a normal `Provider::metadata()` implementation supplies metadata for the provider as a whole. It does not constitute an unrestricted API for attaching an unrelated source identity to every emitted field.

Where configuration combines independently attributed origins, preserving those boundaries through separate providers or an appropriate composition abstraction can be more faithful than representing all values as originating from one synthetic source.

Primary sources:

[https://docs.rs/figment/0.10.19/figment/struct.Metadata.html](https://docs.rs/figment/0.10.19/figment/struct.Metadata.html?utm_source=chatgpt.com)

[https://docs.rs/figment/0.10.19/figment/enum.Source.html](https://docs.rs/figment/0.10.19/figment/enum.Source.html?utm_source=chatgpt.com)

## 3. Provenance semantics under recursive composition

### 3.1 Parent-container provenance is not necessarily child provenance

Figment's value representation associates a `Tag` with every value, including dictionaries, arrays, and their individual descendants. However, composition can produce containers whose nested values originate from several providers.

The published `Coalescible for Value` implementation makes the distinction explicit.

For dictionary conflicts, recursive composition retains the existing dictionary's outer tag under `join` and `adjoin`, or the incoming dictionary's outer tag under `merge` and `admerge`. Nested members are combined individually.

Consequently, the metadata of a composed dictionary does not necessarily describe the origin of every member within that dictionary.

A similar distinction applies to concatenated arrays, but their container-level tag policy is different.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 3.2 Array concatenation retains the existing container tag

For `adjoin` and `admerge`, the published implementation is equivalent to:

```
(A(t, mut existing), A(_, incoming), Adjoin | Admerge) => {
    existing.extend(incoming);
    A(t, existing)
}
```

Here, `A` denotes `Value::Array`. The original array's outer tag is retained, even when `admerge` would otherwise prefer incoming scalar values.

The appended elements retain their individual `Value` objects and associated tags.

This yields a noteworthy difference between the two levels of provenance:

| Configuration object             | Provenance after array concatenation |
| -------------------------------- | ------------------------------------ |
| Concatenated array container     | Existing array's tag                 |
| Elements originally in the array | Their existing individual tags       |
| Appended elements                | Their incoming individual tags       |

For instance, an array representing a list of enabled integrations may contain entries from a file, environment variables, and programmatic configuration, even though the array container itself points to only one source.

A source-aware inspection system should therefore inspect the nested elements rather than assume that the array's outer metadata explains the entire list.

This makes Figment suitable for retaining meaningful element provenance across additive composition, while also establishing a limit: container provenance is not a complete record of composition history.

Source: [https://docs.rs/crate/figment/0.10.19/source/src/coalesce.rs](https://docs.rs/crate/figment/0.10.19/source/src/coalesce.rs?utm_source=chatgpt.com)

### 3.3 Provider metadata inventory differs from effective-value attribution

`Figment::metadata()` exposes an iterator over collected provider metadata in the order that providers were added.

This is distinct from `Figment::find_metadata(path)` or `Figment::get_metadata(tag)`, which resolve provenance associated with particular values or tags.

The distinction permits several forms of inspection.

An application can enumerate registered configuration sources independently of which source supplied a particular surviving field. Conversely, it can inspect a particular field's effective origin without attempting to infer precedence from provider order.

The provider metadata inventory can include sources whose data did not survive a conflict, sources affecting another profile, or sources contributing no values. It should not be interpreted as the set of currently winning providers.

There is an additional diagnostic distinction: `find_metadata(path)` returns an `Option`, using `find_value(path).ok()?.tag()` internally. Therefore, a missing metadata result does not necessarily prove the key was absent; an underlying configuration-evaluation error can also prevent retrieval.

Use `find_value()` when a distinction between absence and evaluation failure is required.

Although Figment retains metadata associated with earlier providers, it does not retain a publicly navigable history of all overwritten values and their conflict decisions. Enumerating metadata cannot reconstruct that history.

Source: [https://docs.rs/figment/0.10.19/figment/struct.Figment.html](https://docs.rs/figment/0.10.19/figment/struct.Figment.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 4. Profile information encoded in `Tag`

### 4.1 `Tag::profile()` is not equivalent to retrieving the originating profile name

A `Tag` encodes both an identifier associated with metadata and a compact profile classification.

The public `Tag::profile()` API returns `Option<Profile>`, but the published implementation cannot reconstruct the names of arbitrary custom profiles.

Internally, the tag allocates two bits to a `ProfileTag` category. The categories are `Default`, `Global`, and `Custom`.

The corresponding published conversion behavior is:

| Encoded profile category | `Tag::profile()`         |
| ------------------------ | ------------------------ |
| Default                  | `Some(Profile::Default)` |
| Global                   | `Some(Profile::Global)`  |
| Custom                   | `None`                   |

All arbitrary custom profile names share the same encoded category. A tag from `staging` and a tag from `production` do not themselves retain those distinct profile names.

Documentation discrepancy: The published rustdoc description of `Tag::profile()` says that a result is returned for `Default` or `Custom`. The published `ProfileTag` conversion implementation instead returns a result for `Default` or `Global`, and `None` for `Custom`. The implementation is the basis for the behavior described here.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



This distinction matters whenever the exact origin profile, rather than merely its category, must be retained.

### 4.2 Default tags versus default-profile values

`Tag::Default` is a distinguished tag without associated provider metadata. It should not be confused with a normal value originating from `Profile::Default`.

Accordingly, `Tag::is_default()` tests whether a tag equals that distinguished default tag. It does not answer whether the value originated from the default configuration profile.

A value emitted into `Profile::Default` by an ordinary provider can have a nondefault tag, with `Tag::profile()` returning `Some(Profile::Default)`.

These are separate properties: one describes whether the tag is the special metadata-free identifier; the other describes its encoded profile category.

`Figment::profile()` also serves a different purpose. It reports the active profile selected for extraction rather than the profile classification associated with an individual value.

Sources:

[https://docs.rs/figment/0.10.19/figment/value/struct.Tag.html](https://docs.rs/figment/0.10.19/figment/value/struct.Tag.html?utm_source=chatgpt.com)

[https://docs.rs/crate/figment/0.10.19/source/src/profile.rs](https://docs.rs/crate/figment/0.10.19/source/src/profile.rs?utm_source=chatgpt.com)

## 5. Configuration key paths and addressability limits

### 5.1 Dotted paths are structural paths, not arbitrary dictionary keys

Figment's dotted key notation represents navigation through nested dictionaries. It does not provide an escaping mechanism for literal dictionary keys containing dots.

Consider the following JSON representation:

```
{
  "service.endpoint": "literal-key",
  "service": {
    "endpoint": "nested-key"
  }
}
```

Both keys are valid and distinct members of the underlying configuration representation.

However, `extract_inner::<String>("service.endpoint")` addresses the nested value `"nested-key"` rather than the literal top-level key `"service.endpoint"`.

The same distinction applies to `find_value()`, `focus()`, `Value::find()`, and `Value::find_ref()`, which interpret a dotted string as a sequence of dictionary-key components.

For literal keys containing dots, extraction of the containing `Dict` or `Value::Dict` followed by direct map access preserves the distinction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.2 Keyed serialized providers share the same limitation

The published `Serialized<T>` provider's keyed constructors, including `Serialized::default()`, `Serialized::global()`, and `.key()`, also interpret their key arguments as dotted paths.

For example, `Serialized::default("a.b", value)` constructs nested dictionaries rather than emitting a literal root key named `"a.b"`.

When arbitrary literal keys must be preserved, an unkeyed serialized dictionary or custom provider can supply the desired dictionary structure directly. This avoids imposing dotted-path interpretation on key names supplied by an external schema.

The relevant distinction is between two different operations: constructing a configuration tree and subsequently addressing a member of that tree. The same dotted syntax is used by several conveniences, but it is not a general-purpose escaped key-path language.

### 5.3 Array elements are not addressed through dotted numeric indices

In published Figment 0.10.19, `Value::find()` and `Value::find_ref()` traverse dictionary keys. They do not interpret numeric path components as array indices.

A path such as `plugins.0.name` therefore does not provide ordinary array-index navigation through `Value::Array`.

The corresponding approach is to retrieve the enclosing array and use `as_array()` or `into_array()` before indexing or iterating over its elements.

This boundary is especially relevant for generic configuration inspectors. Dotted paths address nested dictionaries; arrays require explicit sequence handling.

Primary sources:

[https://docs.rs/figment/0.10.19/figment/value/enum.Value.html](https://docs.rs/figment/0.10.19/figment/value/enum.Value.html?utm_source=chatgpt.com)

[https://docs.rs/figment/0.10.19/figment/providers/struct.Serialized.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Serialized.html?utm_source=chatgpt.com)

## 6. Environment-provider inspection and transformation semantics

### 6.1 `Env::iter()` exposes source strings before structured interpretation

`Env::iter()` yields transformed environment-variable keys paired with `String` values.

It applies the provider's configured key filtering and mapping behavior, but its values remain strings rather than the typed `Value` objects produced through provider evaluation.

For example, an environment variable containing the textual value `[1, 2, 3]` is yielded as a string by `Env::iter()`. The environment provider's data-construction stage then interprets the text according to its supported value syntax.

The distinction matters when inspecting or composing environment-provider adapters. Operating on `Env::iter()` exposes key/value strings; operating on `Provider::data()` exposes profile-indexed dictionaries containing Figment values.

Another difference is lifecycle: repeated calls to `Env::iter()` inspect the environment as it exists at those calls. It is not itself a frozen snapshot. Once an environment provider has been added to a Figment, its data has been collected through Figment's usual eager composition mechanism.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.2 `Uncased` and `UncasedStr` are public building blocks

Figment reexports the `Uncased` and `UncasedStr` types used by its environment-variable transformation API.

`UncasedStr` provides a borrowed, case-insensitive representation, while `Uncased` supports owned or borrowed case-preserving strings.

Their case-insensitive comparison semantics apply to ASCII case differences. They do not provide general Unicode case folding.

This distinction is relevant when implementing filters and mappings because an operation performed on an `UncasedStr` can have different matching semantics from one performed on its ordinary `&str` representation.

The types also provide convenient case-insensitive prefix matching for configuration namespaces without requiring repeated manual lowercase transformations.

Source: [https://docs.rs/figment/0.10.19/figment/value/struct.UncasedStr.html](https://docs.rs/figment/0.10.19/figment/value/struct.UncasedStr.html?utm_source=chatgpt.com)

### 6.3 Diagnostic interpolation is not a reversible environment mapping

The built-in environment provider's metadata identifies environment-variable configuration but does not supply a separate `Source` identifying each original variable.

Its default interpolation converts logical key-path components to uppercase, separated by dots.

This is a readable diagnostic convention, not a guaranteed reconstruction of the original environment-variable name.

For example, a provider might remove a prefix, convert double underscores to dots, and apply further custom renaming. The logical key produced by that transformation may not contain sufficient information to recover the original variable exactly.

A custom provider preserving the original source-key mapping would need to manage that information explicitly when exact reconstruction matters.

### 6.4 Provider thread safety versus composed configuration thread safety

The published `Env` provider implements neither `Send` nor `Sync`, reflecting the type's internal transformation machinery and its trait-object constraints.

The resulting `Figment` implements both `Send` and `Sync`.

This makes source-provider construction and composed-configuration sharing distinct concurrency concerns. A fully constructed Figment can be shared across threads even though an `Env` provider used to construct it is not independently transferable under those same bounds.

This does not imply that reading or modifying process environment variables is synchronized by Figment, or that composition represents an atomic snapshot across independently changing external sources.

Primary source: [https://docs.rs/figment/0.10.19/figment/providers/struct.Env.html](https://docs.rs/figment/0.10.19/figment/providers/struct.Env.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 7. Additional profile construction utilities

Beyond profile creation and selection, the published `Profile` type exposes several methods useful in custom provider implementations and configuration introspection.

| Method                         | Function                                                               |
| ------------------------------ | ---------------------------------------------------------------------- |
| `Profile::collect(dict)`       | Construct a `Map<Profile, Dict>` containing one profile and dictionary |
| `Profile::as_str()`            | Access the profile's case-insensitive string representation            |
| `Profile::starts_with(prefix)` | Perform case-insensitive profile-prefix matching                       |
| `Profile::is_custom()`         | Identify profiles other than `Default` and `Global`                    |

`Profile::collect()` is particularly useful at the provider boundary because it directly creates the return structure required by `Provider::data()` for a single-profile source.

For example:

```
use figment::{Profile, value::{Dict, Map}};

fn single_profile_data(
    profile: Profile,
    values: Dict,
) -> Map<Profile, Dict> {
    profile.collect(values)
}
```

This avoids manually constructing a one-entry profile map.

`Profile::starts_with()` and `Profile::is_custom()` support profile-aware logic without manually normalizing names or comparing string representations.

These are convenience operations rather than an additional profile-inheritance mechanism. Prefix matching does not cause a profile to inherit configuration from another profile with a related name.

Source: [https://docs.rs/figment/0.10.19/figment/struct.Profile.html](https://docs.rs/figment/0.10.19/figment/struct.Profile.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 8. Numeric representation and conversion fidelity

### 8.1 `Num` preserves multiple numeric representations

Figment's internal `Num` enum represents signed and unsigned integers across multiple widths, including `i128` and `u128`, together with `isize`, `usize`, `f32`, and `f64`.

This is more expressive than an intermediate representation that normalizes every integer into `i64` or every numeric value into `f64`.

It can therefore preserve distinctions that matter when constructing or examining configuration programmatically, including signedness and numeric width.

However, the conversion methods are intentionally more restrictive than ordinary mathematical convertibility.

| Method            | Conversion behavior                                |
| ----------------- | -------------------------------------------------- |
| `to_u32()`        | Accepts unsigned variants of width at most 32 bits |
| `to_u128()`       | Accepts unsigned integer variants                  |
| `to_u128_lossy()` | Also accepts nonnegative signed integer variants   |
| `to_i128()`       | Accepts signed integer variants                    |
| `to_f64()`        | Accepts floating-point variants                    |
| `to_actual()`     | Converts to the diagnostic `Actual` representation |

A particularly important detail is that conversion behavior can depend on the stored variant rather than only the numeric value.

For example, `to_u32()` is not a general checked integer cast from every wider integer type. A value represented as a `u64` does not satisfy its documented variant-width restriction simply because its magnitude would fit in `u32`.

Similarly, `to_i128()` is not a universal signed conversion for nonnegative unsigned variants.

### 8.2 Diagnostic conversion has different fidelity requirements

`Num::to_actual()` converts a numeric value into Figment's diagnostic `Actual` representation, which distinguishes signed, unsigned, and floating-point numbers.

The documentation notes that values exceeding the target diagnostic representation's bit width can be truncated.

This is distinct from the underlying `Num` representation and should not be interpreted as a claim that ordinary configuration storage truncates those values.

Consequently, raw numeric inspection and diagnostic formatting can have different fidelity characteristics for extreme integer values.

That distinction is relevant when interpreting errors involving large numeric configuration values or developing tooling that must preserve exact numerical representations.

Primary source: [https://docs.rs/figment/0.10.19/figment/value/enum.Num.html](https://docs.rs/figment/0.10.19/figment/value/enum.Num.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 9. Additional `Jail` testing semantics

### 9.1 Complete environment clearing and restoration

`Jail::clear_env()` removes all environment variables in the temporary test context. It does not merely remove variables previously introduced with `Jail::set_env()`.

The original environment is restored when the jail exits through normal cleanup.

This allows configuration tests to distinguish behavior under an explicitly empty environment from behavior under an environment inheriting unrelated variables from the test runner or host process.

It is particularly useful for verifying implicit defaults, missing-variable behavior, and provider filtering without accidental interference from preexisting process variables.

The distinction remains process-level rather than thread-local. `Jail` coordinates its own invocations, but unrelated code that accesses or mutates process-global environment state is not automatically isolated.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.2 Controlled filesystem operations and path validation

The `Jail` file and directory helpers validate supplied paths against the temporary jail directory.

The relevant methods include `create_file()`, `create_binary()`, `create_dir()`, and `change_dir()`.

File and directory creation operations reject paths outside the allowed temporary directory, while `create_dir()` can recursively construct missing parent directories.

These checks help keep test fixtures within the expected temporary namespace and avoid accidental modifications to external paths.

They do not constitute an operating-system security boundary. The rest of the process can still use ordinary filesystem APIs, and the jail does not isolate the process from its host filesystem or unrelated threads.

Source: [https://docs.rs/figment/0.10.19/figment/struct.Jail.html](https://docs.rs/figment/0.10.19/figment/struct.Jail.html?utm_source=chatgpt.com)

## 10. Published-release versus development-source distinctions

The distinction between the published crate and newer upstream source is unusually important for Figment because the upstream manifest continues to identify the package as version `0.10.19` despite containing changes not present in the published package.

As of the research date, 0.10.19 remains the latest published stable release.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 10.1 File-provider API divergence

The published `Data<F>` API supports `file()`, `file_exact()`, `string()`, `profile()`, and `nested()`.

Newer upstream material introduces additional controls, including `required()` and `search()`, which alter the file-discovery and missing-file model.

These methods are not available in the published 0.10.19 public API and should not be presented as capabilities of that release merely because documentation indexed from the development branch contains them.

### 10.2 Feature and dependency divergence

There are also material differences in the manifest:

| Area                      | Published 0.10.19                      | Upstream development                           |
| ------------------------- | -------------------------------------- | ---------------------------------------------- |
| Default features          | None                                   | `env`                                          |
| TOML dependency           | `toml` 0.8                             | `toml_edit` 0.22                               |
| Environment-value parsing | `env` enables `parse-value` and `pear` | Revised implementation and feature arrangement |
| `Data` file controls      | `file()` / `file_exact()`              | Additional search/required controls            |

In the published crate, enabling `env` automatically enables its `parse-value` dependency feature. There is ordinarily no reason to specify `parse-value` separately when the `env` provider is already enabled.

These are release-specific facts rather than general guarantees across future Figment releases. The development branch is useful for understanding implementation direction, but its capabilities must be verified independently before being represented as released.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Published features: [https://docs.rs/crate/figment/0.10.19/features](https://docs.rs/crate/figment/0.10.19/features?utm_source=chatgpt.com)

Published manifest: [https://docs.rs/crate/figment/0.10.19/source/Cargo.toml](https://docs.rs/crate/figment/0.10.19/source/Cargo.toml?utm_source=chatgpt.com)

Upstream development manifest: [https://github.com/SergioBenitez/Figment/blob/master/Cargo.toml](https://github.com/SergioBenitez/Figment/blob/master/Cargo.toml?utm_source=chatgpt.com)

## 11. Cross-cutting architectural implications

The additional behaviors establish three consequential distinctions for systems using Figment as a shared configuration abstraction.

Configuration acquisition is not a single atomic snapshot operation. File discovery, file reading, environment inspection, provider composition, and typed extraction can occur at different points. Figment provides a reusable composed configuration once inputs are collected, but does not guarantee consistency across external sources that change during construction. The file-provider distinctions are particularly important because discovery can precede acquisition.

Provenance is attached to values and their contributing metadata, not to an exhaustive history of configuration decisions. Retained values can identify their source, and nested values can retain different source tags. However, parent containers can have different provenance from their children, custom profile names are not recoverable from tags alone, and overwritten values are not exposed through a general historical audit interface. Accurate provenance inspection therefore depends on the level of the value tree being examined.

Figment's generic representation is more expressive than its convenience key-path interface. Dictionaries can contain literal keys that cannot be addressed unambiguously through dotted paths, while arrays require explicit sequence operations. The internal numeric representation also preserves distinctions that some conversion helpers do not accept. Direct `Value` and `Dict` inspection remains an important complement to the higher-level extraction APIs.

Together, these distinctions clarify how Figment can support sophisticated configuration composition and diagnostics without requiring a bespoke intermediate representation, while identifying the points at which exact source identity, structural addressing, or acquisition consistency require additional handling outside the core API.