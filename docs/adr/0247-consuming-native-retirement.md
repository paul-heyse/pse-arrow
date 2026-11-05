---
id: ADR-0247
title: Finish native provider owners before releasing custody
status: proposed
date: 2026-11-05
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, DP-24, PS-13]
blueprint: [§9.11]
review: "not-required: consuming kernel retirement contract within the authorized Plan65 E04/D05/F02 target; no D1-D14 change"
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A required provider or native adapter cannot expose physical retirement through its actual owner without a separate custody owner.
verification: Provider-bank multi-failure controls; owner-affine retained callback replacement/reset/final finish; scratch publication after successful finish; real worker acknowledgement/reap/I-O fault combinations and cancellation with original admission retained.
---

# ADR-0247: Finish native provider owners before releasing custody

## Context

ADR-0242 separates handle retirement, worker Shutdown and physical settlement. Production
provider ownership also passes through CaseWorker, retained native callbacks, conditional-unit
evaluators and Horizon dynamic workers. Dropping a binding or closing only NativeSession
cannot report every original provider/process/I/O outcome.

## Scope

Propose consuming retirement at these existing kernel/native ownership boundaries, amending
blueprint §9.11 within the selected local-worker target. Implementation and qualification remain
with Plan65 E04/D05/F02. No protocol observation becomes a retirement grant.

## Drivers

A completed solve must preserve its original scientific result alongside independent cleanup
failures, and admission must survive unknown physical settlement. Native callback aliases,
reusable scratch and operation-owned workflow state must retire on their actual native thread.

## Options

Retaining Drop-only cleanup loses failures. Adding only Remote::finish or NativeSession::close
misses providers erased behind adapter states and operation-owned evaluators. A separate
retirement registry duplicates existing custody. Select explicit consuming finish through
the actual ownership chain, using library transport/protocol machinery where it exposes the
required results.

## Outcome

Provider, ProviderBinding, ProviderBindings and CaseWorker expose consuming, result-bearing
finish. Owning wrappers forward it; borrowed callbacks release their alias and never retire
the shared evaluator. Provider banks attempt every independent retirement with keyed typed
causes. A returned result establishes physical retirement, including settled failure;
unreaped work remains held by the original performing finish/supervisor operation.

Retained adapter/workflow states carry a monomorphized finisher for their exact constructed
type. Replacement, clear, reset and final departure invoke it without discovering owners
through guessed downcasts. Adapter-only clear preserves its existing scope. Scratch publication
moves from normal Drop to successful explicit finish. Native scope/session/lane joins retain
original scientific, panic and independent teardown failures. Drop remains abandonment
containment.

Failed construction consumes the same obligation: preserve the original typed refusal and
finish every already constructed supplier before returning. Generic mathematical suppliers
retain their concrete error type in a shared construction-failure product. Distinguish the
non-cloneable performing root owner from its borrowed native callback; an idle callback alias
must not keep actual providers alive after owner finish. Native aliases are dismantled before
extracting and finishing the actual evaluator.

Each independent retirement isolates unwind, including destruction of a caught `panic_any`
payload. Dispose resulting payload owners under the same containment before visiting siblings;
neither a leaked payload nor an escaped destructor panic establishes settled retirement.

### Consequences

Migrate the concrete algebraic, dynamic, conditional-unit and Horizon owners, native oracle
wrappers and all adapter replacement/failure paths together. No arbitrary consuming callback
may silently erase the owned worker's finish obligation.

### Compensating controls

Keep generated ConnectRPC clients/codecs. Its 0.9.1 shared client detaches workers/drivers;
where this hides required results, use public ClientTransport over explicitly owned Hyper H2
SendRequest/driver state in the existing I/O owner. Retain library-exposed inner connection
errors and outer join results. Hyper 1.11.1 internally logs/erases some HTTP/2 connection
errors; executor completion cannot reconstruct an error the library does not expose.
After actual native Shutdown, the server initiates Hyper's graceful connection drain. The
parent retains its sender through driver completion only for that requested and decoded
Shutdown; other cancellation/startup paths retain abrupt departure. This avoids making
last-sender drop race the server's final writes. Original server connection errors remain typed.
Shutdown acknowledgement, child reap and I/O completion remain distinct;
failed attempts do not skip independent teardown or release uncertain custody.

### Confirmation

Targeted tests challenge multiple provider failures, shared versus owned callbacks, failed
construction/replacement, reset, pending method guards, absent acknowledgements, reap/I/O
failures and scratch reuse. Assembled real-process qualification and support claims stay in
Plan65h. The consuming interfaces and migrated owners are integrated on main; Plan65e owns
targeted acceptance and its remaining scope. This record remains Proposed; source integration
and component controls do not establish formal acceptance or assembled support.

## Pros and cons

Explicit ownership makes retirement reachable and attributable; migration spans actual native
holders and must preserve their alias and neutral-reuse contracts. It adds no second owner
registry, protocol stack or universal workflow model.

## More information

- [Plan65e production finish ownership](../plans/65e-scientific-portability-and-execution.md#production-finish-ownership--execution-refinement)
- [ADR-0240](0240-prepared-local-scientific-workers.md), [ADR-0242](0242-scientific-worker-handle-and-shutdown-scope.md)
- [ConnectRPC 0.9.1 connection ownership](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/http2.rs), [ClientTransport](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs)
- [Hyper 1.11.1 H2 client](https://docs.rs/hyper/1.11.1/hyper/client/conn/http2/index.html)

## Status history

- 2026-11-05 — proposed before kernel/native retirement activation under authorized Plan65 execution.
