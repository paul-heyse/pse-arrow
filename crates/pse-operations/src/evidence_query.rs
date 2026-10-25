// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite selected evidence questions. No service, scientific runtime, source traversal,
//! automatic detail reads or public arbitrary SQL. Sources parse once and become the
//! registry-owned Arrow relation, shared by every question in an analysis.
mod comparison;
use crate::evidence::{EvidenceError, EvidenceSnapshot};
pub use comparison::*;
use datafusion::{
    arrow::{
        array::{Array, Float64Array, Int64Array, StringArray, StringViewArray, UInt64Array},
        datatypes::DataType,
        record_batch::RecordBatch,
    },
    execution::{
        context::SessionContext,
        memory_pool::{GreedyMemoryPool, MemoryPool},
        runtime_env::RuntimeEnvBuilder,
    },
    prelude::{col, lit},
};
use pse_engine::session::{EngineFactory, ExecutionSettings, ThreadBudget, native_engine_profile};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::Arc,
};

fn invalid(error: impl std::fmt::Display) -> EvidenceError {
    EvidenceError::Configuration(error.to_string())
}
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
/// Optional detail selected before acquisition; each section owns its page bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSection {
    /// Selected contexts, independent of primary-row display limits.
    Contexts,
    /// Exact diagnostic declarations.
    DeclaredLinks,
    /// Exact diagnostic action correspondence.
    LinkedActions,
    /// Original measurement populations and censoring.
    Populations,
    /// Original invocation-membership association.
    Association,
    /// Recorded capture overhead.
    CaptureCosts,
    /// Actual invocation timing distributions.
    InvocationDistributions,
    /// Original reported measurement statistics.
    ReportedStatistics,
    /// Per-context conditions underlying the full-population assessment.
    Comparability,
}
impl EvidenceSection {
    fn name(self) -> &'static str {
        match self {
            Self::Contexts => "contexts",
            Self::DeclaredLinks => "declared_links",
            Self::LinkedActions => "linked_actions",
            Self::Populations => "populations",
            Self::Association => "association",
            Self::CaptureCosts => "capture_costs",
            Self::InvocationDistributions => "invocation_distributions",
            Self::ReportedStatistics => "reported_statistics",
            Self::Comparability => "comparability",
        }
    }
    fn allowed(self, operation: EvidenceOperation) -> bool {
        match self {
            Self::Contexts => true,
            Self::DeclaredLinks | Self::LinkedActions => {
                matches!(operation, EvidenceOperation::Diagnostics)
            }
            _ => matches!(operation, EvidenceOperation::Measurements),
        }
    }
    fn query(self) -> Option<&'static str> {
        Some(match self {
            Self::Contexts => {
                "SELECT DISTINCT e.generation,e.context_id,e.context_json FROM evidence e JOIN (SELECT DISTINCT generation,context_id FROM selected WHERE context_id IS NOT NULL) s ON e.generation=s.generation AND e.context_id=s.context_id WHERE e.kind='context' ORDER BY e.generation,e.context_id,e.context_json"
            }
            Self::DeclaredLinks => {
                "SELECT generation,member,link_relation,link_target,status FROM evidence WHERE kind='declared_link' AND (generation IN (SELECT DISTINCT generation FROM selected WHERE kind='diagnostic_tool') OR link_target IN (SELECT detail_ref FROM selected WHERE kind='diagnostic_tool')) ORDER BY generation,member"
            }
            Self::LinkedActions => {
                "SELECT l.generation AS referring_generation, d.generation AS diagnostic_generation, d.status AS tool_outcome FROM evidence l JOIN selected d ON l.link_target=d.detail_ref WHERE l.kind='declared_link' AND l.link_relation='diagnostic_action' AND d.kind='diagnostic_tool' ORDER BY referring_generation,diagnostic_generation,tool_outcome"
            }
            Self::Populations => {
                "SELECT generation,member,context_id,population_id,status FROM selected WHERE kind='measurement_population' ORDER BY generation,member"
            }
            Self::Association => {
                "SELECT generation,population_id,status FROM selected WHERE kind='measurement_association' ORDER BY generation,member"
            }
            Self::CaptureCosts => {
                "SELECT generation,population_id,value_ns,status FROM selected WHERE kind='capture_cost' ORDER BY generation,member"
            }
            Self::InvocationDistributions => {
                "SELECT generation,context_id,population_id,status,count(*) AS invocation_count,count(value_ns) AS timed_invocations,min(value_ns) AS min_ns,max(value_ns) AS max_ns,avg(value_ns) AS mean_ns FROM selected WHERE kind='harness_invocation' GROUP BY generation,context_id,population_id,status ORDER BY generation,context_id,population_id,status"
            }
            Self::ReportedStatistics => {
                "SELECT generation,context_id,population_id,member,status,value_ns FROM selected WHERE kind='measurement_summary' ORDER BY generation,member"
            }
            Self::Comparability => return None,
        })
    }
}
/// An explicit finite page of one optional section on the immutable source cut.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSectionSelection {
    /// Returned detail rows, 1..=10000.
    pub limit: usize,
    /// Continue with the previous next_offset on this same analysis and question.
    #[serde(default)]
    pub offset: usize,
}
impl EvidenceSectionSelection {
    fn validate(self) -> Result<(), EvidenceError> {
        if self.limit == 0
            || self.limit > 10000
            || self.offset.checked_add(self.limit + 1).is_none()
        {
            return Err(invalid(
                "section limit must be 1..=10000 and offset must not overflow",
            ));
        }
        Ok(())
    }
    fn metadata(self, returned: usize, truncated: bool) -> serde_json::Value {
        serde_json::json!({"limit": self.limit, "offset": self.offset, "returned": returned, "truncated": truncated, "complete": self.offset == 0 && !truncated, "next_offset": if truncated { Some(self.offset + returned) } else { None }})
    }
}
struct SectionPage {
    rows: Vec<serde_json::Value>,
    truncated: bool,
}
struct ComparabilityAssessment {
    summary: serde_json::Value,
    detail: Option<SectionPage>,
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
    /// Optional detail; empty means no subsidiary section is acquired.
    #[serde(default)]
    pub sections: BTreeMap<EvidenceSection, EvidenceSectionSelection>,
}
/// Explicit current owner observation, supplied separately from an imported cut.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
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
    contexts: BTreeMap<(String, String), serde_json::Value>,
    contents: BTreeMap<String, String>,
    // Retain the same native assembly/pool for the entire selected analysis.
    _factory: EngineFactory,
}
impl std::fmt::Debug for EvidenceAnalysis {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EvidenceAnalysis")
            .field("sources", &self.snapshots.len())
            .field("history", &self.history)
            .finish_non_exhaustive()
    }
}
impl EvidenceAnalysis {
    /// Read only an explicitly supplied compact snapshot. Never follow its references.
    pub fn read_snapshot(path: &Path) -> Result<EvidenceSnapshot, EvidenceError> {
        let metadata = std::fs::symlink_metadata(path).map_err(invalid)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > 4 * 1024 * 1024
        {
            return Err(invalid("bounded supplied snapshot file required"));
        }
        let snapshot: EvidenceSnapshot =
            serde_json::from_slice(&std::fs::read(path).map_err(invalid)?).map_err(invalid)?;
        snapshot.validate()?;
        Ok(snapshot)
    }
    /// Select exact source versions. Identical imports deduplicate; changed versions
    /// of one original source require an explicit history question.
    pub fn new(sources: Vec<EvidenceSnapshot>, history: bool) -> Result<Self, EvidenceError> {
        if sources.is_empty() || sources.len() > 64 {
            return Err(invalid("select 1..=64 explicit evidence sources"));
        }
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128 * 1024 * 1024));
        let runtime = RuntimeEnvBuilder::new()
            .with_memory_pool(pool.clone())
            .build_arc()
            .map_err(invalid)?;
        let one = std::num::NonZeroUsize::new(1).ok_or_else(|| invalid("invalid query budget"))?;
        let factory = EngineFactory::new(
            runtime,
            pool,
            ExecutionSettings::default(),
            ThreadBudget {
                pool_threads: one,
                target_partitions: one,
            },
            native_engine_profile(),
        )
        .map_err(invalid)?;
        let context = SessionContext::new_with_state(factory.native_state().clone());
        // The common engine deliberately disables automatic namespace creation.
        // This supplied-only analysis owns exactly one empty catalog/schema.
        use datafusion::catalog::{CatalogProvider, MemoryCatalogProvider, MemorySchemaProvider};
        let defaults = context.state().config_options().catalog.clone();
        let catalog_name = defaults.default_catalog.clone();
        let schema_name = defaults.default_schema.clone();
        let catalog = Arc::new(MemoryCatalogProvider::new());
        catalog
            .register_schema(&schema_name, Arc::new(MemorySchemaProvider::new()))
            .map_err(invalid)?;
        context.register_catalog(&catalog_name, catalog);
        let registry = pse_schema::registry().map_err(invalid)?;
        let validation = pse_relations::validate::ValidationContext::new(
            registry,
            pse_engine::validation::NativeValidation(factory.native_state().clone()),
        );
        let mut builder = pse_relations::columnar::RowBuilder::<
            pse_model::generated::runtime::evidence_facts::Row,
        >::new(&validation)
        .map_err(invalid)?;
        let mut snapshots = Vec::new();
        let mut generations = Vec::new();
        let mut identities = BTreeSet::new();
        let mut originals = BTreeSet::new();
        let mut contents = BTreeMap::new();
        let mut contexts = BTreeMap::new();
        for snapshot in sources {
            let (generation, content) = snapshot.identity()?;
            if let Some(old) = contents.insert(generation.clone(), content.clone())
                && old != content
            {
                return Err(EvidenceError::Reused);
            }
            if !identities.insert(generation.clone()) {
                continue;
            }
            if !originals.insert((
                snapshot.authority.clone(),
                snapshot.codec.clone(),
                snapshot.source_id.clone(),
            )) && !history
            {
                return Err(invalid(
                    "ambiguous source versions; select one version or explicitly request history",
                ));
            }
            for fact in &snapshot.facts {
                if fact.kind == "context"
                    && let (Some(id), Some(value)) = (&fact.context_id, &fact.context_json)
                {
                    contexts.insert(
                        (generation.clone(), id.clone()),
                        serde_json::from_str(value).map_err(invalid)?,
                    );
                }
                builder.push(fact.to_row(&generation)?).map_err(invalid)?;
            }
            generations.push(generation);
            snapshots.push(snapshot);
        }
        context
            .register_batch("evidence", builder.finish().map_err(invalid)?.into_batch())
            .map_err(invalid)?;
        Ok(Self {
            context,
            snapshots,
            generations,
            history,
            contexts,
            contents,
            _factory: factory,
        })
    }
    async fn sql(&self, query: &str) -> Result<Vec<RecordBatch>, EvidenceError> {
        self.context
            .sql(query)
            .await
            .map_err(invalid)?
            .collect()
            .await
            .map_err(invalid)
    }
    /// Run one finite relational question over already parsed inputs. Responses retain
    /// all-source coverage and independently selected test denominator even when filtered.
    pub async fn ask(
        &mut self,
        question: &EvidenceQuestion,
        observations: &[EvidenceSourceObservation],
    ) -> Result<serde_json::Value, EvidenceError> {
        let limit = question.limit.unwrap_or(1024);
        if limit == 0 || limit > 10000 {
            return Err(invalid("evidence output limit must be 1..=10000"));
        }
        for (section, selection) in &question.sections {
            selection.validate()?;
            if !section.allowed(question.operation) {
                return Err(invalid(format!(
                    "section {} does not belong to this evidence operation",
                    section.name()
                )));
            }
        }
        for text in [&question.crate_name, &question.context_id, &question.status]
            .into_iter()
            .flatten()
        {
            if text.len() > 4096 {
                return Err(invalid("query text exceeds compact identity bound"));
            }
        }
        let mut observed_sources = BTreeSet::new();
        for observation in observations {
            if !observed_sources.insert((
                &observation.authority,
                &observation.codec,
                &observation.source_id,
            )) {
                return Err(invalid("ambiguous supplied source freshness observations"));
            }
            if observation.snapshot_digest.as_ref().is_some_and(|digest| {
                digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }) {
                return Err(invalid(
                    "supplied source observation needs an exact digest or explicit missing state",
                ));
            }
        }
        let mut selected = self.context.table("evidence").await.map_err(invalid)?;
        if let Some(value) = &question.crate_name {
            selected = selected
                .filter(col("crate_name").eq(lit(value.clone())))
                .map_err(invalid)?;
        }
        if let Some(value) = &question.context_id {
            selected = selected
                .filter(col("context_id").eq(lit(value.clone())))
                .map_err(invalid)?;
        }
        // This ephemeral view belongs only to this question; ask takes &mut self to
        // prevent concurrently replacing another question's selection.
        self.context.deregister_table("selected").map_err(invalid)?;
        self.context
            .register_table("selected", selected.into_view())
            .map_err(invalid)?;
        let denominator = json_rows(&self.sql("SELECT count(*) AS selected_tests, count(crate_name) AS declared_crate_tests, count(context_id) AS declared_context_tests FROM selected WHERE kind IN ('rust_test','python_test')").await?)?;
        let mut subjects = self.context.sql(match question.operation {
            EvidenceOperation::Subjects => "SELECT * FROM selected WHERE kind IN ('rust_test','python_test') ORDER BY generation,member",
            EvidenceOperation::Failures => "SELECT * FROM selected WHERE kind IN ('rust_test','python_test') AND status IN ('failure','failed','error') ORDER BY generation,member",
            EvidenceOperation::Missing => "SELECT * FROM selected WHERE kind IN ('rust_test','python_test') AND status IN ('unreported','not_run','incomplete') ORDER BY generation,member",
            EvidenceOperation::Distribution => "SELECT crate_name,context_id,status,count(*) AS subject_count FROM selected WHERE kind IN ('rust_test','python_test') GROUP BY crate_name,context_id,status ORDER BY crate_name,context_id,status",
            EvidenceOperation::Diagnostics => "SELECT * FROM selected WHERE kind IN ('diagnostic_tool','diagnostic_target','diagnostic_science','diagnostic_artifact') ORDER BY generation,member",
            EvidenceOperation::Measurements => "SELECT generation,context_id,population_id,status,count(*) AS sample_count,min(value_ns) AS min_ns,max(value_ns) AS max_ns,avg(value_ns) AS mean_ns FROM selected WHERE kind='criterion_sample' GROUP BY generation,context_id,population_id,status ORDER BY generation,context_id,population_id,status",
        }).await.map_err(invalid)?;
        if let Some(value) = &question.status {
            subjects = subjects
                .filter(col("status").eq(lit(value.clone())))
                .map_err(invalid)?;
        }
        let batches = subjects
            .limit(0, Some(limit + 1))
            .map_err(invalid)?
            .collect()
            .await
            .map_err(invalid)?;
        let mut rows = json_rows(&batches)?;
        let truncated = rows.len() > limit;
        rows.truncate(limit);
        let mut output = serde_json::json!({"operation":question.operation,"denominator":denominator,"rows":rows,"truncated":truncated,"limit":limit,"history":self.history,"sources":self.sources(observations),"detail_access":"separate explicit owner or supplied-copy selection"});
        output["sections"] = serde_json::json!({});
        if matches!(question.operation, EvidenceOperation::Diagnostics) {
            output["association_rule"] =
                "declared exact references only; absent correspondence is unknown".into();
        }
        if matches!(question.operation, EvidenceOperation::Measurements) {
            let selected_contexts = json_rows(&self.sql("SELECT DISTINCT generation,context_id FROM selected WHERE kind IN ('criterion_sample','measurement_population','measurement_association','capture_cost','harness_invocation','measurement_summary') AND context_id IS NOT NULL ORDER BY generation,context_id").await?)?;
            let contexts = selected_contexts
                .iter()
                .map(|row| {
                    Ok((
                        row["generation"]
                            .as_str()
                            .ok_or_else(|| invalid("selected generation identity"))?
                            .to_owned(),
                        row["context_id"]
                            .as_str()
                            .ok_or_else(|| invalid("selected context identity"))?
                            .to_owned(),
                    ))
                })
                .collect::<Result<BTreeSet<_>, EvidenceError>>()?;
            let missing_context = json_rows(&self.sql("SELECT count(*) AS samples_without_context FROM selected WHERE kind='criterion_sample' AND context_id IS NULL").await?)?;
            let selection = question.sections.get(&EvidenceSection::Comparability);
            let assessment = self.comparability(&contexts, selection)?;
            output["comparability_assessment"] = assessment.summary;
            output["comparability_assessment"]["samples_without_context"] = missing_context
                .first()
                .and_then(|row| row.get("samples_without_context"))
                .ok_or_else(|| invalid("measurement context denominator is unavailable"))?
                .clone();
            if let Some(selection) = selection
                && let Some(detail) = assessment.detail
            {
                output["sections"]["comparability"] =
                    selection.metadata(detail.rows.len(), detail.truncated);
                output["comparability"] = detail.rows.into();
            }
            output["scope"] = "actual supplied CSV samples per original source/context/population; no invocation-membership, regression or causality inference".into();
        }
        for (section, selection) in &question.sections {
            let Some(query) = section.query() else {
                continue;
            };
            let batches = self
                .context
                .sql(query)
                .await
                .map_err(invalid)?
                .limit(selection.offset, Some(selection.limit + 1))
                .map_err(invalid)?
                .collect()
                .await
                .map_err(invalid)?;
            let mut rows = json_rows(&batches)?;
            let truncated = rows.len() > selection.limit;
            rows.truncate(selection.limit);
            output["sections"][section.name()] = selection.metadata(rows.len(), truncated);
            output[section.name()] = rows.into();
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
    fn comparability(
        &self,
        contexts: &BTreeSet<(String, String)>,
        selection: Option<&EvidenceSectionSelection>,
    ) -> Result<ComparabilityAssessment, EvidenceError> {
        let fields = [
            "workload",
            "input",
            "revision",
            "build",
            "profile",
            "features",
            "provider",
            "isa",
            "capture",
            "source",
            "environment",
            "host",
        ];
        let mut unknown_contexts = 0;
        let mut rows = Vec::new();
        // Assess every selected context; display limits cannot turn an unknown
        // condition outside the requested page into a known comparison basis.
        for (index, (generation, context_id)) in contexts.iter().enumerate() {
            let value = self
                .contexts
                .get(&(generation.clone(), context_id.clone()))
                .ok_or_else(|| invalid("selected context declaration is unavailable"))?;
            let unknown: Vec<_> = fields.iter().filter(|key| value[**key].is_null()).collect();
            if !unknown.is_empty() {
                unknown_contexts += 1;
            }
            if let Some(selection) = selection
                && index >= selection.offset
                && rows.len() <= selection.limit
            {
                rows.push(serde_json::json!({"generation":generation,"context_id":context_id,"context":value,"unknown_fields":unknown,"comparable":unknown.is_empty(),"aggregation_scope":"one original source/context/population; unknown conditions never merged across sources"}));
            }
        }
        let assessment = serde_json::json!({"selected_contexts":contexts.len(),"unknown_contexts":unknown_contexts,"context_basis":if contexts.is_empty() { "unavailable" } else if unknown_contexts != 0 { "partial" } else { "declared" },"cross_source_comparability":"not_established","aggregation_scope":"one original source/context/population"});
        let detail = selection.map(|selection| {
            let truncated = rows.len() > selection.limit;
            rows.truncate(selection.limit);
            SectionPage { rows, truncated }
        });
        Ok(ComparabilityAssessment {
            summary: assessment,
            detail,
        })
    }
}
/// Convert only the finite query result scalar types. No text re-parsing of source facts.
fn json_rows(batches: &[RecordBatch]) -> Result<Vec<serde_json::Value>, EvidenceError> {
    let mut rows = Vec::new();
    for batch in batches {
        for index in 0..batch.num_rows() {
            let mut row = serde_json::Map::new();
            for (field, array) in batch.schema().fields().iter().zip(batch.columns()) {
                let value = if array.is_null(index) {
                    serde_json::Value::Null
                } else {
                    match array.data_type() {
                        DataType::Utf8 => serde_json::Value::String(
                            array
                                .as_any()
                                .downcast_ref::<StringArray>()
                                .ok_or_else(|| invalid("string result layout"))?
                                .value(index)
                                .into(),
                        ),
                        DataType::Utf8View => serde_json::Value::String(
                            array
                                .as_any()
                                .downcast_ref::<StringViewArray>()
                                .ok_or_else(|| invalid("string view result layout"))?
                                .value(index)
                                .into(),
                        ),
                        DataType::Int64 => serde_json::json!(
                            array
                                .as_any()
                                .downcast_ref::<Int64Array>()
                                .ok_or_else(|| invalid("int result layout"))?
                                .value(index)
                        ),
                        DataType::UInt64 => serde_json::json!(
                            array
                                .as_any()
                                .downcast_ref::<UInt64Array>()
                                .ok_or_else(|| invalid("uint result layout"))?
                                .value(index)
                        ),
                        DataType::Float64 => serde_json::json!(
                            array
                                .as_any()
                                .downcast_ref::<Float64Array>()
                                .ok_or_else(|| invalid("float result layout"))?
                                .value(index)
                        ),
                        value => {
                            return Err(invalid(format!(
                                "unexpected evidence result scalar {value}"
                            )));
                        }
                    }
                };
                row.insert(field.name().clone(), value);
            }
            rows.push(serde_json::Value::Object(row));
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::EvidenceFact;
    fn sections(
        values: &[EvidenceSection],
        limit: usize,
    ) -> BTreeMap<EvidenceSection, EvidenceSectionSelection> {
        values
            .iter()
            .map(|section| (*section, EvidenceSectionSelection { limit, offset: 0 }))
            .collect()
    }
    fn source(id: &str) -> EvidenceSnapshot {
        let mut facts = Vec::new();
        for (member, status, owner) in [
            ("failure", "failure", Some("pse-alpha")),
            ("missing", "unreported", Some("pse-alpha")),
            ("skipped", "skipped", Some("pse-beta")),
            ("reused", "reused", None),
        ] {
            facts.push(EvidenceFact {
                member: member.into(),
                kind: "rust_test".into(),
                status: status.into(),
                binary_id: Some("actual-binary".into()),
                test_id: Some(format!("actual-binary::{member}")),
                crate_name: owner.map(str::to_owned),
                ..Default::default()
            });
        }
        facts.push(EvidenceFact {
            member: "command".into(),
            kind: "invocation".into(),
            status: "command_failure".into(),
            ..Default::default()
        });
        EvidenceSnapshot {
            version: 2,
            authority: "authentic-owner".into(),
            codec: "exact-resource-files-invocation-v2".into(),
            source_id: id.into(),
            snapshot_digest: "a".repeat(64),
            coverage: "partial".into(),
            expected_members: facts.iter().map(|f| f.member.clone()).collect(),
            facts,
        }
    }
    #[tokio::test]
    async fn evidence_query_denominators_filters_and_distinct_outcomes() {
        let snapshot = source("one");
        let mut analysis = EvidenceAnalysis::new(vec![snapshot.clone(), snapshot], false).unwrap();
        let result = analysis
            .ask(
                &EvidenceQuestion {
                    operation: EvidenceOperation::Failures,
                    ..Default::default()
                },
                &[],
            )
            .await
            .unwrap();
        assert_eq!(result["denominator"][0]["selected_tests"], 4);
        assert_eq!(result["rows"].as_array().unwrap().len(), 1);
        assert_eq!(result["rows"][0]["test_id"], "actual-binary::failure");
        assert_eq!(
            result["sources"].as_array().unwrap().len(),
            1,
            "repeated imports cannot duplicate samples"
        );
        let result = analysis
            .ask(
                &EvidenceQuestion {
                    operation: EvidenceOperation::Missing,
                    crate_name: Some("pse-alpha".into()),
                    ..Default::default()
                },
                &[],
            )
            .await
            .unwrap();
        assert_eq!(result["denominator"][0]["selected_tests"], 2);
        assert_eq!(result["rows"][0]["status"], "unreported");
        let result = analysis
            .ask(
                &EvidenceQuestion {
                    operation: EvidenceOperation::Distribution,
                    ..Default::default()
                },
                &[],
            )
            .await
            .unwrap();
        assert_eq!(result["rows"].as_array().unwrap().len(), 4);
    }
    #[tokio::test]
    async fn evidence_query_source_versions_and_explicit_freshness() {
        let original = source("one");
        let mut changed = original.clone();
        changed.snapshot_digest = "b".repeat(64);
        assert!(EvidenceAnalysis::new(vec![original.clone(), changed.clone()], false).is_err());
        let mut history = EvidenceAnalysis::new(vec![original.clone(), changed], true).unwrap();
        let observations = [EvidenceSourceObservation {
            authority: original.authority,
            codec: original.codec,
            source_id: original.source_id,
            snapshot_digest: None,
        }];
        let result = history
            .ask(&Default::default(), &observations)
            .await
            .unwrap();
        assert_eq!(result["sources"][0]["freshness"], "missing");
        let observations = [EvidenceSourceObservation {
            snapshot_digest: Some("a".repeat(64)),
            ..observations[0].clone()
        }];
        let result = history
            .ask(&Default::default(), &observations)
            .await
            .unwrap();
        assert_eq!(
            result["sources"][0]["freshness"],
            "matches-supplied-observation"
        );
        assert_eq!(result["sources"][1]["freshness"], "stale");
    }
    #[tokio::test]
    async fn evidence_query_exact_links_never_temporal_neighbors() {
        let mut invocation = source("invocation");
        invocation.facts.push(EvidenceFact {
            member: "diagnostic".into(),
            kind: "declared_link".into(),
            status: "declared".into(),
            link_relation: Some("diagnostic_action".into()),
            link_target: Some("sha256:recorded".into()),
            ..Default::default()
        });
        invocation.expected_members.push("diagnostic".into());
        let mut action = source("linked");
        action.codec = "active-diagnostic-action-v1".into();
        action.facts = vec![EvidenceFact {
            member: "action".into(),
            kind: "diagnostic_tool".into(),
            status: "tool_failure".into(),
            detail_ref: Some("sha256:recorded".into()),
            ..Default::default()
        }];
        action.expected_members = vec!["action".into()];
        let mut unrelated = action.clone();
        unrelated.source_id = "standalone-nearby".into();
        unrelated.facts[0].detail_ref = Some("sha256:unrelated".into());
        let mut analysis =
            EvidenceAnalysis::new(vec![invocation, action, unrelated], false).unwrap();
        let result = analysis
            .ask(
                &EvidenceQuestion {
                    operation: EvidenceOperation::Diagnostics,
                    sections: sections(
                        &[
                            EvidenceSection::Contexts,
                            EvidenceSection::DeclaredLinks,
                            EvidenceSection::LinkedActions,
                        ],
                        1024,
                    ),
                    ..Default::default()
                },
                &[],
            )
            .await
            .unwrap();
        assert_eq!(result["rows"].as_array().unwrap().len(), 2);
        assert_eq!(result["linked_actions"].as_array().unwrap().len(), 1);
        assert_eq!(result["linked_actions"][0]["tool_outcome"], "tool_failure");
    }
    #[tokio::test]
    async fn evidence_query_context_keeps_all_action_outcomes_and_only_selected_links() {
        let mut sources = Vec::new();
        for name in ["first", "other"] {
            let mut action = source(name);
            action.codec = "active-diagnostic-action-v1".into();
            action.facts = [
                "diagnostic_tool",
                "diagnostic_target",
                "diagnostic_science",
                "diagnostic_artifact",
            ]
            .into_iter()
            .map(|kind| EvidenceFact {
                member: kind.into(),
                kind: kind.into(),
                status: "unknown".into(),
                context_id: Some(name.into()),
                detail_ref: (kind == "diagnostic_tool").then(|| format!("sha256:{name}")),
                ..Default::default()
            })
            .collect();
            action.facts.push(EvidenceFact {member:"context".into(),kind:"context".into(),status:"declared".into(),context_id:Some(name.into()),context_json:Some(serde_json::json!({"workload":null,"input":null,"revision":null,"build":null,"profile":null,"features":null,"provider":null,"isa":null,"capture":{"action":name},"host":null,"environment":null,"source":null,"mode":null,"process":null}).to_string()),..Default::default()});
            action.facts.push(EvidenceFact {
                member: "link".into(),
                kind: "declared_link".into(),
                status: "declared".into(),
                context_id: Some(name.into()),
                link_relation: Some("run".into()),
                link_target: Some(format!("run:{name}")),
                ..Default::default()
            });
            action.expected_members = action.facts.iter().map(|f| f.member.clone()).collect();
            sources.push(action);
        }
        let mut invocation = source("invocation");
        invocation.facts.push(EvidenceFact {
            member: "diagnostic".into(),
            kind: "declared_link".into(),
            status: "declared".into(),
            link_relation: Some("diagnostic_action".into()),
            link_target: Some("sha256:first".into()),
            ..Default::default()
        });
        invocation.expected_members.push("diagnostic".into());
        sources.push(invocation);
        let mut analysis = EvidenceAnalysis::new(sources, false).unwrap();
        let result = analysis
            .ask(
                &EvidenceQuestion {
                    operation: EvidenceOperation::Diagnostics,
                    sections: sections(
                        &[
                            EvidenceSection::Contexts,
                            EvidenceSection::DeclaredLinks,
                            EvidenceSection::LinkedActions,
                        ],
                        1024,
                    ),
                    context_id: Some("first".into()),
                    ..Default::default()
                },
                &[],
            )
            .await
            .unwrap();
        assert_eq!(result["rows"].as_array().unwrap().len(), 4);
        assert_eq!(result["linked_actions"].as_array().unwrap().len(), 1);
        assert_eq!(result["declared_links"].as_array().unwrap().len(), 2);
        assert_eq!(result["contexts"][0]["context_id"], "first");
        assert!(
            !result["declared_links"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["link_target"] == "run:other")
        );
    }
    #[tokio::test]
    async fn evidence_query_measurements_keep_censored_populations_and_source_context() {
        let mut source = source("measurement");
        source.codec = "process-cost-v5".into();
        source.facts = vec![
            EvidenceFact {
                member: "population".into(),
                kind: "measurement_population".into(),
                status: "censored".into(),
                population_id: Some("workload".into()),
                ..Default::default()
            },
            EvidenceFact {
                member: "membership".into(),
                kind: "measurement_association".into(),
                status: "unavailable".into(),
                population_id: Some("workload".into()),
                ..Default::default()
            },
        ];
        for (index, value) in [10.0, 30.0].iter().enumerate() {
            source.facts.push(EvidenceFact {
                member: format!("sample:{index}"),
                kind: "criterion_sample".into(),
                status: "measured".into(),
                population_id: Some("workload".into()),
                sample_id: Some(format!("row:{index}")),
                value_ns: Some(*value),
                ..Default::default()
            });
        }
        source.expected_members = source.facts.iter().map(|f| f.member.clone()).collect();
        let mut other = source.clone();
        other.source_id = "other-build".into();
        let mut analysis =
            EvidenceAnalysis::new(vec![source.clone(), source, other], false).unwrap();
        let result = analysis
            .ask(
                &EvidenceQuestion {
                    operation: EvidenceOperation::Measurements,
                    sections: sections(
                        &[EvidenceSection::Populations, EvidenceSection::Association],
                        1024,
                    ),
                    ..Default::default()
                },
                &[],
            )
            .await
            .unwrap();
        assert_eq!(
            result["rows"].as_array().unwrap().len(),
            2,
            "unknown context cannot merge independent sources"
        );
        assert_eq!(result["rows"][0]["sample_count"], 2);
        assert_eq!(result["rows"][0]["mean_ns"], 20.0);
        assert_eq!(result["populations"][0]["status"], "censored");
        assert_eq!(result["association"][0]["status"], "unavailable");
        assert_eq!(
            result["comparability_assessment"]["context_basis"],
            "unavailable"
        );
        assert_eq!(
            result["comparability_assessment"]["samples_without_context"],
            4
        );
    }
    #[tokio::test]
    async fn optional_sections_are_explicit_independently_bounded_and_continuable() {
        let mut snapshot = source("sections");
        snapshot.facts.extend((0..4).map(|index| EvidenceFact {
            member: format!("link:{index}"),
            kind: "declared_link".into(),
            status: "declared".into(),
            link_relation: Some("diagnostic_action".into()),
            link_target: Some("sha256:action".into()),
            ..Default::default()
        }));
        snapshot.facts.push(EvidenceFact {
            member: "action".into(),
            kind: "diagnostic_tool".into(),
            status: "tool_failure".into(),
            detail_ref: Some("sha256:action".into()),
            ..Default::default()
        });
        snapshot.expected_members = snapshot
            .facts
            .iter()
            .map(|fact| fact.member.clone())
            .collect();
        let mut analysis = EvidenceAnalysis::new(vec![snapshot], false).unwrap();
        let mut question = EvidenceQuestion {
            operation: EvidenceOperation::Diagnostics,
            limit: Some(1),
            ..Default::default()
        };
        let minimal = analysis.ask(&question, &[]).await.unwrap();
        assert!(minimal.get("declared_links").is_none());
        assert!(minimal.get("contexts").is_none());
        assert_eq!(minimal["sections"], serde_json::json!({}));
        assert_eq!(minimal["denominator"][0]["selected_tests"], 4);
        question.sections = sections(&[EvidenceSection::DeclaredLinks], 1);
        let first = analysis.ask(&question, &[]).await.unwrap();
        assert_eq!(first["declared_links"].as_array().unwrap().len(), 1);
        assert_eq!(first["sections"]["declared_links"]["next_offset"], 1);
        assert_eq!(first["sections"]["declared_links"]["truncated"], true);
        assert!(first.get("linked_actions").is_none());
        question
            .sections
            .get_mut(&EvidenceSection::DeclaredLinks)
            .unwrap()
            .offset = 1;
        let second = analysis.ask(&question, &[]).await.unwrap();
        assert_ne!(
            first["declared_links"][0]["member"],
            second["declared_links"][0]["member"]
        );
        assert_eq!(second["sections"]["declared_links"]["offset"], 1);
        assert_eq!(second["denominator"], first["denominator"]);
        question.sections.insert(
            EvidenceSection::LinkedActions,
            EvidenceSectionSelection {
                limit: 8,
                offset: 0,
            },
        );
        let independent = analysis.ask(&question, &[]).await.unwrap();
        assert_eq!(independent["linked_actions"].as_array().unwrap().len(), 4);
        assert_eq!(
            independent["sections"]["linked_actions"]["truncated"],
            false
        );
        assert_eq!(independent["sections"]["declared_links"]["truncated"], true);
    }

    #[tokio::test]
    async fn section_requests_refuse_invalid_bounds_or_operation_without_silent_omission() {
        let mut analysis = EvidenceAnalysis::new(vec![source("bounds")], false).unwrap();
        let mut question = EvidenceQuestion {
            sections: sections(&[EvidenceSection::CaptureCosts], 1),
            ..Default::default()
        };
        assert!(analysis.ask(&question, &[]).await.is_err());
        question.operation = EvidenceOperation::Measurements;
        question
            .sections
            .get_mut(&EvidenceSection::CaptureCosts)
            .unwrap()
            .limit = 0;
        assert!(analysis.ask(&question, &[]).await.is_err());
        question
            .sections
            .get_mut(&EvidenceSection::CaptureCosts)
            .unwrap()
            .limit = 1;
        question
            .sections
            .get_mut(&EvidenceSection::CaptureCosts)
            .unwrap()
            .offset = usize::MAX;
        assert!(analysis.ask(&question, &[]).await.is_err());
    }

    #[tokio::test]
    async fn original_codec_labels_remain_distinct_for_historical_and_current_imports() {
        for codec in [
            "exact-resource-files-invocation-v2",
            "exact-resource-files-invocation-v3",
            "assessment-v5",
            "assessment-v6",
        ] {
            let mut snapshot = source(codec);
            snapshot.codec = codec.into();
            let mut analysis = EvidenceAnalysis::new(vec![snapshot], false).unwrap();
            let result = analysis
                .ask(&EvidenceQuestion::default(), &[])
                .await
                .unwrap();
            assert_eq!(result["sources"][0]["codec"], codec);
        }
    }

    #[tokio::test]
    async fn large_subsidiary_population_continues_beyond_the_old_fixed_cap() {
        let sources = (0..3)
            .map(|index| {
                let mut snapshot = source(&format!("costs:{index}"));
                snapshot.codec = "process-cost-v5".into();
                snapshot.facts = (0..4000)
                    .map(|row| EvidenceFact {
                        member: format!("cost:{row:05}"),
                        kind: "capture_cost".into(),
                        status: "recorded".into(),
                        value_ns: Some(f64::from(row)),
                        ..Default::default()
                    })
                    .collect();
                snapshot.expected_members = snapshot
                    .facts
                    .iter()
                    .map(|fact| fact.member.clone())
                    .collect();
                snapshot
            })
            .collect();
        let mut analysis = EvidenceAnalysis::new(sources, false).unwrap();
        let mut question = EvidenceQuestion {
            operation: EvidenceOperation::Measurements,
            sections: sections(&[EvidenceSection::CaptureCosts], 10000),
            ..Default::default()
        };
        let first = analysis.ask(&question, &[]).await.unwrap();
        assert_eq!(first["capture_costs"].as_array().unwrap().len(), 10000);
        assert_eq!(first["sections"]["capture_costs"]["truncated"], true);
        assert_eq!(first["sections"]["capture_costs"]["next_offset"], 10000);
        question
            .sections
            .get_mut(&EvidenceSection::CaptureCosts)
            .unwrap()
            .offset = 10000;
        let last = analysis.ask(&question, &[]).await.unwrap();
        assert_eq!(last["capture_costs"].as_array().unwrap().len(), 2000);
        assert_eq!(last["sections"]["capture_costs"]["truncated"], false);
        assert!(last["sections"]["capture_costs"]["next_offset"].is_null());
        assert_eq!(last["sources"], first["sources"]);
    }

    #[tokio::test]
    async fn comparability_assesses_unknown_context_outside_the_display_page() {
        let mut snapshot = source("conditions");
        snapshot.codec = "process-cost-v5".into();
        snapshot.facts.clear();
        for (index, id) in ["a-known", "z-unknown"].iter().enumerate() {
            let mut context = serde_json::Map::new();
            for field in [
                "workload",
                "input",
                "revision",
                "build",
                "profile",
                "features",
                "provider",
                "isa",
                "capture",
                "source",
                "environment",
                "host",
                "mode",
                "process",
            ] {
                context.insert(field.into(), serde_json::json!("declared"));
            }
            if index == 1 {
                context.insert("host".into(), serde_json::Value::Null);
            }
            snapshot.facts.push(EvidenceFact {
                member: format!("context:{id}"),
                kind: "context".into(),
                status: "declared".into(),
                context_id: Some((*id).into()),
                context_json: Some(serde_json::Value::Object(context).to_string()),
                ..Default::default()
            });
            snapshot.facts.push(EvidenceFact {
                member: format!("sample:{id}"),
                kind: "criterion_sample".into(),
                status: "measured".into(),
                context_id: Some((*id).into()),
                population_id: Some("original".into()),
                sample_id: Some((*id).into()),
                value_ns: Some(1.0),
                ..Default::default()
            });
        }
        snapshot.expected_members = snapshot
            .facts
            .iter()
            .map(|fact| fact.member.clone())
            .collect();
        let mut analysis = EvidenceAnalysis::new(vec![snapshot], false).unwrap();
        let question = EvidenceQuestion {
            operation: EvidenceOperation::Measurements,
            limit: Some(1),
            sections: sections(&[EvidenceSection::Comparability], 1),
            ..Default::default()
        };
        let result = analysis.ask(&question, &[]).await.unwrap();
        assert_eq!(result["comparability"][0]["context_id"], "a-known");
        assert_eq!(result["comparability"][0]["comparable"], true);
        assert_eq!(result["sections"]["comparability"]["truncated"], true);
        assert_eq!(result["comparability_assessment"]["selected_contexts"], 2);
        assert_eq!(result["comparability_assessment"]["unknown_contexts"], 1);
        assert_eq!(
            result["comparability_assessment"]["cross_source_comparability"],
            "not_established"
        );
        let mut minimal = question;
        minimal.sections.clear();
        let without_detail = analysis.ask(&minimal, &[]).await.unwrap();
        assert!(without_detail.get("comparability").is_none());
        assert_eq!(
            without_detail["comparability_assessment"],
            result["comparability_assessment"]
        );
    }

    #[tokio::test]
    async fn selected_contexts_join_exact_generation_when_sources_reuse_a_context_spelling() {
        let sources = ["selected", "other"]
            .into_iter()
            .map(|id| {
                let mut snapshot = source(id);
                for fact in &mut snapshot.facts {
                    fact.crate_name = Some(id.into());
                    fact.context_id = Some("same-spelling".into());
                }
                let mut context = serde_json::Map::new();
                for field in [
                    "workload",
                    "input",
                    "revision",
                    "build",
                    "profile",
                    "features",
                    "provider",
                    "isa",
                    "capture",
                    "source",
                    "environment",
                    "host",
                    "mode",
                    "process",
                ] {
                    context.insert(field.into(), serde_json::Value::Null);
                }
                context.insert("capture".into(), serde_json::json!({"original":id}));
                snapshot.facts.push(EvidenceFact {
                    member: "context".into(),
                    kind: "context".into(),
                    status: "declared".into(),
                    context_id: Some("same-spelling".into()),
                    context_json: Some(serde_json::Value::Object(context).to_string()),
                    ..Default::default()
                });
                snapshot.expected_members = snapshot
                    .facts
                    .iter()
                    .map(|fact| fact.member.clone())
                    .collect();
                snapshot
            })
            .collect();
        let mut analysis = EvidenceAnalysis::new(sources, false).unwrap();
        let result = analysis
            .ask(
                &EvidenceQuestion {
                    crate_name: Some("selected".into()),
                    sections: sections(&[EvidenceSection::Contexts], 10),
                    ..Default::default()
                },
                &[],
            )
            .await
            .unwrap();
        assert_eq!(result["contexts"].as_array().unwrap().len(), 1);
        assert_eq!(
            result["contexts"][0]["generation"],
            result["rows"][0]["generation"]
        );
        assert!(
            result["contexts"][0]["context_json"]
                .as_str()
                .unwrap()
                .contains("selected")
        );
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
pub(crate) use comparison::tests::exercise_indexed_comparison;
