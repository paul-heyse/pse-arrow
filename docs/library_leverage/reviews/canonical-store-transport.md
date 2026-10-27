# Canonical store transport and SurrealQL

**Group G3 of the [comparative review](README.md), 2026-10-27.** This page gives focused
comparative advice under the leverage-opportunity-review skill. It authorizes nothing and is not a
formal design review. Every target on this page is *Proposed*.

| Item | Value |
| --- | --- |
| Reassessment | 2026-10-27 at source `4f52fd239`; consumes the complete original review and updated inputs at `b29fcf802`, base patterns 0.2 and additional patterns revision 3 (refined to 4 here). All named dispositions were reconsidered with representative current source, consumers and inherited evidence; unchanged evidence retains its original conditions. |
| Opportunities | [CBO-009](../codebase/persistence-and-transport.md#cbo-009), [CBO-010](../codebase/persistence-and-transport.md#cbo-010) (SurrealQL side), [CBO-016](../codebase/execution-lifecycle-and-observation.md#cbo-016) (store side), [CBO-046](../codebase/registry-generation-and-python-boundary.md#cbo-046) (SurrealQL side), [CBO-048](../codebase/developer-tooling.md#cbo-048) (SurrealDB parts) |
| Consumed accounts | Codebase account `c2f8b485a` (source baseline `dbc4c9436`); library assessment `34f9f07ee` ([surrealdb](../libraries/surrealdb/README.md) pages: rust-sdk, protocol, surrealql-server-constructs, syn-and-sql, server-tooling); [additional patterns](../assessment-patterns.md) revision 2; base patterns edition 0.2 |
| Original source inspected | `main` at `3e9a2525b`. During this pass, uncommitted concurrent edits appeared in `transport.rs`, `canonical_grpc_peer.rs`, `canonical_staging.rs`, `canonical_studies.rs` and `canonical_transport_tests.rs`. They are mostly formatting and tests; no conclusion here depends on them. Since the audit baseline, `transport.rs` gained `operation_scope`. |
| Library realizations | Vendored SDK `surrealdb` and `surrealdb-engine-api` 3.3.4. Clean upstream 3.3.4 was rebuilt in scratch by reverse-applying `vendor/deltas/surrealdb.patch`; the engine-api comparison used the installed registry crate. Also inspected: installed `surrealdb-protocol` 0.13.1, `surrealdb-types` 3.3.4, `surrealdb-rpc` 3.3.4 and `tokio-util` 0.7.19. The server is the source-built 3.3.4 with its patch series (`scripts/surrealdb-source/`). |
| Criteria | Core 3.4, Efficient Architecture 1.0, process-simulator profile 1.5 (not material here), pse-arrow binding |
| Method | Static reading, `rg`, `git diff --no-index -w`, and Context7 `/websites/surrealdb` for function transaction semantics. The reviewer ran no builds. The coordinator-run probe G3-P1 ([direct client](../../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md)) tested the second revealing case for CBO-009. |

## Group summary

**The joint decision.** All five opportunities ask one question: where should the canonical store's
request-lifecycle policy live? That policy covers original deadlines, admission, prepaid response
allowances, acknowledged-only restoration, no replay of application work, and the physical socket
join. Three homes are possible:

- inside a patched supplier SDK (current);
- in a repository-owned client built on the supplier's published protocol layer;
- in upstream SDK seams.

How SurrealQL transaction envelopes are composed, and whether subscriptions are ever admitted,
depends on that answer.

**Recommended set.**

| CBO | Disposition | Target or reason |
| --- | --- | --- |
| CBO-009 | Recommend | A repository-owned bounded client over `surrealdb-protocol`'s generated `SurrealDbServiceClient<T: GrpcService>` and `surrealdb-types`' `proto` conversions. The vendored SDK and engine-API trees leave the production workspace, and the two lifecycle tiers merge into one owner. Execute it in place of the next supplier rebase, or earlier if another transport-level change is needed. |
| CBO-010 (SurrealQL) | Recommend | A typed guarded-script envelope in Rust owns framing, guard-variant selection, the bind set and the result slot. Bodies stay raw SurrealQL. Optionally, move the protected-read prelude and postlude into stored functions at the next schema change. Keep explicit `BEGIN`/`COMMIT` everywhere. Retry pacing remains G2's. |
| CBO-016 (store) | Retain current | The bounded profile refuses `LIVE` by design. Architecture §3.3 requires a qualified consumer for live subscriptions. Changefeeds do not remove polling, and polling cost is unmeasured. A reopen trigger and a target sketch are recorded below. |
| CBO-046 (SurrealQL) | Retain current, premise partly rejected | The generated catalog is already parsed by the maintenance validator. `DEFINE PARAM` would add a second runtime representation of constants that the registry owns. |
| CBO-048 (SurrealDB parts) | Retain current | The authenticated protocol-readiness proof already covers what `/ready` adds for this deployment. `/ready`'s heartbeat semantics are a risk for steady-state readiness. Export is not a backup. SurrealKit's dependence on the edited SDK is incidental, and CBO-009's target removes it. |

**Interactions with other groups.**

- **G2.** These items are designed against whatever client seam CBO-009 lands on:
  - CBO-010 retry pacing and classification;
  - CBO-014 ambient context propagation (the `OPERATION_DEADLINE`, `RESPONSE_OWNER` and
    `METADATA_RESPONSE` task-locals);
  - CBO-015/016 in-process drain.

  The recommended client turns opaque per-request owners into typed request options. It also makes
  `tokio-util` `TaskTracker` tokens a candidate for the unified drain.
- **G4.** The CBO-001 joint judgment covers the cumulative wire allowance (`grpc_wire::Counts`,
  `ResponseLimits`). That allowance is an instance of [LOP-X001](../assessment-patterns.md#lop-x001)
  inside CBO-009. The recommendation moves its seam from engine-API internals to a repository-owned
  tower body. The accounting semantics do not change.
- **G8.** The generated Surreal codec (`pse-codegen/src/codegen/surreal.rs`) emits
  `surrealdb::types::…` paths. These become `surrealdb_types::…`, which is a generator change
  followed by `just codegen`. Templating stays G8's.
- **G9.** The CBO-050 development evidence and knowledge index (`xtask/src/development/{evidence,knowledge}.rs`)
  also uses the edited SDK. Under U3 it is either ported to the repository client or retired with
  that index.
- **G1.** CBO-048's supervisor scope stays G1's. This page only removes the SurrealDB readiness and
  backup substitutions from consideration.

**Shared prerequisite.** CBO-009's target touches roughly 355 query-builder sites. CBO-010's envelope
touches the same call sites. Doing both in one pass avoids rewriting each site twice. If CBO-009 is
not adopted, CBO-010 still stands on its own over the SDK `Query`.

**Reinforcing and competing choices.** CBO-009's direct client reinforces three things:
- CBO-010's envelope, which becomes part of the client's own script type;
- any future subscription class (CBO-016 reopen), which becomes an explicit request class rather
  than a fight with the SDK engine;
- G2's drain unification.

It competes with two alternatives judged below: a patch-shrinking variant, and a hybrid that
implements the SDK's engine SPI.

**Priority by consequence, kept separate from order.** CBO-009 is medium now and high at the next
supplier move. The rebase burden is latent: 3.3.4 is the newest SDK release, and `surrealdb-protocol`
0.14.1 was published on 2026-10-26 without a matching SDK. CBO-010 is medium: the fragmentation is
real and the risk is low. The other three are low.

<a id="cbo-009"></a>

## CBO-009 — Large local SurrealDB SDK delta with a mirrored lifetime layer

### Responsibility and target intent

**What ADR-0214 and §20 promise.** One gRPC production path, with:
- bounded application and control admission;
- original absolute deadlines through admission, session readiness, restoration, dispatch,
  conversion and delivery;
- truthful dispatch uncertainty;
- acknowledged-only, non-coalesced restoration of session setup, for a new undispatched operation only;
- no automatic replay of application queries, mutations or transactions;
- cumulative response allowances validated before protobuf decode;
- a physical HTTP/2 socket join on failure, drain and disconnect;
- typed native errors carried through the `surreal-error-bin` trailer;
- W3C trace propagation.

These are real contracts with existing tests (`canonical_transport_tests.rs`,
`canonical_lifetime_peer_tests.rs`, the scripted HTTP/2 peer in `canonical_grpc_peer.rs`).

**What is incidental.**
- That the policy is realized inside supplier internals.
- The SDK's high-level vocabulary at canonical call sites. Canonical use of it is narrow, measured
  over non-test pse-operations source:
  - `query(...).bind(...)` on about 355 builders;
  - `take::<T>(statement_index)` with only five result shapes: `Option<Object>` (155),
    `Vec<Object>` (58), `Value` (51), `Option<bool>` (11), `Vec<String>` (5);
  - `select`, `signin` and `use_ns`/`use_db` on a handful of control paths;
  - no streaming.
- The opaque `Arc<dyn Debug>` lifetime and payload owners that carry repository authority through
  `surrealdb-engine-api`.

**The delta, re-measured at HEAD.** Whitespace-insensitive, against reconstructed upstream:
- SDK `src`: +2,834 / −477. This includes 395 extracted wire-schema lines, the 316-line test-support
  module and about 60 lines of SDK test edits.
- Engine-API: +1,196 / −153, including the new 643-line `request.rs`.

That is about 3k non-test substantive lines, consistent with the assessment's correction. The
vendored code is dense: lines run up to 530 characters, so line counts understate its logic.

It has four parts:
- **Lifecycle tier.** `grpc_lifecycle.rs`: `BoundedEngine` admission (32+2 slots, 32+32 waiters),
  generation join, `ready()` restoration, `Transport` (owned hyper HTTP/2 as a tower `Service`),
  `WireBody` and `RequestTrace`.
- **Wire validator.** `grpc_wire.rs` and its schema.
- **Engine edits.** In `grpc.rs`: `Channel` replaced by `Transport`, non-coalesced replay, statement
  allocation checks, trailer decode, and `if bounded` branches throughout.
- **Builder additions.** `request_context`, `payload_owner` and `observe_statistics`.

Above it, `transport_lifetime.rs` keeps a second tier: logical caller admission, an activity count
and close rounds that drain readers, the file owner and both SDK clients. That tier is partly
repository policy, namely the logical-caller versus physical distinction and readers-first close
ordering. It partly duplicates the SDK tier's active-work counting and drain. The two tiers meet
through `RequestContext::with_lifetime_owner(Arc::new(activity))`.

Two other consumers compile the edited SDK:
- `xtask` evidence and knowledge tooling;
- the SurrealKit build. `scripts/surrealkit_source.py` rewrites SurrealKit's manifest to the
  vendored path, and hashes the vendored sources into the toolkit artifact identity.

### Credible complete options

- **A — Retain current.** Keep the patched SDK and engine-API trees, the byte-delta reconstruction
  route and the repository tier above them.
- **A′ — Shrink the patch to a transport seam.** Patch the SDK only to accept a caller-built
  `GrpcService`, and move `Transport`, `WireBody`, `grpc_wire` and `RequestTrace` into the
  repository.
  - `BoundedEngine` admission and restoration still need `GrpcEngine` internals: `attach`, `apply`,
    `Replayable` and `connect_generation`.
  - The per-request context still needs the engine-API field.
  - So the patch shrinks but stays woven through the session engine.
- **B — Repository-owned bounded client on the supplier's lower layer.** Use `surrealdb-protocol`
  0.13.1 with feature `rpc`. Its generated `SurrealDbServiceClient<T>` is bounded by
  `T: tonic::client::GrpcService<tonic::body::Body>` and provides `get_capabilities`,
  `attach_session`, `detach_session`, `use`, `set_variable`, `signin`, `query`
  (server-streaming), `kill` and `subscribe`. The client runs over the existing owned hyper
  HTTP/2 transport. Alongside it, use `surrealdb-types` 3.3.4 with feature `proto`:
  - `TryFrom<proto::Value> for Value` and the matching `Object` and `RecordId` conversions;
  - the public `Error` (already what `surrealdb::Error` re-exports), with `Error::from_wire`,
    `with_cause`, `query_details` and `not_found_details`;
  - `SurrealValue` for typed `take`.

  The repository then owns, on these public seams:
  - the lifecycle code that is already written (moved, not new);
  - the narrow façade canonical code actually uses: `query`, `bind`, `take::<T: SurrealValue>(usize)`,
    `take_errors`, `num_statements` and statistics;
  - the client semantics the SDK supplied: the capability handshake, session RPCs, query-frame
    statement assembly, `take` conversion rules and protobuf-error mapping.

  The vendored SDK and engine-API leave the production workspace. SurrealKit builds against
  crates.io `surrealdb` 3.3.4 `protocol-grpc`.
- **D — Hybrid on the SDK's engine SPI.** Implement the repository's engine as a
  `surrealdb_engine_api::SurrealEngine` over B's client and keep the SDK façade. This needs a small
  patch: today `Router::from_engine` is `pub(crate)` and `Connection` is sealed, and a per-request
  extension slot is needed in `EngineContext`.
- **C — Upstream SDK seams (U1 = yes).** Contribute transport injection, per-request context
  extensions, an explicit close-and-join, and acknowledged-only, non-coalesced restoration. None of
  these exists at 3.3.4. Upstream public `main` was last touched 2026-09-04, and the 3.3.4 tag is
  403 commits ahead of it, so acceptance and timing cannot be observed.

### Comparative argument

**Semantic fit and domain authority.** The delta's content is this repository's policy: original
clocks, admission classes, no replay, and prepaid allowances before decode. DP-14 keeps policy out
of adapters, and DP-17 keeps library internals inside one integration owner.
- A places repository policy inside the supplier's client.
- C asks the supplier to adopt that policy as SDK behaviour, which is the wrong owner, especially
  for restoration semantics that deliberately differ from upstream's coalescing replay log.
- B places the policy in the repository on the layer the supplier publishes for exactly this
  purpose. The protocol crate is written for multiple client implementations. Its comments define
  normative semantics: `RequestContext.timeout` is a ceiling, `Limits`, the capability handshake,
  and `LiveQueryCapabilities`. It also ships a client conformance suite.

**Seam stability.**
- A and A′ couple the repository to SDK internals: the connect path, session registry, replay log,
  query assembly and method builders. That is exactly the area a new SDK line rewrites.
- D couples to `surrealdb-engine-api`, whose crate docs state: "an internal interface … carries no
  stability guarantee and may change in any release, including a patch release". `surrealdb-rpc`
  is likewise internal, with no SemVer.
- B couples only to:
  - generated message types, versioned with the server, which is already pinned and source-built;
  - `surrealdb-types`, which the SDK re-exports as public API.

**Change locality and isolated reasoning.** Under A, a server, SDK or protocol move means
reconstructing a 38.8k-line byte delta and rebasing about 3k substantive lines over a rewritten
gRPC engine. Under B, the same move means bumping the protocol and types crates, regenerating the
wire-schema extraction (already a maintained script), and adjusting frame assembly if frames
changed. Protocol 0.14 shows the kind of change involved: it removes `ExportDirectory`, adds f16/f32
value kinds, and renews catalogs.

B also resolves the "mirrored layer". With both tiers in the repository, one connection owner can
hold:
- admission slots;
- the logical-caller phase;
- one drain. `tokio-util` 0.7.19 `TaskTracker::token()`, `close()` and `wait()` fit both detached
  request tasks and continuation activities, and the crate is already used by `ResultReadDrain`.
- a typed request-options struct, replacing `Arc<dyn Debug>` owners threaded through the
  engine-API.

**Reassessment qualification.** `TaskTracker` supplies completion tracking, not admission
closure: `close()` does not prevent new tokens or tasks. Keep serialized Open/Closing/Closed
admission, logical-caller closure, admitted continuation rights, readers-first prerequisites,
independent two-driver settlement and retry progress. These can share one connection owner
without collapsing into the tracker's closed bit.

The repository's own semantics stay where they are and are not mirrors: operation clocks spanning
retries (`transport.rs`), readers-first close ordering, and two clients per store.

**Execution.** The physical path is the same in every option: the tonic-generated client over an
owned hyper connection, with the wire preflight in the response body before prost. No performance
difference is claimed.

**Failure and recovery.** The target can preserve these semantics, but changing this ownership
seam requires requalification of the existing transport and lifetime controls. B must fail closed on
unknown query-frame variants and on missing `QueryEnd` or trailers, as the delta already does.

**Total machinery.**

| Under B | Items |
| --- | --- |
| Disappears | The vendored SDK (about 22.4k source lines, mostly unused engines) and engine-API (about 2.4k) |
| Disappears | `vendor/deltas/surrealdb*.patch`, the reconstruction procedure, `PATCH.md` and the SDK part of `vendor/patches.md` |
| Disappears | The generic/bounded branching inside `grpc.rs`, and the opaque ownership-retention types |
| Disappears | The `grpc-lifecycle-test-support` feature, and the coupling of SurrealKit's artifact identity to vendored SDK inputs |
| Added | Repository ownership of about 1k lines of client semantics (an inference, not a measurement) |
| Added | A façade test suite pinning `take` conversion rules and error precedence |
| Added | Tracking protocol changes at each server move |

- **A′** removes little: the session engine stays patched.
- **D** removes most of the patch but leaves a permanent dependency on an SPI the supplier
  disclaims. The façade it preserves is narrow enough that keeping it is a weak reason.

**Transition.** B is one coherent replacement under the single-production-path rule:
- about 40 pse-operations files, mostly mechanical if the façade keeps the `query`/`bind`/`take`
  names;
- the `xtask` evidence and knowledge tools (subject to U3);
- the `pse-runtime` telemetry target filter (`surrealdb::engine::remote::grpc_lifecycle`);
- the generated codec paths;
- `surrealkit_source.py`;
- re-running ADR-0214's bounded gRPC, lifecycle and sixteen-lane controls.

The existing peer tests exercise protocol behaviour and port directly.

### Revealing cases

1. **A connection fails after a mutation was dispatched; the next undispatched operation must
   restore.** B must re-home the restoration logic in `BoundedEngine::ready` (join the old
   generation, attach, replay the acknowledged log, publish the checkpoint) and `GrpcSession`. The
   bounded profile already permits only one application session per client ("Bounded gRPC permits
   one application session"), so B does not need the SDK's `SessionRegistry`. This simplifies
   rather than weakens the logic.
2. **A subtle SDK result rule.** Examples: `Option<T>` from a statement result; the precedence of
   substantive errors over `NotExecuted`/`Cancelled` in `transport::checked`; the BEGIN/COMMIT
   result slots. A wrong re-implementation would silently change behaviour, so the façade must pin
   these against the current SDK first. Probe G3-P1 ([evidence](../../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md),
   *Measured* on one fixture run per mode against a disposable in-memory server) tested this. A
   minimal client on the generated `SurrealDbServiceClient` matched the vendored SDK on 37 of 37
   observations, covering guarded scripts, thrown refusals, deadlines, both conflict shapes,
   authentication failure and parse failure, once four SDK rules were reproduced: the builder's
   script trim, statement compaction, the `take` conversion rules and the error rebuild. Those
   four rules are exactly what the façade must pin. The error rebuild depends on
   `surrealdb-types` 3.3.4's `#[doc(hidden)]` `Error::from_wire`, the only route to the
   kind-specific codes; the U1 branch below names the public conversion that would remove it. The
   probe's direct client is 527 lines, of which roughly 350 transcribe SDK behaviour; it omits the
   bounded lifecycle tier.
3. **Does B regress anything?** It no longer receives upstream fixes to generic SDK semantics
   passively. The delta already replaced those semantics in the areas canonical code uses, so the
   current system gets such fixes only through a rebase anyway. No promised guarantee is weakened.
4. **SurrealKit needs a different SDK than canonical clients.** Under A both are tied to one
   vendored tree. Under B they decouple. That is the correct ownership: canonical is production and
   SurrealKit is tooling.

### Disposition: Recommend B

**Bounded target.**
- **Module.** A `pse-operations` module (no new crate) owns `SurrealDbServiceClient<OwnedHttp2>`,
  the moved lifecycle tier, the wire preflight and trace injection.
  - `RequestContext` and `ResponseLimits` become repository types.
  - The connection owner unifies admission, caller phase and drain.
  - The façade covers exactly the used surface.
- **Domain rules retained.** Original clocks, two admission classes, no application replay,
  acknowledged-only restoration, truthful reporting of dispatch uncertainty, prepaid allowances
  before decode, physical join, typed trailer errors, and 16-lane capacity.
- **Couplings to declare.** *(Revised after the omission and preservation challenge.)*
  - Typed errors depend on `surrealdb-types`' `#[doc(hidden)]` `Error::from_wire`. That is the
    same kind of unstable coupling this page holds against option D, so record it as a pin hold on
    `surrealdb-types` with the reason, until a public conversion exists (U1).
  - The interactive-transaction test helper (`canonical_conflict_tests::held()`) uses the plain
    SDK profile; the façade must either offer an interactive-transaction test path or the tests
    move to the single-call conflict shape the probe also covered (case 4b).
  - `xtask` uses more SDK surface than pse-operations: `Database` signin, `Query` builders and
    the generated `surrealdb::types` codecs. The façade scope is the union, not the pse-operations
    count alone.
- **Behaviour changes.** None are intended.
- **Consumers.** These are updated:
  - pse-operations call sites;
  - `xtask` development tools;
  - the telemetry target;
  - the codec generator;
  - the SurrealKit build.
- **Timing.** Execute it instead of the next supplier rebase. Execute it earlier if a transport-level
  change is needed, such as a new request class, an allowance change, or a subscription class.
  Until then, A remains the Tested realization.

**U1 branches.**
- **U1 = no.** B as stated.
- **U1 = yes.** Still B. The worthwhile contributions are ones that shrink B:
  - a public `proto::SurrealError → Error` conversion in `surrealdb-types`;
  - gRPC coverage in the conformance runner, whose documented transports are `ws` and `http`;
  - optionally a public `Surreal::from_engine`.

  Do not pursue SDK lifecycle seams (C): they would ask the supplier to own repository policy, and
  they would arrive only with a release matching a newer server. Reconsider A′ or C only if the
  supplier itself ships transport injection, per-request context, close-and-join and
  acknowledged-only restoration.

### Rule impacts (identified, not initiated)

- **Decision.** The change stays within ADR-0214's outcome: a bounded gRPC path with an owned HTTP/2
  driver. ADR-0214 is still `proposed`, and its "Pros and cons" and "Pin the SDK/ABI" text name the
  patched-SDK realization. The maintainer can record the realization change in its decision PR or
  in a short ADR.
- **Architecture text.** These need amendment through the design-edit route with a `blueprint.md`
  revision row:
  - §3.3 ("`pse-operations` uses the remote SDK");
  - §20 and §20.6 transport wording;
  - the architecture overview ("through the remote SDK");
  - scope-and-open-design ("thin native SDK client").
- **Dependencies.** Dropping the vendored path dependency and enabling `surrealdb-protocol/rpc` and
  `surrealdb-types/proto` are ordinary dependency changes.
- **Crate.** A separate client crate would need an ADR and a design review. It is not recommended:
  `xtask` already depends on `pse-operations`.
- **Codegen.** The codec path change requires `just codegen`.

### Evidence labels

- *Interface-checked*, by inspection:
  - the reconstructed upstream versus vendored diff;
  - the generic bound of the generated client and its RPC list (protocol 0.13.1 `*.tonic.rs`);
  - the public `surrealdb-types` `proto` conversions, `Error::from_wire` and accessors;
  - the engine-API and `surrealdb-rpc` stability statements;
  - `Router::from_engine` visibility;
  - the profile gating in vendored `grpc.rs`;
  - the call-site census.
- *Tested*: the current realization (A), per ADR-0214 Plan49a.
- *Proposed*: the B target.

<a id="cbo-010"></a>

## CBO-010 — Conflict retry and guarded-transaction preludes assembled per site (SurrealQL side)

### Responsibility and target intent

Each guarded decision runs as one atomic script within one RPC under the original clock. The
promised parts are:
- guard semantics (pins, grants, fences, deadline checkpoints) atomic with the body;
- typed `THROW 'PSE_REFUSAL:…'` refusals;
- no partial publication;
- each retry attempt rebuilds the complete decision.

Retry pacing and classification belong to G2. The result-slot rule is that the result is the
statement before `COMMIT`. It is used as `num_statements().saturating_sub(2)` 65 times.

### Current state

Non-test source at HEAD:
- 86 lines contain `BEGIN;`. Some of these are in test modules inside production files.
- 61 of them call a stored `fn::` guard.
- 48 are pure `BEGIN; RETURN fn::…(…); COMMIT;` by my pattern. The account says 51.

The `BEGIN;` count misses most of the residue, because the protected-read envelope is composed from
a constant:
- `PROTECTED_BEGIN` (one declaration in `canonical.rs`) is referenced 29 times;
- a pin re-check postlude (`fn::…::deadline(…); IF $pin.expires_at <= time::micros() [AND !fn::pse_files_v1::ticket_open(…)] { THROW … }; RETURN $rpc_result; COMMIT;`)
  appears about 20 times, in two variants;
- the four-variable protection bind set (`problem`, `revision`, `sequence`, `protection`) appears
  29 times.

There are also three near-identical project-authority wrappers. `project_query` strips a leading
`BEGIN;` from its body; `context_query` and `research_query` differ only in the bind name. A
workbench prelude composer and four staging prelude constants complete the picture. So the guard
logic already has single declarations, either a stored function or a constant. The repetition is in
composition: envelope choice, binds, postlude variant and result slot.

### Credible complete options

- **R — Retain.**
- **E — A typed envelope in Rust.** Envelope variants are explicit, with no default:
  - `ProtectedRead { recheck: Pin | PinOrTicket }`
  - `ProtectedAdmission` (no postlude, as at the `RESULT_ADMISSION` sites)
  - `ProjectAuthority { issuer }`
  - `StageGuard`
  - `DeadlineOnly`

  Each variant emits its framing, binds its variables (from `&ProtectedSelection` and similar),
  binds `$pse_rpc_expires_at`/`$pse_rpc_timeout`, and returns a typed result handle that knows its
  slot. Bodies stay raw SurrealQL and may not contain framing.
- **F — Stored-function consolidation.**
  - `fn::pse_protection_v1::open($expiry,$problem,$protection,$revision,$sequence) -> {pin,grant,result_pin}`
  - `confirm($expiry,$guard,$ticketed)`

  Interface-checked through Context7 (2026-10-27):
  - a function called inside a manual transaction joins it, and a failure or `THROW` aborts it;
  - `BEGIN`, `COMMIT` and `CANCEL` inside a function body have no effect.

  Downstream uses of `$pin`, `$grant` and `$result_pin` would read `$guard.*`.
- **I — Implicit single-statement transactions** for the 48 single-call sites.
- **T — RPC-level transactions** (`begin_transaction`/`commit_transaction` with
  `RequestContext.transaction`).
- **No SurrealQL construct wraps a caller body.** This negative result is inherited from the
  assessment.

### Comparative argument

**E.**
- It removes exactly the repeated decisions: envelope selection, binds, postlude variant and slot
  arithmetic.
- It turns the guard variant into a visible typed choice.
- It makes nested framing impossible, so `project_query`'s strip is no longer needed.
- It changes no server semantics.
- Bodies keep their readable raw SurrealQL.

**F** shortens scripts and keeps the protected prelude in the parse-checked, registry-generated
catalog. It is a schema change, though, and its benefit is secondary because the prelude already
has one declaration. It fits best at a natural schema change.

**I** gains nothing once the envelope hides framing. It would break the uniform slot rule and needs
an unverified conflict-typing check (assessment probe P2), so drop it. P2 is then unnecessary.

**T** is ruled out:
- it multiplies round trips per decision;
- it holds server transaction state across client failure;
- it widens SSI conflict windows;
- it conflicts with the bounded one-RPC decision model.

### Revealing case

A builder with a default postlude would silently add or remove the pin re-check at the
`RESULT_ADMISSION` mutation sites (`canonical_results.rs`), which deliberately omit it. Required
explicit variants keep that distinction. Dynamic heads (`canonical_selection.rs`
`{PROTECTED_BEGIN}{head}{suppliers}`) remain body text inside the envelope.

### Disposition: Recommend E, with F optional at the next schema change

- **Under CBO-009 B,** the envelope is the repository client's script type, and the port happens
  once. Because both changes touch the same ~355 query sites, design the envelope from the start
  as the future client's script type and schedule the two together, or land the envelope first
  against an interface the client will implement; do not rewrite the sites twice. *(Revised after the omission and preservation challenge.)*
- **Under A,** it wraps `surrealdb::method::Query`.
- **The retry boundary is unchanged.** The envelope is built inside the attempt closure.
- **Machinery that disappears:** three wrappers, about 20 postlude copies, 29 bind sets and 65
  index computations.
- **Added:** one envelope type and its unit tests. Those tests can parse each variant's frame with
  `surrealdb-syn` under the `canonical-maintenance` feature.

**Rule impacts.** E needs none: it is a refactor within contracts. F is a registry and generated
catalog change: run `just codegen`, which changes the schema digest and therefore requires fresh
activation under ADR-0214. It is at most a short ADR, at the maintainer's discretion.

**Evidence labels.** *Interface-checked*: counts by `rg` at HEAD, and the function transaction
semantics through Context7. *Proposed*: the target.

<a id="cbo-016"></a>

## CBO-016 — Waiting by polling for state owned elsewhere (store side)

### Responsibility and target intent

Waiters discover canonical state written by other processes. The two cases are worker discovery of
claimable attempts (500 ms poll plus a 30 s recovery pass) and study inventory. The promised
properties:
- claims stay the authority;
- observation grants no release;
- correctness never depends on notification;
- bounded requests;
- no daemon.

Most of the pse-operations sleep sites are retry backoffs (G2), a local `flock` try-lock (G1's
CBO-003) or tests. They are not store waits.

### Options

- **R — Retain the documented polling.**
- **L — A `LIVE SELECT` wake hint** over the protocol's `subscribe`, with a polling backstop.
- **CF — Changefeeds** (`DEFINE TABLE … CHANGEFEED`, `SHOW CHANGES … SINCE`) as a cheaper catch-up read.

### Comparative argument

The vendored bounded engine explicitly refuses LIVE. `GrpcEngine::subscribe_live` returns "Bounded
gRPC LIVE subscription is not an admitted canonical operation" when bounded, so the assessment's
probe P1 is moot.

L would add a long-lived request class:
- a stream per waiting process;
- server live-query state;
- possibly a held session-admission permit under the server's `grpc-session-admission` patch.

That is new functionality with tradeoffs, not a clear-win minor change. Architecture §3.3 already
states that live subscriptions "need a concrete qualified consumer".

CF still polls. It adds write amplification on hot tables and cursor and retention handling, and
the claim query is already index-backed.

Protocol 0.13.1 exposes `LiveQueryCapabilities { delivery, resumable, multiple_subscribers, retention }`
and `SubscribeRequest.resume_from: LiveQueryCursor`. The inspected source-built server realization
(`v3.3.4-mx1.0.0-effective-v1-3549c2a97bd565fd2497`, `server/src/rpc/grpc.rs`) advertises
at-most-once delivery, `resumable=false`, `multiple_subscribers=false` and no retention;
`Subscribe` explicitly rejects a nonempty resume cursor. SDK 3.3.4 sends `resume_from: None`.
Protocol expressiveness does not establish this server capability.

### Disposition: Retain current

**Reopen trigger:** measured discovery latency or idle query load that is material at 16 lanes
across several worker processes.

**Target when reopened.** A repository-owned subscription request class in CBO-009's client, with:
- a lifetime separate from per-operation deadlines and allowances;
- an allowance per notification;
- `kill` on drop;
- re-registration after loss followed immediately by authoritative re-read; capability-gated
  `resume_from` only as a future-server branch;
- wake hints only;
- the polling backstop and claim authority unchanged.

This is far easier under CBO-009 B than inside the SDK engine.

**Rule impact if reopened:** a §3.3 consumer qualification, and an ADR-0214 request-class addition
(a short ADR).

**Evidence labels.** *Interface-checked*: vendored `grpc.rs` `subscribe_live`, protocol 0.13.1
message definitions and the worker poll settings. Nothing is measured.

<a id="cbo-046"></a>

## CBO-046 — SurrealQL and Python emitted through string templates (SurrealQL side)

### Responsibility

Generate a deterministic catalog from registry constants and declaration-owned fragments. The
generators have 18 placeholders:
- mostly integers from `pse_schema::catalog`;
- one quoted interpretation string;
- the holder-kind list and holder guard fragments in `surreal_artifacts.rs`.

### Options

- **R — Retain.**
- **V — Parse-validate at codegen** with `surrealdb-syn`.
- **P — `DEFINE PARAM`** for the integer placeholders.
- **AST rendering** with `surrealdb-sql`. The assessment already ruled this out: the crate is
  scheduled for deletion and it normalizes the authored text.

### Comparative argument

**V is largely present already.** `canonical_maintenance::parse_definitions(wire::SCHEMA)` parses the
whole generated catalog in a unit test under `canonical-maintenance`, and again at maintenance
activation, using the exact server grammar. The `xtask` `surrealql-tools` validator exists but is
wired only into tooling experiments. Adding a pse-codegen dependency on the server grammar would
move the same check earlier at little gain.

**P adds a second representation.** Today the authority is one registry constant. It is projected as
a literal into the catalog and, in several cases, also consumed client-side (for example the
message-byte limits behind `ResponseLimits`). `DEFINE PARAM` would add a stored runtime value, a
new definition family that `parse_definitions` refuses today, unverified `LIMIT $param` planning,
and variable-shadowing risk. Integers carry no escaping risk.

### Disposition: Retain current, premise partly rejected

DEFINE PARAM is not a clear win, and validation already exists. The checked-substitution or
templating mechanism (asserting that each placeholder is consumed and none remain) is G8's choice.

**Evidence labels.** *Interface-checked*: `canonical_maintenance.rs` (`parse_definitions` and its
catalog test), `xtask/src/surrealql_tools.rs`, and the generator `replace` chains.

<a id="cbo-048"></a>

## CBO-048 — Host workload admission and service supervision rebuilt above systemd (SurrealDB parts)

### Responsibility

Decide whether the owned server is ready, back it up and restore it, and roll out schema.

### Options and comparison

**Readiness.** `service_readiness` combines:
- systemd unit state and invocation;
- listener ownership;
- `/health`;
- an authenticated native gRPC proof recorded per `InvocationID` (`protocol-readiness.json`).

For this deployment `/ready` would add only node registration and heartbeat freshness: a single
node, no `--import-file`, and root credentials established before the proof can authenticate. Its
503 also does not separate a failed startup from a slow one. Its heartbeat-staleness rule (three
refresh intervals) makes it a false-unready risk if used as steady-state readiness under saturating
load with synchronous AOL fsync. That risk was not measured, but it is plausible, and it would
refuse legitimate admission.

**Backup.** `surreal export` cannot replace the offline state copy, payload-inventory verification
or restore re-rooting. Adding it as an extra artifact is new functionality without a consumer.

**SurrealKit.** It is already adopted for rollout, and settlement and exclusive ownership stay local.
Its build pins the edited SDK, but it runs the unbounded upstream-transport profile:
- the bounded profile is selected only by `Config::bounded_requests()`, which SurrealKit's code
  never calls;
- its fixture profile permits SDK clones.

Only the trailer decode, which is not profile-gated, differs from upstream 3.3.4. Its dependence on
the patch is therefore incidental.

### Disposition: Retain current

SurrealKit's coupling to the vendored SDK dissolves under CBO-009 B. A SurrealKit build against
released 3.3.4 would confirm this; it was not run, so the conclusion rests on the static
profile-gating reading above.

**Evidence labels.** *Interface-checked*: the `surreal_server.py` readiness and start functions,
`surrealkit_source.py`, `surrealkit.py`, the vendored `grpc.rs` gating, and Context7 `/ready`
semantics inherited from the assessment.

## Limits

- **Not read.**
  - SurrealKit source.
  - The bodies of most of `surreal_server.py`.
  - The server-side gRPC `Subscribe` implementation.
  - The server patch bodies (only their recorded purposes).
- **Inherited, not re-verified.** The public `main` and tag observations and the absence of a
  release beyond 3.3.4 come from the assessment (2026-10-27).
- **Not estimated.** The size of B's re-owned semantics is an inference.
