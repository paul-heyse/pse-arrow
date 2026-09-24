# Improving the DataFusion tracing skill as a programming reference

Assessed 2026-09-18. Scope: the portable `datafusion-tracing` skill, independently of the
repository containing it. This assessment and target design were the planning baseline. Implementation is now recorded in
[IMPLEMENTATION_REPORT.md](IMPLEMENTATION_REPORT.md), with retained baseline and candidate evidence.

## Intended result

Give an experienced agent a compact route from an instrumentation task to the relevant
capabilities, their exact contracts, and the observations that support them. Preserve the useful
inventory, private-item captures, macro grammar, and trace evidence. Add the task, operation,
comparison, and bounded retrieval layers demonstrated by the enhanced DataFusion skill.

The reference supplies distinctions an agent cannot safely infer from an API name. It leaves
design judgment with the agent. A comparison explains what changes between alternatives; it
does not prescribe a preferred telemetry stack, collector, deployment, or sequence of research
steps. Known-symbol lookup and direct evidence inspection remain shortcuts.

The central questions are:

1. Which part of the query needs observing: planning, execution, storage, or export?
2. What enters and leaves this operation, and when are fields, metrics, and previews recorded?
3. Which configuration changes the result: rule placement, target, level, field declaration,
   subscriber, feature profile, or provider lifetime?
4. Which observations distinguish missing instrumentation from filtering, incomplete execution,
   sampling, or unsuccessful delivery?
5. What does the evidence establish for this release, and what still needs checking in the
   consumer's actual dependency environment?

## Read the plan

| Document | Purpose |
|---|---|
| [Target design](TARGET_DESIGN.md) | Routes, contract structure, evidence, retrieval, and portability |
| [Worked capability references](CAPABILITY_EXAMPLES.md) | Examples of the proposed depth and neutral comparative style |
| [Implementation and evaluation](IMPLEMENTATION_PLAN.md) | Dependency order, deliverables, completion criteria, and paired tasks |
| [Proposed entry point](examples/SKILL.proposed.md) | A short router using existing files in this planning bundle |
| [Evidence guide](evidence/README.md) | Inspection scope, baseline check, source conflicts, and reproduction |
| [Baseline inventory](evidence/inventory.json) | Counts, source identities, and the boundaries of this assessment |

## Existing foundation

The current reference contains 11 crates, 502 canonical items, 1,834 method rows, 286 aliases,
49 macro arms, 18 routed questions, and eight seam pages. It preserves hosted and private
rustdoc captures, release manifests, upstream source, and upstream trace snapshots. Visibility
distinguishes supported, doc-hidden, reachable-undocumented, and internal items. That distinction
is essential for the two option builders whose methods the hosted documentation omits.

The stored probe index reports 12 probes: 11 `confirmed`, one `recorded`. Two inspect rustdoc;
the ten runtime probes use scenarios built around the same small `VALUES` query. These are existing observations,
not fresh executions in this assessment. The reference also has nine observed span rows and
69 field rows; these describe the captured scenarios, not a closed telemetry vocabulary.

The [baseline check](evidence/baseline-check.log) succeeded with `--skip-rebuild`: 417 recorded
file digests matched, 11 structural rule groups passed, and 20 navigation checks succeeded.
The command did not regenerate the reference or rerun the Rust probes. Its success establishes
the properties it checks, not the semantic correctness of every route or explanation.

## Findings that change the plan

| Priority | Observed gap | Consequence and planned change |
|---|---|---|
| 1 | `Method` retains a display signature and short summary; method pages emit signatures. Module docs are outside `ITEM_KINDS`; fields and variants retain names. | Recover complete upstream documentation and structured types before adding hand-authored explanations of missing contracts. Preserve each capture's identity. |
| 1 | The preview seam says a formatter is required, but retained source supplies a default formatter when `preview_fn` is `None`. The seam describes one batch while the source collects partition previews and caps the combined output. | Correct the authored claim and affected rule guidance after tracing the source path and adding discriminating tests. Separate displayed row limit, retained data, and completion timing. |
| 1 | The object-store question cites S008, which tests physical-optimizer instrumentation. Router verification checks that the probe ID exists. | Join claims to the assertion they support, not merely to an existing probe. Add object-store evidence under its own scope. |
| 1 | Several seam source pointers omit the subject-crate directory. For example, `content/corpus/source/preview.rs` is absent; the source is under `source/datafusion-tracing/`. | Validate actual route and evidence destinations, including prose pointers and anchors. A working search recipe does not prove that its adjacent link is valid. |
| 1 | Whole span rows receive `confirmed` by span name, while the supporting assertion establishes only a particular observation such as span existence. | Keep snapshot aggregates and assertion-level results separate. A probe does not confirm every target, parent, field, and scenario on the aggregate row. |
| 2 | Routing starts with a compulsory question ladder and rejected sources; seam pages end in checklists. | Make task, symbol, crate, lifecycle, and symptom routes independently usable. Move release-specific caveats next to the claims they qualify. |
| 2 | The compatibility catalog promotes observed version pairs into an arithmetic rule. The probe fixture has no OpenTelemetry dependencies. | Distinguish manifest requirements, resolved dependency graphs, compilation, local export, and collector receipt. Preserve the historical matrix as evidence of declared combinations. |
| 2 | Existing runtime checks largely use substring presence. S008 checks different captures rather than the exact selected phase set. | Add structured assertions on fields, parents, phase membership, completion, and result equivalence where those distinctions affect use. |
| 2 | Wiring, object-store lifecycle, repeated execution, errors, early drop, and export lack comparable depth in the current probe suite. | Use the retained source and upstream tests as discovery material; extend consumer probes for specific contracts without implying those tests have already run. |

Evidence locations and source-versus-execution distinctions are collected in the
[evidence guide](evidence/README.md). The examples demonstrate the proposed treatment of these
issues without editing generated material.

## Scope and completion boundary

The design follows the DataFusion skill's progression: preserve full contracts, add useful
routes and reviewed comparisons, expose bounded retrieval, evaluate actual decisions, then
qualify transfer and maintenance. Tracing adds macro invocation, span lifecycle, filtering,
context propagation, and export boundaries to that structure.

The reference remains about DataFusion instrumentation and its immediate wiring. General query
planning, Arrow kernels, object-store semantics, and collector administration retain their own
authorities. A copied skill can name and link those boundaries without depending on a sibling
skill, application service, workspace convention, or installed MCP connection.

Delivered here: the assessment, target design, implementation plan, worked examples, proposed
entry point, and baseline evidence. Contract recovery, semantic corrections, new retrieval,
additional runtime probes, copied-bundle qualification, and comparative agent evaluation are
planned work. No improvement in agent performance is claimed yet.
