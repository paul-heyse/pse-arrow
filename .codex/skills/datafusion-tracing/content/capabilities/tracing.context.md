# Propagate async span and subscriber context

Async instrumentation scopes context around polling. Span context and subscriber dispatch are separate parts of a composition, especially across spawned work.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| Instrument::instrument | A future belongs to a particular span | The wrapper enters that span as the future is polled. |
| WithSubscriber | A future must use an explicit dispatcher | Carries subscriber context; does not choose the intended parent span. |
| synchronous in_scope | Work completes synchronously | An entered guard held across await can misattribute concurrent work. |

## Contract

**Async boundary.** The pinned Instrument docs describe entering the span whenever the future is polled. Spawn boundaries require verifying the actual propagated context.
Claim `tracing.context.1`; source_observation; evidence: source.

**Executed scope.** A spawned future carrying both the parent span and subscriber dispatch produces the expected parent ID. A separate unparented control does not acquire that parent.
Claim `tracing.context.observed`; runtime_observation; evidence: consumer.

## Implementation

- Create the intended parent span before moving work; instrument the future and preserve the dispatcher where needed.

## Limits and unknowns

- No blanket inheritance guarantee for all executors or spawn APIs.

## Exact contracts

- [`tracing::instrument::Instrument`](../operations/tracing.instrument.Instrument.md#op-f3255b14b518b17fdbec7041) — `trait Instrument: Sized`
- [`tracing_futures::WithSubscriber`](../operations/tracing_futures.WithSubscriber.md#op-c39af7ca2f96032e131f7149) — `trait WithSubscriber: Sized`
- [`tracing::span::Span`](../operations/tracing.span.Span.md#op-1283b08586edf8ae3649f1c6) — `struct Span`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/api/tracing.instrument.md): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): A spawned future carrying both the parent span and subscriber dispatch produces the expected parent ID. A separate unparented control does not acquire that parent.
  Tests: spawned_future_propagates_parent_and_dispatch
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
