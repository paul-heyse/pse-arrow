# Previewing partial results

A positive preview_limit enables preview recording; zero disables it. Omitted preview_fn selects the default pretty formatter. A custom formatter changes presentation. Available partition previews are assembled and capped when the shared recorder is released.

## Entry points

| Item | Visibility | Methods | Reached via |
|---|---|---:|---|
| [`pretty_format_compact_batch`](../api/datafusion_tracing.preview_utils.md) | supported | 0 | — |

## Spans it emits

| Span | Target | Level | Fields | Evidence |
|---|---|---|---:|---|
| `InstrumentedExec` | `integration_utils` | INFO | 49 | recorded |

## What this seam cannot tell you

- What a preview will look like for your data. `index/previews.tsv` holds 35 captured renders; the widths and wrapping are a function of the arguments you pass, not of the library.
- A complete or random sample, or a byte bound. Partition previews and retained Arrow buffers are distinct from the final displayed row cap.
- The writable name of `preview_fn`'s type. `PreviewFn` is a type alias in a private module and rustdoc expands it away; pass a closure and let inference do it.

## Read next

- [`catalogs/spans.md`](../catalogs/spans.md)
- [`content/corpus/source/datafusion-tracing/preview.rs`](../corpus/source/datafusion-tracing/preview.rs) — upstream, verbatim
- [`content/corpus/source/datafusion-tracing/preview_utils.rs`](../corpus/source/datafusion-tracing/preview_utils.rs) — upstream, verbatim
