# Canonical selection query plans

**Interface-checked:** the installed `pse.substrate.v1` schema on SurrealDB 3.3.0
was inspected with `EXPLAIN FULL` for bounded scalar output, dense output, result
block range, analysis-edge and study-frontier selectors. The
[captured plans](query-plans.json) retain the complete query text and native response.

All five shapes use `IndexScan` on the intended scoped composite index:
`result_cell_rows`, `result_output_range`, `result_block_page`, `analysis_edges`
and `study_candidates`. Ordering follows the index; none requires a `Sort` or
table scan. Result-range overlap and optional scientific predicates remain filters
within the selected result/output/partition prefix. The study frontier uses
study, settled and assigned state plus its ordinal range in the index itself.

The application tables were empty. This establishes the selected physical paths
under these concrete parameters, without a cardinality, concurrent capacity or
timing claim. Functional controls exercise admitted values and protected reads;
Plan 28e owns assembled workload qualification and measurements.
