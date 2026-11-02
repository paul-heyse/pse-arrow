# Selective technical contract research

Broad discovery finds candidates. Technical research establishes what a selected mechanism
actually requires and guarantees. Keep this research in the existing reusable library type
pages, separately from local applicability and from the concise briefing.

Start with the specific question that could change a design choice. Use Context7, then exact
release documentation, implementation/delegation paths and upstream tests. The useful unit is
often a composition—builder, operation, retained owner and cleanup—not one function signature.
Record the exact APIs and conditions while following the question beyond familiar entry points.

Explain the necessary initialization, features, runtime or thread requirements, ownership and
resource lifetimes. Follow consequential success, error, cancellation and cleanup paths.
Distinguish effects such as I/O, blocking, process creation, mutation, partial publication and
allocation from merely declared types. For numerical work, distinguish mathematical validity,
approximation, rewriting and floating-point realization. Select what matters; this is not a
mandatory field inventory.

Make discharged obligations and residual caller policy understandable. For example, a tracker
can observe completion without closing admission, a pruning predicate can discard impossible
containers without verifying data integrity, and evaluator export can expose optimized
instructions without preserving the original expression shape.

Signatures nominate routes; they do not establish cancellation safety, bounded memory,
idempotence, determinism or a domain validity envelope. Preserve contrary evidence and unknowns.
Use compile probes for disputed access/bounds and runtime probes for specific lifecycle or
error questions. Use retained compiler/MIR observations when compiler-selected behavior is
decisive, rather than as a default census substrate. Reading an example does not execute it.

Keep reusable explanations free of repository paths, `CBO`s and adoption decisions. Put exact
local source/configuration and fit in the applicability account. A type can be researched or
refreshed without a current codebase match; promoting it into a local recommendation still
requires the existing applicability and comparative judgment.

The pilot technical accounts illustrate selective depth:
[conservative pruning](../libraries/datafusion/pruning-contract.md),
[task drain](../libraries/tokio/task-drain-contract.md), and
[optimized evaluator boundaries](../libraries/symbolica/evaluator-contract.md).
