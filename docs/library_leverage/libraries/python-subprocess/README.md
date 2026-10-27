# Python subprocess output ownership

Research date 2026-10-27. Realization: checkout CPython 3.14.8. Context7 `/python/cpython`
was followed by the [Python 3.14 subprocess reference](https://docs.python.org/3.14/library/subprocess.html).
The live reference describes the 3.14 series; no claim depends on a new patch-level API.

## Types

### python-subprocess/demand-shaped-output

- **Need:** run a child while retaining only the output the consumer needs.
- **Mechanisms:** `run(capture_output=True)` uses pipes and `communicate`, which accumulates
  output in memory. `Popen` accepts file descriptors, files or `DEVNULL` for output, allowing
  file-backed capture or discarding unneeded output. Incremental pipe consumers must drain
  stdout and stderr without deadlocking and bound retained chunks, including unterminated lines.
- **Lifetime:** `run(timeout=...)` kills and waits for its child on timeout; `Popen.communicate`
  timeout alone does not. Neither child wait establishes descendant-group drain.
- **Residual obligations:** output extent and redaction, file ownership/storage limits, complete
  input/output handling, original deadline composition and descendant settlement belong to the caller.
- **Evidence:** Interface-checked documentation; no new subprocess workload or performance probe.

## Current applicability

### CBO-053 — Maintenance CLI output retained before bounding

At source `4f52fd239`, `scripts/surreal_server.py::maintenance_sql` captures complete stdout
and stderr, then `maintenance_failure` redacts and keeps only bounded tails. Catalog reads
need full structured results, so a blanket tiny acknowledgement cap would reject legitimate
catalogs. `copy_current_database` also captures export/import control output, although the
data itself travels through an owned file; these calls supply no local timeout. This is an
observed route, not evidence of a runaway process or an unbounded end-to-end lifetime.

**Fits with conditions.** Existing stdlib primitives can change output ownership without adding
another process manager. Compare streaming bounded acknowledgement capture, admitted file-backed
catalog output, and discarded success output plus bounded failure tails. Preserve nonce checks,
redaction, maintenance closure and listener drain. Determine operation-specific output demands
and lifetime policy before selecting caps or imposing a restore timeout. No replacement was tested.

See the [opportunity](../../codebase/developer-tooling.md#cbo-053) and
[comparison](../../reviews/host-processes-and-durable-files.md#cbo-053).
