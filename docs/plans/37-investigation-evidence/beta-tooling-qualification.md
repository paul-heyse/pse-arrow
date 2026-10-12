---
title: Plan 37 beta tooling qualification
status: active
date: 2026-10-12
doc_role: evidence
doc_owner: docs/plans/37-surrealdb-lifecycle-and-integration.md
doc_topics: [current-work, storage, libraries]
doc_retention: while-dependent
---

# Qualified initial tooling boundary

[Plan37](../37-surrealdb-lifecycle-and-integration.md) owns current status and
acceptance. These are bounded tooling-foundation observations and selected outcomes; they do
not qualify persistent rollout, final scientific declarations or assembled Q1.

**Interface-checked:** Context7 `/surrealdb/surrealkit` supplied discovery leads;
exact beta.6 source commit
[`92ebc3cc89b99f00d21c7d2cbf5136d728367061`](https://github.com/surrealdb/surrealkit/tree/92ebc3cc89b99f00d21c7d2cbf5136d728367061)
supplies the actual CLI/config/test/setup/rollout contracts. The standalone release
is declared once in `.config/surrealkit.toml`. Acquisition checks the official
archive digest and executable digest/version, and records that commit, source
hashes, actual committed SDK resolution3.3.0 and official locked-release build
workflow. This does not attest a bitwise reproducible binary. The production
SDK/server remains its separately selected patched3.3.0 family.

**Implemented:** `scripts/surrealkit.py` adapts only registered disposable
namespace owners. Provisioning registration precedes account/namespace effects.
Private namespace credentials restrict every beta-created setup, suite and parity
child to that namespace. The resource authority owns generated schema provenance
and the complete frozen file set, separately from editable tool inputs. Ambient
connection configuration, symlinks, arbitrary test/setup/seed SQL, explicit actors,
file/HTTP inputs, coverage overrides and unowned changed rollout inputs refuse
before child execution. Initial tests are exactly the repository's authoritative
qualified SQL fixture. Their negative assertions require observed server-rejection
signatures; beta otherwise accepts an unfiltered transport error as an expected
failure. A successful SQL response also cannot settle an ASYNC event, which is why
arbitrary SQL is outside this initial subset.

The complete current generated declaration needs no schema/activation split:
it already contains definitions only. Beta's parser may introduce OVERWRITE and
normalize IF NOT EXISTS; direct installation supplies the independent behavioral
comparator. Its generated/frozen SQL hashes do not replace SCHEMA_DIGEST's schema,
codec and interpretation authority. Neither installation path activates the
canonical interpretation.

**Tested, 2026-10-12:** the admitted `scripts.tests.surrealkit_fixture_check`
journey with evidence destination
`/tmp/pse37-execution-SMD3bG/generated-oracle-v4` completed with exit0 and zero
failures against released3.3.0 RocksDB. The exact command and conditions are in the
[selective fixture evidence](operational-capabilities.md#selective-i5-fixture-evidence-2026-10-12).
The complete generated declaration was installed directly; seven expectations
checked deadline behavior, integral/max-u64 decimals, immutable bytes, enforced
endpoints and absent activation. Beta fresh sync/replay and catalog parity passed
15/15; parity compared1192 definitions. Namespace credentials refused another
registered namespace. Checked disposal removed all owned beta children and the
enclosing namespace, preserving the independent fixture until its own removal.

The beta parity client retains an independently authenticated SDK clone. An
explicit disposable `--tooling-fixture` service therefore admits three attached
sessions including the implicit session, versus two for ordinary simulator state.
The initial cap-two probe failed at the clone; this was a fixture configuration
correction, not permission to change the production session contract.

The independent source review found and prompted repairs to service-admission
publication, reuse after unresolved work, HTTP/actor overrides, editable input
provenance, suppressed coverage, unqualified negative assertions and arbitrary
asynchronous SQL effects. Subsequent source review found no remaining material
bypass in the qualified subset. Script tests exercise these refusals and resource
retention without treating mocked SQL as installed-schema qualification.

The direct probe additionally caught two oracle mistakes: line-oriented SQL CLI
framing and decimal JSON strings. Their failed invocations retain their original
outcomes. A further real cleanup control caught the pinned SQL shell reselecting
and recreating a just-removed namespace. Cleanup now removes the namespace from
an unselected root session and verifies catalog absence before disposal. The
previously recreated empty scope was removed using its archived exact ownership
and same service generation; its correction receipt does not relabel the failed
helper invocation. Unknown work remains protective until authentic settlement or
whole-owner drain.

Permission observations cover root/namespace-owner execution and foreign
namespace credential refusal. Database/record-user permission matrices, all actor
modes, arbitrary seed/custom SQL, asynchronous events, hosted/remote deployment,
representative performance and numerical scientific behavior remain unexamined.
Changed A declarations require renewed generated-surface qualification. The
maintenance adapter separately retains unknown SQL until its actual operation
owner settles it; CLI0, lock TTL and catalog parity do not supply that authority.

**Tested:** through `scripts/pse-env --resource-class light --`,
`python -m unittest scripts.tests.test_surrealkit scripts.tests.test_test_resources
scripts.tests.test_surrealkit_maintenance` passed97 controls with zero failures.
Subsequent maintenance probe preparation and bootstrap-readiness correction
expanded its focused module to22/22 passing controls. `python -m unittest scripts.tests.test_surreal_server`
passed200/200 after correcting recovery-profile classification: the explicit
`tooling_fixture` selection is immutable across recovery, including refusal to
toggle it. The earlier supervisor run had one error against the zero baseline;
the corrected full module passed. These are script mechanisms, not actual
persistent recovery or Q1 qualification.

## Frozen forward-chain control

**Tested, 2026-10-12:** `scripts.tests.surrealkit_rollout_fixture_check`
completed the disposable forward-chain journey in
`/tmp/pse37-execution-SMD3bG/forward-chain-qualified`, through the admitted
native-light route and private namespace credentials. Three ordered plans and
their three `rollout up --complete` executions succeeded. Applying the same
frozen chain to a fresh database also succeeded. An independent HTTP oracle
checked the complete generated catalog and preserved a value across the two
probe-owned additive declarations; catch-up produced the same value and added
field. These seven successful commands all had exact owned child drain.

The complete frozen set contains twelve files, including companion SQL and
snapshot inputs. Separate missing-file, falsified-file, fork and gap variants
each refused offline with exit1 and exact child drain. The registered enclosing
namespace was removed only after the successful journey settled. Earlier failed
oracle invocations retain their failure and protective ownership records.

This establishes the selected fresh baseline, additive order, catch-up and
frozen-input refusal controls. It does not qualify arbitrary data transformations,
failed completion repair, persistent deployment rollout or activation. The
exclusive maintenance adapter keeps those operation-specific settlement and
service-owner obligations separate from beta's chain bookkeeping.

## Exclusive reservation-revocation control

**Tested, 2026-10-12:** the corrected `owner-revocation` mode of
`scripts.tests.surrealkit_rollout_fixture_check` completed with exit0 against the
fresh registered `beta-revocation-v2` service, generation
`84c5f229-a798-447a-a751-bcdfae4c1174`. The command used
`scripts/pse-env --native= --resource-class light --`, the private root registry,
exact registered controls selection and pinned beta executable. Evidence lives
in `/tmp/pse37-execution-SMD3bG/maintenance-owner-revocation-v2`.

The observer saw the actual `running_start` barrier and the beta global lock
with a live 900-second expiry, then revoked only its captured reservation nonce.
The native composer remained alive; process death was not exercised. The beta
child reported `owner_lost` and its exact native scope drained. Only after that
drain did the test restore its original still-live reservation and drain its
listener. Status succeeded. One exact repair attempt with a 15-second bound
refused the still-live lock with exit1; no lock deletion or TTL adjustment was
performed. Beta supports `running_start` repair to failed metadata when it can
acquire authority; that transition does not settle the interrupted SQL step.

Ordinary open/start refused, and the actual operation callback refused to claim
the unfinished step settled. Pending nonce
`ba9a55e3476b485e8873cbcad667df37`, frozen input identity and SQL outcome remain
unknown; admission is closed, the exact service/children are physically drained,
and no activation occurred. The enclosing control is retained/incomplete/pinned,
not disposed as a successful rollout. Root independently checked all exact native
scopes and the selected service's stopped state.

The first probe failed before revocation because its observer queried beta
metadata before bootstrap completed. Its deadline also expired after the start
SQL had completed. Exact status, original manifest checksum, stored step
acknowledgements, generated catalog comparison and child/listener drain later
established settlement of that original start only. Its failed probe outcome
remains unchanged; no completion, activation or replay was used. The corrected
observer retries only exact transient beta metadata absence under its original
deadline; permission, syntax and other missing-table errors still fail. The
fixture's finite overall clock includes the observed bootstrap/DDL cost.

This establishes the disposable reservation-revocation and unknown-outcome
refusal foundation. Successful interrupted-SQL settlement, process-death
recovery, arbitrary transformations, persistent rollout and assembled Q1 remain
unqualified.
