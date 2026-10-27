# Rust Error Handling and Diagnostics: thiserror 2.0.21 + miette 7.6.0

Technical capability reference | Research date: October 26, 2026

Verified stable versions:

- thiserror 2.0.21 — typed error definitions and automatic implementations of Rust's standard error traits.
- miette 7.6.0 — structured diagnostics, rich error reporting, source annotations, and customizable presentation.

Both versions are verified against their published rustdoc documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 1. Architectural orientation

The two libraries are complementary rather than competing solutions.

thiserror addresses how errors are represented and propagated within Rust code. It generates implementations of `std::error::Error`, `Display`, and appropriate `From` conversions for strongly typed error enums and structs.

miette addresses how errors are enriched, aggregated, interpreted, and presented. It extends standard errors through the `Diagnostic` trait, adds source-location information and remediation guidance, and supplies reporting machinery for applications, command-line tools, and other diagnostic consumers.

Their combined architecture can be understood as:

Domain operations and external libraries

I/O, parsing, validation, state transitions, execution

thiserror — Error semantics

Typed variants · context fields · source chains · conversions

miette — Diagnostic semantics

Codes · severity · help · source spans · related errors

Reporting and consuming errors

Structured Rust APIs · CLI reports · accessible text · JSON

The important separation is between the semantic identity of an error, its diagnostic metadata, and its presentation. These are distinct responsibilities that can evolve independently.

| Concern                                        | thiserror       | miette                                    |
| ---------------------------------------------- | --------------- | ----------------------------------------- |
| Typed error enums and structs                  | Primary purpose | Uses existing error types                 |
| Standard `Error` and `Display` implementations | Derive macro    | Requires compatible errors                |
| Automatic `From` conversions                   | `#[from]`       | General errors via `IntoDiagnostic`       |
| Error cause chains                             | `#[source]`     | Reads and renders chains                  |
| Stable diagnostic codes                        | Not provided    | `Diagnostic::code`                        |
| Severity and remediation guidance              | Not provided    | `severity`, `help`, `url`                 |
| Source snippets and annotations                | Not provided    | `SourceCode`, `SourceSpan`, `LabeledSpan` |
| Multiple related diagnostics                   | Manual modeling | `Diagnostic::related`                     |
| Terminal and accessible rendering              | Not provided    | Configurable report handlers              |
| Application-level type-erased errors           | Not provided    | `Report` and `Result`                     |
| Public library error contracts                 | Strong fit      | Optional enrichment                       |
| Machine-readable diagnostic rendering          | Not provided    | JSON report handler                       |

A significant design possibility follows: libraries can maintain precise, typed errors without embedding terminal-specific reporting logic. Diagnostic metadata and rendering can then be added where they are valuable, with miette's interfaces providing the connection.

Primary documentation:

`https://docs.rs/thiserror/2.0.21/thiserror/`

`https://docs.rs/miette/7.6.0/miette/`

## 2. thiserror — Typed error definitions and propagation

### 2.1 Core derive mechanism

The central capability is `#[derive(Error)]`, a procedural macro that generates implementations of standard Rust error interfaces while leaving the underlying error representation under the author's control.

It supports all conventional error shapes: enums, named-field structs, tuple structs, and unit structs. The resulting types remain ordinary Rust types and can be matched, inspected, propagated, and exposed through public interfaces without a runtime dependency on a specialized error wrapper.

The primary annotation vocabulary is compact:

| Annotation              | Generated behavior                            | Important distinction                                                           |
| ----------------------- | --------------------------------------------- | ------------------------------------------------------------------------------- |
| `#[error("...")]`       | Implements `Display`                          | Supports Rust formatting and field interpolation                                |
| `#[from]`               | Implements `From<SourceError>`                | Also marks the field as the error source                                        |
| `#[source]`             | Implements `Error::source()`                  | Does not generate a conversion                                                  |
| Field named `source`    | Automatically recognized as an error source   | Explicit `#[source]` unnecessary                                                |
| `#[error(transparent)]` | Delegates `Display` and `source()`            | Does not introduce a new displayed context layer                                |
| `#[backtrace]`          | Forwards backtrace provision through a source | Requires nightly compiler support                                               |
| Field typed `Backtrace` | Supports backtrace provision                  | Also depends on nightly compiler functionality in the documented implementation |

The formatting and propagation behavior is documented in the crate's released API reference.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.2 Error enums as explicit failure contracts

Error enums allow a fallible operation to expose a closed set of semantically distinct failure categories.

For example:

```

use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("unexpected token {token:?} at byte {offset}")]
    Syntax {
        token: String,
        offset: usize,
    },

    #[error("unsupported schema version {0}")]
    Version(u32),
}

```

This is an illustrative error model; it does not require a particular parser or storage library.

Unlike converting all errors immediately to strings, this representation preserves the ability to:

- Distinguish I/O, syntax, and version failures by variant.
- Retain structured context such as paths, offsets, and schema versions.
- Traverse underlying error sources where available.
- Add diagnostic annotations without changing the basic error contract.

The important architectural property is preservation of typed failure information. Error messages are a presentation of that information, not its replacement.

### 2.3 Automatic conversion versus contextual wrapping

`#[from]` generates conversions suitable for the `?` operator:

```

#[derive(Debug, Error)]
enum OperationError {
    #[error("I/O failure: {0}")]
    Io(#[from] std::io::Error),
}

```

When a lower-level operation returns `std::io::Error`, the compiler can convert it to `OperationError::Io` through the generated `From` implementation.

However, a variant containing `#[from]` cannot carry arbitrary additional fields. Beyond its source error, only a supported backtrace field is permitted.

That limitation is consequential. An error requiring operation-specific context must generally be constructed explicitly:

```

let content = std::fs::read_to_string(&path)
    .map_err(|source| LoadError::Read {
        path: path.clone(),
        source,
    })?;

```

This establishes two different mechanisms:

Automatic propagation: use `#[from]` when the conversion is fully determined by the source error type.

Contextual propagation: construct the destination error when context must be supplied from the current operation.

The latter is not a deficiency in the macro. It preserves the distinction between lossless mechanical conversions and domain-contextual transformations, which require additional information.

### 2.4 Error sources and causal structure

Rust's `Error::source()` represents a causal relationship between errors, not a general collection of fields or diagnostic observations.

With thiserror, a source is identified through `#[source]`, `#[from]`, or the conventional field name `source`.

This is particularly useful when retaining low-level causes while introducing higher-level errors:

```
Failed to load configuration
    caused by: Failed to read configuration file
        caused by: Permission denied
```

This chain is different from a collection of independent validation failures. A cause explains an error's origin, whereas several validation diagnostics may be siblings that all contribute to an unsuccessful operation.

That distinction becomes especially important when combining thiserror with miette's `related` diagnostics.

### 2.5 Transparent delegation and stable public interfaces

`#[error(transparent)]` delegates the error message and source behavior to one underlying error value rather than adding another message layer.

This has two particularly valuable uses.

Transparent catch-all variants: an enum can include an underlying error without replacing its display message with another wrapper message.

Opaque public errors: an API can expose a public error type while keeping the complete representation private.

```

use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ApiError(#[from] ErrorRepr);

#[derive(Debug, Error)]
enum ErrorRepr {
    #[error("invalid input")]
    InvalidInput,

    #[error("resource unavailable")]
    Unavailable,
}

```

The public `ApiError` type can remain stable while the private `ErrorRepr` changes. Public accessor methods can selectively expose stable information without exposing the private variants themselves.

The distinction is not merely aesthetic: exposing every error variant makes those variants and their structure part of the downstream compilation contract.

A transparent wrapper, however, does not automatically hide all observable behavior. Its displayed text, exposed source chain, and any public accessors may still be relevant to consumers. It is a tool for controlling the public type contract, not a universal compatibility guarantee.

Source: `https://docs.rs/thiserror/2.0.21/thiserror/`

### 2.6 Formatting and customization

The `#[error]` attribute supports interpolation of named and positional fields, standard format specifiers such as `:?`, and additional formatting expressions.

This allows error messages to be derived directly from their structured fields without storing an additional preformatted message.

Field formatting can reference nested values and formatting helpers. Custom `Display` implementations remain an option when macro-generated formatting is insufficient.

A useful distinction is that thiserror generates formatting implementations; it is not itself a diagnostic renderer. It does not natively format annotated source ranges, select terminal themes, or attach structured help instructions. Those belong to a richer diagnostic layer such as miette.

### 2.7 Backtrace support and compiler constraints

thiserror includes integration with Rust's generic error-member-access mechanism.

When enabled, its derive implementation can provide a `std::backtrace::Backtrace` field through `Error::provide()`, capture a backtrace during certain `From` conversions, and forward backtrace requests through a field marked `#[backtrace]`.

The released documentation explicitly identifies this functionality as nightly-dependent. It must not be assumed available merely because `std::backtrace::Backtrace` itself can be used on stable Rust.

This is separate from application frameworks that capture backtraces within their own report structures.

Source: `https://docs.rs/thiserror/2.0.21/thiserror/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 2.8 Compilation, dependencies, and runtime implications

thiserror 2.0.21 declares a minimum supported Rust version of 1.77. Its default `std` feature supplies convenience formatting for types such as `Path` and `PathBuf`; the feature can be disabled for `no_std` use, subject to the target toolchain's support for the necessary core error interfaces.

The crate itself uses `#![no_std]`, with derive processing supplied by its companion `thiserror-impl` procedural-macro crate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



Its performance characteristics follow from what it generates:

- Typed errors preserve ordinary Rust enum/struct layout and dispatch behavior.
- Formatting work occurs through generated `Display` implementations.
- Automatic error conversion does not require an intermediate `String` representation.
- The generated error types do not require a dynamic reporting framework.

These mechanisms favor direct, low-overhead error representation. Actual cost still depends on the fields, formatting expressions, allocations, and conversions used by an individual error type.

Boundary: thiserror is not an error aggregator, logging framework, tracing system, backtrace collection framework, or diagnostic presentation engine.

Source: `https://docs.rs/crate/thiserror/2.0.21/source/Cargo.toml.orig`

## 3. miette — Structured diagnostics and reporting

### 3.1 The `Diagnostic` protocol

miette's central abstraction is the `Diagnostic` trait, which extends `std::error::Error` with optional, machine-accessible diagnostic metadata.

Its methods describe a diagnostic independently of how that diagnostic will eventually be rendered.

| `Diagnostic` method   | Information provided            | Architectural significance                                                   |
| --------------------- | ------------------------------- | ---------------------------------------------------------------------------- |
| `code()`              | Diagnostic identifier           | Stable categorization, documentation lookup, downstream interpretation       |
| `severity()`          | `Error`, `Warning`, or `Advice` | Distinguishes diagnostic significance from the fact that an operation failed |
| `help()`              | Suggested remediation           | Keeps actionable guidance alongside the originating failure                  |
| `url()`               | Additional documentation        | Connects error codes to external explanations                                |
| `source_code()`       | `&dyn SourceCode`               | Makes source content available through an abstraction                        |
| `labels()`            | Iterator of `LabeledSpan`       | Provides annotated source locations                                          |
| `related()`           | Iterator of other diagnostics   | Represents multiple associated problems                                      |
| `diagnostic_source()` | Another `&dyn Diagnostic`       | Preserves diagnostic-aware causal relationships                              |

All eight methods have default implementations. An ordinary `std::error::Error` can therefore acquire `Diagnostic` support incrementally, without implementing every capability.

The trait is also usable through dynamic dispatch: `&dyn Diagnostic`. This makes it a useful boundary between diagnostic producers and consumers such as terminal renderers, test utilities, editor integrations, and structured-report exporters.

A key detail is that diagnostic codes, help text, and URLs are returned through displayable values, not an enforced central registry. miette supplies the metadata protocol; defining stable code identities and their meaning remains the responsibility of the producer.

Source: `https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.2 Deriving diagnostics from typed errors

miette's `#[derive(Diagnostic)]` is designed to compose with an existing error implementation, commonly one produced by thiserror.

```

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("invalid configuration value")]
#[diagnostic(
    code(config::invalid_value),
    severity(Error),
    help("replace the value with a supported option"),
    url("https://example.org/diagnostics/invalid-value")
)]
pub struct InvalidValue {
    #[source_code]
    pub source: NamedSource<String>,

    #[label(primary, "unsupported value")]
    pub span: SourceSpan,
}

```

The two derives generate different implementations for the same type:

- `thiserror::Error` supplies the standard error semantics and message.
- `miette::Diagnostic` supplies structured metadata, source references, and annotations.

The example's URL is illustrative, not an actual project resource.

The distinction is useful because producers can retain a strongly typed Rust API while also making selected errors understandable to diagnostic-aware tooling.

The derive supports enums as well as structs. Metadata may be specified per variant, and source spans, help text, related diagnostics, and diagnostic sources can be supplied through field attributes.

Important derive annotations include `#[diagnostic(...)]`, `#[source_code]`, `#[label(...)]`, `#[related]`, `#[help]`, and `#[diagnostic_source]`.

Dynamic metadata: help text can be derived from fields rather than a fixed message. This is valuable when guidance depends on the expected value, operation, or available alternatives.

Documentation integration: `url(docsrs)` can generate a documentation URL for the diagnostic type, avoiding manually maintained links to its Rust API reference. Dedicated explanations remain preferable when the relevant guidance is more extensive than API documentation.

Sources:

`https://docs.rs/miette/7.6.0/miette/derive.Diagnostic.html`

`https://docs.rs/miette/7.6.0/miette/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 3.3 Source locations as a reusable diagnostic model

miette's source-annotation mechanism is more general than printing snippets of Rust code.

Its principal abstractions are:

`SourceSpan` — a range represented by an absolute byte offset and byte length, relative to some `SourceCode`.

`SourceCode` — an interface through which a reporter requests the source content covering a range, with optional surrounding context.

`SpanContents` — the returned content and its positional metadata, including line and column information and optional source name and language.

`NamedSource<S>` — an adapter that associates a source name, and optionally a language hint, with an underlying source implementation.

`LabeledSpan` — an annotation that combines a span with optional explanatory text and primary-label semantics.

The important architectural distinction is that source locations are not restricted to Rust source files. A `SourceCode` may represent a configuration document, query, expression, generated program, script, serialized input, or another byte-addressable source.

Because `SourceSpan` identifies an offset within a `SourceCode`, the source object supplies the missing context: source name, contents, and conversion to human-readable line and column positions.

#### Byte offsets versus displayed locations

`SourceSpan` uses byte positions, not character indices or one-based line/column numbers. It accepts common Rust representations such as `(offset, length)`, ranges, and `SourceOffset` values.

Zero-length spans are supported and can identify insertion points or missing tokens.

This distinction matters for Unicode text: UTF-8 byte offsets are not necessarily identical to character counts or terminal display columns. A parser's offsets must correspond to the same source bytes eventually supplied to miette.

`SourceOffset::from_location` provides a conversion from source-relative line and column coordinates, but its documented behavior can clamp out-of-range positions rather than reject them. It is not a general-purpose validator of externally supplied locations.

Sources:

`https://docs.rs/miette/7.6.0/miette/struct.SourceSpan.html`

`https://docs.rs/miette/7.6.0/miette/struct.SourceOffset.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

#### Custom source access

A `SourceCode` implementation provides `read_span(...)`, receiving the requested span and the number of surrounding context lines. It returns a `SpanContents` object or a `MietteError`.

This separation makes it possible to provide source text through a custom abstraction rather than eagerly copying every complete source into each diagnostic.

For example, a source implementation could resolve a span against a shared text buffer, retained document snapshot, source map, or another source repository. These are architectural possibilities enabled by the trait, not built-in storage systems supplied by miette.

The producer must still ensure that the stored source identity and byte offsets refer to the correct source version. A label that points into a modified document may be technically well-formed but semantically wrong.

Sources:

`https://docs.rs/miette/7.6.0/miette/trait.SourceCode.html`

`https://docs.rs/miette/7.6.0/miette/trait.SpanContents.html`

`https://docs.rs/miette/7.6.0/miette/struct.NamedSource.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 3.4 Multiple annotations, primary locations, and dynamic label collections

miette supports multiple annotations on one diagnostic, including optional labels and collections whose size is determined at runtime.

A single error can therefore identify a primary problem and several contributing source locations. This is particularly useful for problems involving a relationship between two or more source expressions—for example, an incompatible declaration and its use.

`#[label(primary, "...")]` designates the main location. Only one primary label is permitted per derived diagnostic.

`#[label(collection, "...")]` accepts a collection of spans, including `Vec<Range<usize>>`, or `Vec<LabeledSpan>` when individual annotations need different messages.

The lower-level `LabeledSpan` API also permits programmatic construction through methods such as `at`, `at_offset`, `underline`, and `new_primary_with_span`.

Importantly, multiple labels do not require multiple independent errors. They describe multiple locations relevant to one diagnostic. Multiple independent findings are better represented through `related()`.

Sources:

`https://docs.rs/miette/7.6.0/miette/struct.LabeledSpan.html`

`https://docs.rs/crate/miette/7.6.0/source/README.md`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 3.5 Related diagnostics: representing multiple independent findings

One of miette's more architecturally consequential capabilities is `Diagnostic::related()`, exposed through `#[related]`.

A causal error chain describes how one failure arose from another. Related diagnostics instead describe multiple findings associated with a larger operation.

For example, a configuration validator may discover several independent problems and return them together without discarding all but the first error.

```

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("invalid configuration entry")]
#[diagnostic(code(config::invalid_entry))]
struct Finding {
    #[label("invalid entry")]
    span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("configuration contains {count} invalid entries")]
#[diagnostic(code(config::validation_failed))]
struct ValidationErrors {
    count: usize,

    #[source_code]
    input: NamedSource<String>,

    #[related]
    findings: Vec<Finding>,
}

```

Notice that individual `Finding` values contain spans but do not own separate copies of the source document.

miette supports source code supplied by an enclosing diagnostic being used to render related diagnostics. This allows a collection of findings to share a single source representation, reducing duplicated source ownership and simplifying the error model.

The resulting structure has a clear interpretation: one operation-level failure containing multiple independently meaningful diagnostic records.

This is useful for batch validation, compilers, schema checking, configuration analysis, and other operations where collecting errors can be more valuable than failing immediately.

miette 7.5 introduced configurable nested rendering for related diagnostics, which is available in 7.6.0. The presentation can therefore reflect whether findings should appear as siblings or as a nested group.

Sources:

`https://docs.rs/crate/miette/7.6.0/source/CHANGELOG.md`

`https://docs.rs/miette/7.6.0/miette/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1

### 3.6 Diagnostic-preserving delegation and wrapping

A subtle but important integration issue arises when a type that already implements `Diagnostic` is wrapped inside another error type.

Rust's standard `Error::source()` exposes the wrapped value as `&dyn Error`. This preserves its causal identity but does not expose the additional `Diagnostic` methods through that reference.

miette offers distinct solutions depending on what should be preserved.

`#[diagnostic_source]` — preserve diagnostic-aware causality. Applied alongside thiserror's `#[source]`, this exposes the underlying value through `Diagnostic::diagnostic_source()` in addition to standard `Error::source()`.

`#[diagnostic(transparent)]` — delegate diagnostic metadata completely. This is the diagnostic analogue of thiserror's transparent wrapper.

`#[diagnostic(forward(field))]` — forward metadata from a selected field. This works with wrappers that contain other fields, including wrappers where the outer type has its own error message. It can also be combined with selected diagnostic metadata overrides.

Consider the difference between these two annotations:

```

#[error(transparent)]
#[diagnostic(transparent)]
Inner(#[from] InnerDiagnostic)

```

The first attribute forwards standard error presentation and source semantics. The second forwards the diagnostic information, including annotations and guidance.

Using only `#[error(transparent)]` does not guarantee that miette's labels or other diagnostic metadata will be forwarded.

The `forward(field)` mechanism is an especially valuable alternative when a wrapper needs to retain its own fields or message while exposing diagnostic information from a particular contained value.

There is also a broader semantic distinction: forwarding another diagnostic's metadata is not always equivalent to introducing a new diagnostic identity. If the outer error represents a materially different failure category, its own code, severity, and guidance may be more appropriate.

Source: `https://github.com/zkat/miette`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2

### 3.7 `Report`, `Result`, and application-level error handling

miette includes an application-oriented, type-erased error wrapper:

`miette::Report`

Its `Result<T>` alias is equivalent to `Result<T, miette::Report>`.

This provides a different interface from a public enum-based error contract.

A typed error enum makes variants statically visible and matchable. A `Report` can contain different diagnostic implementations through one common type, providing convenient propagation and rich reporting without requiring a dedicated enum for every application operation.

Important `Report` capabilities include:

| Operation                                        | Capability                                         |
| ------------------------------------------------ | -------------------------------------------------- |
| `Report::new`                                    | Wraps an existing `Diagnostic`                     |
| `Report::from_err`                               | Wraps an ordinary standard error                   |
| `Report::msg`                                    | Creates a report from a message                    |
| `Report::new_boxed`                              | Accepts a boxed trait-object diagnostic            |
| `with_source_code`                               | Attaches source information later                  |
| `wrap_err` / `context`                           | Adds contextual error information                  |
| `chain` / `root_cause`                           | Traverses standard causal information              |
| `is`, `downcast_ref`, `downcast_mut`, `downcast` | Recovers concrete error information                |
| `handler` / `handler_mut`                        | Accesses the report's associated rendering handler |

`Report::new` requires an owned `Diagnostic + Send + Sync + 'static`. These requirements matter for error values involving borrowed data, thread-unsafe state, or source buffers whose lifetimes do not extend beyond an individual operation.

miette documents backtrace capture within its report machinery when the underlying error does not already provide one. This is separate from thiserror's nightly-dependent `Error::provide()` integration.

Source: `https://docs.rs/miette/7.6.0/miette/struct.Report.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



#### `IntoDiagnostic` and a consequential information-loss boundary

`IntoDiagnostic` supplies `.into_diagnostic()` for a `Result<T, E>` where `E` implements `std::error::Error + Send + Sync + 'static`.

It allows ordinary library errors to enter a miette reporting pipeline:

```

use miette::{IntoDiagnostic, WrapErr};

fn read_settings(path: &str) -> miette::Result<String> {
    std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("reading settings from {path}"))
}

```

However, the official documentation explicitly warns that applying `.into_diagnostic()` to an error that already implements `Diagnostic` loses its richer diagnostic information.

The conversion treats the value through the common `std::error::Error` interface. Its additional code, labels, help, and related-diagnostic information may no longer be accessible.

A diagnostic-aware error should generally be propagated directly with `?` or converted through an appropriate `Into<Report>` path rather than first passing through `.into_diagnostic()`.

This distinction is essential when composing independently developed libraries: merely displaying the correct error message does not establish that the original diagnostic metadata survived the conversion.

Source: `https://docs.rs/miette/7.6.0/miette/trait.IntoDiagnostic.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



#### Context preservation and downcasting

The `WrapErr` trait provides `wrap_err`, `wrap_err_with`, `context`, and `with_context`.

Its implementations cover `Result` and `Option`. The closure-based variants construct contextual messages when needed, rather than requiring eager message construction.

A notable documented guarantee is that wrapping a report with context does not prevent downcasting to an existing underlying error type. miette also supports downcasting to the type of an attached contextual message where applicable.

That is stronger than simply preserving a textual cause chain: typed information can remain discoverable even after contextual messages have been attached.

There are limits around errors that do not satisfy the required trait bounds; the documentation specifically discusses conversions involving `Box<dyn Error>` and errors represented by types that do not implement the standard `Error` trait.

Source: `https://docs.rs/miette/7.6.0/miette/trait.WrapErr.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.8 Dynamic diagnostics and macro-based construction

Not every diagnostic needs a dedicated Rust type.

miette supplies `MietteDiagnostic` and macros such as `miette!`, `diagnostic!`, `bail!`, and `ensure!` for creating error reports and diagnostic values without first defining a custom enum or struct.

The distinction between two principal macros is:

- `diagnostic!` constructs a `MietteDiagnostic`, with options for severity, code, help, URL, and labels.
- `miette!` constructs an application-level `Report`, optionally containing diagnostic metadata.

`MietteDiagnostic` also provides builder methods for adding labels and other metadata programmatically.

This is valuable for runtime-defined findings, dynamically selected error categories, and transformation pipelines where an error's complete structure is not conveniently represented by a small set of static Rust variants.

However, a dynamically constructed report is not automatically equivalent to a typed error contract. Downstream code may need stable diagnostic codes or explicit structured fields to make reliable decisions.

The optional `serde` dependency provides serialization support for selected miette diagnostic data types. It does not automatically make arbitrary user-defined diagnostics serializable.

Source: `https://docs.rs/miette/7.6.0/miette/macro.diagnostic.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 4. miette's rendering architecture

### 4.1 Diagnostic production is independent of report presentation

miette separates the `Diagnostic` protocol from the `ReportHandler` responsible for formatting it.

This is important because a diagnostic can be consumed by several different presentation mechanisms without its producer having to encode terminal layout, ANSI colors, or JSON serialization into the error type.

The main available handlers are:

| Handler                   | Purpose                             | Important characteristics                                            |
| ------------------------- | ----------------------------------- | -------------------------------------------------------------------- |
| `GraphicalReportHandler`  | Rich terminal diagnostics           | Annotations, colored output, cause chains, themes, links             |
| `NarratableReportHandler` | Accessible plain-text presentation  | Suitable for screen readers, braille users, and non-graphical output |
| `JSONReportHandler`       | Machine-readable rendering          | Emits structured JSON derived from the diagnostic protocol           |
| `DebugReportHandler`      | Lightweight default/debug rendering | Avoids requiring the full graphical reporting stack                  |
| Custom `ReportHandler`    | Application-defined reporting       | Can implement an entirely different presentation protocol            |

All of these consume diagnostic information; they do not replace the typed error model.

### 4.2 Graphical rendering and layout controls

`GraphicalReportHandler` can present diagnostic codes, severity, cause chains, source excerpts, multiline annotations, help text, and documentation links.

Its substantial customization surface includes:

Layout: terminal width, wrapping behavior, word breaking, tab width, context lines surrounding labels, and formatting of multiline primary spans.

Appearance: graphical themes, Unicode versus ASCII characters, terminal color controls, and different color capabilities.

Links: inclusion of diagnostic URLs and use of clickable terminal hyperlinks where supported.

Relationships: inclusion or omission of cause chains and nested versus sibling rendering of related diagnostics.

Output policy: custom footers, URL visibility, and syntax highlighting.

miette 7.6.0 includes `without_primary_span_start()` to disable display of the starting line/column for a multiline primary span. It is a comparatively narrow feature, but illustrates that presentation details are controlled by the handler rather than the diagnostic type.

Sources:

`https://docs.rs/miette/7.6.0/miette/struct.MietteHandlerOpts.html`

`https://docs.rs/miette/latest/miette/struct.GraphicalReportHandler.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 4.3 Accessibility and non-interactive environments

`NarratableReportHandler` emits a textual report without relying on graphical alignment, color, or decorative Unicode layout to communicate diagnostic meaning.

It is intended for accessibility tools and environments where terminal graphics are unsuitable.

Its output can include cause chains and source excerpts, and its settings support configurable context lines and footers.

The default handler machinery can select presentation based on environment capabilities, while `MietteHandlerOpts` allows graphical or narrated rendering to be forced explicitly.

An important nuance is that disabling color and disabling graphical layout are separate decisions. `NO_COLOR` is not a universal guarantee that a narratable handler will be selected; miette also supports `NO_GRAPHICS` and explicit reporter configuration.

Source: `https://docs.rs/miette/latest/miette/struct.NarratableReportHandler.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.4 JSON reporting and structured integration

`JSONReportHandler` is a particularly important but easily overlooked capability.

It exposes diagnostic information as machine-readable JSON rather than expecting consumers to parse terminal output.

Inspection of the released handler implementation shows output fields covering:

- Diagnostic message, code, and severity.
- Cause messages.
- Optional help and documentation URL.
- Source-related information where available.
- Labels and their byte offsets and lengths.
- Nested related diagnostics.

This enables integrations that consume diagnostics programmatically, for example editor tooling, automated quality checks, structured test outputs, and downstream reporting interfaces.

However, three distinctions matter.

First, JSON reporting is not serialization of the original Rust error value. It renders the information exposed through the diagnostic interfaces. Arbitrary fields that have not been exposed through those interfaces are not automatically included.

Second, the JSON handler is distinct from miette's optional `serde` support. The former produces a diagnostic report; the latter supports serialization of selected Rust data structures.

Third, the presence of JSON output does not by itself establish a long-term versioned interchange protocol. For persisted reports or distributed interfaces, consumers may need their own compatibility rules, schema versioning, and conventions for source identity.

The handler can be invoked directly:

```

use miette::{Diagnostic, JSONReportHandler};

fn diagnostic_json(
    diagnostic: &dyn Diagnostic,
) -> Result<String, std::fmt::Error> {
    let mut output = String::new();
    JSONReportHandler::new()
        .render_report(&mut output, diagnostic)?;
    Ok(output)
}

```

Direct rendering is particularly useful for tests or programmatic consumers because it avoids depending on application-global error-reporting configuration.

Sources:

`https://docs.rs/miette/7.6.0/miette/struct.JSONReportHandler.html`

`https://docs.rs/miette/latest/src/miette/handlers/json.rs.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 4.5 Global hooks and custom handlers

The `ReportHandler` trait allows alternative rendering implementations.

`miette::set_hook()` installs a process-global factory that determines the handler associated with reports. The hook is installed once, and installation can fail if one has already been registered.

For standard customization, `MietteHandlerOpts` is generally sufficient. It avoids implementing the entire formatting protocol merely to modify themes, context lines, line wrapping, hyperlinks, or narration behavior.

The global nature of the hook has consequences for libraries and tests: independent components cannot safely assume they each control global report formatting.

When deterministic output is necessary, individual handler instances can render diagnostics directly to a `fmt::Write` target. This avoids mutable process-global configuration and allows a single diagnostic to be rendered differently for different consumers.

miette also exposes `set_panic_hook()` for configuring panic reporting. Panic handling is distinct from ordinary `Result` propagation and should not be understood as transforming panics into recoverable typed errors.

Sources:

`https://docs.rs/miette/latest/miette/trait.ReportHandler.html`

`https://docs.rs/miette/latest/miette/fn.set_hook.html`

`https://docs.rs/miette/latest/miette/fn.set_panic_hook.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 4.6 Syntax highlighting and source-aware presentation

With the optional `syntect-highlighter` feature, miette can highlight source excerpts using the `syntect` library.

Language selection follows source metadata: an explicit language hint supplied through `SpanContents::language()` takes precedence, followed by filename-based detection through `SpanContents::name()` where available.

The `NamedSource::with_language()` method offers a convenient way to supply this information.

miette also exposes the `Highlighter` extension interface, making it possible to provide an alternative highlighter rather than use its default syntect integration.

This separation permits different source domains and language definitions without changing how errors and spans are modeled.

Syntax highlighting is an optional presentation cost and should not be confused with parsing, semantic validation, or the derivation of source spans; those remain responsibilities of other components.

Source: `https://github.com/zkat/miette`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 5. Feature flags, dependency boundaries, and platform implications

miette 7.6.0 has a deliberately layered feature system. Its default configuration is not the same as enabling its full graphical reporting capabilities.

| Feature                  | Capability and dependency implications                                       |
| ------------------------ | ---------------------------------------------------------------------------- |
| `derive`                 | Enabled by default; provides `#[derive(Diagnostic)]` through `miette-derive` |
| `fancy-base`             | Foundation for graphical rendering, including styling and text wrapping      |
| `fancy-no-syscall`       | Graphical foundation without the terminal-detection-related dependencies     |
| `fancy-no-backtrace`     | Adds terminal capability detection to the graphical foundation               |
| `fancy`                  | Adds the full graphical stack, including backtrace-related dependencies      |
| `syntect-highlighter`    | Adds syntect-based highlighting and its associated rendering dependencies    |
| `serde`                  | Enables optional serialization-related functionality for selected types      |
| `no-format-args-capture` | Compatibility-oriented formatting feature                                    |

miette's declared minimum Rust version is 1.70.0. The `derive` feature can be disabled if diagnostic implementations are provided manually, reducing reliance on procedural-macro dependencies.

The crate does not declare a general `no_std` mode comparable to thiserror's `no_std` configuration. Combining them therefore does not make miette-based reporting available in every environment where a thiserror-derived type can be compiled.

The manifest distinguishes `fancy-no-syscall` from the fuller terminal-detection modes. This can matter for constrained targets and environments where operating-system terminal detection is unavailable or undesirable.

The full `fancy` feature is primarily a presentation concern, so a library that exposes diagnostic error types does not intrinsically need to activate the complete graphical stack. A consuming application can determine which reporting capabilities to enable.

Source: `https://docs.rs/crate/miette/7.6.0/source/Cargo.toml.orig`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



## 6. Architectural composition: important choices and failure modes

The combination of thiserror and miette is most valuable when the distinction between typed error identity, diagnostic enrichment, and final presentation is preserved.

| Design question                                           | Relevant capability                                          | Consequence                                                       |
| --------------------------------------------------------- | ------------------------------------------------------------ | ----------------------------------------------------------------- |
| Should downstream Rust code match exact failures?         | thiserror enums and structs                                  | Preserves static error identity                                   |
| Does an error need local operation context?               | Explicit variant construction or `WrapErr`                   | Different tradeoffs between typed fields and ad-hoc messages      |
| Does a wrapped diagnostic need to retain its annotations? | `#[diagnostic(transparent)]`, `forward`, `diagnostic_source` | Avoids losing diagnostic information                              |
| Can validation produce several independent errors?        | `#[related]`                                                 | Collects findings without inventing artificial causal chains      |
| Are source locations produced by a parser?                | `SourceSpan` / `SourceCode`                                  | Separates byte offsets from rendering                             |
| Are source buffers large or shared?                       | Custom `SourceCode` and shared ownership                     | Can avoid repeated full-source allocation                         |
| Is the consumer another Rust library?                     | Concrete typed errors                                        | Keeps matching and typed access available                         |
| Is the consumer application-level orchestration?          | `Report`, `Result`, `WrapErr`                                | Simplifies heterogeneous error propagation                        |
| Is the consumer a terminal or accessibility tool?         | Graphical/narratable handlers                                | Changes presentation without changing diagnostic semantics        |
| Is the consumer automated tooling?                        | `Diagnostic` or `JSONReportHandler`                          | Exposes structured findings without parsing human-readable output |

### 6.1 Three separate relationships

A useful way to reason about the libraries is to distinguish:

Causality: Error A occurred because of error B. Expressed through `Error::source()` and, when appropriate, `Diagnostic::diagnostic_source()`.

Association: Several findings belong to one operation or validation result. Expressed through `Diagnostic::related()`.

Location: A diagnostic is associated with one or more byte ranges in a source. Expressed through `SourceCode`, `SourceSpan`, and `LabeledSpan`.

Trying to force all three into standard error-source chains loses information. Equally, treating every annotated span as a separate error can fragment one coherent diagnostic.

### 6.2 Context messages versus structured context

`WrapErr` is convenient because it adds descriptions of what an operation was attempting to do, without requiring a new type at every propagation boundary.

But contextual messages are not a substitute for structured fields when consumers need to inspect the operation, resource, or error category programmatically.

A `LoadError::Read { path, source }` representation can expose the path as typed data. A report wrapped with `"failed to read settings.json"` supplies useful explanatory context but does not automatically make the path available as a distinct machine-readable field.

These mechanisms are complementary. The tradeoff depends on whether a piece of context belongs to the semantic error contract or primarily to human explanation.

### 6.3 Errors and warnings are separate from control flow

miette supports `Warning` and `Advice` severities, but the diagnostic severity is not equivalent to the Rust `Result` success/failure channel.

A diagnostic with warning severity may still be carried inside an `Err`, and a successful operation may separately produce warnings. The library does not prescribe whether such warnings should abort a workflow.

Likewise, collecting related diagnostics does not automatically establish the policy for proceeding after individual findings.

miette supplies diagnostic representation and reporting; control-flow policy and recovery semantics remain with the caller.

### 6.4 Source retention, performance, and lifetime

A source-backed diagnostic may need access to the relevant source contents at report-rendering time.

That creates an ownership and lifetime consideration absent from errors that contain only small messages.

Possible representations include owned strings, shared buffers, and custom source providers. The choice affects allocations, memory retention, and whether an error can safely outlive the operation that created it.

A byte-span representation is compact, but deriving human-readable excerpts requires source retrieval and location resolution. Source rendering, wrapping, and syntax highlighting therefore introduce costs not incurred by simply propagating a typed error.

The general tradeoff is between cheap, structured error propagation and richer, potentially more expensive diagnostic presentation. The separation between the libraries allows those costs to be incurred at different boundaries.

### 6.5 Relationship to adjacent error libraries

thiserror and miette cover a large portion of Rust's typed-error and diagnostic-reporting needs, but they do not replace every adjacent error-handling tool.

`anyhow` and `eyre` address ergonomic application-level error aggregation. miette provides comparable `Report` and context-propagation facilities, while additionally supplying a structured diagnostic protocol and specialized rendering.

`color-eyre` emphasizes colorful application error reporting and related context/backtrace presentation, whereas miette's distinctive mechanism is the producer-facing `Diagnostic` interface with source annotations and related findings.

`tracing` and `tracing-error` address execution-time instrumentation and contextual tracing rather than representing a collection of annotated source-level diagnostics. They may provide complementary evidence about how an error arose, but they do not replace its typed identity or diagnostic source ranges.

These are differences in responsibility, not an assertion that one stack should universally replace another.

## 7. Principal limitations and implementation considerations

Several boundaries deserve particular attention when evaluating these libraries together.

| Limitation or condition                                                         | Technical significance                                                         |
| ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `#[from]` cannot include arbitrary context fields                               | Contextual conversions require explicit construction                           |
| thiserror error variants remain part of the public type contract when exposed   | Enum changes can affect downstream compatibility                               |
| `#[error(transparent)]` does not imply `#[diagnostic(transparent)]`             | Diagnostic annotations may be lost across wrappers                             |
| `.into_diagnostic()` can erase existing rich diagnostic metadata                | Conversion paths must distinguish standard errors from diagnostics             |
| `SourceSpan` uses byte offsets                                                  | External offsets must match the source's exact byte representation             |
| Standard error causes differ from related diagnostics                           | Causal and aggregate error relationships must be modeled separately            |
| `Report` uses type erasure and owned, thread-safe values                        | It has different API and lifetime characteristics from concrete error types    |
| Handler hooks are process-global                                                | Local formatting decisions should not depend on competing global configuration |
| Graphical rendering depends on optional features                                | A successful derive does not imply a full graphical reporter is enabled        |
| The JSON renderer exposes diagnostic metadata, not arbitrary application fields | Additional structure or versioning may be needed for persistent interfaces     |

The general architectural opportunity is to avoid bespoke code for standard error traits, common conversions, diagnostic annotations, source snippet generation, and reporting formats while retaining precise ownership of the error model itself.

## 8. Primary-source reference map

The following direct URLs provide useful entry points for deeper investigation.

thiserror

- Crate API and derive behavior: `https://docs.rs/thiserror/2.0.21/thiserror/`
- Published package metadata and dependency configuration: `https://docs.rs/crate/thiserror/2.0.21/source/Cargo.toml.orig`
- Upstream implementation and examples: `https://github.com/dtolnay/thiserror`

miette — Diagnostic modeling

- Crate documentation and comprehensive examples: `https://docs.rs/miette/7.6.0/miette/`
- Diagnostic protocol: `https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html`
- Derive attributes: `https://docs.rs/miette/7.6.0/miette/derive.Diagnostic.html`
- Source abstraction: `https://docs.rs/miette/latest/miette/trait.SourceCode.html`
- Span contents: `https://docs.rs/miette/latest/miette/trait.SpanContents.html`
- Byte-based source spans: `https://docs.rs/miette/7.6.0/miette/struct.SourceSpan.html`
- Source-offset conversions: `https://docs.rs/miette/latest/miette/struct.SourceOffset.html`
- Named sources: `https://docs.rs/miette/latest/miette/struct.NamedSource.html`
- Label construction: `https://docs.rs/miette/7.6.0/miette/struct.LabeledSpan.html`

miette — Error propagation and reporting

- Report wrapper: `https://docs.rs/miette/7.6.0/miette/struct.Report.html`
- Standard error conversion: `https://docs.rs/miette/7.6.0/miette/trait.IntoDiagnostic.html`
- Context propagation: `https://docs.rs/miette/7.6.0/miette/trait.WrapErr.html`
- Report-handler customization: `https://docs.rs/miette/latest/miette/trait.ReportHandler.html`
- Handler options: `https://docs.rs/miette/7.6.0/miette/struct.MietteHandlerOpts.html`
- Graphical reporter: `https://docs.rs/miette/latest/miette/struct.GraphicalReportHandler.html`
- Narratable reporter: `https://docs.rs/miette/latest/miette/struct.NarratableReportHandler.html`
- JSON reporter: `https://docs.rs/miette/7.6.0/miette/struct.JSONReportHandler.html`
- JSON reporter implementation: `https://docs.rs/miette/latest/src/miette/handlers/json.rs.html`
- Feature declarations: `https://docs.rs/crate/miette/7.6.0/source/Cargo.toml.orig`
- Release history: `https://docs.rs/crate/miette/7.6.0/source/CHANGELOG.md`
- Upstream project and examples: `https://github.com/zkat/miette`

### Overall assessment

The most important capability of thiserror is not simply eliminating implementations of `Display` and `Error`. It makes precise, structured error contracts inexpensive to maintain, including variant-specific context, causal relationships, automatic conversions, and opaque public representations.

The most important capability of miette is not its attractive terminal output. It establishes a shared diagnostic protocol for identifying problems, associating them with source locations, collecting related findings, supplying remediation guidance, and rendering the same information for different consumers.

Combined, they support an error architecture in which semantic failure types remain explicit, rich diagnostic information survives propagation, and application-specific reporting machinery can often be replaced by reusable library capabilities.