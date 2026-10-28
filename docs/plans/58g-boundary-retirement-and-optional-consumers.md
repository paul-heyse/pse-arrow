---
title: "Plan58: Boundary retirement and optional consumers"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, design, libraries, validation]
---

# Boundary retirement and optional consumers

## Target and foundation

**Proposed:** retire first-party Python operating machinery and the Python MCP stack while preserving useful scientific outcomes through Rust owners. Source review O03/O04; coordinator owns selection. No full Python API, FastMCP feature parity, CLI or MCP requirement survives merely because it once existed. Existing CLIs may stay when they are thin useful adapters; remove their mandatory MCP execution dependency or drop the adapter. Future optional PyO3 uses coarse Rust operations; its implementation is deferred.

The current Python Runtime facade, control client, generated documents, native worker bridge, retained MCP sessions, source/extension currency checks and CLI generation serve different purposes. Inspect their callers before deletion. Do not replace a Python helper process with a Rust helper process if a direct owned operation suffices. Do not embed Python in the new host to preserve orchestration.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| G01 | planned | 58g | Retained/lost frontend behavior and migration callers explicitly classified |
| G02 | planned | 58g | Python MCP and consumer-specific bridges removed after retained consumers move |
| G03 | planned | 58g | Python product facade/required dependency retirement integrated across build/generation/tests |
| G04 | planned | 58g | Optional consumer contract/investigation decisions recorded without implementation requirement |

<a id="g01"></a>
### G01 — Select outcomes and deletion boundaries

Consume T01/T02 and A/D/E/F capability designs. Classify public authoring/result/control use, developer commands, prompts/navigation, source editing and telemetry/recovery access by actual retained outcome. Record intentional losses with their rationale and any ordinary tool/API replacement. A shell/editor workflow may replace an agent-specific editing service; no new preview state machine is required by default. No preserved interface can force a generic command catalogue into production. Targeted acceptance is a concrete consumer transition with input/output/failure expectations, not a module-count target.

<a id="g02"></a>
### G02 — Remove Python MCP as a system dependency

After selected product/developer callers work, delete serving, fixed worker/native read subprocess bridges, retained Python Runtime sessions, generic invokers, visibility transforms, generated MCP CLI clients, reconnect/admission/currency checks and fixtures whose only consumer disappears. Migrate or drop every remaining meaningful `scripts/capabilities` consumer rather than deleting the directory by name. Update recipes, agent attachments/configuration, generation and dependency declarations together. Preserve immutable executing-artifact identity and physical settlement where their actual consumers remain. No alternate Python server or hidden compatibility invoker.

Targeted tests exercise retained behavior directly at Rust owners plus startup/help/configuration with Python/MCP unavailable, and explicit refusal for removed entry points without launching setup. Removed interface tests retire; native assertions remain. H owns joined frontend-independent acceptance.

<a id="g03"></a>
### G03 — Retire required Python product/runtime adapters

Migrate remaining native consumers before removing required scientific facade, HTTP orchestration, generated Python mirrors/stubs, pyo3-specific workflow roots and build dependencies. Reconcile version/build metadata authority when Python ceases to own package-facing publication. Remove stale generator outputs through their generator, not hand edits. Delete obsolete interpreter wait policies only after actual retained consumers leave; C owns native lifecycle guarantees. Isolate independent Python oracle execution and any explicitly optional Python data consumer; neither can become a required simulation/bootstrap dependency. Do not ship fake `.py` forwarding compatibility APIs.

Targeted tests establish native operations do not import/start Python, generated contracts reflect retained consumers and optional references cannot contaminate production closure. No assertion that all dependencies must be implemented in Rust: qualified C/Fortran solvers and external tools remain eligible.

<a id="g04"></a>
### G04 — Define future adapters; conditional adoption only

Document an optional PyO3 seam over native request/result/operation handles: Python supplies adaptation, Rust owns policy and lifecycle. Compare in-process borrowed Arrow access with process-isolated durable access for a concrete consumer; no GUI framework is selected. PyO3 `detach`/classes do not eliminate thread/destruction/cancellation obligations. This design deliverable is required; implementing the package is not.

RMCP remains deferred until a retained actual agent workflow needs it beyond direct tools/APIs. If triggered, compare no MCP against a small typed STDIO adapter using `ServerHandler`, `ToolRouter`, `Parameters<T>` and structured results. Reuse existing directional schemas, not an independent protocol type universe. RMCP3.7 service close can cancel handlers after bounded drain; domain owners settle separately. No MCP task scheduler or full FastMCP parity. Actual host support is required before any delivery claim. See [tagged RMCP service](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/crates/rmcp/src/service.rs) and [PyO3 pinned API](https://docs.rs/pyo3/0.29.2/pyo3/).

## Verification and handoff

**Proposed:** all packages. No execution performed. H receives retained/lost capabilities, final dependency closures, actual caller migrations and deletions. Optional consumers cannot hold the production target open. Rule/architecture changes precede their dependent implementation; this document does not change ADR status.
