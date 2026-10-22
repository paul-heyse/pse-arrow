# Salsa — Comprehensive Rust Library Overview

Target release: Salsa 0.28.5 | Research date: October 22, 2026 | Language: Rust

Salsa is an incremental computation framework for Rust that automatically tracks dependencies, memoizes derived results, and selectively recomputes them when underlying inputs change.

Its central abstraction is a database of interconnected, demand-driven computations. Rather than explicitly maintaining dependency graphs, invalidation rules, cache coherence, and recomputation schedules, developers describe computations as ordinary Rust functions and annotate the data structures and functions that participate in incremental evaluation.

Salsa is particularly well suited to systems with three characteristics:

1. A substantial body of derived information depends on a changing collection of inputs.
2. Changes generally affect only some of the derivations.
3. The same derived information is requested repeatedly, including after inputs have changed.

Language servers, compilers, static analyzers, IDE tooling, document processors, reactive analytical engines, and incremental transformation systems are natural examples.

The important distinction is that Salsa is more than a memoization library. It provides an integrated model for dependency discovery, fine-grained change detection, identity management, result reuse, intermediate representation lifecycles, cycle handling, and concurrent access to incremental state.

Official resources:

- Repository: [https://github.com/salsa-rs/salsa](https://github.com/salsa-rs/salsa?utm_source=chatgpt.com)
- Documentation: [https://docs.rs/salsa/0.28.5/salsa/](https://docs.rs/salsa/0.28.5/salsa/?utm_source=chatgpt.com)
- Salsa book: [https://salsa-rs.github.io/salsa/](https://salsa-rs.github.io/salsa/?utm_source=chatgpt.com)
- Changelog: [https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md](https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md?utm_source=chatgpt.com)
- Crate: [https://crates.io/crates/salsa](https://crates.io/crates/salsa?utm_source=chatgpt.com)

Version and security note: The current documented stable release is 0.28.5, published September 24, 2026. This release fixes a use-after-free security vulnerability affecting interned values and cached function results (GHSA-xc3w-55vh-cw3w). Earlier 0.28 releases should not be treated as interchangeable with 0.28.5 for security-sensitive use. Version 0.28 also introduced significant API changes, including replacing the older `Update` mechanism with `SalsaValue` and `PartialEq`, and making borrowed returns the default.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



## 1. Conceptual model and capability map

A Salsa system consists of an incrementally maintained database, externally supplied inputs, tracked computations, and derived values. The database records not only cached results but also the dependency relationships and change information needed to establish whether those results remain valid.

Conceptual dependency and validation flow. Evaluations are demand-driven; the diagram does not imply that Salsa eagerly recomputes all affected queries after each input change.

Salsa exposes several complementary capability families:

| Capability                             | Primary mechanisms                     | Architectural purpose                                                       |
| -------------------------------------- | -------------------------------------- | --------------------------------------------------------------------------- |
| Incremental computation                | `#[salsa::tracked]` functions          | Automatic memoization and dependency tracking                               |
| External state                         | `#[salsa::input]`                      | Explicit mutation boundaries and field-granular inputs                      |
| Derived entities                       | `#[salsa::tracked]` structs            | Stable identities for incrementally maintained intermediate representations |
| Canonical values                       | `#[salsa::interned]`                   | Structural sharing and inexpensive identity equality                        |
| Incremental validation                 | Red-green algorithm, backdating        | Avoid both unnecessary computation and cascading invalidation               |
| Coarse-grained validation optimization | `Durability`                           | Skip entire categories of dependency checks                                 |
| Memory management                      | LRU, interned reclamation              | Control retention of memoized results and canonical values                  |
| Auxiliary outputs                      | `#[salsa::accumulator]`                | Associate diagnostics and other outputs with tracked computations           |
| Alternative evaluation strategies      | `specify`                              | Combine on-demand queries with batched derivation                           |
| Polymorphic query keys                 | `Supertype`                            | Define computations across multiple kinds of Salsa entities                 |
| Recursive computations                 | Cycle fallback, fixed-point iteration  | Handle cycles with explicit semantics                                       |
| Runtime coordination                   | Database handles, cancellation, events | Coordinate changes, concurrent queries, and observability                   |

The key architectural idea is to make derived state a queryable, automatically maintained consequence of explicitly managed inputs, rather than a separately synchronized collection of caches.

This reduces the amount of application-specific machinery needed to preserve consistency across layers of derived data. Its suitability nevertheless depends on whether the computations have sufficiently well-defined, deterministic dependencies and whether their results can be represented effectively within Salsa's ownership and lifecycle model.

Primary references:

- [https://salsa-rs.github.io/salsa/overview.html](https://salsa-rs.github.io/salsa/overview.html?utm_source=chatgpt.com)
- [https://salsa-rs.github.io/salsa/reference/algorithm.html](https://salsa-rs.github.io/salsa/reference/algorithm.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/](https://docs.rs/salsa/0.28.5/salsa/?utm_source=chatgpt.com)

## 2. Tracked functions and the incremental query engine

### 2.1 Automatic query memoization

The primary computational abstraction is the tracked function:

```
#[salsa::tracked(returns(copy))]
fn word_count(
    db: &dyn salsa::Database,
    file: SourceFile,
) -> usize {
    file.text(db).split_whitespace().count()
}
```

Salsa generates machinery that associates the invocation with a query key, records dependencies encountered during execution, caches the output, and validates the result when it is requested again.

Its behavior differs from conventional memoization in a fundamental way.

With ordinary memoization, a key maps to a cached value until explicitly removed or replaced. With Salsa, a query key maps to a cached value whose continuing validity is established from the dependencies observed during its previous execution.

Those dependencies are discovered dynamically. The caller does not have to declare a static list of upstream queries or construct an explicit dependency graph.

For example, a query may examine configuration, select one of several source modules, and invoke additional queries according to the selected module. Salsa records the dependencies actually accessed during that execution.

When the query re-executes, its dependency set can change along with its control flow. This is particularly useful for conditional dataflows, dynamic module references, graph traversal, and derived structures whose dependencies are themselves derived.

### 2.2 Query identity and key construction

In Salsa 0.28.5, the non-database parameters determine query identity.

| Query signature                             | Memoization behavior                                |
| ------------------------------------------- | --------------------------------------------------- |
| `fn q(db: &dyn Db)`                         | One query instance per database                     |
| `fn q(db: &dyn Db, file: File)`             | Uses the Salsa struct's ID directly                 |
| `fn q(db: &dyn Db, file: File, mode: Mode)` | Interns the argument tuple as a synthetic query key |

The single-argument form requires a Salsa struct, including a compatible `Supertype`. Multiple arguments require the appropriate `Clone`, `Eq`, and `Hash` capabilities, with the generated implementation using argument-tuple interning.

The distinction matters because query arguments and query dependencies are different concepts.

A `File` argument identifies the entity being examined. Reading `file.text(db)` establishes a dependency on its contents. Merely receiving the `File` handle does not establish a dependency on every field.

This enables a large number of distinct computations to share the same entity identity while depending on different aspects of its state.

Performance implication: Using a single Salsa entity as the query key avoids the tuple-interning step. Multi-argument queries remain fully supported, but key construction and hashing become part of their execution overhead.

Tracked functions are subject to generated API constraints, including `Send + Sync` requirements on keys and outputs. They are not unrestricted generic Rust functions: the macro has specific rules for database parameters, lifetimes, and generic parameters.

### 2.3 Red-green incremental validation

Salsa's central incremental mechanism is the red-green algorithm.

For each memoized query, Salsa tracks enough information about its dependencies and their change revisions to determine whether the cached result remains valid.

Conceptually, when a query is requested in a later revision:

1. Salsa checks whether its memo is already valid for that revision.
2. Otherwise, it examines the validity of the dependencies recorded during the previous execution.
3. If those dependencies remain unchanged, Salsa validates the existing memo without re-executing the function body.
4. If relevant dependencies changed, Salsa re-executes the query and records its new result and dependencies.
5. The result's equality determines whether the change needs to propagate to downstream computations.

This is demand-driven validation. Changing an input does not ordinarily trigger eager recomputation of every downstream result.

A query is revalidated when it is needed.

### 2.4 Backdating and change propagation

Backdating is one of Salsa's most consequential capabilities.

Suppose a system has the following dependency chain:

Source

Text

Parser

AST

Checker

Types

Source text changed, so parsing executes again.

The newly parsed AST compares equal to the previous AST.

Salsa backdates the parse result, allowing type checking to reuse its prior result.

The parser's inputs changed, and the parser executed, but its observable output did not change.

By default, Salsa compares the previous and new results using `PartialEq`. If they compare equal, Salsa retains their earlier change revision.

Downstream computations can therefore remain valid even when upstream functions have executed again.

This gives Salsa two separate opportunities to avoid work:

- Memo validation: Skip execution because dependencies did not change.
- Backdating: Avoid propagating invalidation because recomputation produced an equal result.

The distinction is central to incremental language processing and other multistage transformations.

For example, a formatting-only source change could cause lexical analysis or parsing to execute while leaving a normalized semantic representation unchanged. Computations depending only on that semantic representation may then be reused.

### 2.5 Equality semantics and `no_eq`

Backdating depends on meaningful equality.

If equality is expensive, Salsa may spend substantial effort comparing newly computed results. If equality is too coarse or incorrectly implemented, it may incorrectly treat changed outputs as unchanged.

Salsa offers the `no_eq` option:

```
#[salsa::tracked(no_eq, returns(copy))]
fn revision_sensitive_value(
    db: &dyn salsa::Database,
    file: SourceFile,
) -> u64 {
    // Compute a value.
    # unimplemented!()
}
```

The code illustrates the attribute combination; the function body is intentionally omitted.

With `no_eq`, a newly computed result is always treated as changed. This removes the `PartialEq` requirement but sacrifices backdating for that query.

It is appropriate when meaningful equality is unavailable or more expensive than permitting downstream recomputation.

A similar `#[no_eq]` field attribute exists for tracked structs. There, the field is treated as changed whenever the entity is recreated, even if its identity remains stable.

Correctness requirement: Equality must capture everything downstream computations are entitled to observe. It is not merely a performance hint.

Primary references:

- [https://salsa-rs.github.io/salsa/reference/algorithm.html](https://salsa-rs.github.io/salsa/reference/algorithm.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html?utm_source=chatgpt.com)
- [https://salsa-rs.github.io/salsa/plumbing/query_operations.html](https://salsa-rs.github.io/salsa/plumbing/query_operations.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

+2

## 3. Input structs: externally controlled incremental state

Salsa's input model establishes a clear boundary between mutable external state and computed derived state.

### 3.1 Field-granular dependencies

Input structs are declared using `#[salsa::input]`:

```
#[salsa::input]
pub struct SourceFile {
    #[returns(deref)]
    pub path: std::path::PathBuf,

    #[returns(deref)]
    pub text: String,
}
```

Salsa transforms this declaration into a compact identity-bearing handle whose values are stored in the database.

Each field receives an accessor and a generated setter.

```
use salsa::Setter as _;

let file = SourceFile::new(
    &db,
    "module.rs".into(),
    "fn main() {}".to_owned(),
);

file.set_text(&mut db)
    .to("fn main() { println!(\"hi\"); }".to_owned());
```

Dependencies are recorded individually for fields.

A query that reads only `text` need not become invalid because `path` changed. Conversely, a query that computes a path-based module identifier does not automatically depend on the text.

This is a substantial improvement over treating every modification to a larger object as invalidating all of its consumers.

### 3.2 Identity is not structural equality

Every input constructor invocation creates a new identity, even when the field values are identical.

Updating a field preserves that identity.

Consequently, input structs represent externally controlled entities, not deduplicated value objects.

Input handles have no database lifetime parameter, so they can persist across revisions. However, the handle remains associated with its originating database and cannot be treated as a portable identifier usable in arbitrary Salsa databases.

### 3.3 Input mutation always constitutes a write

The generated input setter does not automatically compare the incoming value against the old one.

Calling the setter records the field as changed and begins a new database revision, even if the supplied value is equal to the existing value.

This creates an important distinction between inputs and derived values:

- Input writes are controlled by the caller.
- Derived-value change detection is ordinarily controlled by Salsa's result equality and revision-validation machinery.

Applications that receive frequent duplicate external updates may benefit from deduplicating those updates before invoking Salsa's setters.

### 3.4 Input construction and configuration

The input macro supports several useful extensions:

| Mechanism                     | Capability                                                 |
| ----------------------------- | ---------------------------------------------------------- |
| `singleton`                   | Enforces one instance of an input type per database        |
| `try_get(db)` / `get(db)`     | Retrieves singleton inputs                                 |
| `#[default]`                  | Provides default-initialized fields with builder overrides |
| `#[get(...)]` / `#[set(...)]` | Customizes generated accessor names                        |
| `constructor = ...`           | Renames the constructor                                    |
| `debug`                       | Generates database-aware debugging                         |
| `persist`                     | Opts an input type into the optional persistence mechanism |

The singleton capability is useful for configuration or other logical inputs that should have exactly one identity within the database.

It can eliminate the need to pass a global configuration handle explicitly to every query, although queries still need to read the relevant fields to establish their dependencies.

Lifecycle limitation: Standard input identities and their associated database storage remain until the database is dropped. Dropping the Rust handle does not delete the corresponding input. This matters for workloads that continually create new, short-lived input identities.

Sources:

- [https://docs.rs/salsa/0.28.5/salsa/attr.input.html](https://docs.rs/salsa/0.28.5/salsa/attr.input.html?utm_source=chatgpt.com)
- [https://salsa-rs.github.io/salsa/overview.html](https://salsa-rs.github.io/salsa/overview.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1

## 4. Tracked structs: stable identities for derived entities

Tracked structs are significantly more than convenient containers for query results.

They allow Salsa to represent and incrementally maintain entities produced by computations.

### 4.1 Identity and mutable derived properties

Consider an intermediate representation in which a syntax entity has a stable logical identity while its derived properties may change.

```
#[salsa::tracked]
pub struct Symbol<'db> {
    pub name: Name<'db>,

    #[tracked]
    #[returns(copy)]
    pub offset: usize,
}
```

The `name` field participates in the entity's identity.

The `offset` field does not. It can change when the producing query is re-executed without necessarily changing the identity of the `Symbol`.

Salsa matches newly produced tracked structs against entities produced by the same query in an earlier revision.

If identity is preserved, the existing entity remains, and its tracked fields are updated where their values changed.

Downstream queries that depend on unchanged fields may remain valid.

This enables incremental processing of structured data without requiring the application to maintain its own entity reconciliation and field-level invalidation machinery.

### 4.2 The full identity model

A tracked struct's identity is determined by:

- The query invocation that produced it.
- Its fields not marked `#[tracked]`.
- Its occurrence among same-identity structs produced by that query.

The occurrence component is important.

If a producing query creates two tracked entities with the same identity fields, those entities do not necessarily collapse into one. Salsa distinguishes their occurrences.

This means that stable identity depends on both the selected identity fields and the consistency of the producing query's construction behavior.

In particular, changing the order in which otherwise identical entities are produced can change their matching across revisions.

An effective representation therefore uses identity fields that correspond to actual conceptual identity, rather than fields that frequently change as incidental consequences of processing.

### 4.3 Output ownership and reclamation

Tracked structs belong to their producing query.

When that query is validated, its tracked outputs are also validated.

When it re-executes, Salsa reconciles the new set of outputs with the old set. Previously produced entities that are not recreated become stale and can be reclaimed.

This introduces a database-managed lifecycle for derived entities.

The benefit is that the application need not manually delete every derived entity that disappears following an input change.

The constraint is that tracked structs are not independent mutable records. They are products of a defining computation and exist subject to that computation's lifecycle.

### 4.4 Structural sharing versus query-owned identity

Tracked structs and interned structs can look superficially similar because both provide compact handles to database-resident data.

Their semantics are very different.

A tracked struct models an entity produced by a particular computation.

An interned struct models a canonical immutable value shared wherever equal fields occur.

If two independent queries each create the same-looking tracked struct, their entities are distinct. If they intern the same field values, they obtain the same canonical interned identity within the relevant revision.

Architectural opportunity: Tracked structs can represent declarations, graph nodes, semantic entities, or IR objects, while interned structs represent shared names, paths, symbols, types, or other immutable value-like components.

Sources:

- [https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html?utm_source=chatgpt.com)
- [https://salsa-rs.github.io/salsa/plumbing/tracked_structs.html](https://salsa-rs.github.io/salsa/plumbing/tracked_structs.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1

## 5. Interned structs: canonicalization and identity sharing

Interning provides an efficient, database-wide relationship between values and canonical identities.

### 5.1 Interned value semantics

An interned struct is defined with `#[salsa::interned]`:

```
#[salsa::interned]
pub struct Name<'db> {
    #[returns(deref)]
    pub text: String,
}
```

Within the same database and revision:

```
let a = Name::new(&db, "alpha".to_owned());
let b = Name::new(&db, "alpha".to_owned());

assert_eq!(a, b);
```

Both values represent the same canonical interned identity.

This has several consequences:

- Repeated occurrences of an equal value can share storage.
- Equality between handles is an inexpensive identity comparison.
- Interned values can serve directly as components of other Salsa structures.
- Common strings and immutable semantic values need not be copied throughout a large graph of derived entities.

Interning can be particularly beneficial in workloads with a high degree of structural repetition.

### 5.2 Memory reclamation is part of the current model

An important change from older expectations of interning is that low-durability interned values are not necessarily immortal.

Salsa 0.28.5 supports revision-based reclamation of interned values.

The `revisions` option determines how many active revisions an eligible value can go unused before its storage becomes reclaimable. The documented default is 3.

A reclaimed slot may be reused with a new ID generation, allowing Salsa to recognize that dependencies on the previous value have become invalid.

Values with higher durability are not reclaimed by that mechanism. Setting `revisions = usize::MAX` disables reclamation for an interned type.

The tradeoff is between memory retention and the work associated with re-interning values that reappear.

### 5.3 Interning is not always the fastest representation

Interning introduces a shared canonicalization operation involving hashing, equality, and synchronization.

It is most attractive when the benefits of deduplication and cheap downstream equality outweigh the cost of interner lookups.

For derived entities that already belong to a specific query, tracked structs may be a better representation because they avoid unnecessary database-wide structural canonicalization.

The best results may come from combining the two mechanisms: tracked entity identity for mutable derived structure, interned handles for immutable shared components.

Source:

[https://docs.rs/salsa/0.28.5/salsa/](https://docs.rs/salsa/0.28.5/salsa/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 6. Return modes, borrowing, and revision-safe values

Salsa's ownership model is a major part of its performance and correctness design. It stores data inside the database and ordinarily exposes borrowed access, avoiding unnecessary cloning of cached results.

### 6.1 Return modes

The 0.28 API uses borrowed returns by default, with explicit alternatives.

| Mode       | Returned form                      | Typical use                                         |
| ---------- | ---------------------------------- | --------------------------------------------------- |
| `ref`      | `&T`                               | Borrow a stored result without copying; default     |
| `copy`     | `T` via `Copy`                     | Scalars and compact handles                         |
| `clone`    | `T` via `Clone`                    | Owned values that must outlive a database borrow    |
| `deref`    | `&T::Target`                       | Borrow the contents of `String`, `Vec`, `Arc`, etc. |
| `as_ref`   | Reference-oriented associated form | Optional or fallible stored values                  |
| `as_deref` | Dereferenced associated form       | For example, `Option<String>` to `Option<&str>`     |

For tracked functions, the annotation appears on the function:

```
#[salsa::tracked(returns(clone))]
fn representation(db: &dyn Db, node: Node) -> Arc<Representation> {
    // ...
}
```

For fields, it appears on the field as `#[returns(MODE)]`.

The choice affects more than convenience. A borrowed result avoids a clone but retains a borrow of the database, preventing mutation until that borrow ends. An owned result releases that constraint, subject to the ordinary Rust ownership and lifetime rules of its type.

One subtle point is that `returns(clone)` does not disable memoization. Salsa still stores the computed result internally; cloning changes how callers receive it.

### 6.2 The `'db` lifetime

Tracked and interned structs typically carry a database lifetime:

```
#[salsa::interned]
struct Symbol<'db> {
    name: String,
}
```

The lifetime prevents a handle created or accessed in one database revision from being used unsafely across a revision change that might reclaim or replace its backing storage.

In particular:

- Input handles can be copied across revisions, subject to database identity.
- Tracked and interned handles generally must be obtained again in the new revision.
- References to fields or memoized results cannot remain live across a mutable database borrow.
- Holding a tracked or interned handle beyond its valid database borrow is not a supported way to establish permanent entity identity.

This is an important distinction for integrations involving long-lived object models, asynchronous tasks, or external caches. Those systems often need to retain their own stable identifiers or immutable owned representations and reacquire Salsa handles when querying a new revision.

### 6.3 `SalsaValue`: safely retaining lifetime-bearing data

Salsa sometimes needs to retain a value after the borrow under which it was produced has ended. This happens with memoized results, tracked fields, and interned fields.

The `SalsaValue` trait expresses a particular safety guarantee: Salsa may store the value and later rebind the database lifetime when exposing it under a valid database borrow.

Types containing only `'static` data are generally accepted without a custom implementation. Types containing `'db`-parameterized Salsa handles may need to derive the trait:

```
#[derive(salsa::SalsaValue, PartialEq)]
struct SemanticResult<'db> {
    symbol: Symbol<'db>,
}
```

The derive macro checks whether the constituent types support that retention contract. It does not make arbitrary borrowed references safe to retain.

Manually implementing `SalsaValue`, or using the macro's unsafe retention exemptions, requires independently proving lifetime safety. These mechanisms are extension points for unusual representations, not general-purpose lifetime workarounds.

Version distinction: Salsa 0.28 replaced the older `Update`-based approach with the combination of `SalsaValue` for retention safety and `PartialEq` for ordinary change comparison.

Sources:

- [https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/derive.SalsaValue.html](https://docs.rs/salsa/0.28.5/salsa/derive.SalsaValue.html?utm_source=chatgpt.com)
- [https://salsa-rs.github.io/salsa/plumbing/db_lifetime.html](https://salsa-rs.github.io/salsa/plumbing/db_lifetime.html?utm_source=chatgpt.com)
- [https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md](https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+3

## 7. Durability: avoiding unnecessary dependency validation

Durability is an advanced optimization layered on top of Salsa's normal dependency tracking.

It describes how frequently particular input values are expected to change.

The important purpose is to skip dependency validation itself, not merely to avoid executing computations whose values remain unchanged.

### 7.1 Durability levels

Salsa defines the following levels:

| Durability     | Meaning                                                |
| -------------- | ------------------------------------------------------ |
| `LOW`          | Frequently changing inputs; default for input fields   |
| `MEDIUM`       | Inputs expected to change less frequently              |
| `HIGH`         | Rarely changing inputs                                 |
| `NEVER_CHANGE` | Inputs that are permanently immutable after assignment |

For example, source text changing during editing might be low durability, project settings medium durability, and stable platform definitions high durability.

A field's durability can be specified through its generated setter:

```
use salsa::Setter as _;

config
    .set_target(&mut db)
    .with_durability(salsa::Durability::HIGH)
    .to(new_target);
```

The setter supplies both the new value and the durability classification.

`NEVER_CHANGE` is stronger than a performance hint. A field assigned this durability cannot subsequently have its value or durability changed; attempts to modify it panic.

### 7.2 How durability reduces validation work

A tracked computation inherits information about the durability of the values it depends on.

If a database revision changes only low-durability inputs, a computation known to depend exclusively on higher-durability inputs need not have its dependency list recursively examined.

Consider:

- A large collection of high-durability built-in definitions.
- Thousands of symbol-resolution queries dependent on those definitions.
- A small edit to a low-durability source file.

Without durability information, Salsa may need to traverse substantial parts of the query graph to establish that certain definitions and their derived results remain valid.

With appropriate durability classification, entire validation paths can be skipped.

This optimization is most meaningful where the computation graph is large and dependency validation becomes a material portion of query latency.

### 7.3 Durability propagates through computations

A derived result's effective durability is constrained by the dependencies actually read during its computation.

A query that reads one low-durability input cannot simply be assumed high durability because it also reads several high-durability inputs.

Conversely, a computation involving only high-durability dependencies can inherit the relevant high-durability validation benefit.

Durability therefore works particularly well in conjunction with fine-grained input fields and fine-grained tracked results.

The library still performs normal dependency accounting and invalidation when higher-durability inputs actually change. Correct classification is valuable because it improves the selectivity of revision-wide validation.

### 7.4 Relationship to interned-value reclamation

Durability also affects memory behavior.

Salsa's interned-value reclamation is principally applicable to low-durability interned values. Higher-durability values are retained, reducing the risk of discarding canonical values that are expected to remain useful.

The same concept therefore influences two different mechanisms:

- Incremental validation cost.
- Retention eligibility for interned data.

This creates a meaningful architectural tradeoff: a durability choice can make validation cheaper while allowing more interned state to remain resident.

Primary references:

- [https://salsa-rs.github.io/salsa/reference/durability.html](https://salsa-rs.github.io/salsa/reference/durability.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/struct.Durability.html](https://docs.rs/salsa/0.28.5/salsa/struct.Durability.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/trait.Setter.html](https://docs.rs/salsa/0.28.5/salsa/trait.Setter.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

+2

## 8. Accumulators: incremental diagnostics and auxiliary results

Accumulators solve a problem that ordinary memoized function results handle awkwardly.

A computation often needs to produce both a principal result and additional information associated with executing that computation.

Examples include parse errors, warnings, validation findings, source-map annotations, and other diagnostics.

Encoding all such information into the principal result can make equality comparisons, return types, and downstream dependencies unnecessarily complicated.

Printing messages or mutating an external collection during query execution is also unsuitable: when Salsa reuses a memoized result, the function body does not execute again, so those side effects would not recur.

### 8.1 Accumulator declaration and emission

An accumulator is declared with its own attribute:

```
#[salsa::accumulator]
pub struct Diagnostic {
    pub message: String,
    pub offset: usize,
}
```

A tracked computation can then emit values:

```
use salsa::Accumulator as _;

Diagnostic {
    message: "Unexpected token".into(),
    offset: 42,
}
.accumulate(db);
```

The emitted values are associated with the executing query, rather than written into an application-owned global collection.

### 8.2 Transitive retrieval

A query's accumulated values can be retrieved with the generated `accumulated` function:

```
let diagnostics: Vec<&Diagnostic> =
    analyze::accumulated::<Diagnostic>(&db, file);
```

This retrieves accumulated values from the requested query and its transitive callees.

The call first ensures the query is up to date. When the query's prior execution remains valid, Salsa can reuse previously accumulated values without re-executing the entire analysis.

If a query is re-executed, its accumulated data is updated accordingly.

### 8.3 Auxiliary outputs do not drive ordinary result equality

Accumulated values are separate from the main return value.

A difference in diagnostics does not, by itself, imply that a tracked function's primary result has changed.

This separation can preserve reuse of expensive downstream computations that depend on the main semantic result but not on every diagnostic emitted along the way.

The distinction is especially valuable in systems where diagnostics are numerous or change for reasons that do not affect subsequent semantic operations.

There is an associated design consideration: a caller that needs diagnostics must retrieve them through the accumulator mechanism. It cannot assume that inspecting only the function's primary result captures all observable information.

Accumulation is valid only during tracked-function execution. Emitting outside a tracked query is an error and panics.

Sources:

- [https://salsa-rs.github.io/salsa/tutorial/accumulators.html](https://salsa-rs.github.io/salsa/tutorial/accumulators.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/attr.accumulator.html](https://docs.rs/salsa/0.28.5/salsa/attr.accumulator.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

+1

## 9. Explicit result specification and polymorphic query keys

These two features materially expand how Salsa can represent computation graphs.

### 9.1 `specify`: combining batched and on-demand computation

Many systems have operations that are naturally computed in a batch but subsequently queried one entity at a time.

A compiler might analyze an entire module to derive information about every declaration. A graph analysis might calculate many nodes' properties in one traversal.

Salsa's ordinary model is demand-driven: each tracked query knows how to calculate its result for a particular key.

The `specify` mechanism makes it possible to retain that on-demand interface while installing results produced by another computation.

```
#[salsa::tracked(specify)]
fn representation<'db>(
    db: &'db dyn Db,
    item: Item<'db>,
) -> Representation {
    // Fallback on-demand implementation.
    // ...
}
```

A tracked query that creates an `Item` can compute a value through an alternative route and install it:

```
let item = Item::new(db, /* identity and fields */);
let result = /* result produced by batch analysis */;

representation::specify(db, item, result);
```

This enables a dual evaluation model:

- On-demand path: `representation(db, item)` computes the result when required.
- Batch path: The producing computation calculates multiple representations and specifies their values ahead of demand.

Both paths can expose the same tracked-query interface to downstream consumers.

The result is not an arbitrary mutable cache entry. Salsa associates the specified value with the tracked computation that created the entity and manages its revision validity accordingly.

The constraints are consequential:

- The specifiable query must take exactly one non-database argument.
- That argument must be a tracked struct.
- Specification must occur within the same tracked-query invocation that created that entity.
- `specify` and `lru` cannot be combined.

This capability can replace bespoke maps that otherwise duplicate a bulk analysis's results for individual lookup.

It also supports special built-in entities whose values are predetermined rather than derived through the ordinary fallback function.

Source:

[https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2

### 9.2 `Supertype`: heterogeneous entity-oriented queries

A conventional Rust query signature with one entity parameter normally accepts a single concrete type.

However, many analyses naturally operate on several kinds of entities: declarations, expressions, modules, or other typed IR nodes.

Salsa provides the `Supertype` derive macro for heterogeneous query keys.

Conceptually:

```
#[derive(salsa::Supertype)]
enum ItemLike<'db> {
    Function(Function<'db>),
    Variable(Variable<'db>),
}
```

A tracked function can accept `ItemLike<'db>` as its single key type and match on the concrete variant.

The important property is that a supertype does not create another database object or another independent identity. It wraps an existing Salsa struct and reuses that struct's ID in the query-key mechanism.

This provides a type-safe alternative to introducing a custom shared entity table merely so different types of objects can participate in one query family.

The derive imposes structural requirements:

- The enum must be nonempty.
- Every variant wraps exactly one Salsa struct or another `Supertype`.
- A concrete Salsa struct must be reachable through only one variant.
- Nested supertypes are supported when their variant structures remain unambiguous.

A supertype's validity follows the wrapped entity's lifecycle.

Architectural implication: A rich typed IR can preserve distinct domain types while supporting cross-cutting query operations such as type lookup, reference resolution, diagnostics, formatting, or metadata retrieval.

Source:

[https://docs.rs/salsa/0.28.5/salsa/derive.Supertype.html](https://docs.rs/salsa/0.28.5/salsa/derive.Supertype.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

## 10. Recursive queries and cycle recovery

Dependency cycles are a significant issue in incremental graph analysis.

Ordinary memoized recursion assumes that computation dependencies eventually terminate. That assumption fails in systems containing recursive definitions, mutually dependent types, circular imports, or graph properties defined in terms of other graph properties.

Salsa 0.28.5 provides two distinct cycle-recovery strategies.

### 10.1 Default cycle detection

By default, encountering a query that is already active in the same dependency chain produces a panic identifying the cycle head.

This behavior is appropriate when cyclic dependencies are invalid or represent an application error.

The panic is preferable to silently producing an arbitrary value, because the query author has not declared any meaningful semantics for recursion.

### 10.2 Fixed-point iteration

Some cyclic computations have a mathematically meaningful fixed-point solution.

Salsa supports this using `cycle_initial` and, optionally, `cycle_fn`:

```
#[salsa::tracked(
    cycle_initial = cycle_initial,
    cycle_fn = cycle_fn
)]
fn inferred_property(
    db: &dyn Db,
    item: Item<'_>,
) -> Property {
    // ...
}
```

The initial function supplies a provisional value for the cycle head.

Salsa evaluates participating queries using that provisional value, obtains an updated result, and repeats until the value converges.

The optional `cycle_fn` combines the previous provisional value with the newly computed value. This provides an extension point for operations such as lattice joins or widening.

The intended mathematical model is a monotone computation over a finite-height partially ordered domain.

Typical examples include dataflow analyses and recursive propagation of type or property information.

The main constraints are:

- Results must be deterministic.
- Iteration must progress monotonically toward convergence.
- All queries that could become cycle heads need suitable `cycle_initial` support.
- Recovery functions must avoid introducing problematic new cycles.
- Salsa panics if a cycle exceeds 200 iterations.

A `cycle_fn` can inspect the previous provisional result and a `Cycle` identifier to implement domain-specific convergence behavior.

Salsa supplies iteration control, dependency integration, and convergence checking. It does not automatically infer a suitable lattice, a sound merge operation, or widening strategy for arbitrary query results.

### 10.3 Fallback cycle results

The alternative is `cycle_result`.

```
#[salsa::tracked(cycle_result = fallback)]
fn analyze(db: &dyn Db, item: Item<'_>) -> Analysis {
    // ...
}
```

Here, a recovery function supplies an application-defined fallback value when a cycle is detected.

Unlike iterative recovery, this does not attempt to calculate a least or greatest fixed point.

The current semantics ensure that participating queries use their fallback result when a cycle is encountered, rather than returning different values merely because evaluation began at a different entry point.

This is appropriate when a cyclic dependency should produce an explicit sentinel, error-like result, or conservative approximation.

Fixed-point and fallback recovery are alternative approaches. They should not be confused with automatically supported arbitrary recursive computation.

Source:

[https://salsa-rs.github.io/salsa/cycles.html](https://salsa-rs.github.io/salsa/cycles.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

## 11. Database architecture, parallel queries, and concurrency

### 11.1 Database composition

Salsa databases combine a runtime-managed storage system with any application-specific state required to serve queries.

A custom database is declared using `#[salsa::db]`:

```
#[salsa::db]
#[derive(Clone, Default)]
pub struct AnalysisDatabase {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for AnalysisDatabase {}
```

Applications can also define `#[salsa::db]` traits extending `salsa::Database` to expose additional capabilities through trait objects.

For simple uses that do not require custom state, Salsa provides `salsa::DatabaseImpl`, a ready-made database implementation with `Default`, `Clone`, and `Database` support.

The database has two broad categories of state:

- Shared state: Inputs, tracked entities, interned values, query memos, dependency information, revision metadata, and synchronization.
- Handle-local state: The active query stack and other execution context associated with an individual database handle.

Internally, the shared runtime is commonly described through `Zalsa`, while `ZalsaLocal` represents handle-local execution state.

This division is important because it permits multiple threads to evaluate queries against shared incremental state while maintaining their own execution stacks.

### 11.2 Parallel database handles

Salsa permits concurrent query evaluation using cloned database handles.

Each clone shares the database's underlying memoized results and runtime but has a separate active query stack.

This is different from creating independent databases, which would duplicate their caches and input identities.

It also differs from conventional read-only snapshots of historical database versions.

All active Salsa handles operate against the same current database revision. A database does not expose independently mutable, multiversion snapshots.

Consequently, cloned handles provide a way to perform concurrent computations within a revision, not a way to continue reading indefinitely from an old revision while another thread advances the database.

The standard database implementation is `Send` but not `Sync`. Concurrent execution is therefore organized around separate handles, rather than sharing a single database handle by reference across arbitrary threads.

### 11.3 Synchronization between concurrent queries

When multiple threads request the same memoized computation, Salsa coordinates access so that the shared cache and dependency state remain consistent.

If another thread is already evaluating a required query, a consumer may block until that computation completes.

The runtime also tracks cross-thread query dependencies and can detect cycles that arise through parallel execution.

Salsa 0.28.3 introduced more finely sharded synchronization by tracked-query key, reducing unnecessary coordination between unrelated query instances.

These mechanisms make concurrent query execution possible, but they do not mean every dependency graph is automatically distributed across threads.

The application still determines where it introduces parallel work and how it manages execution resources.

### 11.4 Writes cancel and drain readers

Input mutation requires exclusive access to the shared database state.

If cloned handles exist, Salsa initiates cancellation of outstanding work and waits for those handles to be released before completing the mutation.

This introduces a significant operational constraint.

A writer that retains a database clone while attempting a write can deadlock. The same problem can arise when a long-lived worker retains a handle but does not terminate or release it in response to cancellation.

For workloads with frequent edits and expensive concurrent analyses, the duration and lifetime of database handles are therefore material architectural considerations.

A sensible general separation is:

- A controlling context owns input mutation.
- Parallel workers use cloned handles to perform read-only computations.
- Workers relinquish their handles on cancellation or completion.
- New queries begin against the updated revision.

Salsa supplies the coherence and cancellation mechanisms, but not a complete task scheduler or application-level worker lifecycle manager.

Sources:

- [https://salsa-rs.github.io/salsa/plumbing/database_and_runtime.html](https://salsa-rs.github.io/salsa/plumbing/database_and_runtime.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/struct.DatabaseImpl.html](https://docs.rs/salsa/0.28.5/salsa/struct.DatabaseImpl.html?utm_source=chatgpt.com)
- [https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md](https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

+2

## 12. Cancellation and untracked dependencies

### 12.1 Query cancellation

Cancellation is integrated into Salsa's revision model.

When a query becomes obsolete because a database update is occurring, Salsa can interrupt its execution.

The mechanism uses unwinding with a `salsa::Cancelled` panic payload.

Salsa checks for cancellation when queries call other tracked queries, but a computation that spends a long time in an ordinary numerical loop or parsing loop may not reach such a checkpoint promptly.

For these cases, query implementations can explicitly check:

```
db.unwind_if_revision_cancelled();
```

An operation performing substantial work can call this periodically.

This helps obsolete computations release their database handles promptly, allowing writes to proceed.

### 12.2 Cancellation tokens

The database also exposes:

```
db.cancellation_token()
```

The returned `CancellationToken` provides `cancel()` and `is_cancelled()` methods.

This supplies a cancellation signal that can be communicated to other execution contexts without requiring them to own a database handle.

It is distinct from the automatic revision-invalidation mechanism but can participate in application-level cancellation coordination.

Because Salsa uses unwinding internally, application code should not indiscriminately catch and suppress all panics from tracked computations. Doing so can interfere with cancellation propagation and obscure genuine computational failures.

### 12.3 Untracked external state

Salsa normally expects tracked computations to be deterministic functions of Salsa-visible inputs.

Sometimes a computation must read information that Salsa cannot track directly, such as external state accessed through an ordinary Rust API.

The `Database` trait provides:

```
db.report_untracked_read();
```

This reports that the active query depends on information outside the ordinary Salsa dependency graph.

Queries making such a report are scheduled for re-execution in the next revision.

That is useful as an escape hatch, but it is less selective than modeling the external information as an ordinary Salsa input.

It also does not continuously monitor external state: the passage of time or a change in the filesystem does not itself create a Salsa revision.

For repeatably changing external data, explicit input updates are generally the more precise integration boundary.

Sources:

- [https://salsa-rs.github.io/salsa/tuning.html](https://salsa-rs.github.io/salsa/tuning.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/struct.CancellationToken.html](https://docs.rs/salsa/0.28.5/salsa/struct.CancellationToken.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/latest/src/salsa/database.rs.html](https://docs.rs/salsa/latest/src/salsa/database.rs.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

+2

## 13. Memory management and performance tuning

Salsa makes a fundamental tradeoff: retaining computation history enables faster incremental evaluation, but retained history consumes memory.

Several mechanisms control different portions of that cost.

### 13.1 Memoized result eviction

Tracked queries support least-recently-used result eviction:

```
#[salsa::tracked(lru = 128)]
fn parse(db: &dyn Db, source: SourceFile) -> Ast {
    // ...
}
```

This configures a capacity of 128 retained results for that query function.

Eviction occurs at the beginning of a new revision. It discards cached values, while preserving the query keys and dependency metadata needed for future evaluation.

A subsequent call for an evicted value may need to compute it again.

The limit is not a comprehensive bound on all memory associated with the query. In particular, it does not remove input identities or every memo metadata entry.

By default, no LRU eviction is enabled; cached results remain available until invalidated, reclaimed through another lifecycle mechanism, or discarded with the database.

### 13.2 Dynamic capacity configuration

For an LRU-enabled query, Salsa generates:

```
parse::set_lru_capacity(&mut db, 256);
```

This changes the retention policy during execution of the application.

The operation requires mutable database access and therefore participates in Salsa's exclusive-access and cancellation requirements.

The library also exposes:

```
db.trigger_lru_eviction();
```

This requests enforcement of the configured eviction limits.

An LRU cache is useful when query-key cardinality is large and only a subset of results remains actively used. It can be counterproductive when evicted computations are expensive and frequently revisited.

### 13.3 Distinct memory lifecycles

It is important to distinguish what Salsa can reclaim.

| Stored information                 | Retention behavior                                                  |
| ---------------------------------- | ------------------------------------------------------------------- |
| Input identities                   | Remain until database destruction                                   |
| Input field values                 | Replaced by setters; current state remains                          |
| Query memo values                  | Normally retained; optional LRU eviction                            |
| Query keys and dependency metadata | Not generally removed by memo-value LRU                             |
| Tracked entities                   | Reconciled and reclaimed according to producing-query lifecycle     |
| Low-durability interned values     | Eligible for revision-based reclamation                             |
| High-durability interned values    | Not reclaimed by the low-durability mechanism                       |
| Accumulated outputs                | Follow the execution and memo lifecycle of their generating queries |

This means no single configuration option comprehensively caps Salsa's total database memory consumption.

For a workload with continuously growing input-key cardinality, result LRU may reduce value retention without eliminating all growth in metadata and identity storage.

That is an important limitation for systems with large volumes of short-lived entities.

### 13.4 Equality, key choice, and cache granularity

Performance depends strongly on the structure of the query graph.

Fine-grained queries provide more opportunities to reuse unaffected results. They also increase the number of memo entries, dependency edges, hash lookups, and synchronization operations.

Coarse-grained queries reduce orchestration overhead but can force expensive recomputation when only a small part of their result changes.

Likewise, returning a large structure and comparing it with `PartialEq` can make backdating expensive.

A representation with stable tracked entities and independently tracked fields may avoid some whole-structure comparisons and unnecessary downstream work.

Interned values can make equality and sharing cheaper, but their construction involves interner lookups and possible synchronization.

These are qualitative tradeoffs. Salsa does not guarantee that more tracked functions or more interning will improve performance for an arbitrary workload.

Source:

[https://salsa-rs.github.io/salsa/tuning.html](https://salsa-rs.github.io/salsa/tuning.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

+1

## 14. Observability, debugging, and instrumentation

Salsa provides useful facilities for understanding incremental execution.

### 14.1 Runtime events

The `Event` and `EventKind` APIs describe significant runtime activity.

Relevant event kinds include:

| Event                      | Interpretation                                          |
| -------------------------- | ------------------------------------------------------- |
| `WillExecute`              | A query function is about to execute                    |
| `DidValidateMemoizedValue` | A memo was successfully validated for reuse             |
| `WillBlockOn`              | A query is waiting for another thread's computation     |
| `WillIterateCycle`         | Fixed-point iteration is starting another iteration     |
| `DidFinalizeCycle`         | A fixed-point iteration has completed                   |
| `WillCheckCancellation`    | The runtime is checking whether work has been cancelled |
| `DidSetCancellationFlag`   | Cancellation was initiated                              |
| `WillDiscardStaleOutput`   | An output is no longer produced by its query            |
| `DidDiscard`               | Tracked or memoized data was discarded                  |
| `DidDiscardAccumulated`    | Accumulated auxiliary data was discarded                |
| `DidInternValue`           | A value was newly interned                              |
| `DidReuseInternedValue`    | A stale interned slot was reused                        |

An event callback can be supplied to the storage system, allowing an application to collect execution histories or counters.

These events are useful for determining whether a supposed incremental change actually causes only a limited set of computations to execute.

They also help distinguish expensive recomputation from time spent waiting on concurrent queries.

The event callback itself can introduce overhead, so high-volume instrumentation should be treated separately from production-performance measurements.

### 14.2 Tracing integration

Salsa includes the Rust `tracing` dependency.

The optional `detailed-trace` feature enables additional DEBUG- and TRACE-level events and spans that are disabled by default.

This can integrate with an application's tracing subscriber and existing instrumentation.

The separation between ordinary event callbacks and detailed tracing is useful: event callbacks can provide targeted incremental-system diagnostics, while tracing provides a broader execution-oriented view.

### 14.3 Memory diagnostics

With the `salsa_unstable` feature, Salsa exposes database memory-usage reporting.

The `memory_usage()` facility provides per-ingredient and per-query information, including counts, stored-field sizes, metadata sizes, and page occupancy.

A `heap_size` annotation can be supplied on supported Salsa types and queries to account for heap allocations that are not captured by a simple `size_of` calculation.

This is more informative than only measuring the number of memoized query results, particularly for representations dominated by heap-backed strings, vectors, and IR structures.

Important constraint: Gathering this information requires exclusive database access. It triggers cancellation and waits for other handles to be dropped. Using it while the current thread owns another active database handle can deadlock.

The API is currently gated as unstable rather than guaranteed by a stable 1.0 contract.

### 14.4 Debug formatting and query inspection

Salsa provides database-aware debug formatting through the `debug` macro option and database attachment mechanisms.

The `attach` and `attach_allow_change` functions install a database in thread-local context while executing a closure, allowing generated debug implementations to access database-resident field values.

Salsa also exposes ingredient identifiers, debug names, and related query metadata. These can support application-specific diagnostic tooling, although internal ingredient names and identifiers should not be treated as stable persistent application identifiers.

Sources:

- [https://docs.rs/salsa/0.28.5/salsa/enum.EventKind.html](https://docs.rs/salsa/0.28.5/salsa/enum.EventKind.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/latest/src/salsa/database.rs.html](https://docs.rs/salsa/latest/src/salsa/database.rs.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html?utm_source=chatgpt.com)
- [https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig](https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source

+2

## 15. Optional persistent caching

One of Salsa's less widely documented capabilities is an optional persistence mechanism.

This should be distinguished from the ordinary in-memory memoization that is central to Salsa.

### 15.1 Scope and availability

Persistent caching is enabled through the Cargo feature:

```
salsa = { version = "0.28.5", features = ["persistence"] }
```

The feature is not enabled by default.

Supported Salsa declarations can opt into serialization:

```
#[salsa::input(persist)]
struct SourceFile {
    text: String,
}

#[salsa::tracked(persist)]
fn analyze(db: &dyn salsa::Database, file: SourceFile) -> Analysis {
    // ...
}
```

Corresponding options exist for tracked structs and interned structs.

The implementation uses Serde-based serialization and supports custom serialization and deserialization functions for certain Salsa struct fields.

Persisted queries require suitable serialization implementations for their inputs and outputs.

### 15.2 Database serialization and restoration

The implementation exposes methods on `dyn salsa::Database` that allow the database state to be serialized and subsequently restored through a Serde deserializer.

The upstream tests demonstrate use with `serde_json` and cover persisted inputs, interned values, tracked structs, memoized query results, and dependency-related state.

For example, the tested serialization path uses:

```
let serialized = serde_json::to_string_pretty(
    &<dyn salsa::Database>::as_serialize(&mut db)
)?;
```

The complementary method is `deserialize`, called with a mutable database and a Serde deserializer.

The intent is to restore incremental state so selected previously computed results can be reused rather than recomputed from scratch.

### 15.3 What persistence does not provide automatically

This mechanism should not be confused with a complete production-ready durable cache service.

In particular, it does not itself provide:

- Automatic writing and reading of cache files.
- A documented cross-version storage compatibility guarantee.
- Distributed cache coherence.
- Automatic cache-key namespaces for different application builds.
- Persistent transaction management or crash recovery.
- Automatic validation of external inputs that changed while the process was stopped.

Those concerns remain with the application and its selected storage layer.

Serialization also requires exclusive database access, which can be consequential in a concurrently queried database.

The feature originated as a persistent caching prototype and remains an optional, evolving part of the pre-1.0 API. Its behavior is evidenced by upstream source and tests, but it warrants more caution than the primary in-memory incremental query model.

Architectural significance: Persistence creates the possibility of combining in-process incremental recomputation with reuse across process lifetimes. That is a more ambitious capability than simple serialization of final results, because some incremental dependency information can survive restoration.

Primary sources:

- [https://github.com/salsa-rs/salsa/blob/master/tests/persistence.rs](https://github.com/salsa-rs/salsa/blob/master/tests/persistence.rs?utm_source=chatgpt.com)
- [https://docs.rs/salsa/latest/src/salsa/database.rs.html](https://docs.rs/salsa/latest/src/salsa/database.rs.html?utm_source=chatgpt.com)
- [https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html?utm_source=chatgpt.com)
- [https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig](https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+3

## 16. Ecosystem integration and capability boundaries

Salsa's role is to maintain derived computational state. It is not a general-purpose data storage, language-processing, or execution framework.

That distinction helps clarify where it fits alongside other Rust libraries.

### 16.1 Filesystems, parsers, and language tooling

Salsa does not directly supply file watching, parsing, syntax trees, name resolution, type inference, or Language Server Protocol services.

Those capabilities can be implemented by other libraries and exposed through Salsa inputs and queries.

The official book includes an on-demand input pattern in which an application uses ordinary Rust components, including a concurrent map and filesystem watcher, to discover files lazily and update their Salsa inputs when external changes occur.

A file-watcher notification does not inherently enter Salsa's dependency graph. The application translates the external event into a field update.

This separation is beneficial because it allows Salsa to remain independent of the parser, operating system, editor protocol, or external data source.

Source:

[https://salsa-rs.github.io/salsa/common_patterns/on_demand_inputs.html](https://salsa-rs.github.io/salsa/common_patterns/on_demand_inputs.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://salsa-rs.github.io\&sz=32)

Salsa

### 16.2 Rayon and parallel execution

Salsa's default feature set includes `rayon`.

Rayon can support parallel operations performed by Salsa-based applications, while Salsa supplies shared query memoization and synchronization.

The important boundary is that Rayon is a general-purpose parallel execution framework; Salsa is the incremental dependency and cache engine.

Salsa does not automatically transform every tracked function into a distributed or parallel task.

Parallel task decomposition, synchronization costs, and the relationship between worker lifetimes and database writes still matter.

### 16.3 Serde and persistent storage

Serde integration is principally relevant to the optional persistence feature.

Serde defines serialization and deserialization mechanisms; Salsa contributes knowledge of which database entities and query state can be retained.

An external storage system can supply the durable bytes, but it does not automatically inherit Salsa's revision consistency semantics.

### 16.4 Integration with analytical and relational systems

Salsa can be conceptually valuable alongside databases and analytical engines that store or compute structured information.

For example, a system could expose a changed dataset version as a Salsa input, then derive validation results, query plans, or semantic metadata through tracked computations.

However, Salsa does not incrementally update arbitrary SQL queries, relational operators, DataFrame computations, or database rows automatically.

Its dependency tracking observes Salsa-aware queries and fields. Operations inside an external engine are opaque unless their relevant inputs and outputs are represented through Salsa's tracked abstractions.

This can make Salsa useful as a higher-level incremental orchestration layer while leaving vectorized execution, relational optimization, and durable transactional state to specialized engines.

### 16.5 Rust-analyzer and related query systems

Salsa is closely associated with the architecture of Rust tooling, particularly rust-analyzer.

The rust-analyzer development guide describes Salsa as the foundation of its on-demand incremental analysis model.

There is also a separately published `rust-analyzer-salsa` crate, reflecting the history of Salsa variants and forks in the Rust tooling ecosystem.

It should not be assumed that the published `rust-analyzer-salsa` API is compatible with the upstream `salsa` 0.28 API. They have distinct version histories.

Similarly, Rust's compiler has its own red-green incremental query system; the two share architectural ideas but should not be treated as the same library.

Sources:

- [https://rust-analyzer.github.io/book/contributing/guide.html](https://rust-analyzer.github.io/book/contributing/guide.html?utm_source=chatgpt.com)
- [https://docs.rs/crate/rust-analyzer-salsa/latest](https://docs.rs/crate/rust-analyzer-salsa/latest?utm_source=chatgpt.com)
- [https://rustc-dev-guide.rust-lang.org/queries/salsa.html](https://rustc-dev-guide.rust-lang.org/queries/salsa.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://rust-analyzer.github.io\&sz=32)

rust-analyzer

+2

## 17. Release compatibility, feature flags, and build requirements

### 17.1 Version 0.28.5 characteristics

| Property                       | Verified status                           |
| ------------------------------ | ----------------------------------------- |
| Current stable release         | `0.28.5`                                  |
| Publication date               | September 24, 2026                        |
| Rust edition                   | 2024                                      |
| Minimum supported Rust version | 1.88                                      |
| License                        | Apache-2.0 OR MIT                         |
| Primary crate                  | `salsa`                                   |
| Procedural macro crate         | `salsa-macros`                            |
| Supporting macro crate         | `salsa-macro-rules`                       |
| API stability                  | Pre-1.0; breaking changes remain possible |

The crate metadata still describes Salsa as experimental. That designation should be understood in the context of a substantial, actively maintained library with real use cases, rather than as evidence that the core functionality is merely planned.

### 17.2 Important Cargo features

| Feature          | Default | Role                                                              |
| ---------------- | ------- | ----------------------------------------------------------------- |
| `macros`         | Yes     | Procedural macros used to declare Salsa entities and queries      |
| `inventory`      | Yes     | Automatic ingredient registration                                 |
| `accumulator`    | Yes     | Auxiliary query outputs                                           |
| `rayon`          | Yes     | Parallel-operation support                                        |
| `salsa_unstable` | Yes     | APIs not yet under a stable contract, including memory inspection |
| `detailed-trace` | No      | Additional tracing events and spans                               |
| `persistence`    | No      | Serde-based persistence capabilities                              |
| `shuttle`        | No      | Integration for concurrency testing                               |
| `compact_str`    | No      | Supporting value-trait integrations                               |
| `triomphe`       | No      | Supporting value-trait integrations                               |
| `ordermap`       | No      | Supporting value-trait integrations                               |

The default `salsa_unstable` feature means that simply using the default configuration does not imply that every exposed API carries a mature compatibility guarantee.

The two companion macro crates are closely version-coupled to Salsa. They should not be treated as independent alternatives to the runtime.

Sources:

- [https://docs.rs/crate/salsa/latest/source/Cargo.toml](https://docs.rs/crate/salsa/latest/source/Cargo.toml?utm_source=chatgpt.com)
- [https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig](https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig?utm_source=chatgpt.com)
- [https://docs.rs/crate/salsa-macros/latest](https://docs.rs/crate/salsa-macros/latest?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+1

### 17.3 Important changes in the 0.28 release series

The 0.28 series includes several meaningful changes:

| Release  | Selected changes                                                                                                                           |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `0.28.0` | Replaced `Update` with `SalsaValue` plus `PartialEq`; changed default returns to references; added per-ingredient page-occupancy reporting |
| `0.28.1` | Improved interning behavior and storage-lifetime constraints                                                                               |
| `0.28.2` | Added conditional `SalsaValue` derive proofs and cycle-related fixes                                                                       |
| `0.28.3` | Increased MSRV to Rust 1.88; introduced tracked-query synchronization sharding                                                             |
| `0.28.4` | Fixed builds involving optional interning features                                                                                         |
| `0.28.5` | Corrected a use-after-free vulnerability affecting interned values and cached results                                                      |

The 0.28 migration implications are more substantial than a routine patch update.

In particular, examples written for Salsa 0.23–0.27 may use different equality and return conventions. Much older examples may use a different database/query organization entirely.

The current book and 0.28.5 rustdoc should take precedence over older tutorials or code snippets.

### 17.4 Security advisory

RUSTSEC-2026-0308 describes a use-after-free vulnerability in versions from `0.23.0` up to but excluding `0.28.5`.

The problem concerns stale interned-value storage and cached query results.

The 0.28.5 fix introduces validation assertions that prevent unsafe references from being returned when the relevant storage has not been validated for the current revision.

For this reason, 0.28.5 is the appropriate baseline within the 0.28 series rather than an earlier 0.28 patch.

References:

- [https://rustsec.org/advisories/RUSTSEC-2026-0308.html](https://rustsec.org/advisories/RUSTSEC-2026-0308.html?utm_source=chatgpt.com)
- [https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md](https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://rustsec.org\&sz=32)

rustsec.org

+1

## 18. Architectural synthesis: what Salsa enables

The main value of Salsa emerges from combining its mechanisms rather than using each one independently.

### 18.1 Incrementally maintained domain models

Input entities can represent externally supplied state. Tracked queries transform that state into derived entities and properties. Interned structs canonicalize shared immutable components. Accumulators preserve diagnostics independently of principal outputs.

Together, these provide much of the infrastructure needed for an incrementally maintained semantic model.

An application still defines the meaning of the entities and their relationships, but it need not independently implement most of the change-propagation and memo-validation machinery.

### 18.2 Stable entities with selectively changing properties

Tracked-entity identity can survive changes to nonidentity fields.

When combined with field-level dependencies and backdating, this enables localized changes in an intermediate representation without invalidating every computation that refers to the entity.

That is substantially more expressive than caching whole analysis results under a document identifier.

### 18.3 Mixed batch and demand-driven evaluation

`specify` permits one family of results to be generated in bulk and then exposed through individually addressable tracked queries.

This is important because efficient algorithms do not always naturally compute one result at a time.

Salsa can accommodate an externally chosen batch algorithm without requiring all downstream consumers to adopt a batch-only interface.

### 18.4 Incremental recursive analysis

Cycle handling allows selected recursive analyses to participate in the same tracked query model.

For monotone analyses, fixed-point iteration can replace bespoke recursion bookkeeping and provisional memo management.

The domain-specific convergence strategy remains the caller's responsibility.

### 18.5 Low-overhead reuse at several levels

Salsa offers multiple, complementary ways to reduce work:

| Layer                | Mechanism            | Work potentially avoided                                   |
| -------------------- | -------------------- | ---------------------------------------------------------- |
| Query execution      | Memo validation      | Re-running an unchanged computation                        |
| Change propagation   | Backdating           | Re-running dependent computations                          |
| Validation           | Durability           | Traversing unaffected dependency subgraphs                 |
| Value representation | Interning            | Repeated allocations and expensive value comparisons       |
| Entity lifecycle     | Tracked identities   | Rebuilding downstream structures after nonidentity changes |
| Computation strategy | `specify`            | Redundant per-entity work already performed in a batch     |
| Memory retention     | LRU and reclamation  | Keeping infrequently reused data resident                  |
| Process lifetime     | Optional persistence | Rebuilding selected incremental state from scratch         |

These opportunities are conditional. Some require additional metadata, hashing, equality checks, synchronization, or memory retention. Their aggregate benefit depends on the workload's change patterns and query granularity.

### 18.6 What Salsa does not replace

Salsa is not a general replacement for:

- Transactional databases or durable object stores.
- A mutable entity-component system.
- A general-purpose dependency-injection framework.
- An asynchronous execution runtime.
- A parallel task scheduler.
- A parser, compiler, symbolic mathematics engine, or relational query engine.
- Arbitrary side-effect management.
- Automatic incrementalization of opaque external computations.

Its strongest architectural role is as the incremental state and computation kernel surrounding domain-specific logic, not as the implementation of that logic.

## 19. Primary-source reference map

The following resources provide the most useful paths into Salsa's architecture and advanced features. All URLs are written in full.

| Topic                                       | Primary source                                                                                                                                                          |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Release-specific API reference              | [https://docs.rs/salsa/0.28.5/salsa/](https://docs.rs/salsa/0.28.5/salsa/?utm_source=chatgpt.com)                                                                       |
| Conceptual overview and constructs          | [https://salsa-rs.github.io/salsa/overview.html](https://salsa-rs.github.io/salsa/overview.html?utm_source=chatgpt.com)                                                 |
| Query engine and red-green algorithm        | [https://salsa-rs.github.io/salsa/reference/algorithm.html](https://salsa-rs.github.io/salsa/reference/algorithm.html?utm_source=chatgpt.com)                           |
| Tracked structs and tracked functions       | [https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html](https://docs.rs/salsa/0.28.5/salsa/attr.tracked.html?utm_source=chatgpt.com)                                     |
| Input structs                               | [https://docs.rs/salsa/0.28.5/salsa/attr.input.html](https://docs.rs/salsa/0.28.5/salsa/attr.input.html?utm_source=chatgpt.com)                                         |
| Interned structs                            | [https://docs.rs/salsa/0.28.5/salsa/attr.interned.html](https://docs.rs/salsa/0.28.5/salsa/attr.interned.html?utm_source=chatgpt.com)                                   |
| Durability                                  | [https://salsa-rs.github.io/salsa/reference/durability.html](https://salsa-rs.github.io/salsa/reference/durability.html?utm_source=chatgpt.com)                         |
| Cycle recovery                              | [https://salsa-rs.github.io/salsa/cycles.html](https://salsa-rs.github.io/salsa/cycles.html?utm_source=chatgpt.com)                                                     |
| Accumulator-based diagnostics               | [https://salsa-rs.github.io/salsa/tutorial/accumulators.html](https://salsa-rs.github.io/salsa/tutorial/accumulators.html?utm_source=chatgpt.com)                       |
| Database lifetime and safety                | [https://salsa-rs.github.io/salsa/plumbing/db_lifetime.html](https://salsa-rs.github.io/salsa/plumbing/db_lifetime.html?utm_source=chatgpt.com)                         |
| Tracked entity internals                    | [https://salsa-rs.github.io/salsa/plumbing/tracked_structs.html](https://salsa-rs.github.io/salsa/plumbing/tracked_structs.html?utm_source=chatgpt.com)                 |
| Database and runtime architecture           | [https://salsa-rs.github.io/salsa/plumbing/database_and_runtime.html](https://salsa-rs.github.io/salsa/plumbing/database_and_runtime.html?utm_source=chatgpt.com)       |
| Memory and cache tuning                     | [https://salsa-rs.github.io/salsa/tuning.html](https://salsa-rs.github.io/salsa/tuning.html?utm_source=chatgpt.com)                                                     |
| Lazy input discovery                        | [https://salsa-rs.github.io/salsa/common_patterns/on_demand_inputs.html](https://salsa-rs.github.io/salsa/common_patterns/on_demand_inputs.html?utm_source=chatgpt.com) |
| Database and cancellation APIs              | [https://docs.rs/salsa/0.28.5/salsa/trait.Database.html](https://docs.rs/salsa/0.28.5/salsa/trait.Database.html?utm_source=chatgpt.com)                                 |
| Persistent caching implementation and tests | [https://github.com/salsa-rs/salsa/blob/master/tests/persistence.rs](https://github.com/salsa-rs/salsa/blob/master/tests/persistence.rs?utm_source=chatgpt.com)         |
| Cargo features and dependencies             | [https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig](https://docs.rs/crate/salsa/latest/source/Cargo.toml.orig?utm_source=chatgpt.com)                           |
| Release history                             | [https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md](https://github.com/salsa-rs/salsa/blob/master/CHANGELOG.md?utm_source=chatgpt.com)                         |
| Security advisory                           | [https://rustsec.org/advisories/RUSTSEC-2026-0308.html](https://rustsec.org/advisories/RUSTSEC-2026-0308.html?utm_source=chatgpt.com)                                   |
| Source repository                           | [https://github.com/salsa-rs/salsa](https://github.com/salsa-rs/salsa?utm_source=chatgpt.com)                                                                           |

## Conclusion

Salsa's distinctive capability is to provide a general-purpose, type-aware incremental computation substrate in Rust.

It combines dependency discovery, revision validation, result equality, entity identity, structural interning, recursive computation support, diagnostics, cache retention, and concurrent query coordination in one coherent system.

The most consequential architectural distinction is that Salsa manages not only cached computations but also the identities and lifecycles of many of the derived entities participating in those computations.

That combination can eliminate substantial bespoke machinery in systems that repeatedly compute interconnected derived information as their inputs evolve.

The principal limitations are its dependence on deterministic Salsa-visible computations, the ownership and lifetime constraints of database-resident values, memory retained by incremental history, and a concurrency model in which writes must coordinate with and cancel active readers.

For workloads fitting those constraints, Salsa 0.28.5 offers a sophisticated foundation for constructing large, incrementally maintained computational models without implementing a custom dependency graph, cache invalidation engine, and derived-state lifecycle manager.