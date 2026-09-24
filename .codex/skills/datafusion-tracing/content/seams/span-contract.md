# The emitted span contract

The tables summarize upstream snapshot observations, including instrumentation and harness spans. They are not a complete list of possible fields or behavior.

## Spans it emits

| Span | Target | Level | Fields | Evidence |
|---|---|---|---:|---|
| `InstrumentedExec` | `integration_utils` | INFO | 49 | recorded |
| `Phase` | `integration_utils` | INFO | 4 | recorded |
| `Rule` | `integration_utils` | INFO | 1 | recorded |
| `get_opts` | `instrumented_object_store::instrumented_object_store` | INFO | 5 | recorded |
| `get_ranges` | `instrumented_object_store::instrumented_object_store` | INFO | 4 | recorded |

## What this seam cannot tell you

- Completeness. Aggregate snapshot rows remain recorded; assertion-level probe evidence is kept separately.
- Tell you the target your spans will carry. It is your crate's module path unless you pass `target:`.
- Spans from scenarios upstream does not test. `07_scrabble_all_options` sets `ignore_full_trace()` because its ordering is not deterministic, so it contributes no rows at all.

## Read next

- [`catalogs/spans.md`](../catalogs/spans.md)
