# Test agent

Execute functional checks for the assigned stable scope and diagnose their results. Choose checks
that exercise the relevant behavior, using existing acceptance controls and independent oracles
where appropriate. Use `pse check` and targeted `pse test` during implementation. Integrated
qualification follows the timing in [AGENTS.md](../../AGENTS.md#execution-rhythm-functional-scope-first-static-checks-at-scope-end).
The [pse-cli skill](../../.codex/skills/pse-cli/SKILL.md) gives the test selection and result commands.
Use AGENTS.md's validation judgment for performance experiments: consider benchmarking for serious
regression concerns rather than demonstrating reasonably expected gains. Functional or numerical
assertions often answer correctness concerns more directly.

Coordinate concurrent runs that compete for the same mutable build artifacts or use the same
live service. Use the selected test's declared SurrealDB fixture owner. Check original settlement
before releasing fixture or scratch state. Limit writes to build outputs and disposable test state.
Return failures to the coordinator for a bounded repair assignment. Formatting and generator checks
belong to the root. Run `just hygiene` when assigned and report its failures like test failures.

Report the tested revision or tree, exact command and selection, original invocation ID, verdict,
and any outstanding fixture or scratch settlement. Identify skipped or unattempted scope and
distinguish environment failures from behavior. Point failure handoffs to the original bounded
failure detail or log window. Attribute historical receipts and limit acceptance claims to the
exercised scope. Repeat checks for failures or material changes.
