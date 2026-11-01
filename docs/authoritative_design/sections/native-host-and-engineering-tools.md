---
title: Native host and engineering tools
status: current
---

# Native host and engineering tools

### 21.7 Independent host and engineering tools

The independent `pse-host` owns native commands, environment selection, managed processes
and engineering investigations. Scientific preparation, execution and result interpretation
remain with product owners. Source-only reads need no analyzer; native analysis, acquired APIs
and compiler observations keep their own explicit preparation and original evidence.

| Question or effect | Performing owner and boundary |
|---|---|
| Read source, syntax or a known declaration | Host source/query owners; no semantic preparation or product startup |
| Ask a native semantic question | Explicit prepared immutable Cargo/VFS world; qualified source/session identity |
| Inspect dependency APIs or compiler facts | Original acquired artifact and context; passive reads do not reacquire inputs |
| Execute or control work | Foreground owner or explicitly detached service; original capture and actual settlement |
| Decode scientific data | Explicit product owner; the independent host does not import scientific interpretation |

Ordinary host work can use the selected unbounded build placement; explicit finite placement
and scientific execution retain their distinct admission contracts. An uncapped host allocation
is not an unlimited scientific pool. `pse-host::admission` selects placement, while the performing
product owner admits numerical resources. Operating commands and preparation mechanics belong
in the [environment guide](../../dev/agent-environment.md); failures and recorded qualification
remain in [§23–§24](operations-and-validation.md).

The contracts below describe current source responsibilities. Proposed ADR status records
formal decision status, not missing implementation. Historical package references identify
evidence at its original scope and do not establish future direction or a fresh qualification.

#### 21.7.1 Native worlds and compiler observations


> Decision: ADR-0233 (proposed Plan62 extension; operator-confirmed preparation/compiler target).

Explicit native preparation supports build-generated inputs and proc macros with qualified
readiness and retained session/CPU ownership. Build metadata errors and unavailable macro
dylibs refuse preparation. The selected test context is explicit; it is not inferred from a
test-looking source location. Retained compiler, macro-server, dylib and generated-input
identities fence the loaded world. Individual cancellation does not guarantee interruption of
external macro I/O; original managed whole-session stop owns escalation and descendant settlement.

Native explanations preserve import/read/write/test roles, implicit operations, source versus
stored capture types and ordered adjustments. Callable discovery keeps declaration signatures
separate from actual call-site inference. Contextual substitutions, receiver/parameter/result
types and supported obligations attach by native callable identity; hypothetical calls and
unsupported lifetime/associated constraints remain unknown. `Holds`, `NotProven`, `Invalid`
and `Unsupported` are distinct outcomes. Fallible full layouts require a supported Sized proof;
an unsized prefix is separately labelled. Compiler-layout association checks original premises
without manufacturing numerical equality or unique type correspondence.

A separately prepared optional compiler-analysis binary within the host isolates compiler-private
linkage and consumes actual admitted root arguments. The exact pinned compiler copies owned
phase-qualified capture/suspension/cleanup observations and continues normal compilation.
Supported function/closure/coroutine bodies use coordinated native query ordering; constant/CTFE
bodies remain explicitly unavailable. Saved storage does not establish liveness or Send, native
may-initialization does not establish loans or reachability, and static cleanup edges do not
establish physical resource release. Actual ABI and unwind-termination cause remain qualified
compiler observations. Wrapper images, versioned compiler libraries and their loader alias
resolutions are checked before and after acquisition. Publication requires successful original
compilation and completed capture/settlement. Passive reads select only acquired fact families,
keep whole records indivisible, and address original artifacts/context separately. Plan62 owns
the implementation's qualification; the proposed ADR's formal acceptance is unchanged.

#### 21.7.2 Generated contracts and authenticated transport

> Decision: [ADR-0235](../../adr/0235-code-intelligence-contract-foundation.md) (proposed;
> operator-authorized Plan61 foundation and B/C/D realization; the scoped prospective review covers A02).

New code-intelligence wire meanings are authored in proto within the independent host;
existing product-registry meanings remain derived and deliberate CLI presentation remains
Serde/Schemars-owned. Explicit xtask generation compiles one schema input, checks required
constraints and stages messages/views/validators/service bindings before publication. Invalid
schemas or unsupported required lowering refuse before aggregate writes. Full descriptor
metadata remains distinct from the stripped runtime pool; HIR/rustdoc/source identities retain
their original world and artifact qualification. Generated validation must preserve CEL error
and arithmetic semantics without treating evaluator defects as user violations. No ordinary
read/launch starts a generator. These contracts add no workspace crate or universal service;
Plan61 retains package acceptance and assembled qualification ownership.

Consumed source/API/native observations have typed fact constructors and explicit provider/world
guards. Direct VTable projection borrows generated fields without a dynamic-message transcode;
a reflected shape does not establish applicability or native identity. Ordinary callers can decode
native type facts without loading HIR libraries. Source selection, predicate truth, answer demand,
rendering allowance, count precision and stable continuation retain their native owners. Grouped
questions share request-local preparation, authenticated clients and proofs qualified by their
original use-site as well as entity/world identity. Original provider documents remain separately
qualified inspection evidence; decision consumers do not reconstruct facts from category strings.

Offline schema investigations acquire complete immutable descriptor bundles and bind navigation,
payload interpretation and comparisons to their original byte digests. Conflicting logical files
and incomplete declared imports refuse before pool construction. Schema addresses describe their
selected bundle; they are not Rust entities or application truth. Descriptor comments and spans
remain producer metadata until exact original source and coordinate conventions are verified.
Source-only actions and native Rust world guards retain their existing owners.

Payload inspection is passive. Original input identity remains separate from decoded projections
and re-encoding. Generated static reflection and compiled validators serve the exact compiled
schema world; external dynamic schemas expose constraint metadata without claiming evaluated
validation. Descriptor-backed text parsing preserves diagnostics and is distinct from semantic
validation. Contract comparison distinguishes structural observations from native compatibility
classifications and semantic equivalence. Generated-Rust associations derive from explicit emitted
trait/accessor evidence qualified by source, generator configuration and output bytes; unresolved
correspondence stays unresolved. Exact authored byte ranges come from native proto syntax nodes
matched to compiled declaration identities during generation; passive navigation validates
recorded ranges against their original bytes instead of interpreting generic descriptor columns.
Missing, ambiguous or recovered syntax correspondence remains unavailable. These capabilities require no new daemon or mutable
global pool.
Generated lint qualification preserves source comments and restricts assertion permissions to
exact pinned reflection functions and immutable metadata/self-encoding invariants; unknown
assertion patterns refuse generation. Acquired-input decoding remains fallible.

The explicitly prepared semantic helper serves generated binary gRPC over HTTP/2 on its private
Unix socket. Every physical client connection, including reconnects, authenticates the kernel peer
against original managed custody before negotiation. Server request identity comes from Unix
credentials. Fixed status/package/capability operations are unary; a conversation admits one Start
and idempotent Cancel controls. Graceful input half-close permits completion. Session-owned workers
retain admitted native work beyond handler/stream loss, deadline or cancellation and join the actual
CPU task. Stop closes admission, cancels and drains workers before releasing the provider. Bounded
output delivery cannot prevent that drain. Terminal distinguishes cancellation request, original
work outcome and CPU settlement from live-LSP return/detachment; missing delivery establishes no
client-observed settlement. Schema, constraint, context, validator, provider, transport and settlement
diagnostics preserve original causes. Neither transport status nor task detachment proves native
CPU completion. The replaced JSON-line/control-byte protocol has no compatibility listener;
incompatible prepared selectors retain original native administration and require explicit
stop/repreparation for queries. The pinned snapshot VFS requires UTF-8 source paths, with explicit
unsupported refusal outside that scope; source-only raw-path access remains independent.

#### 21.7.3 Product boundary and managed work

> Decision: ADR-0233 (proposed; operator-confirmed Plan59 A07 target).

The agent CLI delegates scientific decoding, response schemas and interpretation to an explicitly
prepared product owner. The host forwards selected bytes and interprets transport metadata without
importing scientific/Arrow/database policy. CLI invocation, scientific run/attempt, process and
observation identities remain separate; supplied associations never imply cancellation authority.
Code intelligence retains provider/configuration/source-generation scope: candidate tests are not
actual selected tests, static calls are not executed causal edges, and watcher updates do not
establish an atomic filesystem snapshot. Owned reusable analysis sessions protect tools and original
children until settlement; canceling a query does not terminate the session, while revision changes
may cancel old snapshots. Probes, generation, compiler preparation and semantic application remain
explicit effects, independently usable from passive source reading and existing-work control.

The independent native host owns developer environment selection, authentic OS process/service
ownership and operator effects, without scientific preparation, Arrow/DataFusion, database clients
or generated product schemas. Heavy native developer composition stays in xtask. Canonical SurrealDB
owns persisted execution transitions and schema/write-admission policy; computation and physical
joins remain Rust/OS effects. Service launch alone does not establish schema readiness; pure local
science needs no database.

> Decision: ADR-0234 (proposed Plan59 managed-invocation realization; native qualification pending).

Foreground operations
use native synchronous caller-bound ownership through reap, capture and settlement; generic
foreground reconnection/crash-finalizer continuity is not promised. Explicit detached operations
use one transient user service containing the fixed wrapper, payload and ordinary descendants.
Nested ordinary work inherits aggregate ownership; independent detach requires admitted placement
and cannot escape finite constraints. Independent nested CLI subtree control is not selected.
Systemd owns forced termination, Rust owns cooperative joins and required output. Missing original
evidence remains incomplete; scientific and fixture owners retain their independent authority.

Optional foreign adapters require a named consumer and remain coarse callers of the same native
operations, documents and protected data services. They acquire no separate scientific, scheduling,
settlement or schema authority. Native product APIs require no PyO3 implementation, full CLI,
RMCP endpoint or cdylib. Plan59 separately selects the agent-facing CLI over those native owners.
Retired producers, callers, configuration and mechanism-only tests leave together; useful scientific
acceptance stays with native callers. Typed fixtures and native documentation/governance/evidence
owners replace first-party Python tooling (§24.1). A successful compiler result is not invalidated
solely by cache-service loss; correctness, real boundary admission and physical release remain.
Implementation of this cut does not close all Plan58 B–G acceptance or qualify H's assembled and
durable-deployment campaigns (§24.2).

> Decision: ADR-0233/0234 (proposed Plan59 A08 clarification; operator-confirmed library-integration RC01–RC04).

The host performs no automatic generation collection. Immutable completed generations remain
available until explicitly authorized administrative cleanup. Executable authentication,
provenance and actual process/resource duties remain independent of collection bookkeeping.
Unknown existing claims/artifacts remain untouched; no retrospective release is inferred.
Native owners enforce valid terminal facts before presentation. Directional wire schemas and
common source/range/conflict proposals establish neither scientific validity, semantic name
resolution nor publication. Source-only readiness has no semantic-session prerequisite;
semantic operations retain their selected context. Representation contracts establish no
new runtime acceptance; recorded source/query evidence retains its original scope (§24.2).

#### 21.7.4 Source questions and addressed evidence

> Decision: ADR-0233/0234 (proposed Plan59 A10 search-contract amendment; operator-confirmed S-RC01–S-RC03).

The developer-query surface has three public passive intents: `search`, direct `inspect`
and immutable `propose` previews. Useful read/query capabilities move with their callers; replaced
routes retire after their replacements pass. Explicit preparation, acquisition, probes, test
construction/execution and control retain their effects and owners. A known file/range/position,
index/revision, declaration or original-evidence locator requires no preceding search. Source-only
discovery works without an analyzer; semantic/API consumers state their actual preparation/context.

Flags, typed JSON requests and saved queries share one validated interpretation. Source selection,
predicate scope, directed relation, requested answer and display controls are distinct. Native
providers own matching, syntax, semantic/API/compiler facts and bounded relationship expansion.
Same-node/entity conjunction requires a common candidate/context; file conjunction can use different
witnesses. Missing or incompletely examined facts stay unknown, including under negation.

#### 21.7.5 Cross-context relationships and guarded comparisons

> Decision: ADR-0235 (proposed Plan60a relationship-contract extension).

**Implemented Plan60 comparison and agent consumers.** Paired inspection consumes explicitly addressed
original source/API/context inputs and separately authenticated before/after native sessions.
Side-local observations and qualified correspondence do not establish universal cross-world
identity. Retained metadata and admitted-unit evidence keep independent custody and hashes;
passive comparisons never acquire missing inputs or prepare worlds. Association and relevance
views use canonical document/lifecycle, inventory/report and source interpreters, preserving
connection basis, original policy/status and unknown assurance. Source-only inspection remains
independent of native preparation. Concise parser discovery and native-encoded references share
the existing parser and address authority. Invalid presentation choices are checked before
providers where the selected schema establishes their domain. Structured validation correction
extends the existing diagnostic envelope with origin-owned disclosure/completeness facts;
unknown peers remain unknown, and ordinary correction never restores redacted selectors from
raw diagnostics. Native wire changes are authored/generated and fenced by protocol7. Existing
preparation, cancellation and settlement owners remain authoritative. The
[Plan60 Outcome](../../plans/60-engineering-code-intelligence.md#outcome) owns selected delivery
and qualification; conditional indexing/compiler/schema/remote branches remain dormant.

Retained Context addresses pair exact raw metadata with admitted units under their original
owner. Compiler generation/probe contexts retain the strict artifact reader; ordinary workflow
admissions retain the authenticated terminal/report reader. Digest domains, original outcomes,
ordered admission groups and original indexes stay explicit. Metadata and units supplied
independently do not establish common acquisition. Missing requested native enrichment does
not suppress valid retained evidence, and omitted optional enrichment does not itself make
that evidence partial. Duplicate unit matches remain ambiguous. Rebuildable Context pages
retain their address outside projected detail and account for producer omissions.

Source relevance exposes supported assertion operand/initializer/setup structure without
claiming coverage or oracle independence. Frozen test/failure addresses retain full original
selection identity and report/inventory digests; current source and current selection actions
remain separate guarded observations. Canonical document associations retain original policy,
section/status owner, explicit connection basis and unresolved references. Optional shared
engineering inquiries support the distinct review/audit/planning processes without prescribing
their conclusions or required tool use. Literal top-level field choices, native opaque address
handoff and disclosed structured correction use the existing parser, reference and diagnostic
owners. Opaque JSON retains runtime projection checks; arbitrary peers do not gain a known
original violation count or redacted location from raw diagnostic custody.

**Implemented extension (Plan60b/c).** Native compatibility compares guarded operands in one
snapshot after fallible generic-owner rebasing; possible unification/coercion, existential
implementation discovery and obligation evaluation retain distinct outcomes. Native origins
distinguish locals, assembly operands, originating ADTs and generated observations. Acquired
API adjacency binds artifact/context/item addresses and never acquires missing artifacts
implicitly. Rich patterns share one structural parser/matcher across entry points. Lifetime
views join native/compiler bodies only with verified original-source and selected-unit
correspondence; normalized/remapped locations and accompanying runtime evidence alone do not
prove that join. Existing owners retain preparation, cancellation and actual settlement.
The existing compiler producer retains checksum-verified original UTF-8 source bytes once per
file inside its acquired facts artifact; source expansion reads those bytes with bounded
UTF-8/base64 output. Historical artifacts without that binding remain readable and unjoined.
Body correspondence checks compiler identity, actual cfg, edition, target, package/role/features,
crate-root bytes and exact original body bytes. Binary body context uses a unique admitted
Cargo target mapping and the actual loader target; it does not become a library API profile.
Pinned native cfg discovery uses optimized settings and adds atomic capability atoms; ordinary
debug compiler acquisitions can therefore remain unjoined. Those actual differences are retained.
Supplied work evidence remains explicitly accompanying evidence, without an inferred execution
link or a zip between native captures and compiler saved fields. Public lifetime cancellation
retains the original native receive owner through terminal settlement.

Artifact adjacency and native traversal share canonical edge indexes while retaining separate
domains, qualified nodes and original witnesses. Adjacency reuses artifact-local definition,
parent and contract lookups within the inquiry; provided defaults, associated bounds, impl
qualifiers and compiler-resolved documentation links do not establish use-site applicability.
Pattern input preserves lifetime/type/const argument kinds and higher-ranked scopes; wildcards
are independent and opaque constant expressions are not evaluated. Native comparison preserves
exact equality, possible unification and directional possible coercion as separate operations.
Format/derive observations retain optional/plural mappings and bounded, cancellable projection;
an originating ADT is not an identity for a generated declaration. The companion Outcomes own
targeted acceptance; the coordinator Outcome records broader E03 process journeys and Q01
qualification under their actual conditions.

**Implemented realization (Plan60a).** Related investigations use nonempty guarded seeds and selected
relations, an independent traversal domain and any/all seed attribution. Output source selection
does not prune supporting intermediates. One native worker and immutable snapshot serve one
bounded traversal; request-local adjacency is shared across seeds. Canonical declaration edges
retain structural roles and original occurrence witnesses, including parallel outer parameter
and field positions. Native declaration identities keep their original session, configuration
and generation; unsupported or unmapped facts remain explicit. Depth bounds describe the
question, while incomplete node/edge/provider examination prevents negative conclusions.
Settled compact graph answers retain guarded expansion and coverage in every presentation.
Traversal intersects selection with the active prepared world; inactive files are identified
as outside that world, while inactive/plural seeds refuse. The subsequent B01 extension adds
associated/supertrait/impl relationships, supported immediate block-local containment and
contextual field witnesses. Direct assignments distinguish read/write roles; aliases, borrowing
and interior mutation remain incomplete. Calls require actual native invocation resolution;
callback values remain uses and unresolved indirect calls remain unknown.
Native `NODE_TYPES` inspection and query-byte error locations remain source-only assistance.
Plan60a owns its original targeted local Linux evidence; the coordinator Outcome records
E03/Q01 engineering-process and assembled/static qualification. No new persistent graph or
universal identity follows.

#### 21.7.6 Bounded delivery and prepared snapshot ownership

Bounded search defaults to 20 results in fast arrival order. Stable ordering and source/query-guarded
continuations are explicit; arrival order has no reproducible continuation promise. Counts state
their unit and exact/at-least/unknown character. A limit can fulfill the requested demand without
exhaustive examination; deterministic ordering and exact totals can require additional work.
Native early stop, compatible preparation/parse reuse and cancellation precede unnecessary full
materialization. Existing native process/CPU completion ownership remains authoritative.

Compact results share verbose source/provider context and carry versioned typed locators usable
across invocations without a new result service. Normal paths remain readable and native fallback
bytes preserve unusual names. Source/provider/query premises govern their lifetime. Lightweight
lexical observations do not imply a full content snapshot; exact-source hydration checks selected
premises before inspection or preview. Stale selections refuse or require an explicit fresh
observation. Independent file snapshots never establish an atomic checkout-wide capture.

Typed search records can stream before computation completes: context declarations precede their
referenced results, complete records remain indivisible, and one terminal summary states coverage
and actual settlement. Missing terminal delivery remains incomplete. Consumer closure stops new
admission and settles admitted work, including no-match reads. No automatic detach, full-result
retention or second task registry follows. Existing non-search envelopes retain their behavior;
product scientific interpretation stays with its prepared owner. K/L/M and their dependencies own
implementation and verification of this selected target, separate from formal ADR acceptance.

> Decision: ADR-0233 (proposed Plan59 A11 native-analysis snapshot binding).

Typed native HIR/IDE investigation uses an explicitly prepared immutable snapshot context,
selected through the existing semantic session owner. It never silently supplies a second
live database for the same project/context. The separately prepared `pse-native-analysis`
binary and optional `native-hir` feature isolate its compatible 0.0.358 family from ordinary
host construction and source-only syntax0.0.360; only plain declared requests/results cross.
The existing prepared-tool and managed-service owners retain executable authentication,
original invocation custody, socket-peer checks, cancellation/completion and stop/drain.

Selected native facts include resolved method identity, original/adjusted types, read/write
reference roles and declared generic constraints. Unsupported fields remain unavailable;
rendered type names do not prove identity, blanket applicability or rustc acceptance. Native
snapshots refuse changed source/configuration/preparation premises and require explicit reprepare;
portable locators bind the original session because generation numbers may repeat across sessions.
Explicit native preparation selects whether build scripts/proc macros participate; selected
extended inputs use the original correctness generation and refuse when unavailable. The live LSP retains its distinct supported context and operations. Native SSR
capture/range search is not claimed where the selected public API supplies only matched text.
G11's isolated evidence establishes feasibility; G12 and P02 own production and assembled acceptance.

**Implemented realization (Plan59 K–M).** Working selection uses the native ignore walker and
grep matcher/searcher pipeline; index/revision selection retains Git object identity. Native
AST rules, Tree-sitter queries and written declaration discovery share the selected source
universe. Structured fields use their actual parsers. Managed Git supplies history operations
whose semantics are not supplied by the embedded Git owner. Source-only questions start no
analyzer or implicit dependency acquisition. File-level, node-level and entity-level witnesses
keep their different meanings through composition.

The native snapshot loads installed standard-library sources and the selected Cargo/VFS world
once during explicit preparation. Each request forks an opt-in cancellation group over shared
Salsa storage for that immutable input world. IDE and Rayon descendants inherit that group while retaining
independent local stacks, cancellation suppression and reset. Other request groups and ordinary
database clones keep their existing behavior. The exact source patches remain with their library
provenance. Original IDE definition/reference/implementation/symbol operations own navigation;
native visibility and use-path queries can prove a naming path at an unused destination. This
does not establish arbitrary trait applicability or compiler acceptance. Cancellation requests
the actual group and retains the CPU completion owner through settlement.

API refinement preserves the facade's original public route separately from the acquired
defining declaration. Original compiler units, target, features, build inputs and dependency
package bytes govern their association with each other and a prepared native world. Registry
identity alone cannot establish source-byte equivalence. Missing original byte evidence,
unexpanded external descendants and unsupported contexts remain unknown. Request-owned proof
contexts reuse acquired graphs/routes and recheck their original evidence at the final barrier.
Definition artifact/hash/context and original session/source guards survive portable references.

Typed defaults, repository configuration, selected requests and explicit flags have one merge
owner; implicit parser defaults do not override authored configuration. Producer demand is
independent of payload allowance, context, field selection and grouping. A bounded queue feeds
the native nonblocking output owner; consumer closure cancels admission and retains actual CPU
and child joins. Existence can be proved by one positive alternative while other coverage stays
unknown. Exact totals and complete absence require their own sufficient examination. The current
qualification boundary remains in §24.2; these mechanisms confer no new scientific qualification.

Correctness and prepared-tool construction bind checkout-selected actual compiler/rustdoc
executables before physical Cargo enters dependency directories. Observation, admission and
execution consume that same selection, with explicit physical selections and compiler wrappers
preserved. A rustup proxy evaluated later in a registry directory cannot silently select another
default compiler. Native source-build diagnostics use Cargo's rendered-stderr mode while artifact
records retain their JSON protocol and original capture owner.
