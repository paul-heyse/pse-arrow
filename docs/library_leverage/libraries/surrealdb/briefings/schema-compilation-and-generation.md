# Schema compilation and generation

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283`, including
the concurrent Plan66 working tree. Representative source and selected upstream
interfaces are **Interface-checked**; all suggested changes and benefits are
**Proposed**. No compiler corpus, generator or live database was executed.

## Current responsibility

[The native generator](../../../../../crates/pse-codegen/src/codegen/surreal.rs)
derives schema declarations and exact native codecs from authoritative declarations.
Its compatibility digest includes generated DDL, codec description and refusal
classification. This already supplies a substantial shared contract; counting
generated lines does not demonstrate duplication.

[The prepared SurrealKit owner](../../../../../crates/pse-host/src/surrealkit.rs)
selects rc.1 and builds with `--no-default-features --features cli`, using the
locked SDK4 gRPC integration. It does not select the embedded engine needed by
offline compilation. Installed-definition observation and generated-contract
fixtures exist; their live status belongs to [Plan68b](../../../../plans/68b-surrealdb-v4-schema-transport-and-authority.md)
and [Plan68d](../../../../plans/68d-surrealdb-v4-developer-and-test-receiving.md).

The responsibility to improve is agreement among intended schema, generated
declarations/codecs and what the database actually installed. Blueprint §4 and
§20 own current authority. Breaking changes can revise placement; they do not
make a second independently editable model a useful default.

## Capability map

| Mechanism | Origin and availability | Useful current application |
| --- | --- | --- |
| Live introspection/typegen | Pre-v4 foundation; selected CLI route | Read normalized installed schema rather than scrape display strings |
| `compile`, `CompileModule`, `CompileOpts` | Later 1.0 prerelease; backend feature change needed | Interpret generated SQL with the engine in a disposable local compilation |
| V2 structured kinds, presence and diagnostics | Later prerelease; draft, version-qualified | Check read/create/merge/content semantics and generator assumptions |
| `semanticDigest` | Later prerelease; limited represented semantics | Compare compiler and live projections under the same interpretation |
| External emitters and managed publication | Later prerelease; custom Rust emitter required | Derive useful consumers and remove stale generated artifacts |
| `prepare_schema_sql` and selection APIs | Existing supplier machinery | Avoid independently reproducing Sync-specific declaration preparation |

Exact upstream conditions live in [the reusable tooling contracts](../v4-tooling-contracts.md),
especially engine-semantic compilation and semantic-document/emission. The supplied
[SurrealKit reference](../../../../libraries/surrealDB/SurrealKit.md), original
§7/§9 and addendum §§1–3/10–13, supplies broader orientation. Its beta.6 SDK3
statements are historical; the rc.1 addendum applies to the selected tool.

## Promising research directions

**First: compare the engine's normalized contract with the generator's intent.**
Feed the generated schema into an explicitly prepared compiler, retain diagnostics,
then compare its semantic projection with live introspection of a fresh native
fixture. This could expose declaration/codec disagreements before application
tests and consolidate local normalization logic. Begin with field kinds, presence,
relation endpoints, permissions and installed functions actually consumed today.

The discriminating question is whether v2 represents the complete selected
contract, including v4 named types and the study extension. A smaller semantic
projection can supplement current validation but cannot replace the existing
schema/codec compatibility identity. The local patched server may also normalize
or admit definitions differently from the compiler's embedded engine.

**Next: use operation-shaped generation where a current writer benefits.**
Derive separate inputs for creation, merging and replacement rather than forcing
every operation through a read-record shape. This can remove manually interpreted
omission/default/removal rules at existing mutation boundaries. Native presence,
exact unsigned/decimal values, bytes and typed identities must survive the selected
codec; a structurally convincing Rust `Option` is insufficient evidence.

This direction loses value if current writes intentionally construct explicit native
values and already centralize their policy with little residual code. A bundled
TypeScript emitter supplies no automatic Rust implementation. Compare the total
new emitter/adapter burden with the specific code it would delete.

**Then: consolidate generated-file publication selectively.** A custom external
emitter could delegate manifest ownership, unchanged-byte handling, stale-file
removal and freshness checks. Use a representative existing generated family to
compare it with the repository's current generator tree. Do not insert a second
publication owner around the same files. The useful alternative may be borrowing
the model rather than adopting the subprocess layer wholesale.

## Questions that decide priority

1. Does strict rc.1 v2 represent every definition needed by the installed witness,
   and does the patched-server result match the compiler for that selected schema?
2. Which existing hand-maintained mapping disappears if a downstream consumer
   uses structured kinds/presence, and which scientific policy still needs its owner?
3. Can one generated mutation family preserve NONE/NULL/omission, exact numbers
   and endpoint identity without additional reinterpretation?
4. Does the managed-output route simplify the complete generation lifecycle,
   including failure, interruption and codegen drift?

## Limits and handoff

Prioritize semantic cross-checking before a new general-purpose emitter. The
effect-surrealkit example demonstrates a downstream composition, not a reason to
add a TypeScript product. Vite, Action, container and CLI-component packaging have
no independently established advantage for these current Rust consumers.
Template/configuration variables need an explicit identity if deployed SQL differs
from the compiler inputs. No present consumer was established for ambient
substitution during these representative reads.

Follow [rollouts and migration](deployment-rollouts-and-migrations.md) for data
effects and [contract testing](contract-testing-and-developer-workflows.md) for
runtime witnesses. Discovery has not selected a new semantic owner or qualified
the compiler/server composition.
