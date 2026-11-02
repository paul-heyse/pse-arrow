# Conservative container pruning

Reusable type: `datafusion/conservative-container-pruning-contract`.
Research date 2026-11-02. **Interface-checked** against DataFusion 55.1.0 source;
no new probe or application qualification was run.

`PruningPredicateBuilder::new().with_file_schema(schema).try_build(physical_expr)` rewrites
a physical predicate into a predicate over container statistics. The caller implements
`PruningStatistics` or uses a suitable library statistics adapter. The containers can be
application-defined blocks or groups: they need not be files.

`PruningPredicate::prune` returns a keep decision per container. False proves that the container
cannot match under the supplied statistics; true includes unknown cases and does not prove a
matching row exists. Preserve the index-to-container correspondence. Evaluate the original
residual predicate on kept data unless a separate exactness argument discharges it.

The caller owns trustworthy min/max, null and row counts, schema compatibility and the treatment
of unavailable statistics. Unknown data conservatively preserves candidates. Null statistics
participate in SQL three-valued reasoning; a hand-written Boolean approximation can change
the result. Row counts are container-level (`row_counts()` has no column argument).

Errors are a policy boundary. An advisory read may choose to keep every container after a
pruning error; that is not a substitute for required validation or an integrity refusal.
Pruning neither authenticates statistics nor verifies scientific validity. Building a pruning
expression and evaluating it are work with allocation/lifetime obligations; this API does not
by itself charge all work to a chosen application memory budget.

Use the schema and physical predicate together across pruning and residual evaluation to avoid
independent semantic translations. A revealing case is an unknown statistic: pruning keeps the
container, while residual evaluation still rejects its nonmatching rows. Another is a false
statistic: the library cannot recover rows that an incorrect summary lets it discard.

Sources: [55.1.0 implementation and contract](https://docs.rs/datafusion-pruning/55.1.0/src/datafusion_pruning/pruning_predicate.rs.html),
[PruningPredicate](https://docs.rs/datafusion-pruning/55.1.0/datafusion_pruning/struct.PruningPredicate.html),
[55 migration guide](https://datafusion.apache.org/library-user-guide/upgrading/55.0.0.html).
