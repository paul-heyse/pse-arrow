# Development guidance

Use the repository command surface (`just --list`) and shared agent instructions. These pages
explain focused development workflows. [Current work](../plans/README.md) owns active plan
status; the current qualification basis is
[§24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2).

- [Qualification commands](validation-assessment.md): assessment, native tests, case measurements and shared fixtures.
- [Native workflow](native-workflow.md): public Rust/Python model, solve, dynamics, fitting and publication usage.
- [Native execution assurance](native-execution-assurance.md): engine observation evidence and its limits.
- [Build reuse](build-performance.md): local compilation setup and cache boundaries.
- [System LLVM selection](llvm-system.md): installing and verifying the shared LLVM prefix.
- [Dependency policy](dependency-policy.md): library eligibility and pinned boundaries.
- [Manual CI checks](ci.md): available remote workflows and local recipes.
- [Documentation publishing](documentation.md): Markdown collections, search and section moves.
