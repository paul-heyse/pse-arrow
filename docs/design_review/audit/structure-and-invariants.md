# Structure, invariants and diagnostic meaning

**Interface-checked, 2026-11-01, source `19e7b49a67df24c1bf73005a6186e145e930d53a`.**
The [collection baseline and limits](README.md) apply throughout.

## <a id="da-061"></a>DA-061 — Parser depth limits do not bound the resulting recursive tree

**AP-05, AP-07, DP-03, DP-12, DP-20. New source diagnosis.**

`pse-authoring/src/span.rs::ParseBudget` exposes a nesting-depth bound, defaulting to 64,
alongside a byte budget. `dsl/parser.rs` guards parser recursion, but `Cursor::binary` repeatedly
replaces `lhs` with a binary node in a flat left-associative expression. Each iteration can
add another ancestor while its right-hand parser recursion returns to the same depth.
The proposition loop has the same shape. `parse_expr` and the other inspected public parse
entrypoints finish token consumption without checking the depth of the constructed tree.

Consequently a generated `x+x+...` expression can stay inside the byte allowance and parser
recursion bound while producing tree ancestry exceeding that recursion bound.
`dsl/ast.rs` owns recursive boxed expression nodes and derived cloning; `dsl/walk.rs` and
`dsl/render.rs` recurse through them. Ordinary modeling occurrence admission retains the parser
result (`pse-modeling/src/expression/occurrences.rs`). This makes downstream stack demand depend
on accepted chain length without a corresponding downstream depth protection.

The lexer does enforce the byte allowance, and the parser guard does bound its own recursive
calls. Its documentation explains recursive-descent safety; it does not explicitly promise an
AST-ancestry limit. Those protections do not bound resulting ancestry to the parser recursion
allowance. Arithmetic and proposition construction are the known footprint; not every parser
was audited. The ancestry growth follows from source. Actual stack overflow, a failure threshold and its
platform dependence were not tested.

## <a id="da-007"></a>DA-007 — Consumers re-spell canonical vocabularies and key schemes

**DP-01, DP-02, DP-04, AP-05. Revalidated examples, broader footprint unrefreshed.**

Runtime worker and local study code parse the `run:` key convention themselves
(`workflow/worker.rs` and `workflow/study_execution.rs`). Authored SurrealQL repeats terminal
phase subsets (`pse-codegen/src/codegen/surreal_studies.rs` and `surreal_execution.rs`).
`workflow/durable.rs` separately maps terminal `AttemptState` values into `TerminalClass` and
rejects nonterminal states.

The problem is not that Rust and SurrealQL need different executable representations. It is
that key decomposition and terminal membership must stay aligned with declared meanings at
each hand-written consumer. An added terminal state or changed key framing creates a
multi-owner maintenance obligation. No currently disagreeing terminal classification was
demonstrated. The older literal artifact-profile inventory was not fully revalidated and is
not carried forward as fresh evidence.

## <a id="da-050"></a>DA-050 — Semantic derivation recipes are repeated beneath the frame catalogue

**AP-04, AP-05, DP-01, DP-04, DP-09, DP-24. Revalidated with exact examples.**

`pse-ids/src/derive.rs` catalogues derivation contexts, but consumers independently repeat
sub-contexts and operand sequences. `workflow/durable.rs` and `workflow/worker.rs` both derive
attempt lineage using `FrameCanonicalPayloadV1`, `scientific.attempt.lineage.v1` and the attempt
string. `math/strategy/target.rs` and `math/staged/driver.rs` both derive the original-coordinate
start with `FrameDerivedBindingV1`, `actual-original-coordinate-start`, the original identity
and coordinate sequence.

Changing one meaning therefore requires finding every recipe, although its context appears
catalogued. The same concern applies to minting and decoding lowered `f_<hex>` function names
in modeling. Framing and distinct contexts provide real collision separation; no collision or
current digest disagreement was found. Historical tag counts and unexamined hash variants
are not current evidence.

## <a id="da-051"></a>DA-051 — Hand positional calls and settlement leave invocation premises implicit

**AP-04, AP-05, AP-02, DP-01, DP-02, DP-03, DP-24. Revalidated, substantially narrowed.**

`pse-operations/src/canonical_execution.rs` invokes a claim through a positional SurrealQL
call. `canonical_studies.rs` assembles observation inputs separately. This is a coupling
between caller argument order and authored server function signatures outside a shared typed
invocation contract.

The observation/settlement path in `pse-codegen/src/codegen/surreal_studies.rs` does not compare a
supplied module hash with the current module, although progression paths do check the module.
Local canonical mutation admission also checks the configured module
(`pse-operations/src/canonical.rs`). The narrower unresolved premise is whether an already
admitted observation must remain bound to that interpretation throughout transaction settlement.
No stale-module settlement was executed.

The older claim that registry vocabularies, Rust enums, database constraints and orchestration
automatically form four competing authorities is withdrawn. Multiple representations can
implement one authority. The remaining evidence concerns hand invocation coupling and the
specific transactional premise, not a general absence of transition enforcement.

## <a id="da-039"></a>DA-039 — Error crossings discard distinctions that their original owners retain

**AP-02, AP-04, AP-05, AP-06, DP-02, DP-19, DP-21, PS-10. Revalidated and extended.**

Three concrete crossings lose diagnostic meaning:

- `workflow/mod.rs::contract` always constructs `WorkflowError::Input`. In `workflow/worker.rs`,
  duplicate publication handoff, missing or idle lane state and ordinal overflow use it.
  Internal execution facts therefore project as invalid input.
- `pse-operations/src/canonical.rs::TypedDiagnostic` maps every `CanonicalError` except `Codec`
  to `RuntimeInfrastructure`, including typed refusal and study cancellation.
  The owner's operational predicates distinguish these facts more precisely.
- `workflow/native_worker.rs::infrastructure` accepts `Display` and constructs
  `WorkflowError::Internal(String)`. Launch, readiness, shutdown and observation pass typed
  `HostError` values through it, losing unknown-launch, deadline and compound-settlement
  distinctions and structured causes.

There is also a narrower host-local manifestation. In
`pse-host/src/foreground.rs::settle_result`, an ordinary child exit status plus cleanup failure
becomes one `HostError::Ownership` string. Cleanup can contain a structured `Settlement` tree;
both that tree and the independent child status become prose. The branch with an original
`Err` preserves both failures structurally.

The consequence is a diagnostic contract that requires prose interpretation or reports the
wrong class despite the original owner possessing the relevant distinction (blueprint §23.2).
This does not establish unsafe replay, false drain or incorrect scientific settlement:
operational retry/fencing predicates are more precise, and native worker ownership retains the
manager, original intent and unit for reconciliation.

The host's general diagnostic projection has improved substantially: distinct codes, typed
forwarding, related errors and structured envelopes are present. Its former blanket
infrastructure description is obsolete. Historical call-site counts and the unrefreshed
native-service/math error footprint are not evidence for this revision.

## Earlier identities

These are inherited dispositions from the preceding account at source `19e7b49a6`, not
fresh whole-footprint qualification.

| Identity | Retained lineage |
|---|---|
| <a id="da-008"></a>DA-008 | Resolved 2026-10-25: typed store refusals |
| <a id="da-009"></a>DA-009 | Previously resolved: per-kind request facts and replay constructors |
| <a id="da-010"></a>DA-010 | Resolved 2026-10-25: covariance origin and authentication |
| <a id="da-011"></a>DA-011 | Previously resolved: generated document catalogue; old Python consumers are now absent |
| <a id="da-040"></a>DA-040 | Previously resolved: enum-keyed telemetry attributes |
