# Mathematics, solvers and scientific workers

Inspected 2026-11-03 at `f55951cfe995a689f9b75185b8e89cb362021616`, with the supplied
uncommitted Proto reference additions. Source interpretations and selected library APIs are
**Interface-checked**; the target composition is **Proposed**. No scientific execution,
native ABI qualification or performance measurement was performed. This account uses core
3.4, efficient-architecture heuristics 1.0 and process-simulator profile 1.5. It excludes
SurrealDB v4 and assumes a local target with future extension seams.

## Target: portable scientific contracts around a native numerical core

Use generated, inspectable scientific contracts for admitted source exchange, provider
capabilities, selected preparation inputs, execution requests and truthful outcomes. Keep
library-owned mathematical objects, callback workspaces and solver kernels in their useful
native representations. A representation is selected for the complete operation: portability,
inspection and isolation justify encoding at some boundaries; repeated evaluator calls often
justify direct native access.

The target is broader than Plan64's current water worker. It lets a new physical provider,
reaction implementation, dynamic formulation or solver class publish its precise supported
contract without teaching every client another ad hoc format. It does not claim that a generic
message grants numerical capability, serializes arbitrary executable closures or admits an
otherwise unsupported formulation.

## Current routes and the actual opportunity

| Route and source owner | Meaning that the target must carry or leave native |
| --- | --- |
| Authored definitions → compiler → `pse-math` → class-specific `pse-backend-native` | Original physical conventions, admitted domains, derivative meaning, specialization dependencies, native build/profile, original-coordinate screening and distinct termination/qualification facts. Blueprint §7/§14–§18 own these contracts. |
| [`pse-kernels`](../../../../../crates/pse-kernels/src/lib.rs) | `ProviderSpec`, `ProviderRequest`, `PreparedProviderRequest`, `ProviderDemand`, `ProviderRuntime`, `ProviderLinearizationToken` and `ProviderBuffer` distinguish ordered products, exact derivative actions, attempt-local state and requested extents. `ExecutionScope` retains the original optional absolute deadline and cancellation owner. |
| [`scientific_worker/proxy.rs`](../../../../../crates/pse-runtime/src/scientific_worker/proxy.rs) | `LocalWaterFactory` launches an explicitly prepared implementation. Preparation binds exact outputs/order/demand to a handle. `fill` transmits inputs, directions, cotangents and weights, then checks handle, invocation, extents and finiteness before copying results. This route already incurs per-evaluation vector construction; a faster codec alone does not remove it. |
| [`scientific_worker/transport.rs`](../../../../../crates/pse-runtime/src/scientific_worker/transport.rs), [`process.rs`](../../../../../crates/pse-runtime/src/scientific_worker/process.rs) | Authenticated Unix HTTP/2 ConnectRPC and child process custody are distinct owners. `ProcessCustody::drain` and `settle_failure` establish actual child settlement; a completed RPC is insufficient. Reconnect cannot silently substitute another provider instance. |
| [`math/native_child.rs`](../../../../../crates/pse-runtime/src/math/native_child.rs) | `PortableRootSource` reconstructs a closed provider-free authored implicit-root problem for one KINSOL child, using a bounded private Serde protocol. `PreparedSolve` and opaque provider closures are deliberately unsupported. This is a concrete non-Proto execution-placement boundary to compare, not evidence that every solve is already portable. |
| [`reference_exchange.rs`](../../../../../crates/pse-runtime/src/reference_exchange.rs) | Generated source-derived Buffa messages have a checked native interpretation and separately retained original message. This is an existing reusable pattern for portable scientific records, not a proposed first serialization mechanism. |

The opportunity is to make these portable meanings explicit once and reusable across
preparation, execution, retained diagnostics and independent references. Internal numerical
types need not become wire-shaped to accomplish it. Conversely, internal preparation records
are eligible for Proto when persistence or independent inspection provides concrete value;
the investigation is not restricted to public RPC.

## Precise capability composition

**1. Generate scientific records from their semantic owners.** Extend the owner-derived
projection route rather than hand-maintaining a second provider or solver vocabulary.
Descriptors and custom options should expose physical quantity identities, ordered ports,
supported derivative forms, formulation/profile identity and available result products.
Opaque mathematical artifacts must retain their producer/interpreter identity and admissible
consumer context. Descriptor identity is structural; it cannot replace semantic dependency
keys or native implementation/build evidence. The source-ownership alternatives and exact
generation mechanisms are assessed in [contract ownership](contract-ownership-and-generation.md).

**2. Use Buffa representations according to lifetime.** `MessageView::decode_view` can read
borrowed strings/bytes; `OwnedView` retains the backing `Bytes` across async lifetimes;
`ViewEncode` avoids constructing owned fields merely to serialize them. Buffa 0.9.2 also
provides bounded encoding and reusable size-cache storage. `Rope::with_backing` can preserve
eligible slices, `into_segments` yields ordered `Bytes`, and `to_contiguous_bytes` explicitly
flattens them. Those mechanisms matter for source/artifact and large diagnostic transport;
they do not automatically remove the scientific worker's `inputs.to_vec()` or the native
solver's required numerical buffers. Retaining a small slice can retain a large backing
allocation. See [representation](representation-reflection-and-validation.md) for exact
encoding conditions and [Buffa's tagged sink source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs).

**3. Separate generated constraints from scientific admission.** `buf.validate` annotations
and generated `Validate` implementations can discharge local required alternatives, shape
and bounded-field rules. Resolve units, material/phase conventions, property validity,
formulation smoothness and solver eligibility with their existing scientific owners.
Unsupported CEL must remain a refusal or explicit coverage gap. A message claiming a Hessian
capability is not evidence of an actual derivative implementation. The maintained
[`protoc-gen-protovalidate-buffa`](../../../../../vendor/protoc-gen-protovalidate-buffa/README.md)
contract, rather than an upstream conformance count, bounds local generated coverage.

**4. Make an isolation boundary represent one useful scientific operation.** Reuse generated
ConnectRPC services and the prepared provider handle where an implementation benefits from a
separate process or foreign runtime. Preserve exact provider/configuration identity, demand,
input context, invocation, rejection class and output extents. Add a batched evaluation only
where the provider can promise coherent independent states or an explicitly ordered stateful
operation; a bag of scalar calls is not automatically a valid batch. Prepared linearizations
and derivative tokens need explicit session ownership and invalidation. Neither a connection
cache nor a descriptor pool owns that state. Tagged [ConnectRPC client source](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs)
supplies transport reuse; application handles supply scientific continuity.

**5. Keep native solve outcomes richer than transport status.** Record backend termination,
candidate presence, numerical convergence, physical closure, approximation/validity scope and
final usability separately. Send a typed scientific failure or outcome record; use transport
errors for failure to perform/observe the operation. Diagnostic projections must retain
original statuses, affected source identities and derivative/scale context. See
[diagnostics and results](diagnostics-and-results.md).

## Complete alternatives and judgment

| Alternative | Complete operating consequence | Judgment |
| --- | --- | --- |
| Preserve separate native/private/export representations everywhere | Least immediate conversion change, but each portable source/provider/result extension repeats adapter and interpretation work. | Retain native hot paths; replace repeated portable interpretation where source-derived generation can cover the whole consumer population. |
| Universal Proto mathematical IR and mandatory solver RPC | Uniform-looking records but new algebraic ownership, decoding, preparation and lifecycle work; library-native evaluator and solver capabilities must be reconstructed. | Do not select. No inspected consumer establishes the value of moving all algebra and callbacks into this representation. Reopen for a concrete otherwise valuable portable computation backend. |
| Owner-derived portable contracts plus direct and explicitly isolated execution | One scientific meaning can have generated exchange/inspection while native libraries keep efficient internal operations. New worker families implement an admitted capability slice. | Recommended proposed target. Compare and remove each superseded private mapping together with its consumers; do not retain two independently editable definitions. |

The private KINSOL child is a candidate for source-derived portable request/outcome generation,
not a reason to require an engineering service for every solve. Adopt that replacement only
when it covers reconstruction, limits, original screening and child settlement with less total
machinery than the current private framing. A local service is justified by useful independent
consumers or isolation, not by the availability of a generator.

## Extension and failure controls

A new derivative-capable provider should add its semantic capability and implementation once,
derive inspection/client records, and refuse unsupported actions before numerical execution.
A new solver family should extend its problem-class admission and native outcome mapping;
generic clients should inspect the added metadata without guessing capability from an enum name.
Dynamic/spatial products must retain time/axis/layout and approximation scope instead of being
flattened into an unexplained repeated-double field.

Discriminating controls are source/admission → wire → independent decoder → native adoption
with changed datum/precision/order, stale prepared handle, wrong invocation, truncated or
nonfinite derivative arrays, lost response after work completion, and worker death during a
native call. Compare direct and isolated execution over preparation, repeated evaluation and
drain. Any performance claim must include vector construction, validation, encoding, runtime
bridges, synchronization and disposal, not just wire throughput.

## Sources and remaining limits

The inspected source owners above supply the local mechanisms. The supplied
[Local RPC addendum](../../../../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md),
[Buffa addendum](../../../../libraries/proto/Buffa.md),
[ConnectRPC](../../../../libraries/proto/ConnectRPC_for_Rust.md) and
[validation](../../../../libraries/proto/protobuf_validation_rust.md) broaden capability discovery.
Context7 Buffa results were cross-checked against tagged 0.9.2 material rather than treating
`main` as the installed contract. Current manifest realizations are Buffa 0.9.2, ConnectRPC
0.9.1, Protovalidate-Buffa 0.7.2 and Tonic 0.14.6. Native solver contracts were interpreted with
the active `native-solver-libraries` skill; no native release or ABI move is proposed here.

The general provider batching contract, portable compiled-artifact population and replacement
of the private root protocol remain conditional implementation choices requiring their named
consumer controls. This account establishes a target direction, not support for arbitrary
remote providers, distributed solving, universal executable serialization or additional science.
