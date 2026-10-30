---
title: Plan59 CLI architecture and observability improvements
status: proposed
date: 2026-10-29
doc_role: review
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [architecture, design, libraries, environment, validation]
---

# Plan59 CLI architecture and observability improvements

## Combined judgment and assessed footprint

**Proposed:** retain the lean native front door, but revise its control, identity, outcome,
artifact-lifetime and diagnostic contracts before implementing the remaining packages.
Add simulation inspection as a coherent product capability behind that front door. Plan59's
selection, explanation, fixtures, compact output and engineering aids remain valuable;
its current detailed mechanisms do not yet compose safely in several ordinary cases.

This review assesses [Plan59 and its A–E companions](../plans/59-agent-cli.md) at source/plan
commit `890aa07f76fac5739cfc9811a47e823c326be663`, initially clean, on 2026-10-29.
Execution is recorded stopped/reverted; A01/E05 and the D02 trial retain their existing status.
No plan package, rule, ADR status, production implementation or existing service is changed here.
The [functional companion](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md)
owns new O01–O08 propositions; original CLI O01–O25 remain qualified by their original review.

The maintainer authorized reconsidering major choices. This is a combined library-informed,
first-principles plan improvement and functional review, not a broad diagnostic audit or
completion audit. Core3.4, efficiency heuristics1.0, test assessment1.2 and process-simulator1.5
apply through the repository binding. Scientific outcome interpretation, original units,
partiality and result/cancellation authority must survive presentation changes (PS-01/PS-10).

**Interface-checked:** repository paths, plan interactions and researched library contracts.
**Proposed:** all amendments below. The failure scenarios are reasoned counterexamples and
future acceptance cases, not newly executed regressions. No product tests, native-provider
builds, database changes, host attachments or performance campaigns ran for this assessment. Earlier
Plan58 qualification is not promoted to evidence for the proposed CLI.

## Architecture alternatives

| Complete alternative | Benefit | Burden/consequence | Judgment |
| --- | --- | --- | --- |
| Expand `pse-host` to link all product diagnostics/database/science | Direct typed access in one binary | Breaks independent construction, widens every hook/correctness adapter build and couples passive host diagnosis to product dependencies | Decline. |
| Small host front door; delegate product operations to their prepared native owner | Preserves cold host and current scientific APIs while giving agents one discoverable interface | Requires explicit delegation/schema and executable lifetime contracts | Preferred initial design; amend rather than replace host placement. |
| New lightweight frontend crate over host and product adapters | Can isolate command development from compatibility/tool bins | New crate/decision route and migration without a demonstrated current benefit | Conditional if dependency or build isolation proves useful; not needed merely for file organization. |
| Required database/daemon for all invocation state and observation | Central querying and remote access | Adds availability, startup, migration and lifecycle requirements to cancellation/basic tooling; canonical science still has its own authority | Decline as a baseline. Revisit for a concrete shared/remote consumer. |

Use existing parser/library facilities and typed domain operations. A derived command catalogue
is useful discovery; it must not become a universal product policy registry. Existing systemd,
canonical and diagnostic mechanisms remain authorities within their respective scopes.

## Recommended amendments

<a id="I01"></a>
### I01 — Separate invocation, scientific and observation identities

**Necessary contract correction · A03/A04, B01/B05, product observation addition.**
Plan59 calls its supervised command a run, while current runtime already has scientific
`RunId`, `AttemptId`, canonical run/attempt keys, diagnostic `OperationId` and process identity.
`workflow::run::RunHandle` and `invocation::InvocationEvidenceDocument`
already preserve several of those relationships; use them rather than recording scientific
correlation independently in the host.

Introduce a distinct CLI invocation identity and explicit references in envelopes/records.
One invocation can contain many scientific attempts; an observer invocation need not own them.
Keep process outcome, scientific outcome, observation completeness and physical settlement
separate fields/states. This improves AP-04/AP-05, DP-02/DP-04/DP-05 and prevents a successful
CLI observation from becoming a claim of scientific success.

**Closure:** a study with retries and two independent observing commands retains exact existing
identities; partial observations never imply absent execution or grant cancellation.

<a id="I02"></a>
### I02 — Keep control of existing work independent of broken current sources

**High-priority architectural correction · A05, B03, proposed ADR-0233 amendment.**
A05 rebuilds top-level effectful verbs and fails them when compilation fails. `cancel` is
effectful and is not among its exemptions. B03 requires later-process cancellation. Thus an
agent's own invalid host edit can prevent cancelling its already-running work; separate
worktrees do not address that case.

Keep fail-closed rebuild for launching new work. Read/wait/control of an existing invocation
must use a compatible prepared control protocol and recorded generation without rebuilding
current sources. Resolve cancellation identity before mutation; unsupported record versions
remain explicit and retain a documented recovery route. Pin the original supervisor/finalizer
generation for its real lifetime. Do not blindly trust arbitrary executable paths from a record.
The one-reexec marker is loop prevention, not evidence of a coherent build: sources changing
during construction must not cause new work to run under a misattributed fingerprint.

This narrows R3's application; it does not reverse the selected fresh-host requirement for new
effects. AP-02/AP-06, DP-18 and H24/H25 favor the smaller independent recovery boundary.

**Closure:** start detached work, make host sources uncompilable, cancel from another process,
and prove original physical settlement. A mismatched invocation refuses. No rebuild is attempted.

<a id="I03"></a>
### I03 — Make result finality and observation failure explicit

**Necessary contract correction · A03, B01–B04, C02.**
The plan's headline “managed exit0 only on reconciled pass” conflicts with its intentionally
successful `launched` and wait-timeout responses. Define command success separately from the
subject state. `wait` timeout means the observation interval ended while work may continue;
it is not a workload timeout. Preserve the exec-adapter's raw child exit contract.

Represent child termination, supervisor failure, settlement and scientific acceptance separately.
An `ExecStopPost` finalizer may record that the supervisor crashed without claiming that all
separately placed workload scopes drained. Foreground source currently settles workload,
operation and owner resources plus child reap/permit release; preserve that full obligation.
Where finalization cannot establish settlement, publish an explicit unknown/failed settlement
state instead of an ordinary completed run.
The current `manager::foreground_placement` accepts scope units, so B03's service-hosted
supervisor also needs its planned real placement/admission journey before claiming reuse works
from a service context. This is an implementation prerequisite, not an existing passing path.

Reconciled test verdicts require the actual selected binary/test population, setup outcomes and
terminal runner evidence. A generic log fallback is diagnostic evidence, not a substitute
reconciled pass. Preserve an incomplete outcome when required stream evidence is unavailable.
AP-04, DP-03/DP-18/DP-22 and PS-10 govern these distinctions.

**Closure:** launch acknowledgement, nonterminal wait, actual workload timeout, test failure,
missing records, supervisor SIGKILL and partial drain all produce distinguishable responses.
Attempt every independent cleanup even after one fails, retaining the original failure too.

<a id="I04"></a>
### I04 — Publish the existing diagnostic model without losing information

**Necessary preservation plus enhancement · A03, B02, new product inspection package.**
A03's proposed miette projection omits the ordinary cause chain and existing native structured
facts. `pse_diagnostics::TypedDiagnostic::diagnostic_facts` supplies sources, locations, rules
and observations. Source `HostError::Settlement` also needs both operation and cleanup failures
exposed; changing its code alone does not preserve those children.

Use one native diagnostic interpretation, adapted into a versioned presentation record.
Include standard error sources, diagnostic delegation, source identity and resolved locations,
related errors and structured facts where supplied. miette is a diagnostic protocol/presenter,
not the semantic owner of every native failure. Use thiserror for typed error ownership.
Preserve machine-readable detail while giving a short useful summary and explicit next action.

A custom Serde error projection is reasonable: exact miette7.6.0 source research confirms its
JSON handler's URL escaping issue and offset-oriented labels. That does not justify flattening
causes or implementing another graphical renderer. Read spans through the source interface,
including inherited source context for related diagnostics. AP-01/AP-04, DP-01/DP-21 apply.

**Closure:** non-ASCII labelled input, nested causes, related diagnostics, multiple settlement
failures, secret-bearing settings and clap usage errors retain actionable structured meaning.

<a id="I05"></a>
### I05 — Separate semantic records, opaque streams and rendering budgets

**Necessary contract correction · A03, B01/B02, schema generation.**
Schemars1.2.2's `RawValue` implementation delegates to the arbitrary JSON value schema.
Validating an envelope containing raw Cargo/nextest records therefore does not validate those
records' semantics. Retain typed normalized outcomes/failures and qualify each extractor's
actual upstream stream separately. Raw records can remain opaque diagnostic artifacts.

Keep version/identity/state/coverage core fields present. Define `--fields` as payload
projection with explicit omitted/projected information and a matching view schema. A budget
never cuts serialized JSON bytes or suppresses a required terminal state. Page large results
or return a reference to retained detail. A renderer and a transport are not scientific owners.

Tee/output processing needs independent readers and an explicit disk/queue/error policy.
Slow terminal rendering or optional telemetry cannot block scientific progress indefinitely.
Coalesce or drop optional observations with disclosed loss; required test-verdict data needs
its own completeness path. Futures streams/Sinks help compose this, but cancellation/drop
does not establish original native completion. AP-07, DP-10/DP-18 and H26 apply.

**Closure:** oversized lines, a full output destination, a broken pipe and a slow consumer
cannot deadlock child settlement or produce malformed JSON/a false reconciled pass.

<a id="I06"></a>
### I06 — Add a coherent simulation inspection and escalation capability

**High-value functional expansion · A06, B viewing, D product delegation; new package needed.**
Implement functional O01–O04 through existing `tooling::inspection`, diagnostic statements,
study observation, capture, frozen analysis and review export. Show owner phases, related
operations, exact scientific references and coverage; make numerical diagnostics explicit
effectful operations distinct from passive inspection.

Keep local checkpoints useful without a database/backend. Canonical scientific reads use their
existing capability. The host delegates typed product requests to a prepared product owner;
do not move runtime/model/Arrow into the host to decode them. Ordinary progress can use
indicatif over copied state; tracing layers and optional OTLP retain causal attribution.
An initially small one-shot snapshot and follow view is sufficient; no mandatory full-screen UI.

Expose remote `LiveNumericsUnavailable` honestly. If remote current numerical checkpoints are
selected, recognize that another local process also cannot acquire an `Arc<Progress>` from
the original `RunHandle`. Existing process checkpoints and terminal canonical history are
directly exposable; cross-process live study samples need an owner publication bridge too.
For any selected live-study bridge, assign publication/retention/sequence policy to the product state
owner, not the CLI ledger. Preserve already available terminal history and cursor semantics.
Live OTel traces are useful navigation, not a complete durable numerical log.

**Closure:** preparation wait, shared producer, silent foreign work, publication, cancellation
and drain can be inspected without guessing. Closing an observer leaves scientific ownership
unchanged. Optional capture/backend failure is independent from scientific success.

<a id="I07"></a>
### I07 — Preserve the exact selection and actual fixture owners

**Necessary current-baseline correction · C01–C05, A05, E01.**
C04's hand-plumbed baseline is incomplete. Current `native_fixture.rs::NativeTestCommand` builds
and captures source metadata first, lends `PSE_NATIVE_CANONICAL_FIXTURE`/`FixtureSelection`, and
uses `ChildExitPremise` to protect state after signals or unknown scientific joins. Other
targets, including `native_study.rs`, still consume helper/module inputs and construct their
own service. Integrate both real modes: runner-owned service borrowing and target-owned
service construction. Routing every database target through an extra runner service could
create an unused service while omitting the target's required preparation inputs. Do not add
a global fixture-permission table or duplicate either lifecycle.

Normalize package, target kind/name, features, platform, filter and ignored-test policy once.
Adding all targets' required features into a union can erase absence/refusal controls; the
current justfile explicitly has feature-absence cases. Group only compatible selections.
Rerun identities include binary/package/target and test, not a potentially ambiguous test name.
Explain selection and preparation separately so listing/help does not accidentally prepare work.

Capture/pin actual immutable helper/tool outputs before later Cargo selections. This directly
addresses the current-session overwritten-xtask failure. A cached path or `current` pointer
alone is not immutable selection. AP-02/AP-05, DP-04/DP-19 and H19 apply.

**Closure:** duplicate test names across binaries, absence controls, setup failure and a later
different-feature build preserve the intended selection. Fixture cleanup remains with the
actual owner even if the invocation record is terminal.
Exercise both service ownership modes, including preparation inputs for a target-owned study.

<a id="I08"></a>
### I08 — Make retention and worktree removal follow actual users

**Necessary lifecycle correction · A04/A05, B03, E01.**
A05 protects generations referenced by live ledger entries, but supported adapter/boundary
routes can be unrecorded. `correctness.rs` exports `CARGO` and `PSE_CORRECTNESS_EXECUTABLE` from
the current executable: a child may need that generation later. Ledger-only scanning cannot
establish that old generations have no users.

Use a small generation-use lease/pin protocol covering every supported entry that can retain
an executable, including supervisor/finalizer and compatibility adapter paths. Prune only
after those actual users release it; do not invent a global process scanner. Keep current and
previous as convenience retention, not a proof of safe deletion.

E01's clean/merged worktree removal checks also need active invocations and referenced artifacts.
Refuse removal with live users; an explicit stop-and-settle operation may precede removal, but
`--force` must not silently erase an executing checkout. Run-log retention may not remove
active state, uncertain fixture evidence or required finalizer generations. Separate disposable
presentation logs, scientific data, probes and protected artifacts instead of one age-based GC.
Worktree creation also needs an honest partial outcome: Git creation can succeed while host
preparation fails. Report the existing checkout and failed preparation rather than calling it
ready or deleting a checkout another participant may already be using.
DP-18/DP-20 and H23/H25 apply. No existing cache/data deletion is authorized by this proposal.

**Closure:** an unrecorded long adapter, finalizer, warm worktree run and clean checkout with
active work all survive ordinary pruning/removal attempts. Terminal log pruning cannot erase
the independent fixture owner's retained-state decision.

<a id="I09"></a>
### I09 — Delegate grammar and configuration without duplicating policy

**Alignment improvement · A02/A04/A06, D01/D02/D07.**
Retain clap derive/builder composition, typed selections, parser introspection and provenance.
Use Figment providers/metadata where they replace handwritten settings merging; CLI explicit
values must override defaults without silently writing every default into an overriding layer.
Keep loaded environment maps explicit; the separate direnv effort remains untouched.
Clap ValueSource cannot distinguish a manually exported variable from one inherited through
direnv. Report it as caller environment. Derived PATH values need contributing-layer context;
unevaluated local configuration remains pending. Figment's built-in Env provider reads globals
and stringifies lossily, so retain byte-exact activation and use named captured-input providers
only for a worthwhile typed configuration subset.

A serialized clap catalogue cannot generally reconstruct arbitrary typed parsers, validators
or dynamic semantics. Keep delegated parsing authoritative in its owner. Use owner-generated
schemas/help for discovery, with actual executable/schema version association; validate before
effects at that same owner. Compare narrow external-subcommand forwarding with the proposed
generated federation before committing to a full serialized parser reconstruction. Shared
`#[path]` code is a source-sharing mechanism, not proof of semantic equivalence.

Schema generation is explicit development publication. Help/status must remain useful with
unprepared richer tools. AP-01/AP-03/AP-04, DP-01/DP-13 and H4/H27 favor delegation over mirrors.

**Closure:** a delegated typed parser rejects the same input through direct and federated
entrypoints. Stale discovery data never silently dispatches a different meaning. Explain and
execution consume the same resolved configuration while redacting secrets.

<a id="I10"></a>
### I10 — Keep engineering aids useful without fragile side effects

**Preservation/enhancement · D01–D08, E02/E03.**
Retain exact dependency/API lookup, structural reading and targeted document checks. The API
cache key needs source identity, target/toolchain and the actual feature context, not only
crate/version/features. A probe needs a genuinely isolated target/build context as well as a
seeded lock; a copied workspace lock does not freeze every changed scratch manifest resolution.
Preserve the original workspace lock and report scratch resolution explicitly.

D08's private index isolates proposed tree construction, but HEAD compare-and-swap and shared
index refresh are separate transactions. Preserve concurrent same-path staging and unrelated
staged changes. Before adoption choose a concrete tested Git transaction strategy; if HEAD
advances but refresh cannot safely apply, report committed-with-unreconciled-index rather than
retrying the commit or overwriting concurrent staging. Whole-file formatting must preserve a
concurrent edit and hunk-mode intent. Do not claim private index alone solves all concurrency.

Keep changed-doc checks scoped to the named change and classify external/historical issues
without initiating cleanup. Handoff records link authoritative status and immutable invocation
evidence; they are not an additional plan ledger. DP-01/DP-18/DP-23 and H26 apply.

**Closure:** concurrent same-path staging, moved HEAD, deleted/renamed paths, formatter failure
and a failed post-commit index update all preserve work and report whether a commit occurred.

## Recommended dependency and acceptance changes

<a id="I11"></a>
### I11 — Scope discovery and early parsing to the consumer's request

**Useful extension and contract preservation · A02/A03/A06, D07.**
Offer command-path and argument-candidate queries with prefix/page limits alongside the full
catalogue. Report candidate source, freshness and whether the list is exhaustive. Missing
metadata does not mean no valid packages, and custom parsers may accept an infinite language.
Full clap build/export legitimately expands deferred subtrees; scoped help need not hydrate
all dynamic value domains. No measured speed claim follows (AP-07, H5/H26).

Early format detection must stop at raw adapter/child boundaries. A child's `--format` or
`-h` must not become a host option. Normalize compact NDJSON separately from retained raw
records: RawValue can carry pretty-printed internal newlines, so raw JSON validity alone is
insufficient for one-record-per-line framing.

**Closure:** a fresh checkout without cached metadata gives useful syntax and explicit unknown
candidates; `pse run -- child --format json` preserves child tokens; pretty raw records cannot
break NDJSON framing. Share resolved operation values between explain and execution.

### Integrated order

Settle I01–I05 first: identity, control of existing work, terminality, diagnostics and output.
Then deliver one small foreground/detached invocation journey with result/wait/cancel and
real process settlement. Include generation pinning before pruning or worktree removal.
This establishes B's usable contract without waiting for the entire command catalogue.

In parallel develop the product-side passive observation interface and reconcile C's actual
fixture/selection owners. Integrate one real test/fixture journey and one local simulation
observation journey before expanding the D verb catalogue. Follow with bounded streaming,
diagnostic escalation and compact handoff. Migrate current documentation/consumers and delete
replaced wrappers with each usable capability; historical records are not a cleanup backlog.

| Representative acceptance | What it settles |
| --- | --- |
| Broken current host sources while cancelling an existing detached invocation | Independent control protocol and original generation lifetime. |
| Supervisor crash with separately placed descendants and a retained fixture | Observation/command terminality is distinct from physical/scientific settlement and disposal. |
| Two feature selections overwrite mutable Cargo output | The consumer executes its selected pinned artifact. |
| Same test name in two binaries; selected feature-absence control | Exact selection, result matching and rerun preserve actual meaning. |
| Local observer, remote ongoing attempt and retained terminal progress | Honest live/history/coverage distinctions and passive ownership. |
| Slow NDJSON reader, broken pipe, full log storage, absent backend | Output handling cannot deadlock settlement or invent a complete result. |
| Quoted URL, non-ASCII location and multiple nested/related errors | Library-backed diagnostics retain original causes and usable structure. |
| Clean worktree with a live run; old generation used by an unrecorded adapter | Pruning/removal follows actual users rather than ledger absence. |
| Concurrent HEAD/index/content changes during a private-index commit | No lost staging, duplicate commit or ambiguous success. |

These are focused closure scenarios for later execution, not a mandate for a broad benchmark
or scientific reference campaign. Simple parser/source-model tests complement actual native
journeys; snapshots alone cannot establish physical ownership, stream fidelity or frontend use.

## Authority changes and remaining investigations

I02 changes the proposed R3/ADR-0233 control/rebuild wording; I01/I03/I06 refine the applicable
§21/§23/§24 contracts through the normal decision route if selected. A new crate, required
service or stop-authority change requires its actual scoped decision/review. No accepted ADR
is amended by this review. Operator-accepted R1–R5 are not silently rescinded.

Before detailed implementation, resolve the compatible control-record/generation protocol,
delegated product executable/schema interface and Git index reconciliation strategy. Remote
live numerical publication and RMCP remain separately selected extensions with explicit
consumer/retention requirements. The preferred baseline requires neither a new database for
host invocations nor an always-on monitoring service.

The functional companion owns capabilities and library research; this document owns proposed
amendments. Plan59 remains the single eventual status/disposition owner after an authorized
plan revision. No duplicated mutable implementation ledger is introduced.
