# Declare and record custom span fields

A custom field value and its declaration are separate inputs. Tracing fixes a span's field set at creation.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| macro field plus options value | Custom metadata belongs on instrumented node spans | The key must exist in the macro-created field set. |
| parent span field | Metadata belongs on the parent operation | Backend presentation of parent attributes is distinct from copying them to children. |

## Contract

**Field schema.** add_custom_field inserts a value into the options map; it does not add a new field to a span after creation.
Claim `tracing.fields.1`; source_observation; evidence: source.

**Construction.** InstrumentationOptions also exposes custom_fields publicly; a struct literal can supply a map. The builder is a convenience, not the only way to configure values.
Claim `tracing.fields.2`; source_observation; evidence: source.

**Executed scope.** The declared query_id field receives its configured value; an undeclared key is absent on the execution spans. The comparison also isolates metrics enablement.
Claim `tracing.fields.observed`; runtime_observation; evidence: consumer.

## Implementation

- Declare env = tracing::field::Empty in the macro and set env in the options.
- Distinguish span metadata name/target from exported otel.name and other attributes.

## Limits and unknowns

- No assumption that arbitrary dynamic keys are exportable or inherited by child spans.

## Exact contracts

- [`datafusion_tracing::options::InstrumentationOptions`](../operations/datafusion_tracing.options.InstrumentationOptions.md#op-07a1777b923d8d00bd1e5318) — `struct InstrumentationOptions`
- [`datafusion_tracing::options::InstrumentationOptions`](../operations/datafusion_tracing.options.InstrumentationOptions.md#op-e8a1e87352fc8239e55d0d93) — `struct InstrumentationOptions`
- [`datafusion_tracing::options::InstrumentationOptionsBuilder::add_custom_field`](../operations/datafusion_tracing.options.InstrumentationOptionsBuilder.md#op-d0d5bef2332c095ef38fac56) — `fn add_custom_field<K: Into<String>, V: Into<String>>(self, key: K, value: V) -> Self`
- [`tracing::span::Span`](../operations/tracing.span.Span.md#op-1283b08586edf8ae3649f1c6) — `struct Span`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/datafusion-tracing/options.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): The declared query_id field receives its configured value; an undeclared key is absent on the execution spans. The comparison also isolates metrics enablement.
  Tests: metrics_and_custom_fields_have_separate_controls
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
