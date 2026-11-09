# Contract testing and developer workflows

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the
concurrent Plan66 working tree. Representative local and tagged supplier source
are **Interface-checked**. Proposed extensions below are **Proposed**; no tests,
tool preparation, database attachment or MCP configuration were executed.

## Existing deployment and test consumers

[The native integration fixture](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/tests/native_service/surrealkit.rs)
already runs SurrealKit against a Rust-owned server, stages generated schema,
seeds and frozen rollout inputs, checks structured reports, and verifies disposal.
It exercises direct Sync, authored rollout replay/parity and a drift control.
This is additional-leverage discovery, not a proposal for first SurrealKit use.

[The declarative suite](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/scripts/tests/fixtures/surrealkit/canonical.toml)
includes exact Decimal/bytes, deadline functions, endpoint enforcement, API
commit/refusal/rollback and an inspection actor. It explicitly follows an empty
Viewer create response with privileged readback. That concrete distinction is
more useful than a generic claim that the suite tests permissions.

[Tool preparation](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-host/src/surrealkit.rs) selects a
standalone rc.1 CLI with SDK4/gRPC. The build omits `mcp` and embedded backends.
Blueprint §20/§24 and [Plan68d](../../../../plans/68d-surrealdb-v4-developer-and-test-receiving.md)
own current lifecycle and receiving evidence.

## Capability families

| Mechanism | Origin and availability | Current-purpose hypothesis |
| --- | --- | --- |
| SQL, permission, metadata, behavior and HTTP cases | Pre-v4 testing foundation, enhanced 1.0 assertions | Express database-owned invariants near generated definitions |
| Actors and independent authorization readback | Earlier capability, retained | Reduce repeated authentication/probe setup |
| Sync/rollout/both construction and parity | Earlier prerelease; already locally exercised | Verify alternative schema construction routes |
| External `test --exec` | Later prerelease, CLI route | Reuse isolated DB/actors for application-side codec/reader tests |
| Structured report APIs and JSON | Available in rc.1 | Consume case outcomes without parsing terminal text |
| Modules, seeds, selection and templates | Pre-v4/1.0 tooling | Reuse coherent fixture ingredients where actually shared |
| Local project MCP and bounded progress | rc.1 source; unselected `mcp` feature | Structured access to existing schema-development workflows |

The [reusable test/settlement and progress contracts](../v4-tooling-contracts.md)
carry exact release details. The supplied [SurrealKit reference](../../../../libraries/surrealDB/SurrealKit.md)
original §§8–10 and addendum §§4/9–13 covers wider tooling.

## Research priorities

**First: extend declarative coverage where it deletes duplicate harness logic.**
Inspect current generated assertions and native contract tests to find repeated
record setup, actor signing, expected refusal and no-effect readback. Move only
the database-owned part into shared suites; preserve an independent application
reader as the oracle for client interpretation. Favor useful denial/visibility
and stored-value witnesses over merely repeating schema strings.

This loses value if a suite duplicates a Rust assertion without removing setup or
covering a different trust boundary. TOML cannot express every native value;
compare exact numeric/presence properties within SurrealQL or the real client.
Schema parity is a definition check, not a historical data migration test.

**Next: compare external suite composition with existing Rust fixture ownership.**
Run a narrow native reader/codec consumer against suite-provided endpoints and
actor tokens, instead of separately creating its logical database. This could
remove bootstrap and fixture duplication while keeping the external consumer's
independent interpretation. Compare whether the provisioning benefit outweighs
an extra subprocess, credential handoff and separate deadline domain.

The supplier's logical database lifecycle cannot establish the native fixture's
physical server ownership, connection drain or process settlement. The existing
root fixture may remain the best outer owner. An integration should delegate
only the inner database preparation that genuinely becomes redundant.

**Then: expose existing development facts through a useful structured interface.**
The rc.1 MCP surface could replace repeated agent shell parsing for schema status,
rollout lint and test authoring. Compare it with current native command outputs
before adding an agent server. Its process context and configured target do not
replace native operation/issuer authority. Confirmation fields are client inputs,
not independently enforced authorization. Broad `apply` access needs a concrete
consumer and appropriately scoped credentials.

## Questions before substitution

1. Which current Rust fixture setup disappears while its independent consumer
   and original service owner remain exercised?
2. Does the report distinguish infrastructure failure, case failure and incomplete
   cleanup? An outer successful Rust `Result` need not mean zero failing cases.
3. Do fail-fast, timeout, Ctrl-C, child exit and transport loss leave no live
   connection or suite database under the actual chosen composition?
4. Can an actual developer command consume supplier JSON or library reports
   directly without introducing another interpretation of the same result?

The tagged [runner](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/runner.rs)
has explicit `--exec` interruption handling and parallel task-abort paths. The
reference's general cleanup wording must be qualified against these paths rather
than promoted to universal physical-settlement evidence.

## Less relevant territory and handoff

Vite/Effect client generation, web templates, hosted Cloud MCP and container/Action
packaging do not supply an established missing current product capability. They
remain available if an existing consumer demonstrates a concrete benefit. No
frontend, remote agent listener or new multi-database product is assumed.

Prioritize the existing native contract lane, then a selected external consumer;
leave MCP behind demonstrable operator benefit. Use
[generation](schema-compilation-and-generation.md) and
[rollout](deployment-rollouts-and-migrations.md) briefs for their semantic owners.
No broad test campaign or new test framework was selected here.
