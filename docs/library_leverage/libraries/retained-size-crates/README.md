# Retained-size crates (get-size2, memuse, datasize, deepsize)

This file covers one native capability family: deriving the retained heap size of Rust values.

Realizations considered (none is in the workspace lockfile):
- `get-size2` 0.11.0. The source is in the local Cargo registry from another project. The latest
  release is 0.11.1 (2026-10-10).
- `memuse` 0.2.2 (2024-12-13).
- `datasize` 0.2.15 (2023-09-11).
- `deepsize` 0.2.0 (2020-10-27).

Every route would be a new dependency at an exact pin.

Research date 2026-10-27. Sources:
- `get-size2-0.11.0/src/{lib.rs, tracker.rs, impls/alloc_impls/collections.rs,
  impls/std_impls/collections.rs, impls/alloc_impls/ownership.rs}` from the local Cargo registry.
- docs.rs pages for `memuse::DynamicUsage` 0.2.2 and `datasize::DataSize` 0.2.15.
- crates.io API for release dates.

`deepsize` was assessed by release metadata only.

## Types

### retained-size/derived-heap-measure
- **Need it serves:** one declaration of "bytes this value holds", derived from type definitions
  instead of hand-written `heap_bytes()` per struct.
- **Recognizable code shapes:**
  - A local `HeapUsage` or `retained_bytes()` trait implemented by hand on many structs.
  - Repeated `capacity() * size_of::<T>()` sums.
- **Mechanisms and APIs:**
  - **get-size2:** `#[derive(GetSize)]` provides `get_heap_size()`, `get_size()` and
    `get_heap_size_with_tracker`. `StandardTracker` deduplicates shared `Arc`/`Rc` payloads.
    Field attributes (`#[get_size(ignore)]`, `size_fn`) handle exceptions.
  - **memuse:** `DynamicUsage::{dynamic_usage(), dynamic_usage_bounds() -> (usize,
    Option<usize>)}`. The lower bound is exact; the upper bound is present only when a property
    of the type guarantees it.
  - **datasize:** `DataSize::{IS_DYNAMIC, STATIC_HEAP_SIZE, estimate_heap_size()}`, plus a
    `detailed` feature for per-field trees. `STATIC_HEAP_SIZE` is a compile-time constant for
    fixed-heap types.
- **Obligations discharged:** a measure, recomputed by derive, that follows field changes in the
  type declaration.
- **Residual caller obligations:**
  - prediction before a value exists;
  - collection overhead conventions;
  - third-party types without impls, such as petgraph, faer and Symbolica atoms, which need
    wrappers or manual impls;
  - deciding the policy for shared ownership.
- **Semantic conditions:** these crates measure an existing value after the fact; none of them
  predicts. The direction of error differs:
  - **get-size2 0.11.0, `BTreeMap`:** sums the key and value sizes per entry, with no node or
    edge overhead. It under-counts relative to a node-geometry bound.
  - **get-size2 0.11.0, `HashMap`:** counts `capacity() × size_of::<(K, V)>()` with no control
    bytes. It under-counts slightly.
  - **get-size2 0.11.0, `Vec`:** counts capacity.
  - **memuse:** documents `HashMap` as a "best estimate" and states no `HashMap` upper bound.
    docs.rs lists no `BTreeMap` or `Arc` implementation.
  - **datasize:** describes itself as an approximation. Its static figures are lower bounds
    for dynamic types.
- **Version / build conditions:**
  - get-size2 is actively released.
  - memuse, datasize and deepsize are dormant: last releases in 2024, 2023 and 2020.
- **Limits and negative results:**
  - No inspected crate guarantees a conservative upper bound for tree or hash containers.
  - get-size2 cannot host a conservative container policy globally (added 2026-10-27): the crate
    owns both the trait and the std container impls, so the orphan rule blocks an application
    from overriding the `BTreeMap` or `HashMap` impl. The only escape is a per-field `size_fn`,
    and a field that omits it under-counts silently.
  - None provides a pre-allocation estimate for a container of n entries, although
    `STATIC_HEAP_SIZE` covers fixed-heap types.
- **Evidence:** Interface-checked:
  - get-size2: source read;
  - memuse and datasize: docs.rs pages;
  - deepsize: release metadata only.

## Current applicability

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) (part) — Retained-size derive for owned structs
- **Types:** retained-size/derived-heap-measure.
- **Realization:** upgrade route as new dependencies, for example `get-size2` =0.11.1. None is
  installed in the workspace.
- **Conclusion:** fits with conditions, and only for the *retained-bytes measurement* role. It is
  ruled out for prepaid admission.
- **What it would discharge / what remains local:**
  - *Could discharge:* hand-written `heap_bytes()` and `retained_bytes()` bodies for plain owned
    structs. These are the measurements that feed Salsa `heap_size` and retention reporting.
  - *Remains local:*
    - every `*_allocation_bound` made before construction;
    - std BTree node geometry;
    - library workspaces;
    - Arc-sharing policy.
- **Prerequisites, configuration or extension:**
  - A custom `size_fn` or wrapper for third-party types.
  - A policy override that adds node overhead to `BTreeMap` where the conservative direction
    matters.
- **Semantic gaps against the CBO's preservation constraints:**
  - The default `BTreeMap` and `HashMap` impls err low, against "conservative direction of
    error".
  - Measurement is after the fact, against "reservation before allocation".
  - Counting traverses the whole value graph. It does not allocate, apart from the tracker's
    dedupe set when `StandardTracker` is used.
- **Unsettled premises and what would settle them:**
  - How many of the roughly 260 local `HeapUsage` impls are plain owned structs and how many are
    deliberate bounds. A census of `impl HeapUsage` bodies would settle this.
  - Whether the existing `heap_bytes` generator already covers the derivable share. Reading that
    generator would settle this.
- **Evidence:** Interface-checked; Proposed.

**Reviewer correction (2026-10-27).** The census differs from the count above: there are about
771 `HeapUsage` impls in 214 files, 204 of them registry-generated; the hand-written residue is
about 205 `fn retained_bytes` and about 465 `fn *_bytes(` outside generated code (rg counts). A
derive for hand-written measures would have to coexist with the generator rather than add a
second derive over registry types. Salsa consumes the local measure, so there are not two
accounting systems. Because get-size2 cannot override its std container impls (see the type), the
conservative B-tree policy would have to be repeated per field; the reviewers suggest instead one
conservative B-tree extent helper beside the local trait, since the math crates use a different
B-tree formula at each of at least four sites. The derive sub-question remains unresolved pending
a census of the hand-written bodies. See the [comparative review](../../reviews/result-data-and-prepaid-accounting.md#cbo-001).
