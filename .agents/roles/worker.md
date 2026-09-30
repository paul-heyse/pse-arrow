# Shared worker contract

Read AGENTS.md, then the assigned role and relevant workflow or library skills. The coordinator's
brief bounds the assignment and permitted effects. Resolve paths from the repository root; Codex
discovers skills through `.agents/skills`, and their tracked source is `.codex/skills`.

Preserve concurrent work. Do not commit, push or delegate further unless assigned. Report a missing
prerequisite, conflicting contract or material discovery beyond scope to the coordinator; continue
independent in-scope work where useful. Make ordinary local decisions within the brief yourself.

Return the result with precise source locations or artifact pointers, the baseline examined,
material uncertainties and anything unfinished. Distinguish observed facts from interpretation.
For edits, identify changed ownership or behavior and deletions. For checks, give commands and
`passed` / `failed` / `blocked` / `not_run`, retaining raw evidence where useful. Keep the response
compact enough for integration without hiding consequential details.

Follow AGENTS.md's test timing and pinned tool routes. Non-functional checks and their repair belong
to the end-of-turn hook. An assignment does not authorize running or troubleshooting those checks.

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
| Data, storage and query tracing | `datafusion`, `deltalake`, `datafusion-tracing` |
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
