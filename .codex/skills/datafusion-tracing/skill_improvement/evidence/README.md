# Evidence for the improvement plan

Inspected 2026-09-18. Evidence paths below are relative to this planning bundle or the containing
skill. This assessment uses local retained sources and a read-only baseline check. It does not
repin dependencies, regenerate content, execute Rust probes, or qualify a telemetry deployment.

## Baseline inventory and executed check

[inventory.json](inventory.json) records index row counts, the existing provenance's corpus
identities, inherited probe verdicts, missing seam source pointers, and hashes of the main files
used for the assessment. Those hashes identify the inspected state even if later implementation
changes the source documents. Counts were read from the actual index files.

[baseline-check.log](baseline-check.log) records the command, UTC timestamp, exit status, and
output. For an already provisioned Python environment, the corresponding portable invocation
from the skill directory is:

```bash
python3 build/verify.py --skip-rebuild
```

The actual command used the enclosing development environment's Python runner; the command is
retained in the log for provenance, not imposed on consumers of the skill.

| Check | Observation in this assessment | Limit |
|---|---|---|
| Recorded content digests | 417 files matched | `--skip-rebuild` does not prove fresh deterministic regeneration |
| Index/model integrity | 502 symbols checked | The current check covers selected pointer classes |
| Structural fixtures | 11 rule groups passed | Syntax fixtures do not prove the semantic guidance is correct |
| Navigation | 20 checks passed | Known regexes in designated files do not test unfamiliar task decisions |
| Transferability scan | 62 files scanned | Static pattern checking is not copied-bundle execution |
| Stored behavior index | 12 records accepted; none re-executed | Inherited verdicts retain their original scope and date |
| Recipes and router | Two recipes and 18 questions checked | Probe-ID existence does not establish relevance to the routed claim |
| Registered counts | 12 registered claims matched | Counts do not measure reference quality |

## Source findings

| Finding | Inspection locations | Interpretation |
|---|---|---|
| Member/module contract loss | [`Method`, `Item`, `ITEM_KINDS`, `_attach_impls`](../../build/model.py); [`_model_record`, `_method_block`, `write_api`](../../build/emit.py) | Full method docs, module docs and structural member details are not retained in the current explanatory model |
| Default preview formatter | [`InstrumentationOptions`](../../content/corpus/source/datafusion-tracing/options.rs); [`InstrumentedExec` recorder construction](../../content/corpus/source/datafusion-tracing/instrumented_exec.rs); [`PreviewRecorderBuilder::preview_fn`](../../content/corpus/source/datafusion-tracing/preview.rs) | A positive limit constructs the recorder; an omitted callback selects the default formatter in retained source |
| Preview accumulation and finalization | [`PreviewRecordingStream` and `PreviewRecorder` drop paths](../../content/corpus/source/datafusion-tracing/preview.rs) | Available partition previews are concatenated and capped before formatting; displayed cap, retention and lifecycle are distinct |
| Contradictory preview explanation | [Current seam](../../content/seams/preview.md); [authored topic definitions](../../build/topics.json); [preview rule](../../queries/rules/project/project-options-without-preview.yml) | Formatter requirement and single-batch language need correction; rule fixture success does not resolve the conflict |
| Unrelated probe citation | [Object-store route in router source](../../build/router.json); [S008 definition](../../build/probes.json); [`check_router`](../../build/verify.py) | S008 concerns physical-optimizer phase selection; the checker verifies ID existence, not claim support |
| Missing source destinations | [Seam pages](../../content/seams/); [inventory](inventory.json) | Seven code-formatted corpus pointers omit `datafusion-tracing/` in the source path |
| Coarse evidence promotion | [`PROMOTES` and `read_behaviours`](../../build/build.py); [`write_all`](../../build/spans.py) | Confirmed span names label aggregate rows even though an assertion does not establish every aggregate property |
| Probe scope | [Probe definitions](../../build/probes.json); [fixture source](../../build/fixtures/probe-crate/src/main.rs); [fixture manifest](../../build/fixtures/probe-crate/Cargo.toml) | Small-query substring checks supply useful limited observations; object-store and OTEL claims need their own assertions |
| Compatibility overstatement | [Catalog](../../content/catalogs/compatibility.md); [catalog authoring](../../build/catalogs.py); [release matrix](../../content/index/compatibility.tsv) | Observed manifest pairs do not establish a universal arithmetic compatibility rule or consumer compilation |

The preview finding is a source-backed conflict, not a newly executed result. The plan calls for
an omitted-formatter probe before reporting runtime qualification. Existing S002/S005 use their
recorded constructions; neither can be silently reassigned to this new claim.

The retained execution wrapper includes upstream lifecycle tests. Reading those tests identifies
useful requirements and possible oracles. It does not mean those upstream tests or equivalent
public consumer fixtures ran during this assessment.

## Documentation discovery

Context7 was queried for `datafusion-tracing` and instrumentation builder/lifecycle documentation.
The resolver returned Apache DataFusion and tracing-related libraries, with no exact subject
match. `/tokio-rs/tracing` was selected for a narrowly scoped query about async instrumentation
versus an entered span guard across `await`.

The result pointed to the primary [tracing README](https://github.com/tokio-rs/tracing/blob/main/README.md)
and [tracing-futures README](https://github.com/tokio-rs/tracing/blob/main/tracing-futures/README.md).
They identify future instrumentation and subscriber propagation as relevant research surfaces.
These current-branch documents are discovery leads, not exact-version evidence for the skill's
recorded profile. No source pin or compatibility judgment was changed using them.

## Reproduction and remaining work

The index counts can be re-read from `content/index/*.tsv`; source findings name their exact
files and functions above. `content/PROVENANCE.json` identifies the existing generated corpus;
the additional hashes in `inventory.json` identify the inspected authoring and fixture files.

The planning bundle's local links, JSON, whitespace, spelling and source preservation are
checked separately from subject behavior. The result is recorded in [plan-validation.log](plan-validation.log).
No comparative agent evaluation, fresh subject probe, full regeneration, or copied-bundle
qualification is claimed. Those remain `not_run` in the [implementation plan](../IMPLEMENTATION_PLAN.md).
