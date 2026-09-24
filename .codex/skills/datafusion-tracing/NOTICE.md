# Source notices

This reference contains upstream documentation, source excerpts, macro definitions and trace
snapshots. Their original copyrights and licenses apply. No ownership of upstream material is
claimed. [The license manifest](content/licenses/manifest.json) names each indexed crate,
exact archive hash, declared license, retained license files and source URL.

The subject source corpus is retained from the commit recorded in `content/PROVENANCE.json`;
the subject packages use Apache-2.0. The tracing and OpenTelemetry wiring packages carry their
respective MIT/Apache declarations and texts. Generated contract projections do not replace
upstream licensing or API documentation.

The reader, contract projection and capability-routing design adapt the companion DataFusion
skill's implementation. All code needed here is retained locally; no sibling skill is required.

For a repin, `build/licenses.py` extracts notices from exact crate archives into
`build/acquired/licenses`. Regeneration copies those retained bytes into `content/licenses`.
