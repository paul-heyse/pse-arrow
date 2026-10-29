---
description: Independent Python references and repository tool environments
paths:
  - "python/**"
  - "tests/reference/**/*.py"
  - "scripts/*_reference.py"
  - "pyproject.toml"
  - "uv.lock"
---

# Independent Python references

Plan58 G retires the first-party Python product, its generated document/stub mirrors,
PyO3 delivery and operating controllers under ADR-0233 and blueprint §21.7. Do not extend
those retiring interfaces or restore a compatibility adapter. The owning plan records which
actual callers still require native acceptance; the target alone is not a completed cutover.

Keep independent IDAES/Pyomo, SciPy and other scientific oracles isolated in their explicitly
selected environments. Exchange explicit cases/results with native candidates; do not import
`pse` or calculate expected answers through the production implementation. Preserve source,
versions, physical meaning, tolerances and deliberately wrong-candidate controls. A missing
required reference dependency is a failure, never a skipped parity claim.

`typing.Any` remains banned. Tools come from the checkout's selected environment and locked
dependency groups; do not use a global formatter/type checker or install an unpinned tool.
Run targeted reference checks during implementation. Formatting and comprehensive quality
checks follow AGENTS.md's scope-end rhythm. Ordinary third-party tools may use Python;
they do not make a first-party Python controller part of the product or cold host.
