# Study execution and database boundary

## Scoped baseline

Inspected 2026-10-28 at `64a217de2`, including uncommitted inline-child corrections and
tests in operations `canonical_transport.rs`, `canonical_transport_tests.rs` and
`transport_lifetime.rs`. The concurrent Plan57j checkpoint and recorded receipts were
read and preserved. Standard: Core 3.4, heuristics 1.0, test assessment 1.2,
process-simulator 1.5 and current binding. This coordinator-integrated account incorporates
the read-only transport researcher's diagnostic contribution. No tests, builds,
services, debugger sessions or measurements ran for this audit.

The whole-account baseline remains `87f2eb9050ae24976956628c16e0e7376190be7a`.
This page refreshes only selected Plan57d–f fixture, route, custody, process and boundary
evidence. It allocates no new DTA identity and does not revalidate the entire population.

## Selection and claim correspondence

Primary homes: [DTA-050](evidence-routes-and-execution.md#dta-050) and
[DTA-058](oracles-and-acceptance.md#dta-058). AP-02/04/05/06, DP-21–24 and test
assessment §§3–6 apply.

`.config/nextest.toml` excludes `managed_primary_` by default. The dedicated selection
in `scripts/native_tests.py` overrides that default and intersects the caller selection
with the managed partition; `test_requirements.py` declares that role. An ordinary
passing linked selection therefore cannot prove managed identities ran merely because
its requested expression included their names. Actual selected/executed identities
and route determine evidence. Plan57j now corrects the twenty-case overclaim and records
the empty ordinary selection and subsequent dedicated failures. This is a scoped
manifestation of existing concerns, with a corrected account and executable dedicated
route as material counterevidence, not a diagnosis that the tests have no route.

## Fixture identity, partial construction and drain

Related home: [DTA-055](doubles-and-coupling.md#dta-055). AP-01/02/03/05/06/07,
DP-15/18–21 and test assessment §§2/3/5/6/8 apply.

Recorded failures searched for storage process metadata in a receiver directory and
invoked a receiver program for storage-controller operations. Later managed worker
failure also encountered an active launch reservation during registry settlement.
These are setup/routing/cleanup observations, not equation or solver-result failures;
setup refusal cannot count as a passing negative scientific test.

Current filesystem-backed tests create distinct storage and receiver directories,
omit receiver-local process metadata, preserve receiver placement, and challenge
missing/changed/foreign controller bindings. The current routing fixes are concrete
counterevidence against claiming that the historical conflation remains unchanged.

`ManagedStudyFixture` registers controls, private nonce-bound files, worker identity
and a fixed drain deadline. Its failure path signals the original worker; Drop attempts
registry settlement. Refusing an active launch protects against treating an absent
recorded PID as proof of drain. Construction, partial startup and final settlement
must nevertheless be assessed as one fixture lifecycle. Production authority diagnosis
belongs to the production account, without another test finding for the same cause.

## Physical future contracts and composed acceptance

AP-02/03/06/07, DP-10/19–23, H2/H11/H17/H20/H24/H27 and PS-10–12 apply.
Existing debugger captures identify different composition sites: main-stack overflow
in Streamer construction through runtime activity, and Tokio-stack overflow submitting
canonical protection through nested activity/response ownership. Neither proves an
upstream runtime defect.

Current dirty inheritance tests directly challenge the repaired mechanism: a 64 KiB
child traverses both wrappers with bounded added state, while separate controls cover
Ready, Pending, repoll, panic, absent/unpolled context, owner release and ambient
restoration. Thus an allegation that all local controls are shallow is unsupported.
Those controls still do not establish complete managed construction, native entry,
sixteen-owner execution, publication and physical drain.

The following existing logs are in `build/qualification/57df-current-acceptance-20261028/`.
The failure target is zero; this audit did not execute their commands.

| Command/mode and receipt | Actual recorded result | Evidence boundary |
| --- | --- | --- |
| `just unit-package pse-operations 'test(inherited_)' --features canonical-tests,canonical-maintenance`, force-validated; `inherited-custody-controls-clean.log` | 12 passed, zero failed, 460 excluded | Focused inherited custody, not managed execution |
| Expanded custody/protection selection; `inherited-custody-and-protection-controls-corrected-macro.log` | 41 passed, zero failed, 431 excluded; one historical compile warning | Selected lifetime/protection; warning subsequently removed |
| Dedicated managed route; `managed-primary-owned-stack-probe.log` | Zero passed, one failed, 1716 excluded | Actual composed failure plus launch-reservation refusal |
| Worker diagnostic route; `worker-owned-stack-probe.log` | Worker exit 134, main-stack overflow | Original worker failure, not acceptance |
| Validated worker build; `inherited-custody-worker-build.log` | Build completed | Compilation only |

The separate two-case managed retry and the single debugger probe are different
observations, not independent successful acceptance campaigns. Earlier build failures,
interrupted runs, corrected controls and recovered drain retain their original scope.

## Consequences for the proposed boundary review

No direct Python/database alternative has been implemented or tested here. Missing
tests for an unselected architecture are not current defects. Candidate verification
must discriminate lost acknowledgements, duplicate commands, worker generations,
cancellation versus physical completion, final stream completion, notification catch-up,
restricted actors, large payloads and database-unavailable recovery. A permissive peer
cannot establish the effective server's connection-bound sessions or LIVE replay.

AP-04 distinguishes scientific authority, scenario intent and observed outcomes;
AP-06 preserves local controls and their explicit dependencies; AP-07 examines repeated
failed preparation and composed future state without weakening actual process isolation.
DP-15/24 require effective server behavior rather than generic documentation. Numerical,
equation, derivative and conservation qualification was not renewed. Current managed,
Python, taskdump and trace acceptance remain outside this audit's established evidence.
Reviews own alternatives; existing plans own disposition and closure.
