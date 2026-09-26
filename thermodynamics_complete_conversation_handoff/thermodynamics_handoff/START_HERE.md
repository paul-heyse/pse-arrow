# Process simulator thermodynamics: complete conversation handoff

Start with [the implementation brief](thermodynamics_agent_implementation_brief.md). It distills the research-intensive decisions and practical implementation recommendation for an LLM programming agent. Then use [the resource index](thermodynamics_resource_index.md) for upstream repositories, focused documentation, and test entry points.

## Contents and preservation

`originals/` contains **all 23 original file attachments in this conversation**, byte-for-byte, including all six original ZIP bundles. `expanded/` contains all **128 files** within those six bundles, under one directory per stage and with each original internal layout intact. Repeated files are intentionally retained so provenance and original relative links are not lost. No original report or test result was rewritten.

The nine-stage reports are:

| Stage | Report | Role |
|---|---|---|
| B1 | [Simulation behavior scope](originals/thermodynamics_simulation_behavior_scope_v0_1.md) | Scope authority and breadth commitments. |
| B2 | [DWSIM workflow investigation](originals/dwsim_workflow_reverse_engineering_v0_1.md) | Pinned workflow evidence and design rationale. |
| B3 | [Comparative library research](originals/thermodynamic_package_comparative_research_v0_1.md) | Library boundaries, evidence, and limitations. |
| B4 | [Functional requirements](expanded/09_validation/baselines/thermodynamics_functional_requirements_v0_1.md) | Normative observable behavior. |
| B5 | [Conceptual packaging](expanded/09_validation/baselines/thermodynamics_conceptual_packaging_v0_1.md) | Semantic ownership and responsibility boundaries. |
| B6 | [Information dictionary](expanded/09_validation/baselines/thermodynamics_semantic_information_dictionary_v0_1.md) | Information meaning and invariants, not physical storage schemas. |
| B7 | [Action and lifecycle blueprint](expanded/09_validation/baselines/thermodynamics_action_workflow_lifecycle_blueprint_v0_1.md) | Action contracts, execution and adoption behavior. |
| B8 | [Cross-library integration](expanded/09_validation/baselines/thermodynamics_cross_library_solver_integration_v0_1.md) | Integration choices, solver arrangements, candidate profiles and gates. |
| B9 | [Blueprint validation](expanded/09_validation/thermodynamics_blueprint_validation_v0_1.md) | Conditional design acceptance and bounded evidence overlay. |

The latest validation directory is the best already-linked working collection of the reports, later reference scripts, and tests. Earlier extracted directories additionally preserve stage-specific audit/README/manifest variants that were not all copied forward. All original standalone registers remain in `originals/`; corresponding expanded registers remain beside the reports where originally bundled.

## Evidence precedence and practical reading

The brief is a synthesis, not a rewrite of the baseline. B1 sets scope; B4 defines requirements; B5 allocates responsibility; B6 defines information; B7 defines actions; B8 defines integration admissibility; B9 supplies evidence and outstanding qualification. B2/B3 explain research findings at their recorded versions. Earlier blanket backend suggestions should not override the later profile-specific decisions.

Do not read the large registers linearly unless needed. Read the brief, then B8/B9 for a chosen integration, and consult B5/B6/B7 at the relevant boundary. Requirement, concept, action, and test identifiers provide drill-down, not a requirement to create equally many code objects.

This archive contains generated files and bundled artifacts, not a verbatim export of the chat or raw tool-response history. Those were not conversation file attachments. All 23 attachment records returned by the conversation file inventory were found in the working filesystem and preserved.

## Integrity and re-execution

[MANIFEST.json](MANIFEST.json) records every payload path, size and SHA-256; preserved entries also identify their original file or archive member. [PACKAGE_AUDIT.json](PACKAGE_AUDIT.json) records the packaging checks. [CHECKSUMS.sha256](CHECKSUMS.sha256) covers all files except itself.

Run from this directory:

```bash
python verify_handoff.py
```

This verifies preserved bytes and checksums without executing any thermodynamics or modifying the archive. To rerun scientific/reference scripts, follow [the original validation README](expanded/09_validation/README.md) in a working copy and inspect prerequisites first.

The packaging task checked originals, archive-member bytes, original ZIP integrity, predecessor hashes and handoff links. It did **not** rerun the archived numerical probes or qualify any backend. Original environmental blockers and test limitations are retained unchanged. No upstream library binary or font file was added by the handoff.
