# Intent-first CLI and Codex operating workflows

**Proposed.** Make `pse` the routine front door for development and production. A caller
supplies the intent, observes the effective selection, and gets a result reference. The performing
owner resolves the environment, stages intermediate work, settles resources, and owns release.
Developers can still edit source directly. Recurring changes that cross several source or
generated boundaries merit an action that understands the whole change.

This review inspects main at `9e301a03d` on the maintainer's Linux workstation. The assessment
uses core 3.4, its efficiency and test guidance, the pse-arrow binding, and process-simulator
profile 1.5 where model changes, test selection, or reporting can affect scientific assurance.
AP-01 and H4 preserve performing owners. H8 and H24 favor composing intent and retrying only
the failed unit. H16 and H27 govern retention and copies. PS-10, PS-12, and PS-13 preserve
verdict, result scope, and the shared model checks. Plan66 owns scientific acceptance, Plan68 owns
the database version cutover, and completed Plan67 supplies the custody and reuse baseline.
[Selected standard](../design_review/design_principles/standard.toml) ·
[Native host contracts](../authoritative_design/sections/native-host-and-engineering-tools.md#section-21-7) ·
[Plan67 outcome](../plans/67-storage-lifecycle-and-build-reuse.md#outcome)

## What the current interface already does

| Workflow | Current owner and route | Opportunity |
|---|---|---|
| Check, list, test, and rerun | `pse test`, `tests`, `check`, `inspect test-selection`, and `test --rerun-failed` use Cargo and Nextest selection, native admission, and original custody. | Define ordinary coverage clearly and make the actual lane and failure-recovery choices discoverable. |
| Run, observe, and retrieve | `run`, `history`, `wait`, `cancel`, `inspect invocation`, and `inspect log` distinguish original work, observation, and settlement. Output envelopes carry schema, budget, artifacts, and next actions. | Make the common result visible without reading raw paths or reconstructing JSON shapes. |
| Operate on project data | The captured `pse-project` binary owns project creation, branch and draft metadata, runtime requests, and portable pack inspection. Typed requests and operation IDs already carry meaningful authority. | Give routine product actions stable front-door routes over that owner and absorb caller-managed transport files. |
| Evolve source and database schema | `just codegen` stages Rust contracts, recorded results, and Surreal definitions before the full generator. Canonical maintenance observes installed definitions and separately admits an exact settled rollout for activation. | Compose common source changes from one typed request and expose an independently guarded database workflow. |
| Prepare or retire resources | Native fixture, tool, server, Study-module, and worktree owners exist. Ordinary Git removal plus exact external release is the accepted checkout route. | Present readiness and handoffs together. Keep resource cleanup with its original owner. |

**Interface-checked.** Bare `pse inspect test-selection --format json` and its explicit
`--workspace` counterpart each reported one intent-stage group with 36 unique package roots.
The current bare syntax selects Cargo default members. This observation establishes roots at
this revision. The admitted test population depends on features, target eligibility, profile
filters, and original fixture readiness. `pse result --help` produced a structured usage
refusal. `pse inspect invocation --help` identifies the supported passive result reader. The
environment guide still advertises the rejected spelling.
[Selection owner](../../crates/pse-host/src/cli/selection.rs) ·
[Nextest configuration](../../.config/nextest.toml) ·
[Environment guide](../dev/agent-environment.md) ·
[Project operations](../../crates/pse-runtime/src/bin/project/tools.rs) ·
[Canonical maintenance](../../crates/pse-operations/src/canonical_maintenance.rs)

## What Codex had to do by hand

**Measured.** A read-only scan covered the 48 hours from 2026-11-05 21:16:40 UTC to
2026-11-07 21:16:40 UTC. It selected session files by a project-root or project-sibling
starting directory and deduplicated keyed tool inputs and outputs by event kind and call ID.
Ninety-two files contributed 14,215 distinct input records and 14,210 output records. This
starting-directory filter does not establish every command's effective directory. The
table counts matching records. Nested cells, repeated output, diagnosis, and commands that
mention a pattern without executing it can contribute. Truncated or missing results limit
coverage. These counts establish where to inspect, with no failure-rate or time estimate.

| Input mentions | Records | Output signals | Records |
|---|---:|---|---:|
| `jq` | 1,015 | Host generation input mismatch | 210 |
| `direnv exec` or user-scope launcher | 747 | Manager membership or scope refusal | 54 |
| Host, native, or fixture preparation | 156 | Pinned compiler crash | 35 |
| Temporary directory or build-capture staging | 79 | Command or argument rejection | 22 |
| Ordinary Git worktree removal | 17 | jq shape mismatch | 7 |
| | | Compiler/KLU prerequisite unavailable | 3 |
| | | Unix socket pathname limit | 1 |

Four reconstructed sequences explain the cost better than the counts.

- A bare workload invocation received a manager membership refusal. The agent retried using
  approved checkout activation, a user scope, and a parent that waits while the native
  controller moves to its own workload. The launch mechanics are stable enough to package.
  A host generation mismatch has a different cause. It needs a matching seed or a checkout-local
  build and a truthful report of which happened.
- Worktree and invocation results required JSON extraction. One read tried to interpret a missing
  registration as a byte-array path. The worktree identity defect was subsequently repaired in
  Plan67. The wider lesson is to expose a selected resource and its available fields directly,
  with a typed unknown identity when the owner cannot authenticate it.
- The Study coordinator accepted a private output root whose derived frontend socket exceeded
  Linux's pathname limit. Recovery required draining the original coordinator and choosing a
  short root before reuse of its captured inputs. Default placement and early path checks can
  remove that sequence. This is placement evidence.
- A pinned compiler crash interrupted discovery. The CLI can preserve the failed stage,
  selected build intent, and useful bounded diagnostics so the operator can retry deliberately.
  The evidence does not identify the compiler failure's cause.

The probe used text signals in keyed tool records and kept only aggregates here. The raw session
records remain local. [Workload and scratch routes](../dev/agent-environment.md) ·
[Worktree regression and retirement evidence](../plans/67e-resource-retirement-and-storage-verification.md)

<a id="O01"></a>
## O01 — A front door organized around intent

Give routine work one public route per intent. Keep the parser-derived command catalog as the
source of syntax, argument relationships, availability hints, and examples. Include a brief
effect classification and owner route where that helps distinguish reading, preparing, applying,
and recovering. Correct stale guidance by linking to the actual parser and test the advertised
paths against it. A machine can then discover an action by capability and use its declared
arguments instead of scripting help text or learning the helper binaries.

Use clap `CommandFactory`, `Args` and `Subcommand` composition, `ArgGroup` for mutually
exclusive forms, and typed parsers for identifiers, input sizes, and OS-native paths.
Use `ArgMatches::value_source` when explicit input changes authority or precedence.
Schema discovery, ordinary help, and generated static completion should share the same tree.
Prefer stable completion where a human consumer benefits. Treat dynamic candidates and
`#[command(defer = true)]` as conditional enhancements. Deferred construction must still
produce complete catalog, schema, and completion views, and startup cost should justify the
change. Clap handles syntax and local value constraints. The performing owner checks identity,
authorization, and current state.
[clap guide §§2–11](../libraries/cli/clap.md) ·
[clap 4.6.7 derive reference](https://docs.rs/clap/4.6.7/clap/_derive/index.html) ·
[Current catalog](../../crates/pse-host/src/catalog.rs)

<a id="O02"></a>
## O02 — Resolve defaults once and explain where they came from

Use the selected checkout and session for developer operations. Use a uniquely configured and
authenticated project or store for product actions. Where identity remains ambiguous, return
the valid selectors and ask for the missing choice before admission. A profile can supply
preferences, but an inherited default cannot serve as consent for a consequential mutation.
Expose a redacted effective-settings view that distinguishes explicit input, approved checkout
settings, and fallback policy. The approved local execution mode is a workstation setting.
Scientific resource selection and production target identity remain owner policy.

The investigation owner already composes Figment layers and resets changed tagged alternatives
before merging. Extend that pattern where several actions share real configuration sources.
Use `Serialized::defaults`, `Toml::file_exact` or a source-tagged provider, and an overlay of
explicit CLI values. Use strict extraction and `find_metadata` for selected provenance.
`Tagged<T>` and `RelativePathBuf` fit fields where origin affects interpretation. `merge`
replaces arrays and recursively combines objects, so typed alternatives may need whole-value
replacement. Figment Global profiles override selected profiles. Declare the precedence
deliberately. Keep the native environment map for byte-exact environment and tool paths.
Simple settings with one source should stay with their current typed owner.
[Figment guide §§2–8, 10–12](../libraries/cli/figment.md) ·
[Figment 0.10.19 composition and extraction](https://docs.rs/figment/0.10.19/figment/struct.Figment.html) ·
[Current composition](../../crates/pse-host/src/investigation/config.rs)

<a id="O03"></a>
## O03 — Make all ordinary tests a zero-argument operation

`pse test` should run the complete ordinary workspace population, including ordinary integration
tests. Publish the effective roots, enabled capabilities, profile, and admitted test population.
Use the existing `--changed`, package, target, and filter selections for deliberately narrower
work. Expose separately named specialized lanes for scientific, release, and qualification
work. Use the existing Cargo declarations and Nextest profiles and inventory as inputs.
Check that current defaults and filters actually implement the intended split before adding a
new profile or changing parser syntax. Distinguish outside-lane tests from tests in the lane
that cannot be admitted. For the latter, name the unmet requirement and return a nonpass.

Preserve force-validation for actual Arrow-consuming closures and the native provider and fixture
contracts. Use the existing exact failed-rerun path while its original report is available. On
current-source rerun, an absent or ambiguous test identity remains a refusal. A passive inspect
view can explain selection without listing binaries or running tests. Success should require a
complete selected population and its original verdict, with settlement reported independently.
[Qualification contracts, blueprint §24.1](../authoritative_design/sections/operations-and-validation.md#section-24-1) ·
[Nextest profile](../../.config/nextest.toml) ·
[Plan67 test consolidation](../plans/67c-surrealkit-and-rust-test-consolidation.md)

<a id="O04"></a>
## O04 — Address results by identity and give retained bytes a purpose

Return an exact invocation or artifact reference alongside each outcome. The ordinary path is
history or a returned reference, then passive inspection, then a bounded detail only if needed.
Keep exact IDs and session-scoped `latest` under the existing resolution rules. Provide a
focused result view over the native envelope rather than a second wire protocol or open-ended
JSON-path language. A result reference can expose typed children such as failed cases,
configuration origin, log excerpts, and linked product outputs where the owner already knows
them. Export to stdout or an explicitly selected destination when a caller actually needs a
copy. Normal execution and retrieval need no output-directory argument.

| Class | Proposed contents and release condition |
|---|---|
| Working scratch and executable captures | Keep only inputs and intermediate bytes the original operation or a current borrower needs. Release after actual settlement and final readback commitment. Preserve unresolved custody for owner recovery. |
| Ordinary developer result | Retain an identity, reproducible selection and source context, terminal summary, required inventory facts, and meaningful owner links. Trial a seven-day expiry for this compact class. Start the clock after settlement. |
| Failure detail | Retain the exact failed identities for current rerun and a redacted bounded excerpt while the compact result lives. A separately requested diagnostic capture declares its consumer, byte budget, and expiry or release condition. |
| Protected product results | Retain the canonical owner reference. Product retention, leases, recovery obligations, and read protection govern the underlying data. |

An explicit hold names the consumer and release condition. A short-lived expiry witness can
distinguish retired results from unknown IDs while its own retention lasts. Trial retaining
only the identity and retirement disposition for seven more days. A typed expired result can
offer a fresh source selection when one is still available. Keep ordinary output to the
summary and selected excerpts. Full streams need the separately declared diagnostic use. Existing
`RetentionClass::eligible` is a starting predicate for disposable presentation. The owner
must recheck actual settlement and holds during release. Trial automatic expiry only for
explicitly approved regenerable classes. Preview and owner-issued release remain the route
for published generations. A dedicated SurrealDB index earns its place only when a named
cross-run query cannot be served adequately from compact owner records. Plan67 withdrew the
general persistent history pilot.
[Retention predicate](../../crates/pse-host/src/paths.rs) ·
[Plan67 invocation custody](../plans/67b-invocation-custody-and-developer-history.md) ·
[Plan67 retirement](../plans/67e-resource-retirement-and-storage-verification.md)

<a id="O05"></a>
## O05 — Bring routine production actions to the front door

Expose project discovery, branch and draft inspection, routine project writes, bounded data
queries, and typed import/export actions over the existing product operator. Start with the existing bounded
metadata and pack capabilities and verify the actual import or export owner before exposing
additional actions. A front-door action may dispatch a prepared product binary or use its
existing typed transport. Prefer the route that carries the complete request and original
acknowledgement without a permanent host dependency on scientific code.

Accept typed inputs through arguments, stdin, or an explicitly supplied document where the
document is itself the user's intent. Return the operation ID and authoritative receipt.
Where the owner permits, the front door can allocate an operation identity and reuse it for
recovery after an uncertain acknowledgement. The caller can still supply a meaningful key
for an external workflow. Preserve typed failures for stale premises, ambiguous targets,
duplicate identities with changed inputs, and unresolved outcomes. Bulk operations need the
owner's actual transaction or checkpoint boundary. Compare composed owner actions with a
generic SQL or shell escape. The former expose permissions, identity, and settlement as
structured facts, which are the obligations that make these actions routine.
[Product operator](../../crates/pse-runtime/src/bin/project/tools.rs) ·
[Native host and product separation](../authoritative_design/sections/native-host-and-engineering-tools.md#section-21-7)

<a id="O06"></a>
## O06 — Compose recurring model changes and schema operations

Start with adding a supported optional field and changing a field name or interpretation
with existing consumers. Compare two candidate scopes. The first improves the generator
invocation, affected selection, and review of changes from a human-authored declaration.
The second accepts a typed operation over a known declaration and prepares the complete
source-side change. Select the second for change forms with an unambiguous existing owner
and a validated impact model. Keep the first for custom algorithms, novel physical meaning,
or changes requiring editorial judgment.

For a supported operation, stage a preview bound to source revision and file identities.
Show the authored change, affected consumers, compatibility implications, generated changes,
and selected checks. Apply the exact reviewed preview through the source and generator owners.
Refuse a changed premise before writing and report completed stages and recoverable state if
publication fails. Use the existing generator ordering and equivalence checks. A field with
physical meaning still needs its units, role, validity, and the applicable shared model
checks. An operation should surface those choices to its author.

Expose installed-schema inspection, an exact rollout preview, apply, and recovery as
distinct operations under canonical maintenance. Bind the preview to the server, namespace,
database, predecessor marker, target declaration and relevant module identity. Require actual
rollout settlement and a fresh generated-target definition witness before activation, reusing
`SchemaMaintenance::validate_definitions` and `activate_definitions`. Treat source publication
and live activation as separate committed effects with a visible handoff. Compare the existing
owner and SurrealKit rollout route against a new wrapper by what it can actually prove.
Plan68 owns version-specific migration selection.
[Registry and generation](../authoritative_design/sections/schema-and-relations.md#section-4-2) ·
[Maintenance implementation](../../crates/pse-operations/src/canonical_maintenance.rs) ·
[Plan68 schema and authority](../plans/68b-surrealdb-v4-schema-transport-and-authority.md)

<a id="O07"></a>
## O07 — Remove repeated workstation orchestration at its source

Package the observed Codex scope launch sequence in the existing workstation bridge or launcher,
whichever can establish the authenticated parent and checkout environment once for ordinary
effects. Measure it on a cold linked checkout and an already prepared main checkout. Preserve
passive help and inspection even when the environment is unavailable. Classify host setup as
absent, stale, or mismatched. One explicit preparation action should be able to select an
equivalent seed or prepare locally, report that choice, and hand back the original effective
selection for continuation. A repeat should reuse the verified preparation. Compare a lightweight
bootstrap entry for the missing-host case with adding behavior to the existing launcher.

Give managed commands short owner-allocated scratch paths when they create local sockets.
Validate derived path lengths before service launch. Use `just` for fixed bundles whose
component owners already expose the intended effects, such as targeted dependency refresh
followed by host preparation. Keep typed admission, ownership, and retry logic in native owners.
For checkout retirement, retain ordinary Git removal and add owner previews or exact release
surfaces only where the observed sequence has a real gap. A composite mutation would reopen
the R-56 decision. Use the local Codex bridge and advice hook to surface canonical commands;
prefer fixing the operational route over adding instructions that restate it. Verify effective
hook and shell behavior on this workstation before proposing configuration changes.
[Activation and hooks](../dev/agent-environment.md) ·
[Bootstrap owner](../../scripts/bootstrap-host) ·
[Plan67 checkout lifecycle](../plans/67e-resource-retirement-and-storage-verification.md)

<a id="O08"></a>
## O08 — Actionable diagnostics and quiet progress from owner events

Extend the existing `ErrorRecord`, `RecoveryRecord`, and `NextAction` projection across
routine host and product workflows. Use typed thiserror variants for operation-specific context.
Use miette source labels, related findings, and diagnostic-source delegation to retain the
relationships when crossing the application boundary. Preserve structured clap usage context.
The current versioned envelope remains the machine contract. Graphical or narrated miette
rendering can improve human output. The miette JSON renderer can help inspect the diagnostic
projection, while application-specific identity, settlement and compatibility stay in the
native envelope. Recovery should provide exact safe argv or an artifact reference. The
redaction owner can omit advice whose operands cannot be disclosed safely.
[Diagnostic guide §§2–6](../libraries/cli/thiserror_and_miette.md) ·
[thiserror 2.0.21](https://docs.rs/thiserror/2.0.21/thiserror/) ·
[miette 7.6.0 diagnostic protocol](https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html) ·
[Current recovery projection](../../crates/pse-host/src/envelope/diagnostic.rs)

Use the existing indicatif search rendering as a starting point for foreground activity.
Choose indeterminate activity during discovery and known totals only where the owner can
account for them. Use a stderr draw target for an attended terminal, hidden rendering for
machine output, and `MultiProgress` only when several simultaneous states are useful.
Register bars before drawing, retire them as jobs settle, and keep logging independent of
hidden progress. Compare this with occasional phase lines for unattended observers. Make
terminal behavior testable with the existing harness or `InMemoryTerm` if its test feature
earns the dependency. The owner's verdict, capture, and settlement drive final presentation.
[Indicatif guide §§2–7](../libraries/cli/indicateif.md) ·
[Indicatif 0.18.6 MultiProgress](https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html) ·
[Existing draw target](../../crates/pse-host/src/cli/investigation.rs)

## Walkthroughs and adoption order

The interactions below describe target behavior. Command spellings for new actions remain
illustrative until the implementation plan reconciles the parser and its consumers.

| Walkthrough | Evidence that would support adoption |
|---|---|
| Bare ordinary tests, then rerun one failure | `pse test` reports the ordinary lane and complete selected population. An exact failed rerun resolves the original failure against current inventory and publishes a separate verdict. A missing ordinary prerequisite is visible as a nonpass. |
| Retrieve and expire a developer result | A returned ID opens a compact result and one selected detail. A settled disposable result expires under the declared policy. An active hold blocks release, and a known expired reference exposes its disposition. |
| Production action with ambiguous selection | Two plausible configured stores produce a typed choice before any write. A uniquely selected target receives the typed request and returns the original operation reference. |
| Model change across generated consumers | One supported field change produces a coherent source preview, regenerated views, and affected checks. A stale preview refuses before edits. A custom physical change remains with its author and existing checks. |
| Stale schema preview | A changed predecessor, target, or module witness refuses activation. The original owner can reconcile an interrupted application with the same operation identity. |
| Cold Codex checkout | Passive help works. A missing host or provider produces a precise preparation action. The supported launcher establishes activation and scope, prepares only what is needed, and resumes the selected operation without manually staging paths. |

Begin with O01, O02, and O07 where the observed discovery, activation, and prerequisite
friction can be removed without changing product contracts. Deliver O03 and O04 with O08's
structured recovery so a routine test is usable from request to expiry. Prioritize O05's
existing typed operators over adding new product effects. Prototype O06 in two supported
source-change forms, then extend the surface where the generator, current consumers, and
maintenance owner demonstrate the necessary guarantees. A new retention service, broad
automatic collection, or generalized patch engine requires a concrete consumer and its own
design decision. Verify the walkthroughs with focused parser, owner, and terminal checks;
measure speed or storage claims only for a named changed workflow.
