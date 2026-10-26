// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::development::evidence::EvidenceFact;
#[cfg(feature = "canonical-tests")]
use crate::development::evidence::EvidenceStore;

fn source(id: &str, facts: Vec<EvidenceFact>) -> EvidenceSnapshot {
    EvidenceSnapshot {
        version: 2,
        authority: "authentic-owner".into(),
        codec: "exact-resource-files-invocation-v3".into(),
        source_id: id.into(),
        snapshot_digest: "a".repeat(64),
        coverage: "partial".into(),
        expected_members: facts.iter().map(|row| row.member.clone()).collect(),
        facts,
    }
}
fn test(member: &str, test_id: &str, status: &str) -> EvidenceFact {
    EvidenceFact {
        member: member.into(),
        kind: "rust_test".into(),
        status: status.into(),
        test_id: Some(test_id.into()),
        binary_id: Some("original-binary".into()),
        crate_name: Some("pse-example".into()),
        ..Default::default()
    }
}
fn selection(sources: &[&EvidenceSnapshot]) -> EvidenceComparisonSelection {
    EvidenceComparisonSelection {
        generations: sources
            .iter()
            .map(|source| source.identity().unwrap().0)
            .collect(),
        crate_name: None,
        context_id: None,
    }
}
fn request(left: &EvidenceSnapshot, right: &EvidenceSnapshot) -> EvidenceComparisonRequest {
    EvidenceComparisonRequest {
        version: pse_model::document::Version,
        baseline: selection(&[left]),
        candidate: selection(&[right]),
        mappings: Vec::new(),
        measurement_mappings: Vec::new(),
        continuation_cut: None,
        sections: BTreeMap::new(),
    }
}
fn fixtures() -> (
    EvidenceSnapshot,
    EvidenceSnapshot,
    EvidenceComparisonRequest,
) {
    let baseline = source(
        "baseline",
        vec![
            test("resolved", "resolved", "failure"),
            test("persistent", "persistent", "failure"),
            test("new", "new", "pass"),
            test("missing", "missing", "failure"),
            test("old-name", "old-name", "failure"),
            test("removed", "removed", "failure"),
            test("ambiguous", "ambiguous", "pass"),
        ],
    );
    let mut renamed = test("new-name", "new-name", "pass");
    renamed.binary_id = Some("new-original-binary".into());
    let candidate = source(
        "candidate",
        vec![
            test("resolved", "resolved", "pass"),
            test("persistent", "persistent", "failure"),
            test("new", "new", "failure"),
            test("missing", "missing", "unreported"),
            renamed,
            test("added", "added", "pass"),
            test("ambiguous-one", "ambiguous", "pass"),
            test("ambiguous-two", "ambiguous", "failure"),
        ],
    );
    let mut request = request(&baseline, &candidate);
    request.mappings.push(EvidenceSubjectMapping {
        baseline: EvidenceMemberIdentity {
            generation: baseline.identity().unwrap().0,
            member: "old-name".into(),
        },
        candidate: EvidenceMemberIdentity {
            generation: candidate.identity().unwrap().0,
            member: "new-name".into(),
        },
    });
    (baseline, candidate, request)
}
#[tokio::test]
async fn evidence_comparison_independent_pairs_denominators_and_bounded_sections() {
    let (baseline, candidate, mut request) = fixtures();
    request.sections.insert(
        EvidenceComparisonSection::Subjects,
        EvidenceSectionSelection {
            limit: 1,
            offset: 0,
        },
    );
    request.sections.insert(
        EvidenceComparisonSection::Members,
        EvidenceSectionSelection {
            limit: 1,
            offset: 0,
        },
    );
    let mut analysis =
        EvidenceAnalysis::new(vec![baseline.clone(), candidate, baseline], true).unwrap();
    let result = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(result.baseline_denominator.selected_tests, 7);
    assert_eq!(result.candidate_denominator.selected_tests, 8);
    assert_eq!(result.baseline_denominator.failures, 5);
    assert_eq!(result.candidate_denominator.failures, 3);
    assert_eq!(result.candidate_denominator.unexecuted, 1);
    assert_eq!(result.summary.paired, 5);
    assert_eq!(result.summary.resolved_failures, 2);
    assert_eq!(result.summary.new_failures, 1);
    assert_eq!(result.summary.persistent_failures, 1);
    assert_eq!(result.summary.unexecuted, 1);
    assert_eq!(result.summary.ambiguous, 1);
    assert_eq!(result.summary.unmatched_baseline, 1);
    assert_eq!(result.summary.unmatched_candidate, 1);
    assert!(result.summary.changed_selection_denominator);
    assert!(result.summary.selection_membership_changed_or_unresolved);
    let subjects = result.subjects.as_ref().unwrap();
    assert_eq!(subjects.total_rows, 8);
    assert!(!subjects.complete);
    assert_eq!(subjects.next_offset, Some(1));
    let members = result.members.as_ref().unwrap();
    assert_eq!(members.total_rows, 15);
    assert!(!members.complete);
    assert_eq!(
        members.rows[0].identity.binary_id.as_deref(),
        Some("original-binary")
    );
    assert_eq!(
        result.baseline_sources.len(),
        1,
        "duplicate imports cannot double populations"
    );
    assert_eq!(result.baseline_sources[0].coverage, "partial");
    request.sections.clear();
    let compact = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(compact.cut, result.cut);
    assert_eq!(compact.summary.resolved_failures, 2);
    assert!(compact.subjects.is_none());
    assert!(serde_json::to_vec(&compact).unwrap().len() < 8192);
}
#[tokio::test]
async fn evidence_comparison_mapping_conflicts_unknowns_and_no_binary_name_guess() {
    let (baseline, candidate, mut request) = fixtures();
    let mut analysis = EvidenceAnalysis::new(vec![baseline, candidate], true).unwrap();
    request.mappings.push(request.mappings[0].clone());
    assert!(analysis.compare(&request, &[]).await.is_err());
    request.mappings.truncate(1);
    request.mappings[0].candidate.member = "outside".into();
    assert!(analysis.compare(&request, &[]).await.is_err());
    request.mappings.clear();
    let result = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(result.summary.paired, 4);
    assert_eq!(result.summary.unmatched_baseline, 2);
    assert_eq!(result.summary.unmatched_candidate, 2);
    let mut changed = source(
        "different-binary",
        vec![test("same-looking", "resolved", "pass")],
    );
    changed.facts[0].binary_id = Some("different".into());
    let baseline = source(
        "original",
        vec![test("same-looking", "resolved", "failure")],
    );
    let request = request_for(&baseline, &changed);
    let mut analysis = EvidenceAnalysis::new(vec![baseline, changed], true).unwrap();
    let result = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(result.summary.paired, 0);
    assert_eq!(result.summary.unmatched_baseline, 1);
    assert_eq!(result.summary.unmatched_candidate, 1);
}
fn request_for(left: &EvidenceSnapshot, right: &EvidenceSnapshot) -> EvidenceComparisonRequest {
    request(left, right)
}
#[tokio::test]
async fn evidence_comparison_repeated_mixed_outcomes_remain_original_evidence() {
    let baseline = source("first", vec![test("attempt", "same", "failure")]);
    let repeated = source("second", vec![test("attempt", "same", "pass")]);
    let candidate = source("third", vec![test("attempt", "same", "pass")]);
    let mut request = request(&baseline, &candidate);
    request.baseline = selection(&[&baseline, &repeated]);
    request.sections.insert(
        EvidenceComparisonSection::Subjects,
        EvidenceSectionSelection {
            limit: 10,
            offset: 0,
        },
    );
    let mut analysis = EvidenceAnalysis::new(vec![baseline, repeated, candidate], true).unwrap();
    let result = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(result.summary.paired, 1);
    assert_eq!(result.summary.mixed_outcome_cohorts, 1);
    let pair = &result.subjects.unwrap().rows[0];
    assert!(pair.mixed_outcomes);
    assert_eq!(pair.baseline_outcomes.failures, 1);
    assert_eq!(pair.baseline_outcomes.successes, 1);
    assert_eq!(result.baseline_denominator.selected_tests, 2);
    assert_eq!(result.candidate_denominator.selected_tests, 1);
}
fn measurements(id: &str, provider: &str, values: &[f64]) -> EvidenceSnapshot {
    let context = serde_json::json!({"workload":"same","input":"same","revision":"same","build":"same","profile":"same","features":"same","provider":provider,"isa":"same","capture":"same","host":"same","environment":"same","source":"same","mode":"same","process":"same"});
    let mut facts = vec![
        EvidenceFact {
            member: "context".into(),
            kind: "context".into(),
            status: "declared".into(),
            context_id: Some("ctx".into()),
            context_json: Some(context.to_string()),
            ..Default::default()
        },
        EvidenceFact {
            member: "association".into(),
            kind: "measurement_association".into(),
            status: "unavailable".into(),
            context_id: Some("ctx".into()),
            population_id: Some("cohort".into()),
            ..Default::default()
        },
    ];
    for (index, value) in values.iter().enumerate() {
        facts.push(EvidenceFact {
            member: format!("sample:{index}"),
            kind: "criterion_sample".into(),
            status: "measured".into(),
            context_id: Some("ctx".into()),
            population_id: Some("cohort".into()),
            sample_id: Some(format!("row:{index}")),
            value_ns: Some(*value),
            ..Default::default()
        });
    }
    let mut source = source(id, facts);
    source.codec = "process-cost-v5".into();
    source
}
#[tokio::test]
async fn evidence_comparison_noisy_incompatible_measurements_and_original_membership() {
    let baseline = measurements("baseline", "first-provider", &[10., 30.]);
    let candidate = measurements("candidate", "other-provider", &[11., 29.]);
    let mut request = request(&baseline, &candidate);
    for section in [
        EvidenceComparisonSection::Measurements,
        EvidenceComparisonSection::Samples,
        EvidenceComparisonSection::Associations,
        EvidenceComparisonSection::Contexts,
    ] {
        request.sections.insert(
            section,
            EvidenceSectionSelection {
                limit: 1,
                offset: 0,
            },
        );
    }
    let mut analysis = EvidenceAnalysis::new(vec![baseline, candidate], true).unwrap();
    let result = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(result.baseline_denominator.samples, 2);
    assert_eq!(result.candidate_denominator.samples, 2);
    assert_eq!(result.measurement_summary.incompatible, 1);
    assert_eq!(result.measurement_summary.comparable, 0);
    assert_eq!(
        result
            .measurement_summary
            .unavailable_association_populations,
        1
    );
    let measurements = result.measurements.unwrap();
    assert!(measurements.complete);
    assert_eq!(
        measurements.rows[0].baseline.as_ref().unwrap().mean_ns,
        Some(20.)
    );
    assert_eq!(
        measurements.rows[0].candidate.as_ref().unwrap().mean_ns,
        Some(20.)
    );
    assert_eq!(measurements.rows[0].verdict, "inconclusive");
    assert_eq!(measurements.rows[0].changed_fields, vec!["provider"]);
    let samples = result.samples.unwrap();
    assert_eq!(samples.total_rows, 4);
    assert!(!samples.complete);
    assert_eq!(samples.rows[0].fact.sample_id.as_deref(), Some("row:0"));
    let associations = result.associations.unwrap();
    assert_eq!(associations.total_rows, 2);
    assert_eq!(associations.rows[0].fact.status, "unavailable");
    assert_eq!(result.contexts.unwrap().total_rows, 2);
}
#[tokio::test]
async fn evidence_comparison_large_selected_cut_keeps_hidden_ambiguity_and_denominators() {
    let facts = (0..1200)
        .map(|i| test(&format!("member:{i}"), &format!("id:{i}"), "pass"))
        .collect();
    let baseline = source("large-baseline", facts);
    let mut candidate = baseline.clone();
    candidate.source_id = "large-candidate".into();
    candidate
        .facts
        .push(test("hidden-duplicate", "id:1199", "failure"));
    candidate.expected_members.push("hidden-duplicate".into());
    let mut request = request(&baseline, &candidate);
    request.sections.insert(
        EvidenceComparisonSection::Subjects,
        EvidenceSectionSelection {
            limit: 1,
            offset: 0,
        },
    );
    let mut analysis = EvidenceAnalysis::new(vec![baseline, candidate], true).unwrap();
    let result = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(result.baseline_denominator.selected_tests, 1200);
    assert_eq!(result.candidate_denominator.selected_tests, 1201);
    assert_eq!(result.summary.ambiguous, 1);
    assert_eq!(result.summary.paired, 1199);
    assert_eq!(result.subjects.as_ref().unwrap().total_rows, 1200);
    assert_eq!(result.subjects.as_ref().unwrap().rows.len(), 1);
    assert!(result.members.is_none());
    assert!(result.samples.is_none());
    assert!(result.contexts.is_none());
    let original_cut = result.cut.clone();
    request.continuation_cut = Some(original_cut.clone());
    request
        .sections
        .get_mut(&EvidenceComparisonSection::Subjects)
        .unwrap()
        .offset = 1199;
    let end = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(end.cut, original_cut);
    assert!(!end.subjects.as_ref().unwrap().complete);
    assert_eq!(end.subjects.unwrap().next_offset, None);
    request.continuation_cut = Some("stale".into());
    assert!(analysis.compare(&request, &[]).await.is_err());
    request.continuation_cut = None;
    assert!(analysis.compare(&request, &[]).await.is_err());
}
/// Called by the existing owned service fixture; it controls initialization and cleanup.
#[cfg(feature = "canonical-tests")]
pub(crate) async fn exercise_indexed_comparison(
    importer: &EvidenceStore,
    reader: &EvidenceStore,
) -> Result<(), EvidenceError> {
    let (baseline, candidate, request) = fixtures();
    let first = importer.import(&baseline).await?;
    let duplicate = importer.import(&baseline).await?;
    let second = importer.import(&candidate).await?;
    assert_eq!(first.generation, duplicate.generation);
    let cut = reader
        .source_cut(&[first.generation, second.generation], true)
        .await?;
    let mut offline = EvidenceAnalysis::new(vec![baseline, candidate], true)?;
    let mut indexed = EvidenceAnalysis::new(cut, true)?;
    let expected = offline.compare(&request, &[]).await?;
    let actual = indexed.compare(&request, &[]).await?;
    assert_eq!(actual.baseline_denominator.selected_tests, 7);
    assert_eq!(actual.candidate_denominator.selected_tests, 8);
    assert_eq!(actual.summary.paired, 5);
    assert_eq!(actual.summary.resolved_failures, 2);
    assert_eq!(actual.summary.ambiguous, 1);
    assert_eq!(
        serde_json::to_value(expected).unwrap(),
        serde_json::to_value(actual).unwrap()
    );
    Ok(())
}

#[tokio::test]
async fn evidence_comparison_explicit_measurement_mapping_and_censored_population() {
    let baseline = measurements("baseline", "same-provider", &[1., 100., 2.]);
    let mut candidate = measurements("candidate", "same-provider", &[2., 99., 1.]);
    candidate.authority = "different-original-owner".into();
    let mut request = request(&baseline, &candidate);
    request.sections.insert(
        EvidenceComparisonSection::Measurements,
        EvidenceSectionSelection {
            limit: 10,
            offset: 0,
        },
    );
    let mapping = EvidenceMeasurementMapping {
        baseline: EvidenceMeasurementIdentity {
            generation: baseline.identity().unwrap().0,
            context_id: Some("ctx".into()),
            population_id: "cohort".into(),
        },
        candidate: EvidenceMeasurementIdentity {
            generation: candidate.identity().unwrap().0,
            context_id: Some("ctx".into()),
            population_id: "cohort".into(),
        },
    };
    let mut analysis = EvidenceAnalysis::new(vec![baseline, candidate], true).unwrap();
    let unpaired = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(unpaired.measurement_summary.unavailable, 2);
    assert_eq!(unpaired.measurement_summary.comparable, 0);
    request.measurement_mappings.push(mapping.clone());
    let paired = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(paired.measurement_summary.comparable, 1);
    assert_eq!(
        paired.measurements.unwrap().rows[0].verdict,
        "inconclusive",
        "compatible noisy samples do not produce a statistical verdict"
    );
    request.measurement_mappings.push(mapping);
    assert!(analysis.compare(&request, &[]).await.is_err());
    let mut baseline = measurements("censored", "same-provider", &[]);
    baseline.facts.push(EvidenceFact {
        member: "population".into(),
        kind: "measurement_population".into(),
        status: "censored".into(),
        population_id: Some("cohort".into()),
        context_id: Some("ctx".into()),
        ..Default::default()
    });
    baseline.expected_members = baseline
        .facts
        .iter()
        .map(|fact| fact.member.clone())
        .collect();
    let candidate = measurements("measured", "same-provider", &[5.]);
    let mut request = request_for(&baseline, &candidate);
    request.sections.insert(
        EvidenceComparisonSection::Measurements,
        EvidenceSectionSelection {
            limit: 10,
            offset: 0,
        },
    );
    let mut analysis = EvidenceAnalysis::new(vec![baseline, candidate], true).unwrap();
    let result = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(result.measurement_summary.populations, 1);
    assert_eq!(result.measurement_summary.unavailable, 1);
    assert_eq!(result.baseline_denominator.samples, 0);
    assert_eq!(
        result.measurements.unwrap().rows[0]
            .baseline
            .as_ref()
            .unwrap()
            .censored_populations,
        1
    );
}

#[test]
fn evidence_comparison_request_native_version_and_strict_codec() {
    let (baseline, candidate, request) = fixtures();
    let bytes = serde_json::to_vec(&request).unwrap();
    let decoded = EvidenceComparisonRequest::decode_json(&bytes).unwrap();
    assert_eq!(
        decoded.baseline.generations,
        selection(&[&baseline]).generations
    );
    assert_eq!(
        decoded.candidate.generations,
        selection(&[&candidate]).generations
    );
    assert!(
        EvidenceComparisonRequest::decode_json(br#"{"version":2,"baseline":"invalid-body"}"#)
            .is_err()
    );
    assert!(EvidenceComparisonRequest::decode_json(br#"{"baseline":"missing-version"}"#).is_err());
    let mut original: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    original["unknown"] = true.into();
    assert!(
        EvidenceComparisonRequest::decode_json(&serde_json::to_vec(&original).unwrap()).is_err()
    );
}

#[tokio::test]
async fn evidence_comparison_inquiry_reuses_one_analysis_for_selected_questions() {
    let (baseline, candidate, request) = fixtures();
    let mut analysis = EvidenceAnalysis::new(vec![baseline, candidate], true).unwrap();
    let source_cut = analysis.generations.clone();
    let source_contents = analysis.contents.clone();
    let subjects = analysis
        .ask(
            &EvidenceQuestion {
                operation: EvidenceOperation::Subjects,
                ..Default::default()
            },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(subjects["denominator"][0]["selected_tests"], 15);
    assert_eq!(subjects["rows"].as_array().unwrap().len(), 15);
    let supplied_cut = subjects["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|source| source["generation"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(supplied_cut, source_cut.iter().cloned().collect());
    let invalid_question = EvidenceQuestion {
        operation: EvidenceOperation::Subjects,
        sections: BTreeMap::from([(
            EvidenceSection::Populations,
            EvidenceSectionSelection {
                limit: 1,
                offset: 0,
            },
        )]),
        ..Default::default()
    };
    assert!(
        analysis.ask(&invalid_question, &[]).await.is_err(),
        "a failed inquiry neither runs missing subjects nor discards their evidence"
    );
    let failures = analysis
        .ask(
            &EvidenceQuestion {
                operation: EvidenceOperation::Failures,
                ..Default::default()
            },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(failures["denominator"][0]["selected_tests"], 15);
    assert_eq!(failures["rows"].as_array().unwrap().len(), 8);
    let missing = analysis
        .ask(
            &EvidenceQuestion {
                operation: EvidenceOperation::Missing,
                ..Default::default()
            },
            &[],
        )
        .await
        .unwrap();
    assert_eq!(missing["denominator"][0]["selected_tests"], 15);
    assert_eq!(missing["rows"].as_array().unwrap().len(), 1);
    assert_eq!(missing["rows"][0]["member"], "missing");
    assert_eq!(missing["rows"][0]["status"], "unreported");
    assert_eq!(missing["sources"], subjects["sources"]);
    let comparison = analysis.compare(&request, &[]).await.unwrap();
    assert_eq!(comparison.baseline_denominator.selected_tests, 7);
    assert_eq!(comparison.candidate_denominator.selected_tests, 8);
    assert_eq!(comparison.candidate_denominator.unexecuted, 1);
    assert_eq!(comparison.summary.paired, 5);
    assert_eq!(comparison.summary.resolved_failures, 2);
    assert_eq!(comparison.summary.unexecuted, 1);
    assert_eq!(analysis.generations, source_cut);
    assert_eq!(analysis.contents, source_contents);
    let comparison_cut = comparison
        .baseline_sources
        .iter()
        .chain(&comparison.candidate_sources)
        .map(|source| source.generation.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(comparison_cut, supplied_cut);
}
