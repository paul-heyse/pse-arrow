---
title: "Plan61 contract foundation audit refresh"
date: 2026-10-31
doc_role: review
doc_owner: docs/design_review/audit/README.md
doc_topics: [design, libraries, validation, diagnostics]
---

# Plan61 contract foundation refresh

## Baseline and coverage

**Interface-checked:** source `580a5bd4f`, inspected 2026-10-31 by the independent A02
reviewer. Covered owners: Plan61a, blueprint §21.7, semantic/investigation models, explicit
xtask generator/writer and A01 qualification fixtures. Core 3.4, efficiency heuristics 1.0,
test assessment 1.2 and the binding apply. No scientific scope or broad audit is refreshed.
No new tests/services/generation/benchmarks ran for this refresh; Plan61a owns its prior 22-test evidence.

## Current observations

The native model already separates selection, predicates, answer demand and presentation;
three-valued truth, count precision, native bytes and original context must survive the wire change.
The helper's generic Reply answer remains a subsequent consumed-fact migration, while typed
Failure preserves refusal state, backend code and opaque original observations.

A01 demonstrates a validator division panic and malformed CEL in runtime_error. Successful
generation does not establish required-rule support. The aggregate xtask route currently
writes other trees before Plan61 generation: A02 preflight must precede those effects.
Separate source-based buf generate/build also repeat compilation and do not bind every
consumer to one compiled input.

## Preservation and limits

Existing explicit generation, prepared-plugin admission, GeneratedTree ownership, directional
schemas and native guards are foundations. The full descriptor retains source metadata;
its stripped runtime pool is a different artifact, and names do not establish HIR/rustdoc identity.
DA-039/DA-052 explain related historical projection/authority concerns; unrelated footprints
are neither revalidated nor resolved. A02 owns these current corrections. No new crate,
parser, universal service, test runner or whole-system qualification follows.
