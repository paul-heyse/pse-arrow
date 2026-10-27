# arrow

Library family account from the first applicability assessment (2026-10-27). Each page keeps its reusable types separate from its current applicability; this index routes to them.

| Page | Types | Current applicability |
| --- | --- | --- |
| [Apache Arrow IPC (arrow-ipc)](ipc.md) | `arrow-ipc/untrusted-file-admission`<br>`arrow-ipc/caller-owned-push-decoding`<br>`arrow-ipc/encoded-size-without-body-copy` | CBO-002, CBO-005 |
| [Apache Arrow data layout and memory sizing (arrow-data, arrow-buffer)](data.md) | `arrow-data/layout-description`<br>`arrow-data/memory-size-functions`<br>`arrow-data/validation-and-layout` | CBO-001, CBO-005 |
| [Apache Arrow compute kernels (arrow-ord, arrow-select, arrow-arith, arrow-buffer bit iterators)](compute.md) | `arrow-compute/run-partitioning`<br>`arrow-compute/predicate-selection`<br>`arrow-compute/extrema` | CBO-006, CBO-007 |
| [Apache Arrow JSON (arrow-json)](json.md) | `arrow-json/serde-rows-to-arrow` | CBO-008 |
| [Arrow schema serde and IPC (arrow-schema, arrow-ipc, serde_arrow)](schema-serde-and-ipc.md) | `arrow-schema/serde-schema-types`<br>`arrow-ipc/deterministic-schema-and-batch-embedding`<br>`serde-arrow/serde-structs-as-arrow-tables` | CBO-040 |
