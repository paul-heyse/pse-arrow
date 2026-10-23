// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Short native selections over compact study metadata, separate from worker readiness.
use crate::{
    canonical::transport::{original_deadline, within_clock},
    canonical::{CanonicalError, CanonicalStore, REQUEST_TIMEOUT, protected_query},
    canonical_codec as codec,
};
use surrealdb::types::{Object, Value};

/// Indexed predicates over actual original scientific and engineering owners.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchFilter {
    /// Exact study key, including terminal studies.
    pub study: Option<String>,
    /// Exact owning project; branch membership is resolved by its native owner.
    pub project: Option<String>,
    /// Exact branch key; not its mutable display name.
    pub branch: Option<String>,
    /// Exact canonical modeling source revision.
    pub source: Option<String>,
    /// Exact admitted physical source revision.
    pub physical_revision: Option<String>,
    /// Exact immutable historical engineering context key.
    pub context: Option<String>,
    /// Original canonical terminal class, independent from scientific usability.
    pub outcome: Option<pse_model::generated::enums::CanonicalTerminalOutcome>,
    /// Dataset/selection keys select consumed ancestry; publication operation keys
    /// select the exact resulting parameter source. Existing ResearchReference keys
    /// select their explicitly named source/run neighborhood, without proving consumption.
    pub research_reference: Option<String>,
}
/// Compact immutable attribution and current header facts. No definitions or trajectories.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyWorkbenchHeader {
    /// Exact study identity.
    pub study: String,
    /// Exact parent execution.
    pub run: String,
    /// Original source owner.
    pub problem: String,
    /// Exact original modeling revision.
    pub source: String,
    /// Exact admitted physical revision, unavailable for historical untyped metadata.
    pub physical_revision: Option<String>,
    /// Exact physical source interpretation.
    pub physical_identity: Option<String>,
    /// Exact engineering context, if selected.
    pub context: Option<String>,
    /// Complete activated population size, before outcome filtering or analysis.
    pub point_count: u64,
    /// Actual cancellation authority generation; not a metadata revision.
    pub generation: u64,
    /// Original complete membership is activated.
    pub active: bool,
    /// Actual persistent cancellation flag.
    pub cancelled: bool,
    /// Parent summary concluded.
    pub terminal: bool,
    /// Exact terminal parent attempt, when admitted.
    pub result_attempt: Option<String>,
    /// Original operational outcome, not scientific qualification.
    pub outcome: Option<pse_model::generated::enums::CanonicalTerminalOutcome>,
}
/// One frozen point's compact original facts; no operation, report or payload is decoded.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyCohortMember {
    /// Exact occurrence key.
    pub key: String,
    /// Exact owning study.
    pub study: String,
    /// Actual immutable member position.
    pub ordinal: u64,
    /// Exact original run.
    pub run: String,
    /// Exact currently assigned attempt; retries remain separate sources.
    pub attempt: Option<String>,
    /// Actual native point metadata revision captured in this selection.
    pub metadata_revision: u64,
    /// Whether the policy has settled this occurrence.
    pub settled: bool,
    /// Original lifecycle, scientific availability, attempt count and effect facts.
    pub facts: pse_model::study::PointFacts,
}
/// One successful native snapshot, bounded before transport and decoding.
#[derive(Clone, Debug)]
pub struct NativeStudySelection {
    /// Compact matched headers, in stable key order.
    pub headers: Vec<StudyWorkbenchHeader>,
    /// Exact points when a complete cohort was requested.
    pub members: Vec<StudyCohortMember>,
    /// More headers exist after this live browsing page.
    pub has_more: bool,
}

type CompactStudyAttribution = (Option<String>, Option<String>, Option<String>);

/// Project immutable scientific metadata into the same original create transaction.
/// Legacy opaque metadata has no invented physical/context attribution.
pub(crate) fn index_metadata(bytes: &[u8]) -> Result<CompactStudyAttribution, CanonicalError> {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return Ok((None, None, None));
    };
    if value.get("version").and_then(serde_json::Value::as_u64) != Some(5) {
        return Ok((None, None, None));
    }
    let string = |v: Option<&serde_json::Value>| -> Result<Option<String>, CanonicalError> {
        v.filter(|v| !v.is_null())
            .map(|v| {
                v.as_str()
                    .filter(|s| !s.is_empty() && s.len() <= 4096)
                    .map(str::to_owned)
                    .ok_or_else(|| {
                        CanonicalError::Configuration("invalid compact study attribution".into())
                    })
            })
            .transpose()
    };
    // PhysicalSource transports an algorithm-qualified ContentHash. Canonical
    // context/source records compare its digest form, not its JSON encoding.
    let physical_identity = string(value.pointer("/physical/identity"))?
        .map(|encoded| {
            pse_ids::ContentHash::parse_prefixed(&encoded)
                .map(|identity| identity.to_hex())
                .map_err(|error| {
                    CanonicalError::Configuration(format!(
                        "invalid study physical source identity: {error}"
                    ))
                })
        })
        .transpose()?;
    Ok((
        string(value.pointer("/physical/revision"))?,
        physical_identity,
        string(value.pointer("/context/context/key"))?,
    ))
}
fn optional(row: &mut Object, name: &str) -> Result<Option<String>, CanonicalError> {
    match row.remove(name) {
        None | Some(Value::None) | Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(codec::decode_string(v)?)),
    }
}
fn header(mut row: Object) -> Result<StudyWorkbenchHeader, CanonicalError> {
    use codec::{decode_boolean as flag, decode_string as text, decode_uint as uint, required};
    let outcome = optional(&mut row, "outcome")?
        .map(|s| serde_json::from_value(serde_json::Value::String(s)))
        .transpose()
        .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
    Ok(StudyWorkbenchHeader {
        study: text(required(&mut row, "key")?)?,
        run: text(required(&mut row, "run")?)?,
        problem: text(required(&mut row, "problem")?)?,
        source: text(required(&mut row, "revision")?)?,
        physical_revision: optional(&mut row, "physical_revision")?,
        physical_identity: optional(&mut row, "physical_identity")?,
        context: optional(&mut row, "context")?,
        point_count: uint(required(&mut row, "point_count")?)?,
        generation: uint(required(&mut row, "generation")?)?,
        active: flag(required(&mut row, "active")?)?,
        cancelled: flag(required(&mut row, "cancelled")?)?,
        terminal: flag(required(&mut row, "terminal")?)?,
        result_attempt: optional(&mut row, "result_attempt")?,
        outcome,
    })
}
fn member(mut row: Object) -> Result<StudyCohortMember, CanonicalError> {
    use codec::{decode_boolean as flag, decode_string as text, decode_uint as uint, required};
    let facts = codec::decode_bytes(required(&mut row, "facts")?)?;
    if facts.len() > 16384 {
        return Err(CanonicalError::PayloadLimit);
    }
    let metadata_revision = uint(required(&mut row, "revision")?)?;
    let occurrence = uint(required(&mut row, "occurrence")?)?;
    let mut facts: pse_model::study::PointFacts = serde_json::from_slice(facts.as_slice())
        .map_err(|e| CanonicalError::Configuration(e.to_string()))?;
    if u64::from(facts.key.0) != occurrence {
        return Err(CanonicalError::Configuration(
            "cohort occurrence differs".into(),
        ));
    }
    facts.revision = metadata_revision;
    Ok(StudyCohortMember {
        key: text(required(&mut row, "key")?)?,
        study: text(required(&mut row, "study")?)?,
        ordinal: uint(required(&mut row, "ordinal")?)?,
        run: text(required(&mut row, "run")?)?,
        attempt: optional(&mut row, "attempt")?,
        metadata_revision,
        settled: flag(required(&mut row, "settled")?)?,
        facts,
    })
}
/// Bound original values separately from generated SQL. Every supplied predicate is native.
pub fn workbench_sql(filter: &StudyWorkbenchFilter, frozen: bool) -> String {
    let mut prelude = String::new();
    if filter.project.is_some() {
        prelude.push_str("LET $problems=SELECT VALUE problem FROM canonical_project_branches WHERE project=$project LIMIT 65; IF array::len($problems)>64 { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; }; ");
    }
    let mut predicates = vec!["key>$after".to_owned()];
    for (present, predicate) in [
        (filter.study.is_some(), "key=$study"),
        (filter.source.is_some(), "revision=$source"),
        (
            filter.physical_revision.is_some(),
            "physical_revision=$physical_revision",
        ),
        (filter.context.is_some(), "context=$context"),
        (
            filter.branch.is_some(),
            "problem=(SELECT problem FROM ONLY type::record('canonical_project_branches',$branch)).problem",
        ),
        (filter.project.is_some(), "problem IN $problems"),
        (
            filter.outcome.is_some(),
            "type::record('canonical_runs',run).terminal_class=$outcome",
        ),
        (
            filter.research_reference.is_some(),
            "(type::record('canonical_runs',run).research_selection=$research_reference OR type::record('canonical_runs',run).research_dataset=$research_reference OR run=type::record('canonical_research_references',$research_reference).run OR revision=type::record('canonical_research_references',$research_reference).revision OR (type::record('canonical_research_publications',$research_reference).selection!=NONE AND revision=type::record('canonical_research_publications',$research_reference).revision AND type::record('canonical_runs',run).research_selection=type::record('canonical_research_publications',$research_reference).selection))",
        ),
    ] {
        if present {
            predicates.push(predicate.into());
        }
    }
    let sql = format!(
        "LET $rows=SELECT key,run,problem,revision,physical_revision,physical_identity,context,point_count,generation,active,cancelled,terminal FROM canonical_studies WHERE {} ORDER BY key LIMIT $extent; RETURN fn::pse_workbench_v1::capture($pse_rpc_expires_at,$rows,$maximum,{},$members);",
        predicates.join(" AND "),
        if frozen { "true" } else { "false" }
    );
    format!("BEGIN; {prelude}{sql} COMMIT;")
}
impl CanonicalStore {
    /// Compact selected-run retry head; never fetch requests, source payloads or completion bytes.
    pub async fn workbench_run_generation(&self, run: &str) -> Result<Option<u64>, CanonicalError> {
        if run.is_empty() || run.len() > 4096 {
            return Err(CanonicalError::PayloadLimit);
        }
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT),async {
            let mut response=protected_query("canonical_study_workbench::run_generation",||Ok(self.db.query(crate::canonical_history::run_query("SELECT current_generation FROM ONLY type::record('canonical_runs',$run);","run")).bind(("run",run.to_owned())))).await?;
            response.take::<Option<Object>>(0)?.map(|mut row|codec::decode_uint(codec::required(&mut row,"current_generation")?).map_err(Into::into)).transpose()
        })).await
    }
    /// Select one live browsing page or one complete frozen population in one short SSI read.
    /// Frozen requests refuse header or point overflow; pages cannot certify completeness.
    pub async fn select_study_workbench(
        &self,
        filter: &StudyWorkbenchFilter,
        after: Option<&str>,
        maximum: usize,
        maximum_members: Option<usize>,
    ) -> Result<NativeStudySelection, CanonicalError> {
        if !(1..=64).contains(&maximum) || maximum_members.is_some_and(|n| n == 0 || n > 128) {
            return Err(CanonicalError::PayloadLimit);
        }
        for value in [
            &filter.study,
            &filter.project,
            &filter.branch,
            &filter.source,
            &filter.physical_revision,
            &filter.context,
            &filter.research_reference,
        ]
        .into_iter()
        .flatten()
        .map(String::as_str)
        .chain(after)
        {
            if value.is_empty() || value.len() > 4096 {
                return Err(CanonicalError::PayloadLimit);
            }
        }
        let sql = workbench_sql(filter, maximum_members.is_some());
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = protected_query("canonical_study_workbench::select", || {
                    Ok(self
                        .db
                        .query(sql.clone())
                        .bind(("study", filter.study.clone()))
                        .bind(("project", filter.project.clone()))
                        .bind(("branch", filter.branch.clone()))
                        .bind(("source", filter.source.clone()))
                        .bind(("physical_revision", filter.physical_revision.clone()))
                        .bind(("context", filter.context.clone()))
                        .bind(("research_reference", filter.research_reference.clone()))
                        .bind((
                            "outcome",
                            filter
                                .outcome
                                .map(serde_json::to_value)
                                .transpose()
                                .map_err(|e| CanonicalError::Configuration(e.to_string()))?
                                .and_then(|v| v.as_str().map(str::to_owned)),
                        ))
                        .bind(("after", after.unwrap_or("").to_owned()))
                        .bind(("maximum", maximum))
                        .bind(("extent", maximum + 1))
                        .bind(("members", maximum_members.unwrap_or(0))))
                })
                .await?;
                let index = response
                    .num_statements()
                    .checked_sub(2)
                    .ok_or(CanonicalError::IncompleteResponse)?;
                let mut row = match response.take::<Value>(index)? {
                    Value::Object(row) => row,
                    _ => return Err(CanonicalError::IncompleteResponse),
                };
                let has_more = codec::decode_boolean(codec::required(&mut row, "has_more")?)?;
                let rows = match codec::required(&mut row, "headers")? {
                    Value::Array(rows) => rows,
                    _ => return Err(CanonicalError::IncompleteResponse),
                };
                let headers = rows
                    .into_iter()
                    .map(|v| match v {
                        Value::Object(v) => header(v),
                        _ => Err(CanonicalError::IncompleteResponse),
                    })
                    .collect::<Result<_, _>>()?;
                let rows = match codec::required(&mut row, "members")? {
                    Value::Array(rows) => rows,
                    _ => return Err(CanonicalError::IncompleteResponse),
                };
                let members = rows
                    .into_iter()
                    .map(|v| match v {
                        Value::Object(v) => member(v),
                        _ => Err(CanonicalError::IncompleteResponse),
                    })
                    .collect::<Result<_, _>>()?;
                Ok(NativeStudySelection {
                    headers,
                    members,
                    has_more,
                })
            }))
            .await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workbench_query_has_real_predicates_and_never_interpolates_values() {
        let filter = StudyWorkbenchFilter {
            study: Some("';DELETE canonical_studies;".into()),
            source: Some("source".into()),
            physical_revision: Some("physical".into()),
            outcome: Some(pse_model::generated::enums::CanonicalTerminalOutcome::Succeeded),
            ..Default::default()
        };
        let sql = workbench_sql(&filter, true);
        assert!(!sql.contains("DELETE"));
        assert!(sql.contains("revision=$source"));
        assert!(sql.contains("physical_revision=$physical_revision"));
        assert!(sql.contains(".terminal_class=$outcome"));
        assert!(!sql.contains("eligible"));
    }
    #[test]
    fn index_attribution_is_exact_optional_and_not_invented_for_opaque_metadata() {
        assert_eq!(index_metadata(&[9]).unwrap(), (None, None, None));
        let identity = pse_ids::ContentHash::from_bytes([7; 32]);
        let metadata = serde_json::json!({
            "version": 5,
            "physical": {"revision": "p", "identity": identity},
            "context": {"context": {"key": "cut"}},
        });
        assert_eq!(metadata["physical"]["identity"], identity.to_prefixed());
        assert_eq!(
            index_metadata(&serde_json::to_vec(&metadata).unwrap()).unwrap(),
            (
                Some("p".into()),
                Some(identity.to_hex()),
                Some("cut".into())
            )
        );
        assert_eq!(
            index_metadata(br#"{"version":5,"physical":null,"context":null}"#).unwrap(),
            (None, None, None)
        );
        assert_eq!(
            index_metadata(br#"{"version":4,"physical":{"identity":"opaque historical value"}}"#)
                .unwrap(),
            (None, None, None)
        );
    }
    #[test]
    fn index_attribution_rejects_untyped_or_malformed_physical_checksums() {
        let identity = pse_ids::ContentHash::from_bytes([7; 32]);
        for invalid in [
            serde_json::json!(identity.to_hex()),
            serde_json::json!(format!("sha256:{}", identity.to_hex())),
            serde_json::json!("blake3:short"),
            serde_json::json!(17),
        ] {
            let metadata = serde_json::json!({"version": 5, "physical": {"revision": "p", "identity": invalid}});
            assert!(index_metadata(&serde_json::to_vec(&metadata).unwrap()).is_err());
        }
    }
}
