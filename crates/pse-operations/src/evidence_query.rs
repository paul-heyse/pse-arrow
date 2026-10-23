// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite selected evidence questions. No service, scientific runtime, source traversal,
//! automatic detail reads or public arbitrary SQL. Sources parse once and become the
//! registry-owned Arrow relation, shared by every question in an analysis.
use crate::evidence::{EvidenceError, EvidenceSnapshot};
use datafusion::{
    arrow::{array::{Array, Float64Array, Int64Array, StringArray, StringViewArray, UInt64Array}, datatypes::DataType, record_batch::RecordBatch},
    execution::{context::SessionContext, memory_pool::{GreedyMemoryPool, MemoryPool}, runtime_env::RuntimeEnvBuilder},
    prelude::{col, lit},
};
use pse_engine::session::{EngineFactory, ExecutionSettings, ThreadBudget, native_engine_profile};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, path::Path, sync::Arc};

fn invalid(error: impl std::fmt::Display) -> EvidenceError { EvidenceError::Configuration(error.to_string()) }
/// The operation selects a meaning, never arbitrary executable query text.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOperation {
    /// Exact selected tests, including skipped/reused/transferred and unreported rows.
    #[default]
    Subjects,
    /// Subject failure population, distinct from tool or command failures.
    Failures,
    /// Selected subjects without reported execution.
    Missing,
    /// Selected test distributions by declared crate, context and outcome.
    Distribution,
    /// Diagnostic tool/target/science summaries and declared associations only.
    Diagnostics,
    /// Separate measurement populations and actual sample distributions.
    Measurements,
}
/// Typed, reusable selected question, with filters applied before denominator selection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceQuestion {
    /// Finite question.
    #[serde(default)]
    pub operation: EvidenceOperation,
    /// Exact owner-declared crate; no path inference.
    pub crate_name: Option<String>,
    /// Exact declared context identity.
    pub context_id: Option<String>,
    /// Exact subject outcome, applied after denominator selection.
    pub status: Option<String>,
    /// Bounded output rows, default 1024, maximum 10000.
    pub limit: Option<usize>,
}
/// Explicit current owner observation, supplied separately from an imported cut.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSourceObservation {
    /// Original authority.
    pub authority: String,
    /// Original codec.
    pub codec: String,
    /// Original source.
    pub source_id: String,
    /// None explicitly means source missing; Some permits exact stale-cut comparison.
    pub snapshot_digest: Option<String>,
}
/// Parsed selected sources and one native engine assembly. Reuse for sibling questions.
pub struct EvidenceAnalysis {
    context: SessionContext,
    snapshots: Vec<EvidenceSnapshot>,
    generations: Vec<String>,
    history: bool,
    // Retain the same native assembly/pool for the entire selected analysis.
    _factory: EngineFactory,
}
impl EvidenceAnalysis {
    /// Read only an explicitly supplied compact snapshot. Never follow its references.
    pub fn read_snapshot(path: &Path) -> Result<EvidenceSnapshot, EvidenceError> {
        let metadata = std::fs::symlink_metadata(path).map_err(invalid)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 4 * 1024 * 1024 {
            return Err(invalid("bounded supplied snapshot file required"));
        }
        let snapshot: EvidenceSnapshot = serde_json::from_slice(&std::fs::read(path).map_err(invalid)?).map_err(invalid)?;
        snapshot.validate()?;
        Ok(snapshot)
    }
    /// Select exact source versions. Identical imports deduplicate; changed versions
    /// of one original source require an explicit history question.
    pub fn new(sources: Vec<EvidenceSnapshot>, history: bool) -> Result<Self, EvidenceError> {
        if sources.is_empty() || sources.len() > 64 { return Err(invalid("select 1..=64 explicit evidence sources")); }
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 * 1024 * 1024));
        let runtime = RuntimeEnvBuilder::new().with_memory_pool(pool.clone()).build_arc().map_err(invalid)?;
        let one = std::num::NonZeroUsize::new(1).ok_or_else(|| invalid("invalid query budget"))?;
        let factory = EngineFactory::new(runtime, pool, ExecutionSettings::default(), ThreadBudget { pool_threads: one, target_partitions: one }, native_engine_profile()).map_err(invalid)?;
        let context = SessionContext::new_with_state(factory.native_state().clone());
        let registry = pse_schema::registry().map_err(invalid)?;
        let validation = pse_relations::validate::ValidationContext::new(registry, pse_engine::validation::NativeValidation(factory.native_state().clone()));
        let mut builder = pse_relations::columnar::RowBuilder::<pse_model::generated::runtime::evidence_facts::Row>::new(&validation).map_err(invalid)?;
        let mut snapshots = Vec::new();
        let mut generations = Vec::new();
        let mut identities = BTreeSet::new();
        let mut originals = BTreeSet::new();
        let mut contents = BTreeMap::new();
        for snapshot in sources {
            let (generation, content) = snapshot.identity()?;
            if let Some(old) = contents.insert(generation.clone(), content.clone()) && old != content { return Err(EvidenceError::Reused); }
            if !identities.insert(generation.clone()) { continue; }
            if !originals.insert((snapshot.authority.clone(), snapshot.codec.clone(), snapshot.source_id.clone())) && !history {
                return Err(invalid("ambiguous source versions; select one version or explicitly request history"));
            }
            for fact in &snapshot.facts { builder.push(fact.to_row(&generation)?).map_err(invalid)?; }
            generations.push(generation);
            snapshots.push(snapshot);
        }
        context.register_batch("evidence", builder.finish().map_err(invalid)?.into_batch()).map_err(invalid)?;
        Ok(Self { context, snapshots, generations, history, _factory: factory })
    }
    async fn sql(&self, query: &str) -> Result<Vec<RecordBatch>, EvidenceError> {
        self.context.sql(query).await.map_err(invalid)?.collect().await.map_err(invalid)
    }
    /// Run one finite relational question over already parsed inputs. Responses retain
    /// all-source coverage and independently selected test denominator even when filtered.
    pub async fn ask(&mut self, question: &EvidenceQuestion, observations: &[EvidenceSourceObservation]) -> Result<serde_json::Value, EvidenceError> {
        let limit = question.limit.unwrap_or(1024);
        if limit == 0 || limit > 10000 { return Err(invalid("evidence output limit must be 1..=10000")); }
        for text in [&question.crate_name, &question.context_id, &question.status].into_iter().flatten() {
            if text.len() > 4096 { return Err(invalid("query text exceeds compact identity bound")); }
        }
        let mut selected = self.context.table("evidence").await.map_err(invalid)?;
        if let Some(value) = &question.crate_name { selected = selected.filter(col("crate_name").eq(lit(value.clone()))).map_err(invalid)?; }
        if let Some(value) = &question.context_id { selected = selected.filter(col("context_id").eq(lit(value.clone()))).map_err(invalid)?; }
        // This ephemeral view belongs only to this question; ask takes &mut self to
        // prevent concurrently replacing another question's selection.
        self.context.deregister_table("selected").map_err(invalid)?;
        self.context.register_table("selected", selected.into_view()).map_err(invalid)?;
        let denominator = json_rows(&self.sql("SELECT count(*) AS selected_tests, count(crate_name) AS declared_crate_tests, count(context_id) AS declared_context_tests FROM selected WHERE kind IN ('rust_test','python_test')").await?)?;
        let mut subjects = self.context.sql(match question.operation {
            EvidenceOperation::Subjects => "SELECT * FROM selected WHERE kind IN ('rust_test','python_test')",
            EvidenceOperation::Failures => "SELECT * FROM selected WHERE kind IN ('rust_test','python_test') AND status IN ('failure','failed','error')",
            EvidenceOperation::Missing => "SELECT * FROM selected WHERE kind IN ('rust_test','python_test') AND status IN ('unreported','not_run','incomplete')",
            EvidenceOperation::Distribution => "SELECT crate_name,context_id,status,count(*) AS subject_count FROM selected WHERE kind IN ('rust_test','python_test') GROUP BY crate_name,context_id,status",
            EvidenceOperation::Diagnostics => "SELECT * FROM selected WHERE kind IN ('diagnostic_tool','diagnostic_target','diagnostic_science','diagnostic_artifact')",
            EvidenceOperation::Measurements => "SELECT generation,context_id,population_id,status,count(*) AS sample_count,min(value_ns) AS min_ns,max(value_ns) AS max_ns,avg(value_ns) AS mean_ns FROM selected WHERE kind='criterion_sample' GROUP BY generation,context_id,population_id,status",
        }).await.map_err(invalid)?;
        if let Some(value) = &question.status { subjects = subjects.filter(col("status").eq(lit(value.clone()))).map_err(invalid)?; }
        let batches = subjects.limit(0, Some(limit + 1)).map_err(invalid)?.collect().await.map_err(invalid)?;
        let mut rows = json_rows(&batches)?;
        let truncated = rows.len() > limit;
        rows.truncate(limit);
        let mut output = serde_json::json!({"operation":question.operation,"denominator":denominator,"rows":rows,"truncated":truncated,"limit":limit,"history":self.history,"sources":self.sources(observations),"detail_access":"separate explicit owner or supplied-copy selection"});
        if matches!(question.operation, EvidenceOperation::Diagnostics) {
            output["declared_links"] = serde_json::to_value(json_rows(&self.sql("SELECT generation,member,link_relation,link_target,status FROM evidence WHERE kind='declared_link' ORDER BY generation,member LIMIT 10001").await?)?).map_err(invalid)?;
            output["linked_actions"] = serde_json::to_value(json_rows(&self.sql("SELECT l.generation AS referring_generation, d.generation AS diagnostic_generation, d.status AS tool_outcome FROM evidence l JOIN evidence d ON l.link_target=d.detail_ref WHERE l.kind='declared_link' AND l.link_relation='diagnostic_action' AND d.kind='diagnostic_tool' LIMIT 10001").await?)?).map_err(invalid)?;
            output["association_rule"] = "declared exact references only; absent correspondence is unknown".into();
        }
        if matches!(question.operation, EvidenceOperation::Measurements) {
            output["populations"] = serde_json::to_value(json_rows(&self.sql("SELECT generation,member,context_id,population_id,status FROM selected WHERE kind='measurement_population' ORDER BY generation,member LIMIT 10001").await?)?).map_err(invalid)?;
            output["association"] = serde_json::to_value(json_rows(&self.sql("SELECT generation,population_id,status FROM selected WHERE kind='measurement_association' LIMIT 10001").await?)?).map_err(invalid)?;
            output["capture_costs"] = serde_json::to_value(json_rows(&self.sql("SELECT generation,population_id,value_ns,status FROM selected WHERE kind='capture_cost' LIMIT 10001").await?)?).map_err(invalid)?;
            output["comparability"] = self.comparability();
            output["scope"] = "actual supplied CSV samples per original source/context/population; no invocation-membership, regression or causality inference".into();
        }
        Ok(output)
    }
    fn sources(&self, observations: &[EvidenceSourceObservation]) -> serde_json::Value {
        self.snapshots.iter().zip(&self.generations).map(|(snapshot,generation)| {
            let observed = observations.iter().find(|v| v.authority==snapshot.authority && v.codec==snapshot.codec && v.source_id==snapshot.source_id);
            let freshness = match observed { None=>"imported-cut-only", Some(v) if v.snapshot_digest.is_none()=>"missing", Some(v) if v.snapshot_digest.as_deref()!=Some(snapshot.snapshot_digest.as_str())=>"stale", Some(_)=>"matches-supplied-observation" };
            serde_json::json!({"generation":generation,"authority":snapshot.authority,"codec":snapshot.codec,"source_id":snapshot.source_id,"snapshot_digest":snapshot.snapshot_digest,"coverage":snapshot.coverage,"expected_members":snapshot.expected_members.len(),"freshness":freshness})
        }).collect()
    }
    fn comparability(&self) -> serde_json::Value {
        let fields = ["workload","input","revision","build","profile","features","provider","isa","capture","source","environment","host"];
        self.snapshots.iter().zip(&self.generations).flat_map(|(snapshot,generation)| snapshot.facts.iter().filter(|f| f.kind=="context").map(move |f| (f,generation)))
            .map(|(fact,generation)| {
                let value: serde_json::Value = fact.context_json.as_deref().and_then(|v| serde_json::from_str(v).ok()).unwrap_or_default();
                let unknown:Vec<_> = fields.iter().filter(|key| value[**key].is_null()).collect();
                serde_json::json!({"generation":generation,"context_id":fact.context_id,"context":value,"unknown_fields":unknown,"comparable":unknown.is_empty(),"aggregation_scope":"one original source/context/population; unknown conditions never merged across sources"})
            }).collect()
    }
}
/// Convert only the finite query result scalar types. No text re-parsing of source facts.
fn json_rows(batches: &[RecordBatch]) -> Result<Vec<serde_json::Value>, EvidenceError> {
    let mut rows = Vec::new();
    for batch in batches {
        for index in 0..batch.num_rows() {
            let mut row = serde_json::Map::new();
            for (field, array) in batch.schema().fields().iter().zip(batch.columns()) {
                let value = if array.is_null(index) { serde_json::Value::Null } else { match array.data_type() {
                    DataType::Utf8 => serde_json::Value::String(array.as_any().downcast_ref::<StringArray>().ok_or_else(|| invalid("string result layout"))?.value(index).into()),
                    DataType::Utf8View => serde_json::Value::String(array.as_any().downcast_ref::<StringViewArray>().ok_or_else(|| invalid("string view result layout"))?.value(index).into()),
                    DataType::Int64 => serde_json::json!(array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| invalid("int result layout"))?.value(index)),
                    DataType::UInt64 => serde_json::json!(array.as_any().downcast_ref::<UInt64Array>().ok_or_else(|| invalid("uint result layout"))?.value(index)),
                    DataType::Float64 => serde_json::json!(array.as_any().downcast_ref::<Float64Array>().ok_or_else(|| invalid("float result layout"))?.value(index)),
                    value => return Err(invalid(format!("unexpected evidence result scalar {value}"))),
                }};
                row.insert(field.name().clone(), value);
            }
            rows.push(serde_json::Value::Object(row));
        }
    }
    Ok(rows)
}
