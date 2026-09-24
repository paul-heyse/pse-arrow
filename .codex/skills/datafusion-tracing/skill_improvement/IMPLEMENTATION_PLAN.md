# Implementation and evaluation plan

Proposed 2026-09-18. All implementation stages below are `not_run` in this planning pass.
Changes belong to the portable tracing skill; no application instrumentation, dependency
upgrade, collector deployment, or user installation is part of this plan.

The [assessment](README.md) establishes the baseline, [target design](TARGET_DESIGN.md) describes
the result, and [worked examples](CAPABILITY_EXAMPLES.md) demonstrate the reference style.
This sequence defines dependencies and reviewable outcomes while leaving implementation choices
to the agent carrying out the work.

## Build in this dependency order

| Stage | Concrete work | Reviewable completion criterion |
|---|---|---|
| 1. Preserve contracts and repair evidence links | Extend `build/model.py` and `build/emit.py` for complete module/member/field/variant docs, raw types, impl context, and source/doc locators. Preserve hosted/private capture identity and callable reachability. Audit `build/router.json`, `build/topics.json`, `build/catalogs.py`, and span promotion in `build/build.py` / `build/spans.py`. Resolve the preview and compatibility overstatements at their authoring sources. | Full builder, field, subscriber, and provider lifecycle contracts are addressable. Missing source links and unrelated probe citations are detected. Corrections retain their evidence, and unexecuted behavior stays unexecuted. |
| 2. Introduce useful routes | Author task, crate-role, lifecycle, and symptom routes over the eleven-crate inventory. Keep direct symbol/macro lookup. Include current alternatives and explain boundaries with DataFusion, object_store, and collectors. | Unfamiliar task wording reaches plausible mechanisms without knowing the macro name. Every crate has a role. Route targets resolve and the reader can distinguish unreviewed surface from reviewed contracts. |
| 3. Establish semantic depth | Convert the worked examples into maintained capability/claim/comparison records. Add async context, error/early-drop behavior, repeated execution, and subscriber/export lifecycle contracts. Preserve distinctions between source interpretation and observed behavior. | A reader can identify inputs, outputs, integration point, lifecycle, and evidence limits for each reviewed capability. Comparisons expose changed conditions without prescribing a stack or preferred answer. |
| 4. Strengthen executable evidence | Extend the existing isolated consumer fixture with structured capture assertions, specific controls, and source-to-assertion links. Add object-store and exporter fixtures only for the corresponding contracts. Record locks, selected features, tools, targets, command results, raw captures and normalization. | Each promoted claim names a discriminating assertion and its scope. A store test cannot qualify a planner claim; compilation cannot qualify receipt. Failure and prerequisite status remain explicit. |
| 5. Add bounded retrieval | Implement task/alias/facet lookup and `find`, `show`, `compare` over the reviewed records and exact operation index. Retain direct files and existing compact views. | Short results expose matching reasons, alternatives, evidence and unknowns. Large contracts have lossless continuation. Known symbols, public aliases, and reachable builders resolve without inventing import paths. |
| 6. Evaluate and refine | Freeze baseline/candidate bundles and run the paired tasks below. Evaluate discovery, contract fidelity, integration and efficiency separately. Expand or correct the reference from observed task failures. | Improvements and regressions are supported by task artifacts and independent oracles. An accurate statement of uncertainty is distinguished from an unsupported claim. Corpus growth alone is not an improvement result. |
| 7. Package and maintain | Supply reader/research bundles, dependency-aware invalidation, notices, and separate integrity, regeneration, probe and transfer checks. Replace the live entry point with a short router when its destinations exist. | A copied reader works offline from an unrelated directory without the source checkout. Research artifacts reproduce the declared probes with named prerequisites. Delivered evidence and remaining limits are recorded. |

Stage 1 establishes the evidence that stages 2–3 explain. A small useful set of routes and
contracts can ship before a query helper. Stage 4 can proceed as individual briefs become ready;
stage 5 depends on stable records. Comparative evaluation needs frozen reader artifacts and
task oracles. Packaging is designed early and qualified against the finished candidate.

## First contract backlog

The initial set spans the mechanisms already present and the most consequential missing depth:

| Capability family | First distinctions | Evidence work |
|---|---|---|
| Planning instrumentation | Full, phase-only, selected phases; state before/after wrapping | Exact phase-set assertions and per-rule parentage, beyond S007/S008 substring checks |
| Physical execution | Rule installation, later rewrites, span creation and stream lifecycle | Instrumented/uninstrumented result equivalence; positive and negative plan-placement controls |
| Fields and targets | Declaring versus recording keys; call-site versus explicit target; tracing name versus exported name | Strengthen S004/S006 with isolated field and target controls |
| Preview | Zero/positive limit, omitted/custom formatter, multi-batch/partition accumulation | Correct source-backed conflict; assert actual preview content, cap, timing, and error path |
| Metrics and execution groups | Metrics enabled/disabled, native availability, completion, early drop, repeated use | Specific field/value and lifecycle assertions without claiming a complete metric vocabulary |
| Subscriber and async context | Layer scope, level/target filters, construction versus polling/spawn context | Parsed parent/child identity and independently observable output branches |
| Object-store instrumentation | Wrapped/unwrapped calls, method/result/error coverage, returned-stream lifetime | New store-specific fixture and call-result equivalence; S008 is not store evidence |
| Bridge and export | Declared/resolved versions, local SDK export, flush/shutdown, optional receiver receipt | Isolated feature/lock profiles; distinguish in-memory export from OTLP transport |

Upstream source tests are useful designs and evidence leads. They are not counted as locally
executed consumer tests unless run and recorded. Their assumptions, visibility access, and
dependency profile may differ from what an application can use.

## Discriminating probes

Prefer assertions on structured observations over text that merely appears somewhere in a log.
Retain the raw capture, then extract the exact relationship needed by the claim: field value,
phase membership, parent ID, close event, forwarded error, output rows, or exporter receipt.

Controls change the relevant condition and keep the others stable. Where several mechanisms
could explain silence, capture a positive observation at an earlier boundary. A missing span
without such a control cannot distinguish filtering from a fixture that did no work.

The existing `VALUES` query is useful for configuration differences. Other contracts need
different fixtures: a plan with multiple partitions, a stream that yields then errors, repeated
execution with a retained plan handle, a controlled storage implementation, or an in-memory SDK
exporter. Real OTLP delivery can use a separately declared loopback receiver profile; a collector
service and credentials are not prerequisites for the reader or ordinary local assertions.

Record workload and measurement scope for performance claims. A preview row cap does not by
itself establish peak retained bytes. A span count does not establish query slowdown. An SDK
exporter call does not establish backend ingestion or persistence.

## Evaluate choices and integration

Use standalone Rust consumer fixtures and concise design/debugging tasks, independent of any
host application. Keep task wording, dependency profile, tools, and response budget constant
between baseline and candidate. Preserve the reference snapshots, retrieved evidence, proposed
composition, assumptions, implementation, and observed result. Hold evaluation answers outside
the reference visible to the evaluated agent.

Each pair changes one material condition. A correct response may change the chosen mechanism,
the configuration, or the evidence it says is still needed. Multiple implementations can satisfy
the oracle; exact wording and a single preferred symbol are not the criteria.

| Pair | Case A | Changed condition in case B | Decision distinction and oracle |
|---|---|---|---|
| E01 | Explain total planning-phase duration | Identify which rule changes a plan | Phase-only versus per-rule detail; parsed phase/rule structure, unchanged query result |
| E02 | Observe all planning phases | Observe only physical optimization | Builder reachability and exact phase membership; selected phases present, excluded phases absent |
| E03 | Instrument the final physical plan | Another optimizer rewrite runs afterward | Registration/composition consequence; inspect wrappers and emitted nodes for a controlled rewrite |
| E04 | Use the macro's default target | Supply an explicit target under a narrow filter | Correct target/filter relation; separate positive captures under matching and nonmatching filters |
| E05 | Record a key declared in the macro | Record the same undeclared key | Field schema versus value update; assert key/value presence and absence on the intended span |
| E06 | Positive preview limit with no custom formatter | Same limit with a custom formatter | Default formatting versus customization; exact captured preview and callback evidence |
| E07 | Preview one batch in one partition | Same display limit with multiple batches/partitions | Output cap, accumulation and finalization; assert rows and closure without inferring a memory bound |
| E08 | Fully consume execution streams | Consume one item, then drop the streams | Partial versus complete metrics/previews; compare lifecycle and result observations |
| E09 | Retain a plan after execution completes | Keep an execution stream alive instead | Ownership controlling span closure; observed close timing with independent handles |
| E10 | Await work under an instrumented future | Spawn it across a task boundary | Explicit context/dispatch composition; assert parent identity rather than matching span names |
| E11 | A local formatting layer receives the span | The OTEL branch filters or samples it out | Branch-specific suppression; independent local and SDK observations |
| E12 | Register and use the wrapped object store | Register and use the underlying store | Separate storage integration; same returned data, store-specific spans only for the wrapper |
| E13 | A store call immediately returns bytes | A store call returns a lazily consumed stream | Request versus payload lifetime; controlled consumption/error observations at both boundaries |
| E14 | Emit spans while provider and runtime remain alive | End the program after emitting buffered spans | Flush/shutdown ownership; exporter records or receiver acknowledgement, with the asserted boundary named |
| E15 | Use the retained bridge/SDK dependency profile | Use an uncharacterized version/feature combination | Evidence-scoped compatibility answer; dependency graph and compilation, no arithmetic inference |
| E16 | Look up the reachable options builder | Look up the internal execution wrapper | Callable route versus source-only identity; public consumer compile fixture and an explicit boundary |

These 32 cases are an initial pilot, not representative coverage of every crate. Held-out tasks
should vary wording and introduce combinations such as repeated/concurrent execution, query
failure, missing native metrics, formatter failure, store listing/multipart, and exporter timeout.
Those additional cases are selected from actual gaps, not a quota of pages or probes.

## Keep evaluation dimensions separate

| Dimension | Evidence |
|---|---|
| Discovery | Relevant mechanisms and alternatives found; unrelated candidates surfaced |
| Contract fidelity | Correct callable path, field/target behavior, lifecycle and dependency scope |
| Integration | Evaluator-produced implementation compiles and meets its behavioral oracle |
| Diagnosis | Competing explanations distinguished with appropriate observations |
| Evidence use | Claims cite the right source/assertion; inherited and fresh results stay distinct |
| Efficiency | Tool calls, retrieved bytes, context use and elapsed time to a defensible result |
| Generalization | Changed conditions, unfamiliar wording and held-out tasks |

Independent judging can inspect task artifacts against the oracle. Repeated trials matter where
variance changes the conclusion. A shared probe suite can support a contract without proving
every evaluator-produced implementation; record which sketches were actually compiled or run.
Do not combine these dimensions into a synthetic quality percentage.

## Qualification and maintenance

Relevant checks include:

- Raw-to-record preservation for docs/types/source identity, including multiple rustdoc formats,
  private captures, impl contexts and stable public aliases.
- Route/link/anchor integrity and claim-to-assertion relevance; unresolved material remains visible.
- Snapshot repair and normalization provenance, with no whole-row promotion from one assertion.
- Structural rules with near misses, aliases and unrelated same-name calls; syntax stays a lead.
- Deterministic regeneration separate from recorded digest integrity.
- Declared versus resolved features and dependencies; compile results separate from runtime results.
- Behavioral controls for the reviewed claims, including errors and lifecycle where consequential.
- Bounded lookup continuation and direct-file usability from another working directory.
- Copied reader and research bundle checks with the original tree/caches unavailable as appropriate.
- Comparative task outcomes before claiming an improvement in agent decisions or efficiency.

Update the claim and its authoring source, regenerate affected views, and rerun the affected
checks. Repins and changes to capture/normalization/identity may require broader qualification.
The final implementation report should identify the frozen candidate, commands and artifacts,
executed outcomes, known failures, untested scope, and the next useful characterization work.
