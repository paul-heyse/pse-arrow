// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native paired evidence semantics over one immutable, explicitly supplied source cut.
use super::*;
use crate::development::evidence::EvidenceFact;
use schemars::JsonSchema;
use sha2::{Digest, Sha256};

/// An explicit side of a comparison. Generation IDs identify exact source versions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonSelection {
    /// Exact generations; repeated IDs and unavailable IDs are refused.
    pub generations: Vec<String>,
    /// Owner-declared crate selection, before all denominators.
    pub crate_name: Option<String>,
    /// Owner-declared context selection, before all denominators.
    pub context_id: Option<String>,
}
/// An original source member, never a guessed spelling or timestamp.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMemberIdentity {
    /// Exact source version.
    pub generation: String,
    /// Original owner member.
    pub member: String,
}
/// Explicit correspondence supplied by the caller, including a renamed subject.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSubjectMapping {
    /// Original baseline member.
    pub baseline: EvidenceMemberIdentity,
    /// Original candidate member.
    pub candidate: EvidenceMemberIdentity,
}
/// Exact original measurement cohort, without invented invocation membership.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMeasurementIdentity {
    /// Exact source version.
    pub generation: String,
    /// Original context, including its explicit absence.
    pub context_id: Option<String>,
    /// Original declared population identity.
    pub population_id: String,
}
/// Explicit correspondence between two original measurement populations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMeasurementMapping {
    /// Exact baseline population.
    pub baseline: EvidenceMeasurementIdentity,
    /// Exact candidate population.
    pub candidate: EvidenceMeasurementIdentity,
}
/// Details selected before acquisition; no section is implicitly hydrated.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceComparisonSection {
    /// One row per paired or unpaired subject cohort.
    Subjects,
    /// Original subject outcomes and binary IDs, independently paged.
    Members,
    /// Original measurement sample membership and order, independently paged.
    Samples,
    /// Measurement cohort compatibility and original summary values.
    Measurements,
    /// Original selected context declarations.
    Contexts,
    /// Declared measurement association and censoring, without inferred membership.
    Associations,
}
/// Finite native request. Both sides and every mapping remain explicit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonRequest {
    /// Exact native request interpretation; required before decoding the body.
    pub version: pse_model::document::Version<1>,
    /// Baseline source population.
    pub baseline: EvidenceComparisonSelection,
    /// Candidate source population.
    pub candidate: EvidenceComparisonSelection,
    /// Exact caller-supplied mappings; conflicting or unknown endpoints are refused.
    #[serde(default)]
    pub mappings: Vec<EvidenceSubjectMapping>,
    /// Explicit measurement population mappings; unknown or conflicting endpoints are refused.
    #[serde(default)]
    pub measurement_mappings: Vec<EvidenceMeasurementMapping>,
    /// Cut returned by the preceding page; mandatory when any section offset is nonzero.
    #[serde(default)]
    pub continuation_cut: Option<String>,
    /// Optional independently bounded detail pages.
    #[serde(default)]
    pub sections: BTreeMap<EvidenceComparisonSection, EvidenceSectionSelection>,
}
impl EvidenceComparisonRequest {
    /// Admit the exact native request version before decoding the bounded body.
    pub fn decode_json(bytes: &[u8]) -> Result<Self, EvidenceError> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(invalid("comparison request exceeds 4 MiB"));
        }
        pse_model::document::decode_versioned::<Self, 1>(bytes).map_err(invalid)
    }
}
/// Coverage and freshness of one selected original source.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonSource {
    /// Exact source cut.
    pub generation: String,
    /// Original authority.
    pub authority: String,
    /// Original codec.
    pub codec: String,
    /// Original source identity.
    pub source_id: String,
    /// Original source digest.
    pub snapshot_digest: String,
    /// Original complete/partial coverage.
    pub coverage: String,
    /// Original declared selected membership, including non-test facts.
    pub expected_members: usize,
    /// Freshness against explicit owner observations only.
    pub freshness: String,
}
/// Full semantic population counts, computed before detail limits.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonDenominator {
    /// Selected original test rows, including repeats and missing execution.
    pub selected_tests: usize,
    /// Original test rows declaring a crate.
    pub declared_crate_tests: usize,
    /// Original test rows declaring a context.
    pub declared_context_tests: usize,
    /// Original failure rows.
    pub failures: usize,
    /// Original explicit success rows.
    pub successes: usize,
    /// Original missing, skipped, reused or transferred execution rows.
    pub unexecuted: usize,
    /// Other original statuses; these are never treated as passing.
    pub other_outcomes: usize,
    /// Selected actual measurement samples.
    pub samples: usize,
    /// Selected original expected members of all categories.
    pub selected_members: usize,
}
/// Outcome counts retain repeated mixed results without a flakiness verdict.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceOutcomeCounts {
    /// Original failures.
    pub failures: usize,
    /// Explicit successes.
    pub successes: usize,
    /// Missing execution, skipped, reused or transferred.
    pub unexecuted: usize,
    /// All other original statuses.
    pub other: usize,
}
impl EvidenceOutcomeCounts {
    fn total(&self) -> usize {
        self.failures + self.successes + self.unexecuted + self.other
    }
    fn mixed(&self) -> bool {
        self.failures > 0 && self.successes > 0
    }
    fn all_success(&self) -> bool {
        self.successes > 0 && self.successes == self.total()
    }
}
/// Owner identity of one cohort. Original binary IDs are never normalized away.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSubjectIdentity {
    /// Compatible owner authority.
    pub authority: String,
    /// Exact codec; codec migrations need an explicit mapping.
    pub codec: String,
    /// Rust or Python test category.
    pub kind: String,
    /// Declared owning crate.
    pub crate_name: Option<String>,
    /// Original binary ID, required for automatic Rust pairing.
    pub binary_id: Option<String>,
    /// Original owner test identity.
    pub test_id: Option<String>,
}
/// A paired cohort or an honestly unpaired original population.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSubjectComparison {
    /// Deterministic reference within this exact comparison cut.
    pub cohort: String,
    /// Exact-compatible or explicitly supplied correspondence; absent for unpaired rows.
    pub correspondence: Option<String>,
    /// Classification never rewrites either original outcome.
    pub classification: String,
    /// Original baseline identity.
    pub baseline: Option<EvidenceSubjectIdentity>,
    /// Original candidate identity.
    pub candidate: Option<EvidenceSubjectIdentity>,
    /// Full baseline outcome counts.
    pub baseline_outcomes: EvidenceOutcomeCounts,
    /// Full candidate outcome counts.
    pub candidate_outcomes: EvidenceOutcomeCounts,
    /// Repeated success and failure occurred in at least one side; no statistical inference.
    pub mixed_outcomes: bool,
}
/// Original membership retained separately from compact paired cohorts.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonMember {
    /// Baseline or candidate.
    pub side: String,
    /// Exact cohort reference in subjects.
    pub cohort: String,
    /// Original generation and member.
    pub original: EvidenceMemberIdentity,
    /// Original owner identity.
    pub identity: EvidenceSubjectIdentity,
    /// Original binary outcome, including unknowns.
    pub status: String,
    /// Original declared context.
    pub context_id: Option<String>,
}
/// Full selected populations. Missing execution and mixed outcomes are independent
/// axes and may overlap failure classifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonSummary {
    /// Subjects paired by exact identity or supplied mapping.
    pub paired: usize,
    /// Candidate failures following an all-success baseline.
    pub new_failures: usize,
    /// Baseline failures followed by an all-success candidate.
    pub resolved_failures: usize,
    /// Failure evidence in both sides, retaining mixed outcomes.
    pub persistent_failures: usize,
    /// Pairs lacking execution in either side.
    pub unexecuted: usize,
    /// Unknown or mixed change evidence without an eligible change verdict.
    pub inconclusive: usize,
    /// Compatible pairs with only explicit successes.
    pub unchanged_successes: usize,
    /// Baseline-only cohorts.
    pub unmatched_baseline: usize,
    /// Candidate-only cohorts.
    pub unmatched_candidate: usize,
    /// Duplicate owner identities within a source or unavailable exact identity.
    pub ambiguous: usize,
    /// Cohorts with original failure and success evidence.
    pub mixed_outcome_cohorts: usize,
    /// Full selected test-row counts differ.
    pub changed_selection_denominator: bool,
    /// Unmatched or ambiguous cohorts prevent a same-subject selection claim.
    pub selection_membership_changed_or_unresolved: bool,
}
/// A bounded section with a full row denominator and explicit continuation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonPage<T> {
    /// Immutable comparison cut; reuse only with the same request selection.
    pub cut: String,
    /// Full section rows before display limits.
    pub total_rows: usize,
    /// Requested offset.
    pub offset: usize,
    /// Requested page limit.
    pub limit: usize,
    /// Whole section is present in this response.
    pub complete: bool,
    /// Next offset in this exact cut, if any.
    pub next_offset: Option<usize>,
    /// Bounded original rows.
    pub rows: Vec<T>,
}
/// Original codec fact with its exact source and comparison side.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparisonFact {
    /// Baseline or candidate selected population.
    pub side: String,
    /// Original immutable generation.
    pub generation: String,
    /// Original native fact, including member, sample identity and context declaration.
    pub fact: EvidenceFact,
}
/// Descriptive statistics and membership counts of one original measurement cohort.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMeasurementCohort {
    /// Original immutable generation.
    pub generation: String,
    /// Original declared context identity, if available.
    pub context_id: Option<String>,
    /// Original owner population identity.
    pub population_id: String,
    /// Actual original sample rows.
    pub sample_count: u64,
    /// Minimum actual sample value, if any.
    pub min_ns: Option<f64>,
    /// Maximum actual sample value, if any.
    pub max_ns: Option<f64>,
    /// Mean actual sample value, if any; no change verdict follows from it.
    pub mean_ns: Option<f64>,
    /// Original censored or unmeasured population declarations.
    pub censored_populations: u64,
    /// Original association declarations, regardless of their status.
    pub declared_associations: u64,
    /// Original explicitly unavailable, unsupported or unknown association declarations.
    pub unavailable_associations: u64,
}
/// Paired or honestly unpaired original measurement cohorts.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMeasurementComparison {
    /// Exact compatible owner identity or explicitly supplied mapping.
    pub correspondence: String,
    /// Baseline original cohorts before display bounds.
    pub baseline_cohorts: usize,
    /// Candidate original cohorts before display bounds.
    pub candidate_cohorts: usize,
    /// Baseline descriptor only when exactly one original cohort exists.
    pub baseline: Option<EvidenceMeasurementCohort>,
    /// Candidate descriptor only when exactly one original cohort exists.
    pub candidate: Option<EvidenceMeasurementCohort>,
    /// Comparable conditions, incompatible conditions, or unavailable basis.
    pub conditions: String,
    /// Relevant declared conditions that differ.
    pub changed_fields: Vec<String>,
    /// Relevant conditions not declared by at least one original owner.
    pub unknown_fields: Vec<String>,
    /// Explicit scope of supported original association.
    pub association: String,
    /// Descriptive-only verdict; statistical and causal conclusions are not inferred.
    pub verdict: String,
    /// Revision/build interpretation retained with the assessment.
    pub revision_build: String,
}
/// Typed comparison result shared by native, offline and canonical consumers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceComparison {
    /// Native comparison codec version.
    pub version: pse_model::document::Version<1>,
    /// Exact selected cut and mappings, independent of page bounds.
    pub cut: String,
    /// Original selected baseline sources and coverage.
    pub baseline_sources: Vec<EvidenceComparisonSource>,
    /// Original selected candidate sources and coverage.
    pub candidate_sources: Vec<EvidenceComparisonSource>,
    /// Full baseline denominator, independent of optional details.
    pub baseline_denominator: EvidenceComparisonDenominator,
    /// Full candidate denominator, independent of optional details.
    pub candidate_denominator: EvidenceComparisonDenominator,
    /// Full semantic subject comparison summary.
    pub summary: EvidenceComparisonSummary,
    /// Full measurement context comparison, without sample hydration.
    pub measurement_summary: EvidenceMeasurementSummary,
    /// Scope and non-inference limits.
    pub interpretation: String,
    /// Requested subject detail only.
    pub subjects: Option<EvidenceComparisonPage<EvidenceSubjectComparison>>,
    /// Requested original outcome rows only.
    pub members: Option<EvidenceComparisonPage<EvidenceComparisonMember>>,
    /// Requested original measurement sample rows only.
    pub samples: Option<EvidenceComparisonPage<EvidenceComparisonFact>>,
    /// Requested measurement cohorts only.
    pub measurements: Option<EvidenceComparisonPage<EvidenceMeasurementComparison>>,
    /// Requested original context rows only.
    pub contexts: Option<EvidenceComparisonPage<EvidenceComparisonFact>>,
    /// Requested original measurement association and population declarations only.
    pub associations: Option<EvidenceComparisonPage<EvidenceComparisonFact>>,
}
/// Cross-side measurement compatibility counts, retaining inconclusive cohorts.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMeasurementSummary {
    /// Exact owner/population cohorts examined across both sides.
    pub populations: usize,
    /// Pairs with declared and equal relevant conditions; sample association remains separate.
    pub comparable: usize,
    /// Pairs with at least one different declared condition.
    pub incompatible: usize,
    /// Missing/unknown conditions, absent samples, ambiguity or unpaired populations.
    pub unavailable: usize,
    /// Cohorts lacking any declared association or reporting unavailable/unsupported association.
    /// This axis is independent of condition compatibility.
    pub unavailable_association_populations: usize,
    /// No automatic regression, causality or flakiness verdict is produced.
    pub verdict: String,
}

#[derive(Clone)]
struct Subject {
    original: EvidenceMemberIdentity,
    identity: EvidenceSubjectIdentity,
    status: String,
    context_id: Option<String>,
}
#[derive(Default)]
struct Cohort {
    rows: Vec<Subject>,
}
impl Cohort {
    fn counts(&self) -> EvidenceOutcomeCounts {
        let mut counts = EvidenceOutcomeCounts::default();
        for row in &self.rows {
            match outcome(&row.status) {
                0 => counts.failures += 1,
                1 => counts.successes += 1,
                2 => counts.unexecuted += 1,
                _ => counts.other += 1,
            }
        }
        counts
    }
    fn ambiguous(&self) -> bool {
        let mut sources = BTreeSet::new();
        self.rows
            .iter()
            .any(|r| !sources.insert(&r.original.generation))
    }
}
fn outcome(status: &str) -> u8 {
    match status {
        "failure" | "failed" | "error" => 0,
        "success" | "passed" | "pass" => 1,
        "unreported" | "not_run" | "incomplete" | "skipped" | "reused" | "transferred" => 2,
        _ => 3,
    }
}
fn text(row: &serde_json::Value, name: &str) -> Result<String, EvidenceError> {
    row[name]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| invalid("comparison scalar identity unavailable"))
}
fn optional(row: &serde_json::Value, name: &str) -> Option<String> {
    row[name].as_str().map(str::to_owned)
}
fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
fn page<T: Clone>(
    cut: &str,
    rows: &[T],
    selection: EvidenceSectionSelection,
) -> EvidenceComparisonPage<T> {
    let end = selection
        .offset
        .saturating_add(selection.limit)
        .min(rows.len());
    EvidenceComparisonPage {
        cut: cut.into(),
        total_rows: rows.len(),
        offset: selection.offset,
        limit: selection.limit,
        complete: selection.offset == 0 && end == rows.len(),
        next_offset: (end < rows.len()).then_some(end),
        rows: rows.get(selection.offset..end).unwrap_or_default().to_vec(),
    }
}
impl EvidenceAnalysis {
    async fn comparison_selection(
        &self,
        name: &str,
        selection: &EvidenceComparisonSelection,
    ) -> Result<(), EvidenceError> {
        if selection.generations.is_empty() || selection.generations.len() > 64 {
            return Err(invalid(
                "comparison requires 1..=64 exact generations per side",
            ));
        }
        let mut seen = BTreeSet::new();
        for id in &selection.generations {
            if !self.generations.contains(id) || !seen.insert(id) {
                return Err(invalid("comparison generation unavailable or repeated"));
            }
        }
        if [&selection.crate_name, &selection.context_id]
            .into_iter()
            .flatten()
            .any(|value| value.len() > 4096)
        {
            return Err(invalid("comparison selection identity exceeds bound"));
        }
        let mut selected = self.context.table("evidence").await.map_err(invalid)?;
        selected = selected
            .filter(
                col("generation").in_list(
                    selection
                        .generations
                        .iter()
                        .map(|id| lit(id.clone()))
                        .collect(),
                    false,
                ),
            )
            .map_err(invalid)?;
        if let Some(value) = &selection.crate_name {
            selected = selected
                .filter(col("crate_name").eq(lit(value.clone())))
                .map_err(invalid)?;
        }
        if let Some(value) = &selection.context_id {
            selected = selected
                .filter(col("context_id").eq(lit(value.clone())))
                .map_err(invalid)?;
        }
        self.context.deregister_table(name).map_err(invalid)?;
        self.context
            .register_table(name, selected.into_view())
            .map_err(invalid)?;
        Ok(())
    }
    async fn comparison_subjects(&self, name: &str) -> Result<Vec<Subject>, EvidenceError> {
        let rows = json_rows(&self.sql(&format!("SELECT generation,member,kind,test_id,binary_id,crate_name,status,context_id FROM {name} WHERE kind IN ('rust_test','python_test') ORDER BY generation,member")).await?)?;
        rows.into_iter()
            .map(|row| {
                let generation = text(&row, "generation")?;
                let index = self
                    .generations
                    .iter()
                    .position(|id| *id == generation)
                    .ok_or_else(|| invalid("comparison source unavailable"))?;
                let snapshot = &self.snapshots[index];
                Ok(Subject {
                    original: EvidenceMemberIdentity {
                        generation,
                        member: text(&row, "member")?,
                    },
                    identity: EvidenceSubjectIdentity {
                        authority: snapshot.authority.clone(),
                        codec: snapshot.codec.clone(),
                        kind: text(&row, "kind")?,
                        crate_name: optional(&row, "crate_name"),
                        binary_id: optional(&row, "binary_id"),
                        test_id: optional(&row, "test_id"),
                    },
                    status: text(&row, "status")?,
                    context_id: optional(&row, "context_id"),
                })
            })
            .collect()
    }
    async fn comparison_denominator(
        &self,
        name: &str,
        rows: &[Subject],
    ) -> Result<EvidenceComparisonDenominator, EvidenceError> {
        let selected = json_rows(&self.sql(&format!("SELECT count(*) AS members,sum(CASE WHEN kind='criterion_sample' THEN 1 ELSE 0 END) AS samples FROM {name}")).await?)?;
        let scalar = selected
            .first()
            .ok_or_else(|| invalid("comparison denominator unavailable"))?;
        let mut denominator = EvidenceComparisonDenominator {
            selected_tests: rows.len(),
            selected_members: scalar["members"].as_u64().unwrap_or(0) as usize,
            samples: scalar["samples"].as_u64().unwrap_or(0) as usize,
            ..Default::default()
        };
        for row in rows {
            denominator.declared_crate_tests += usize::from(row.identity.crate_name.is_some());
            denominator.declared_context_tests += usize::from(row.context_id.is_some());
            match outcome(&row.status) {
                0 => denominator.failures += 1,
                1 => denominator.successes += 1,
                2 => denominator.unexecuted += 1,
                _ => denominator.other_outcomes += 1,
            }
        }
        Ok(denominator)
    }
    /// Compare full selected populations before applying independently requested detail pages.
    /// The original source bytes, outcomes and IDs remain unchanged.
    pub async fn compare(
        &mut self,
        request: &EvidenceComparisonRequest,
        observations: &[EvidenceSourceObservation],
    ) -> Result<EvidenceComparison, EvidenceError> {
        for selection in request.sections.values() {
            selection.validate()?;
        }
        if request.mappings.len() > 10000 || request.measurement_mappings.len() > 10000 {
            return Err(invalid("comparison mapping exceeds 10000 exact pairs"));
        }
        validate_observations(observations)?;
        self.comparison_selection("comparison_baseline", &request.baseline)
            .await?;
        self.comparison_selection("comparison_candidate", &request.candidate)
            .await?;
        let baseline = self.comparison_subjects("comparison_baseline").await?;
        let candidate = self.comparison_subjects("comparison_candidate").await?;
        let baseline_denominator = self
            .comparison_denominator("comparison_baseline", &baseline)
            .await?;
        let candidate_denominator = self
            .comparison_denominator("comparison_candidate", &candidate)
            .await?;
        let selected_generations: BTreeSet<_> = request
            .baseline
            .generations
            .iter()
            .chain(&request.candidate.generations)
            .collect();
        let content: BTreeMap<_, _> = self
            .contents
            .iter()
            .filter(|(id, _)| selected_generations.contains(id))
            .collect();
        let cut = Sha256::digest(
            serde_json::to_vec(&(
                &request.baseline,
                &request.candidate,
                &request.mappings,
                &request.measurement_mappings,
                content,
            ))
            .map_err(invalid)?,
        )
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
        if request
            .continuation_cut
            .as_ref()
            .is_some_and(|expected| expected != &cut)
            || (request
                .sections
                .values()
                .any(|selection| selection.offset > 0)
                && request.continuation_cut.is_none())
        {
            return Err(invalid(
                "comparison continuation does not belong to this exact source cut and selection",
            ));
        }
        let (subjects, members, mut summary) =
            pair_subjects(baseline, candidate, &request.mappings)?;
        summary.changed_selection_denominator =
            baseline_denominator.selected_tests != candidate_denominator.selected_tests;
        summary.selection_membership_changed_or_unresolved =
            summary.unmatched_baseline + summary.unmatched_candidate + summary.ambiguous > 0;
        let (measurement_summary, measurement_rows) = self
            .comparison_measurements(&request.measurement_mappings)
            .await?;
        let all_sources: Vec<EvidenceComparisonSource> =
            serde_json::from_value(self.sources(observations)).map_err(invalid)?;
        let selected_sources = |selection: &EvidenceComparisonSelection| {
            let mut sources: Vec<_> = all_sources
                .iter()
                .filter(|source| selection.generations.contains(&source.generation))
                .cloned()
                .collect();
            sources.sort_by(|left, right| left.generation.cmp(&right.generation));
            sources
        };
        let mut result = EvidenceComparison { version:pse_model::document::Version, cut:cut.clone(), baseline_sources:selected_sources(&request.baseline),candidate_sources:selected_sources(&request.candidate),baseline_denominator,candidate_denominator,summary,measurement_summary,
            interpretation:"Explicit source cut only; partial coverage and stale/missing sources remain partial. Exact compatible owner identity or supplied mapping only. Mixed outcomes are retained; no causal, regression or flakiness verdict. Original binary IDs and unsupported measurement association remain unchanged.".into(), subjects:None,members:None,samples:None,measurements:None,contexts:None,associations:None };
        for (section, selection) in &request.sections {
            match section {
                EvidenceComparisonSection::Subjects => {
                    result.subjects = Some(page(&cut, &subjects, *selection))
                }
                EvidenceComparisonSection::Members => {
                    result.members = Some(page(&cut, &members, *selection))
                }
                EvidenceComparisonSection::Measurements => {
                    result.measurements = Some(page(&cut, &measurement_rows, *selection))
                }
                section => {
                    let predicate = match section {
                        EvidenceComparisonSection::Samples => "kind='criterion_sample'",
                        EvidenceComparisonSection::Contexts => "kind='context'",
                        _ => {
                            "kind IN ('measurement_population','measurement_association','measurement_summary','capture_cost')"
                        }
                    };
                    let detail = Some(
                        self.comparison_detail(
                            &cut,
                            predicate,
                            *section == EvidenceComparisonSection::Contexts,
                            *selection,
                        )
                        .await?,
                    );
                    match section {
                        EvidenceComparisonSection::Samples => result.samples = detail,
                        EvidenceComparisonSection::Contexts => result.contexts = detail,
                        _ => result.associations = detail,
                    }
                }
            }
        }
        if serde_json::to_vec(&result).map_err(invalid)?.len() > 4 * 1024 * 1024 {
            return Err(invalid(
                "comparison response exceeds 4 MiB; select smaller independent section pages",
            ));
        }
        Ok(result)
    }
    async fn comparison_detail(
        &self,
        cut: &str,
        predicate: &str,
        contexts: bool,
        selection: EvidenceSectionSelection,
    ) -> Result<EvidenceComparisonPage<EvidenceComparisonFact>, EvidenceError> {
        let side = |table: &str, label: &str| {
            if contexts {
                format!(
                    "SELECT {} AS side,e.* FROM evidence e JOIN (SELECT DISTINCT generation,context_id FROM {table} WHERE context_id IS NOT NULL) s ON e.generation=s.generation AND e.context_id=s.context_id WHERE e.kind='context'",
                    quote(label)
                )
            } else {
                format!(
                    "SELECT {} AS side,* FROM {table} WHERE {predicate}",
                    quote(label)
                )
            }
        };
        let query = format!(
            "{} UNION ALL {}",
            side("comparison_baseline", "baseline"),
            side("comparison_candidate", "candidate")
        );
        let counts = json_rows(
            &self
                .sql(&format!("SELECT count(*) AS total FROM ({query}) detail"))
                .await?,
        )?;
        let total_rows = counts
            .first()
            .and_then(|row| row["total"].as_u64())
            .ok_or_else(|| invalid("comparison section denominator unavailable"))?
            as usize;
        let rows = json_rows(
            &self
                .sql(&format!(
                    "{query} ORDER BY side,generation,member LIMIT {} OFFSET {}",
                    selection.limit, selection.offset
                ))
                .await?,
        )?;
        let rows = rows
            .into_iter()
            .map(|mut row| {
                let side = text(&row, "side")?;
                let generation = text(&row, "generation")?;
                let object = row
                    .as_object_mut()
                    .ok_or_else(|| invalid("original comparison fact layout unavailable"))?;
                object.remove("side");
                object.remove("generation");
                object.remove("key");
                let fact = serde_json::from_value(row).map_err(invalid)?;
                Ok(EvidenceComparisonFact {
                    side,
                    generation,
                    fact,
                })
            })
            .collect::<Result<Vec<_>, EvidenceError>>()?;
        let end = selection.offset.saturating_add(rows.len());
        Ok(EvidenceComparisonPage {
            cut: cut.into(),
            total_rows,
            offset: selection.offset,
            limit: selection.limit,
            complete: selection.offset == 0 && end == total_rows,
            next_offset: (end < total_rows).then_some(end),
            rows,
        })
    }
    async fn comparison_measurements(
        &self,
        mappings: &[EvidenceMeasurementMapping],
    ) -> Result<
        (
            EvidenceMeasurementSummary,
            Vec<EvidenceMeasurementComparison>,
        ),
        EvidenceError,
    > {
        let mut baseline_rows = BTreeMap::new();
        let mut candidate_rows = BTreeMap::new();
        let mut groups: BTreeMap<(String, String, String), MeasurementGroup> = BTreeMap::new();
        for (table, baseline) in [
            ("comparison_baseline", true),
            ("comparison_candidate", false),
        ] {
            let rows=json_rows(&self.sql(&format!("SELECT generation,context_id,population_id,sum(CASE WHEN kind='criterion_sample' THEN 1 ELSE 0 END) AS sample_count,min(CASE WHEN kind='criterion_sample' THEN value_ns END) AS min_ns,max(CASE WHEN kind='criterion_sample' THEN value_ns END) AS max_ns,avg(CASE WHEN kind='criterion_sample' THEN value_ns END) AS mean_ns,sum(CASE WHEN kind='measurement_population' AND status IN ('censored','not_measured','incomplete') THEN 1 ELSE 0 END) AS censored_populations,sum(CASE WHEN kind='measurement_association' THEN 1 ELSE 0 END) AS declared_associations,sum(CASE WHEN kind='measurement_association' AND status IN ('unavailable','unsupported','unknown') THEN 1 ELSE 0 END) AS unavailable_associations FROM {table} WHERE kind IN ('criterion_sample','measurement_population','measurement_association','measurement_summary') AND population_id IS NOT NULL GROUP BY generation,context_id,population_id ORDER BY generation,context_id,population_id")).await?)?;
            for row in rows {
                let identity = EvidenceMeasurementIdentity {
                    generation: text(&row, "generation")?,
                    context_id: optional(&row, "context_id"),
                    population_id: text(&row, "population_id")?,
                };
                if baseline {
                    baseline_rows.insert(identity, row);
                } else {
                    candidate_rows.insert(identity, row);
                }
            }
        }
        let mut pairs = Vec::new();
        for mapping in mappings {
            let left = baseline_rows.remove(&mapping.baseline).ok_or_else(|| {
                invalid(
                    "baseline measurement mapping is conflicting or outside selected population",
                )
            })?;
            let right = candidate_rows.remove(&mapping.candidate).ok_or_else(|| {
                invalid(
                    "candidate measurement mapping is conflicting or outside selected population",
                )
            })?;
            pairs.push(("supplied_mapping".to_owned(), vec![left], vec![right]));
        }
        for (source_rows, baseline) in [(baseline_rows, true), (candidate_rows, false)] {
            for (identity, row) in source_rows {
                let index = self
                    .generations
                    .iter()
                    .position(|id| *id == identity.generation)
                    .ok_or_else(|| invalid("measurement source unavailable"))?;
                let key = (
                    self.snapshots[index].authority.clone(),
                    self.snapshots[index].codec.clone(),
                    identity.population_id,
                );
                let group = groups.entry(key).or_default();
                if baseline {
                    group.baseline.push(row)
                } else {
                    group.candidate.push(row)
                }
            }
        }
        for (_, group) in groups {
            pairs.push((
                "exact_compatible_owner_identity".to_owned(),
                group.baseline,
                group.candidate,
            ));
        }
        let mut summary=EvidenceMeasurementSummary { populations:pairs.len(), verdict:"inconclusive: descriptive supplied populations only; no automatic causal, regression or flakiness inference".into(), ..Default::default() };
        let mut rows = Vec::new();
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
        for (correspondence, baseline, candidate) in pairs {
            let mut changed = Vec::new();
            let mut unknown = Vec::new();
            if baseline.is_empty()
                || candidate.is_empty()
                || baseline.iter().chain(&candidate).any(|row| {
                    row["declared_associations"].as_u64().unwrap_or(0) == 0
                        || row["unavailable_associations"].as_u64().unwrap_or(0) > 0
                })
            {
                summary.unavailable_association_populations += 1;
            }
            let context_for = |row: &serde_json::Value| {
                row["context_id"].as_str().and_then(|id| {
                    self.contexts
                        .get(&(row["generation"].as_str()?.into(), id.into()))
                })
            };
            let state = if baseline.len() != 1
                || candidate.len() != 1
                || baseline[0]["sample_count"].as_u64().unwrap_or(0) == 0
                || candidate[0]["sample_count"].as_u64().unwrap_or(0) == 0
            {
                "unavailable"
            } else if let (Some(left), Some(right)) =
                (context_for(&baseline[0]), context_for(&candidate[0]))
            {
                for field in fields {
                    if left[field].is_null() || right[field].is_null() {
                        unknown.push(field)
                    } else if left[field] != right[field] {
                        changed.push(field)
                    }
                }
                if !changed.is_empty() {
                    "incompatible"
                } else if !unknown.is_empty() {
                    "unavailable"
                } else {
                    "comparable_conditions"
                }
            } else {
                "unavailable"
            };
            match state {
                "incompatible" => summary.incompatible += 1,
                "comparable_conditions" => summary.comparable += 1,
                _ => summary.unavailable += 1,
            }
            let cohort = |original: &[serde_json::Value]| -> Result<Option<EvidenceMeasurementCohort>,EvidenceError> {
                if original.len()==1 {serde_json::from_value(original[0].clone()).map(Some).map_err(invalid)} else {Ok(None)}
            };
            rows.push(EvidenceMeasurementComparison {correspondence,baseline_cohorts:baseline.len(),candidate_cohorts:candidate.len(),baseline:cohort(&baseline)?,candidate:cohort(&candidate)?,conditions:state.into(),changed_fields:changed.into_iter().map(str::to_owned).collect(),unknown_fields:unknown.into_iter().map(str::to_owned).collect(),association:"original declared association only; criterion-to-invocation membership is not inferred".into(),verdict:"inconclusive".into(),revision_build:"included among compatibility conditions; differences do not establish causality".into()});
        }
        Ok((summary, rows))
    }
}
#[derive(Default)]
struct MeasurementGroup {
    baseline: Vec<serde_json::Value>,
    candidate: Vec<serde_json::Value>,
}

fn validate_observations(observations: &[EvidenceSourceObservation]) -> Result<(), EvidenceError> {
    if observations.len() > 64 {
        return Err(invalid("comparison freshness observation bound exceeded"));
    }
    let mut seen = BTreeSet::new();
    for value in observations {
        if !seen.insert((&value.authority, &value.codec, &value.source_id))
            || [&value.authority, &value.codec, &value.source_id]
                .iter()
                .any(|value| value.len() > 4096)
            || value.snapshot_digest.as_ref().is_some_and(|value| {
                value.len() != 64
                    || !value
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err(invalid(
                "comparison freshness observation is ambiguous or invalid",
            ));
        }
    }
    Ok(())
}
fn pair_subjects(
    baseline: Vec<Subject>,
    candidate: Vec<Subject>,
    mappings: &[EvidenceSubjectMapping],
) -> Result<
    (
        Vec<EvidenceSubjectComparison>,
        Vec<EvidenceComparisonMember>,
        EvidenceComparisonSummary,
    ),
    EvidenceError,
> {
    let mut baseline: BTreeMap<_, _> = baseline
        .into_iter()
        .map(|row| (row.original.clone(), row))
        .collect();
    let mut candidate: BTreeMap<_, _> = candidate
        .into_iter()
        .map(|row| (row.original.clone(), row))
        .collect();
    let mut subjects = Vec::new();
    let mut members = Vec::new();
    let mut summary = EvidenceComparisonSummary::default();
    for mapping in mappings {
        let left = baseline.remove(&mapping.baseline).ok_or_else(|| {
            invalid("supplied baseline mapping is conflicting or outside selected test population")
        })?;
        let right = candidate.remove(&mapping.candidate).ok_or_else(|| {
            invalid("supplied candidate mapping is conflicting or outside selected test population")
        })?;
        if left.identity.kind != right.identity.kind {
            return Err(invalid(
                "supplied mapping crosses incompatible subject categories",
            ));
        }
        append_pair(
            Cohort { rows: vec![left] },
            Cohort { rows: vec![right] },
            Some("supplied_mapping"),
            false,
            &mut subjects,
            &mut members,
            &mut summary,
        );
    }
    let mut cohorts: BTreeMap<EvidenceSubjectIdentity, (Cohort, Cohort)> = BTreeMap::new();
    for (rows, left) in [(baseline, true), (candidate, false)] {
        for (_, row) in rows {
            let group = cohorts.entry(row.identity.clone()).or_default();
            if left {
                group.0.rows.push(row)
            } else {
                group.1.rows.push(row)
            }
        }
    }
    for (identity, (left, right)) in cohorts {
        let ambiguous = identity.test_id.is_none()
            || (identity.kind == "rust_test" && identity.binary_id.is_none())
            || left.ambiguous()
            || right.ambiguous();
        let correspondence = (!ambiguous && !left.rows.is_empty() && !right.rows.is_empty())
            .then_some("exact_compatible_owner_identity");
        append_pair(
            left,
            right,
            correspondence,
            ambiguous,
            &mut subjects,
            &mut members,
            &mut summary,
        );
    }
    Ok((subjects, members, summary))
}
#[allow(
    clippy::too_many_arguments,
    reason = "single native cohort emission keeps mandatory counts and original membership consistent"
)]
fn append_pair(
    left: Cohort,
    right: Cohort,
    correspondence: Option<&str>,
    ambiguous: bool,
    subjects: &mut Vec<EvidenceSubjectComparison>,
    members: &mut Vec<EvidenceComparisonMember>,
    summary: &mut EvidenceComparisonSummary,
) {
    let baseline_outcomes = left.counts();
    let candidate_outcomes = right.counts();
    let classification = if ambiguous {
        summary.ambiguous += 1;
        "ambiguous"
    } else if left.rows.is_empty() {
        summary.unmatched_candidate += 1;
        "unmatched_candidate"
    } else if right.rows.is_empty() {
        summary.unmatched_baseline += 1;
        "unmatched_baseline"
    } else {
        summary.paired += 1;
        let unexecuted = baseline_outcomes.unexecuted > 0 || candidate_outcomes.unexecuted > 0;
        summary.unexecuted += usize::from(unexecuted);
        if baseline_outcomes.failures > 0 && candidate_outcomes.failures > 0 {
            summary.persistent_failures += 1;
            "persistent_failure"
        } else if baseline_outcomes.all_success() && candidate_outcomes.failures > 0 {
            summary.new_failures += 1;
            "new_failure"
        } else if baseline_outcomes.failures > 0 && candidate_outcomes.all_success() {
            summary.resolved_failures += 1;
            "resolved_failure"
        } else if unexecuted {
            "unexecuted"
        } else if baseline_outcomes.all_success() && candidate_outcomes.all_success() {
            summary.unchanged_successes += 1;
            "unchanged_success"
        } else {
            summary.inconclusive += 1;
            "inconclusive"
        }
    };
    let mixed_outcomes = baseline_outcomes.mixed() || candidate_outcomes.mixed();
    summary.mixed_outcome_cohorts += usize::from(mixed_outcomes);
    let cohort = format!("subject:{}", subjects.len());
    subjects.push(EvidenceSubjectComparison {
        cohort: cohort.clone(),
        correspondence: correspondence.map(str::to_owned),
        classification: classification.into(),
        baseline: left.rows.first().map(|row| row.identity.clone()),
        candidate: right.rows.first().map(|row| row.identity.clone()),
        baseline_outcomes,
        candidate_outcomes,
        mixed_outcomes,
    });
    for (rows, side) in [(left.rows, "baseline"), (right.rows, "candidate")] {
        for row in rows {
            members.push(EvidenceComparisonMember {
                side: side.into(),
                cohort: cohort.clone(),
                original: row.original,
                identity: row.identity,
                status: row.status,
                context_id: row.context_id,
            });
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
