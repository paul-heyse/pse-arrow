# Shared worker contract

Use [design principles](../../docs/design_review/design_principles/core/design-principles.md)
together with [Heuristics for Efficient Architecture](../../docs/design_review/design_principles/core/efficient-architecture-heuristics.md)
when consequential architectural or implementation choices fall within the brief. Consider
relevant execution patterns before committing to physical organization, interfaces, preparation,
assurance and lifecycles; address material mismatches while the design remains easy to change.
Use qualitative judgment without an exhaustive checklist, cost models or new proof machinery.
Stay within assigned effects. During execution, focus on choices left open or exposed mismatches;
do not restart settled reviews.

Use applicable instructions already supplied in context; if AGENTS.md is absent, load it before
acting. Load the common and assigned role contracts once, plus relevant workflow or library skills.
The root's general startup tour is not repeated by workers: use the brief and relevant owners,
following additional dependencies when evidence requires it. All permission, preservation and test
rules still apply; named files are not a restriction on necessary read-only investigation. The coordinator's
brief bounds the assignment and permitted effects; a role contract may grant standing write
scopes, which a brief can narrow. Name a markdown file you write for its content (`README.md`,
`<topic>.md`): Claude Code refuses a subagent's Write of a `.md` file whose name begins with
report, summary, findings or analysis. Resolve paths from the repository root; Codex
discovers skills through `.agents/skills`, and their tracked source is `.codex/skills`.

Preserve concurrent work. Do not commit, push or delegate further unless assigned. Report a missing
prerequisite, conflicting contract or material discovery beyond scope to the coordinator; continue
independent in-scope work where useful. Make ordinary local decisions within the brief yourself.

Return the result with precise source locations or artifact pointers, the baseline examined,
material uncertainties and anything unfinished. For consequential negative claims, state the search
scope, versions or paths covered and limitations; an empty search does not establish absence. Distinguish observed facts from interpretation.
For edits, identify changed ownership or behavior and deletions. For checks, give commands and
`passed` / `failed` / `blocked` / `not_run`, retaining raw evidence where useful. Keep the response
compact enough for integration without hiding consequential details.

Follow AGENTS.md's test timing and pinned tool routes. Formatting and generators belong to the
root's `just turn-end`. Non-functional checks (`just hygiene`) run once at scope end, by the integrator
unless the assignment includes them.

Local commands and protections remain in AGENTS.md and its shared rules. Never edit generated
paths; regenerate through `just codegen` when the declaration or generator changes. Keep the
explicit force-validation feature on correctness tests. Run native solver, conformance, parity
and linked-Python commands through `bash scripts/memory-cap.sh <command>`.

**Skills and library documentation.** Before working in an area a repository skill covers,
load that skill: in Claude, use the Skill tool; in Codex, use `.codex/skills/<name>/SKILL.md`.

| Area | Skills |
|---|---|
| Native solvers | `native-solver-libraries` |
| Symbolica, faer, Oximo, FeOS, POUNCE presolve | `symbolica-faer-oximo` |
| Data, storage and query tracing | `datafusion`, `datafusion-tracing` |
| Incremental compilation | `salsa` |
| Graphs | `rust-graphs` |
| Symbolic reasoning | `rust-reasoning` |
| Rust code facts; search and rewrite | `rust-code-model`; `ast-grep-ripgrep` |
| IDAES behaviour and parity | `pyomo-and-solvers` |
| Decisions and reviews | `adr`, `design-review`, `design-review-process-simulator` |

For any other library, framework, SDK or tool API (for example sqlx, PostgreSQL, tokio, PyO3,
maturin, bindgen or SCIP's C API), first load the Context7 tools through ToolSearch
(`select:mcp__context7__resolve-library-id,mcp__context7__query-docs`). Then query current
documentation before relying on memory.
