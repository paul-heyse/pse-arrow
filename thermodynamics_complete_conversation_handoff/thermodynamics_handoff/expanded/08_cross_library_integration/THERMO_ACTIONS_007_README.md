# THERMO-ACTIONS-007 | Deliverable guide

This bundle is Step 7 of the Process simulator thermodynamics architecture work: actions, information flows, lifecycle behavior, failure/recovery paths, and result publication. All baseline documents remain version 0.1 and are included unchanged.

## Read

Start with `thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md`.
Sections 1–4 define shared behavior and operation promises. Section 5 contains 56 action contracts. Sections 6–8 contain eight lifecycle views, 18 handoff facts and 18 workflow routes. Section 9 resolves ordering and replay behavior. Sections 10–13 provide traceability, open questions, audit interpretation and sources.

The corresponding `_register.json` is an architecture-management record, not a runtime API or storage schema. It preserves mappings to all 94 requirements, 72 semantic concepts, 18 information products and 34 scenarios. The preceding requirements, packaging and semantic registers retain their original content and evidence status.

## Inspect the evidence

`thermodynamics_action_workflow_lifecycle_blueprint_v0_1_audit.json` records structural and artifact checks. `thermodynamics_action_workflow_lifecycle_blueprint_v0_1_reference_checks.json` records the 26 executed authored arithmetic/sequential reference-model checks.

These checks do not constitute execution of a thermodynamic provider or the simulator's acceptance suite. There are no numerical provider, native interruption, concurrent transaction, independent representability or real-fluid validation claims. The 112 new action witnesses and all original test/probe/scenario statuses remain unexecuted or unassessed as stated.

## Rerun the authored checks

Use Python 3 with the standard library. From the extracted folder:

```sh
python reference_action_tests.py
python validate_action_blueprint.py
```

The first command prints the result without changing the supplied historical report. The second reruns structural checks and writes the audit report. The scripts use a conservative sequential reference model; their classes are executable sketches, not a proposed production implementation.

## Keep the folder intact

All Markdown predecessors and companions are at the same folder level so relative links resolve. `artifact_manifest.json` records SHA-256 values of every included artifact except the manifest itself. No source libraries, binaries, font files or nested predecessor ZIP files are included.

## Next decision gate

Step 8 resolves actual cross-library and solver integration rules for chosen profiles: compatibility, data/reference conventions, initializer/derivative coverage, exposed operation boundaries and demonstrated session behavior. Step 9 executes the relevant conformance/validation witnesses. This Step-7 blueprint does not select a backend, database, event bus, Rust library or implementation architecture.
