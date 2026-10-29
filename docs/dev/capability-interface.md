# Native tools and agent access

Rust APIs and their existing commands own scientific execution, canonical operations and
physical lifetimes. The optional Python MCP endpoint, generated capability CLI, retained
frontend sessions, reconnect helper and mcpls presentation clients have been retired under
[Plan58 G02](../plans/58g-boundary-retirement-and-optional-consumers.md#g02).
The checkout no longer requires a Python MCP attachment at Codex startup.

## Direct owner routes

Use `just --list` and `just --usage RECIPE` for the existing command surface. Native host
selection and process ownership enter through `scripts/pse-env`; `scripts/pse-env --explain`
reads the selected environment without building a host tool. `just activity` observes current
or named manager/kernel ownership directly. Native binaries must be prepared explicitly;
help and ordinary reads do not build a missing executable.

The [native workflow guide](native-workflow.md) describes scientific construction and results.
The [operational store guide](operational-store.md) describes canonical project, study and
worker operations. `just project-build` explicitly builds the project operator;
`just project --help` forwards arguments to that prepared native executable. Other retained
recipes and Python scientific/reference adapters have their own remaining Plan58 cutovers;
removing the MCP frontend does not establish their migration or qualification.

Use ordinary editor and shell workflows for source editing and navigation. The retained
structural migration helper owns its source-cut, preview, apply and reconciliation contract.
No replacement agent server, universal operation catalogue or new generated CLI is required.

## Native schemas, actions and protected reads

A native typed request, actual prepared artifact and performing owner establish their own
premises. A generated schema describes an external document; it grants no execution,
placement, service or release authority. Original native handles retain cancellation and
physical settlement even when observation or a client disappears. Read completeness and
scientific success remain separate claims.

Historical comparison reports remain available through `scripts/evidence_comparison_report.py`.
Its `scripts/evidence_schema.py` loader validates the selected external comparison against the
Rust-owned evidence schema and retains source identity/digest checks while reading those files.
It imports no endpoint or scientific extension, follows no original source reference and
starts no service. Existing assessment and invocation readers retain their original result
and artifact provenance; see [qualification commands](validation-assessment.md).

## Typed telemetry investigations

`just telemetry configuration` and `just telemetry health` retain their explicit configuration
and passive health behavior. Host telemetry requests and copied observations belong to
`scripts/telemetry_investigation.py` and `scripts/telemetry_selection.py`; their report records
live in `scripts/report_contracts.py`. These readers do not acquire scientific or service
ownership. A copied page retains its original source cut and cannot establish current live
population completeness. Native diagnostic capture and its physical cleanup remain with the
[diagnostic owner](simulation-diagnostics.md), independently of telemetry query success.

MCP-only resource projections, generic invokers, visibility selection, endpoint readiness and
source-currency mirrors have no replacement. Meaningful native operations, passive report
interpretation and independent scientific/reference assertions remain with their owners.

## Historical interface evidence

The [2026-10-25 MCP assessment](https://github.com/paul-heyse/pse-arrow/blob/87ebc72ebc487d97a07acdd077503f2a5303f8ff/docs/dev/mcp-capability-assessment-2026-10-25.md)
retains its original passing and failed attempts, conditions and limits in immutable Git history.
The [former interface guide](https://github.com/paul-heyse/pse-arrow/blob/87ebc72ebc487d97a07acdd077503f2a5303f8ff/docs/dev/capability-interface.md)
records the retired presentation. Those observations do not qualify the native replacements.
Current package progress and remaining acceptance belong to [Plan58](../plans/58-rust-owned-system.md).
