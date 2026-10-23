// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Factual identity headers; executable Replay constructors are never decoded here.
use super::*;
use pse_ids::ContentHash;
use std::collections::BTreeSet;

#[derive(serde::Deserialize)]
struct Coordinates {
    run_id: RunId,
    step: i64,
    source_identity: ContentHash,
    physical_identity: ContentHash,
}
#[derive(serde::Deserialize)]
struct SourceReference {
    source: usize,
}
#[derive(serde::Deserialize)]
struct Decisions {
    step: usize,
}
#[derive(serde::Deserialize)]
struct Invocation {
    version: Version<2>,
    run_id: RunId,
    attempt_id: Option<AttemptId>,
    operation: Option<OperationId>,
    attempts: InvocationAttempts,
}
#[derive(serde::Deserialize)]
struct InvocationAttempts {
    run_id: RunId,
}
fn decode<T: serde::de::DeserializeOwned>(fact: &ReviewContextFact) -> Result<T, WorkflowError> {
    serde_json::from_slice(&fact.payload)
        .map_err(|error| contract(format!("review {} identity header: {error}", fact.codec)))
}
/// An exact top-level step coordinate. Unknown path layouts do not acquire a step identity.
pub(super) fn step(path: &str) -> Option<usize> {
    let mut parts = path.strip_prefix("/steps/")?.split('/');
    parts.next()?.parse().ok()
}
fn source_reference(document: &ReviewContextDocument, source: usize) -> Result<(), WorkflowError> {
    if document.selection.include_sources && !document.source_positions.contains(&source) {
        return Err(contract(
            "review constructor references a source outside its retained selection",
        ));
    }
    Ok(())
}
pub(super) fn validate(document: &ReviewContextDocument) -> Result<(), WorkflowError> {
    let mut coordinates = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    for source in &document.sources {
        if !source_ids.insert(source.identity) {
            // Repeated source closures in different original positions are legitimate.
            if document
                .sources
                .iter()
                .filter(|s| s.identity == source.identity)
                .any(|s| {
                    s.physical_identity != source.physical_identity
                        || s.interpretation != source.interpretation
                        || s.build != source.build
                })
            {
                return Err(contract(
                    "review source identity has contradictory physical/build meaning",
                ));
            }
        }
    }
    for fact in &document.facts {
        if let Some(step) = step(&fact.path)
            && !document.selection.selects(step)
        {
            return Err(contract("review fact is outside selected original steps"));
        }
        match fact.codec.as_str() {
            "scientific-context.v1" | "scientific-coordinates.v1" => {
                let body: Coordinates = decode(fact)?;
                let ordinal = usize::try_from(body.step)
                    .map_err(|_| contract("negative review scientific step"))?;
                if document.run_id != Some(body.run_id)
                    || !document.selection.selects(ordinal)
                    || step(&fact.path).is_some_and(|step| step != ordinal)
                {
                    return Err(contract(
                        "review scientific coordinates contradict enclosing run/step identity",
                    ));
                }
                if !coordinates.insert((
                    body.run_id,
                    body.step,
                    fact.path
                        .rsplit_once('/')
                        .map_or(fact.path.as_str(), |(scope, _)| scope),
                )) {
                    return Err(contract("duplicate review scientific coordinates"));
                }
                // Constructors supply the exact original source position. No hash of
                // a compiled structure is compared with a source closure identity.
                let prefix = fact.path.strip_suffix("/context");
                let source_position = prefix
                    .and_then(|prefix| {
                        document
                            .facts
                            .iter()
                            .find(|f| f.path == format!("{prefix}/constructor"))
                    })
                    .map(decode::<SourceReference>)
                    .transpose()?
                    .map(|r| r.source)
                    .or_else(|| step(&fact.path));
                let source = source_position
                    .and_then(|position| {
                        document
                            .source_positions
                            .iter()
                            .position(|p| *p == position)
                    })
                    .map(|i| &document.sources[i]);
                if let Some(source) = source {
                    if source.identity != body.source_identity
                        || source.physical_identity != body.physical_identity
                    {
                        return Err(contract(
                            "review scientific coordinates contradict retained original source/physical identity",
                        ));
                    }
                } else if !document.sources.is_empty()
                    && !document.sources.iter().any(|source| {
                        source.identity == body.source_identity
                            && source.physical_identity == body.physical_identity
                    })
                {
                    return Err(contract(
                        "review scientific coordinates have no matching retained source identity",
                    ));
                }
            }
            "replay-case-constructor.v2"
            | "replay-simulation-constructor.v2"
            | "replay-fit-constructor.v3" => {
                let source = decode::<SourceReference>(fact)?.source;
                source_reference(document, source)?;
                if step(&fact.path).is_some_and(|step| step != source) {
                    return Err(contract(
                        "review original step constructor contradicts enclosing source position",
                    ));
                }
            }
            "replay-horizon-constructor.v2" => {
                #[derive(serde::Deserialize)]
                struct Analysis {
                    analysis: SourceReference,
                }
                #[derive(serde::Deserialize)]
                struct Horizon {
                    plant: SourceReference,
                    estimator: Option<Analysis>,
                    controller: Option<Analysis>,
                }
                let body: Horizon = decode(fact)?;
                source_reference(document, body.plant.source)?;
                for reference in body.estimator.iter().chain(body.controller.iter()) {
                    source_reference(document, reference.analysis.source)?;
                }
            }
            "replay-shooting-constructor.v2" => {
                #[derive(serde::Deserialize)]
                struct Shooting {
                    simulation: SourceReference,
                }
                source_reference(document, decode::<Shooting>(fact)?.simulation.source)?;
            }
            "replay-decisions.v2" => {
                let body: Decisions = decode(fact)?;
                if step(&fact.path) != Some(body.step) {
                    return Err(contract("review decisions contradict enclosing step"));
                }
            }
            "invocation-evidence.v2" => {
                let body: Invocation = decode(fact)?;
                let _version = body.version;
                if document.run_id != Some(body.run_id)
                    || body.run_id != body.attempts.run_id
                    || document.attempt_id != body.attempt_id
                    || document.operation != body.operation
                {
                    return Err(contract(
                        "review invocation contradicts enclosing run/attempt/operation identity",
                    ));
                }
            }
            "modeling-source-map.v1" => {
                let map: super::super::ModelingSourceMapDocument = decode(fact)?;
                map.validate()?;
                if !fact
                    .path
                    .strip_suffix("/source_map")
                    .is_some_and(attribution::valid_scope)
                {
                    return Err(contract(
                        "review source map requires an exact selected step/component scope",
                    ));
                }
            }
            "initialization-request.v1" | "declared-recycle-request.v1" => {
                source_reference(document, decode::<SourceReference>(fact)?.source)?;
            }
            "initialization-outcome.v1"
            | "initialization-attempt.v1"
            | "declared-recycle-outcome.v1"
            | "dynamic-native-statistics.v1" => {
                #[derive(serde::Deserialize)]
                struct StrategyRun {
                    run_id: RunId,
                }
                if document.run_id != Some(decode::<StrategyRun>(fact)?.run_id) {
                    return Err(contract(
                        "joined outcome fact contradicts enclosing original run identity",
                    ));
                }
            }
            "engine-execution-observations.v2" => {
                let _: pse_engine::session::CopiedExecutionObservations = decode(fact)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context(run: RunId, step: usize) -> ReviewContextDocument {
        let request = ReviewContextRequest {
            steps: vec![step],
            ..Default::default()
        };
        let mut out = builder(&request, Some(run), None);
        out.fact(
            format!("/steps/{step}/context"),
            "scientific-coordinates.v1",
            &serde_json::json!({
                "run_id":run,"step":step,"source_identity":ContentHash::from_bytes([2;32]),
                "physical_identity":ContentHash::from_bytes([3;32])
            }),
        )
        .unwrap();
        out.document
    }
    #[test]
    fn context_headers_reject_run_step_and_retained_source_contradictions() {
        let run = RunId::from_bytes([1; 16]);
        let mut document = context(run, 2);
        document.validate().unwrap();
        document.run_id = Some(RunId::from_bytes([4; 16]));
        assert!(document.validate().is_err());
        document.run_id = Some(run);
        document.facts[0].path = "/steps/1/context".into();
        assert!(document.validate().is_err());
        document.facts[0].path = "/steps/2/context".into();
        document.sources.push(ReviewSourceDocument {
            research: None,            context: None,
            identity: ContentHash::from_bytes([7; 32]),
            interpretation: "source".into(),
            build: ContentHash::from_bytes([8; 32]),
            physical_identity: ContentHash::from_bytes([3; 32]),
            physical_sources: vec![],
            providers: vec![],
            objects: vec![],
        });
        document.source_positions.push(2);
        assert!(document.validate().is_err());
        document.sources[0].identity = ContentHash::from_bytes([2; 32]);
        document.validate().unwrap();
    }
    #[test]
    fn context_constructor_headers_do_not_decode_executable_bindings() {
        let mut document = context(RunId::from_bytes([1; 16]), 2);
        document.selection.include_sources = false;
        document.facts.push(ReviewContextFact {
            path: "/steps/2/constructor".into(),
            codec: "replay-case-constructor.v2".into(),
            payload: br#"{"source":2,"bindings":"opaque and deliberately not executable"}"#
                .to_vec(),
        });
        document.validate().unwrap();
        document.facts[1].payload = br#"{"source":1,"bindings":"opaque"}"#.to_vec();
        assert!(document.validate().is_err());
    }
}
