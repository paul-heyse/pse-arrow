# tokio

Read the [library-led briefing](briefing.md) for the 2026-11-02 current-use synthesis and
research priorities. The [task-drain contract](task-drain-contract.md) distinguishes task
tracking, admission, cancellation and settlement; the older applicability pages retain their
original baselines.

Library family account from the first applicability assessment (2026-10-27). Each page keeps its reusable types separate from its current applicability; this index routes to them.

| Page | Types | Current applicability |
| --- | --- | --- |
| [Tokio](tokio.md) | `tokio/ordered-stop-race`<br>`tokio/change-notification`<br>`tokio/fd-readiness-await`<br>`tokio/task-local-context` | CBO-011, CBO-014, CBO-015, CBO-016, CBO-018 |
| [tokio-util](tokio-util.md) | `tokio-util/cancellation-tree`<br>`tokio-util/task-supervision` | CBO-011, CBO-015, CBO-016 |
| [tokio::process (Rust)](process.md) | `tokio-process/async-child-with-pidfd-reaping` | CBO-004 |
