# Golden stores

Golden relation snapshots and fingerprints per vertical slice (blueprint §3.2, §24.1).
Each subdirectory is a published artifact store (`refs/`, `snapshots/`, `relations/`)
produced by `cargo xtask golden <name>` and read back by the Rust engine tests, the
pushdown wrapper test, the Python boundary tests and the parity harness. Python never
writes here; a governance test regenerates and diffs them.

Phase 0 ships none: the first stores are slice A's heater/mixer flowsheet.
