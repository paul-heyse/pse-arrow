---
name: pse-cli
description: Use pse for routine repository operations it owns, including correctness checks, Nextest, project and schema operations, managed execution and result custody. Optional Rust code intelligence can help with resolved relationships, types, guarded edit previews, pinned APIs and compiler evidence.
---

# Routine repository operations

Default to `pse` for the workflows below. Use the configured context and command defaults,
with overrides for a deliberate change of selection. Documented recipes and supervised raw
tools cover specialized workflows such as `just doctest`, independent references and tests
that require independent process scopes.
The native environment puts the checkout's `scripts/` on `PATH`, so `pse` resolves to
`scripts/pse`. Without activation, use that explicit launcher. Prepare a missing host with
`scripts/bootstrap-host`. Help and reads are passive. `pse COMMAND --help` gives syntax,
`pse help COMMAND` gives structured parser metadata and `pse schema` describes response shapes.

| Task | Command and purpose |
|---|---|
| Compile or run a focused test | `pse check PACKAGE --all-targets` checks eligible targets. `pse test PACKAGE --lib -E 'test(=NAME)'` runs a focused Nextest selection. Use `PACKAGE:TARGET` for an integration target and unfiltered `pse test` for the intended ordinary workspace run. |
| Explain or list test scope | `pse inspect affected` explains conservative changed-package reach. `pse inspect test-selection --admitted` explains selected targets and features without building. `pse tests PACKAGE` builds and lists the selected Nextest population. |
| Run, observe or stop managed work | `pse run -- COMMAND` supervises an ad hoc workload. `pse history` locates prior invocations. `pse wait ID` and `pse inspect invocation ID` read the original outcome. `pse inspect log ID` reads bounded original output. `pse cancel ID` requests cancellation from the authenticated owner and reports original settlement evidence. |
| Read or rerun test results | `pse inspect tests ID` reads the retained population. `pse inspect invocation ID --detail failures` reads original failure detail. `pse test --rerun-failed ID` selects the original failed tests for a new run. |
| Select project context | `pse inspect settings` shows effective settings and their sources. `pse project targets` checks configured issuers. `pse project show` reads a project through its prepared product reader. |
| Import or export a project pack | `pse project import-issue --operation KEY PACK TITLE` records the receiving intent. `pse project import --operation KEY PACK` applies that intent. `pse project export --operation KEY DESTINATION` uses a typed intent on stdin or `--intent FILE` and publishes the pack and receipt to a fresh directory. |
| Recover a project operation | `pse project acknowledgement --operation KEY` reconciles the original operation. Use `pse project import-status --operation KEY` and `pse project import-recover --operation KEY` for the original import. |
| Inspect installed schema | `pse observe prepare --installed-schema` explicitly prepares a Viewer. `pse project installed-schema --reader ID` inspects the configured issuer through that reader. Configured live schema rollout remains deferred. |
| Preview or publish a supported authored change | `pse source-change preview --case CASE` checks generated consequences. `pse source-change status ID` reads the recorded stage. `pse source-change apply ID`, `recover ID` and `discard ID` publish, recover or release the original receipt respectively. |
| Inspect an active calculation | `pse observe prepare` explicitly prepares the product reader. `pse observe discover`, `schema`, `query` and `follow` inspect through the product owner. |
| Hold, export or retire an ordinary result | `pse tools hold-result ID --consumer NAME --condition CONDITION` records an active use. `pse tools export-result ID --output FILE` writes a compact deliverable. `pse tools retire-result ID` previews release. Use `pse tools retire-result ID --release TOKEN` for that exact selection. |
| Close an authored scratch probe | `pse probe clean NAME` previews allocated bytes and the output directory identity. `pse probe clean NAME --release IDENTITY` releases that output after the original owners drain. |

Read the original verdict and settlement separately from the success of a status request.
A wait timeout can leave completion unknown. A source-change stage records the owner's last
observation and may remain after interruption. Follow returned continuations and release identities
as issued. Keep the compact evidence needed for the decision and regenerate expired ordinary
evidence for a named need. Ad hoc scratch follows the original producer's settlement and the
[scratch closeout](../../../docs/dev/agent-environment.md#closing-ad-hoc-scratch).

Native field selection and record budgets keep a read focused while preserving its exit status
and diagnostics. `pse jq` can extract nested data from supported inspection reads when jq is
available on `PATH`.

```bash
pse inspect dependency-units arrow-array pse-columnar --lib \
  --fields dependency,selection,contexts --budget 1 --format human
pse jq --filter '.payload.dependency.selected.selection' -- \
  inspect dependency-units arrow-array pse-columnar --lib --fields dependency
```

The adapter leaves producer diagnostics on stderr and runs jq after a successful, complete
JSON response. A producer failure keeps its original exit status. A jq failure is reported
separately. A read's exit status and the observed work's `subject_state` and `settlement`
answer different questions. Include the relevant fields in a custom filter when both matter.
For an original test population or failure page, use `pse inspect tests ID --limit 20
--fields records,next`. Pass the returned cursor to the original page reader. The
[focused output guide](../../../docs/dev/agent-environment.md#focused-cli-output) covers
the adapter's limits and error handling.

The [environment guide](../../../docs/dev/agent-environment.md) owns preparation, placement,
run controls and detailed command examples. The
[qualification guide](../../../docs/dev/validation-assessment.md) describes what each check establishes.

## Code intelligence that may help

File search and source reads may be the shortest path for many questions. A prepared semantic
session or an existing artifact can make a focused code intelligence query inexpensive when
resolved Rust semantics or compiler evidence would help.

| Question | Capability that may help |
|---|---|
| Which references, callers or implementations resolve to this item? | `pse inspect entity` with `refs`, `calls` or `implementations` can distinguish aliases and resolved targets. |
| What type, trait predicate, method or macro expansion is involved? | `pse inspect entity` offers `explain`, `predicate`, `candidates`, `macro` and related views, subject to backend support. |
| What could a structural edit change? | `pse propose rename` and `pse propose ssr` can help assess guarded previews before normal edit and validation. |
| What does the exact locked dependency expose, or why is it included? | `pse search api` and `pse inspect api` query acquired API artifacts. `pse inspect dependency` and `pse inspect dependency-units` can explain dependency and feature context. |
| What explains a compiler failure or generated behavior? | `pse inspect diagnostic`, `pse inspect compiler-facts` and `pse inspect compiler-artifact` can read captured evidence. `pse compiler generate` or an authored probe can capture a specific missing observation. |

The [code intelligence reference](references/code-intelligence.md) and
[engineering inquiries](references/engineering-inquiries.md) describe preparation, examples
and the scope of the evidence.

When a working-source result already identifies the code in question,
`pse inspect test-selection --from-result REF --admitted` can use its verified witness to select
the current containing package. This yields a test candidate. `pse inspect affected` supplies the
separate conservative changed-source view.
