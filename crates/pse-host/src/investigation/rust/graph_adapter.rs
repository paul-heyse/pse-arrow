// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One bounded relationship operation, retaining original source and native world guards.
use super::*;

/// Normalized public intent; the native operation does not receive the composition DSL.
#[derive(Debug)]
pub struct GraphIntent<'a> {
    /// Original guarded source/declaration addresses.
    pub seeds: &'a [ResultReference],
    /// Canonical relationship meanings to examine.
    pub relations: &'a [Relationship],
    /// Walk direction, independent of canonical edge orientation.
    pub direction: Direction,
    /// Source selection admitting seeds and supporting intermediates.
    pub traversal: &'a SourceSelector,
    /// Ordinary answer-node selection.
    pub output: &'a SourceSelector,
    /// Required seed attribution.
    pub seed_match: graph::SeedMatch,
    /// Positive existence demand may stop once the selected seed rule is established.
    pub exists: bool,
    /// Intentional maximum edge distance.
    pub depth: usize,
    /// Distinct admitted node allowance, including seeds.
    pub max_nodes: usize,
    /// Edge-observation examination allowance.
    pub max_edges: usize,
}

/// Selected rows and their mandatory supporting graph travel together.
#[derive(Debug)]
pub struct GraphEvaluation {
    /// Selected positive answer nodes, separate from support.
    pub rows: Vec<SearchResult>,
    /// Shared original provider/world context.
    pub sources: Vec<SourceContext>,
    /// Relevant within-domain/depth examination completed.
    pub complete: bool,
    /// Original coverage limitations or refusals.
    pub diagnostics: Vec<QueryDiagnostic>,
    /// Mandatory reusable edges, witnesses, attribution and coverage.
    pub evidence: RelationshipEvidence,
}

/// Select paths against the already prepared project without refreshing Cargo metadata.
fn paths(
    root: &Path,
    selector: &SourceSelector,
    token: &CancellationToken,
) -> Result<Vec<std::path::PathBuf>, Error> {
    providers::select_prepared_context(root, selector, token)?
        .into_iter()
        .map(|source| root.join(source.path()).canonicalize().map_err(Error::from))
        .collect()
}

/// Resolve all guarded seeds, then run exactly one native worker and immutable snapshot.
pub async fn graph_with_context(
    root: &Path,
    intent: &GraphIntent<'_>,
    token: &CancellationToken,
    context: &ResolutionContext,
) -> Result<GraphEvaluation, Error> {
    if intent.traversal.kind != SourceKind::Working || intent.output.kind != SourceKind::Working {
        return Err(Error::Unavailable("native relationships require working source selections; index/revision facts cannot become current native semantics".into()));
    }
    if intent.seeds.is_empty() {
        return Err(Error::Invalid(
            "relationship seeds must not be empty".into(),
        ));
    }
    let env = environment();
    let record = crate::semantic::load(root, &env)?.ok_or_else(|| {
        Error::Unavailable("explicitly prepare the native analyzer session".into())
    })?;
    let project = record.configuration.project.canonicalize()?;
    let mut seeds = Vec::with_capacity(intent.seeds.len());
    let mut guarded_generation = None;
    for seed in intent.seeds {
        let locator = source_semantic_with_context(root, &seed.locator, token, context).await?;
        let Locator::Semantic {
            project: seed_project,
            path,
            line,
            column,
            generation,
            source_sha256,
            candidate_fingerprint,
            session_invocation,
        } = locator
        else {
            return Err(Error::Invalid(
                "relationship seed has no native position".into(),
            ));
        };
        if session_invocation.as_deref() != Some(record.invocation.as_str())
            || seed_project.canonicalize()? != project
        {
            return Err(Error::Stale(
                "relationship seeds must belong to the selected original session and project"
                    .into(),
            ));
        }
        let generation = generation
            .as_deref()
            .map(str::parse::<u64>)
            .transpose()
            .map_err(|_| Error::Invalid("invalid semantic generation".into()))?;
        if let Some(generation) = generation {
            if guarded_generation.is_some_and(|original| original != generation) {
                return Err(Error::Stale(
                    "relationship seeds span native snapshot generations".into(),
                ));
            }
            guarded_generation = Some(generation);
        }
        seeds.push(crate::semantic::Query {
            kind: crate::semantic::Kind::Explain,
            file: path.to_path_buf(),
            line,
            column,
            if_source: source_sha256,
            if_generation: generation,
            if_candidate: candidate_fingerprint,
            limit: intent.max_nodes,
            end_line: None,
        });
    }
    let request = crate::semantic::Request::Graph(crate::semantic::GraphQuery {
        seeds,
        relations: intent.relations.to_vec(),
        direction: intent.direction,
        traversal_paths: paths(&project, intent.traversal, token)?,
        output_paths: paths(&project, intent.output, token)?,
        depth: intent.depth,
        max_nodes: intent.max_nodes,
        max_edges: intent.max_edges,
        seed_match: intent.seed_match,
        exists: intent.exists,
    });
    let answer = context
        .request(root, &env, &record, &request, token)
        .await?
        .into_graph()?;
    if answer.session.configuration != record.configuration
        || guarded_generation.is_some_and(|expected| expected != answer.generation)
    {
        return Err(Error::Stale(
            "relationship answer changed its original prepared world".into(),
        ));
    }
    let configuration = answer.session.configuration.identity()?;
    let mut sources = BTreeMap::new();
    let mut nodes = Vec::with_capacity(answer.nodes.len());
    let mut ranges = Vec::with_capacity(answer.nodes.len());
    for (id, node) in answer.nodes.iter().enumerate() {
        let (reference, range) = address(
            &record,
            answer.generation,
            &node.location,
            &node.source_sha256,
            &mut sources,
        )?;
        nodes.push(RelationshipNode {
            id,
            entity: node.declaration.then(|| EntityIdentity {
                session_invocation: record.invocation.as_str().into(),
                configuration: configuration.clone(),
                generation: answer.generation,
                declaration: node.location.path.clone().into(),
                byte_start: node.location.start,
                byte_end: node.location.end,
            }),
            reference,
            name: node.name.clone(),
        });
        ranges.push(range);
    }
    let mut witnesses = Vec::with_capacity(answer.witnesses.len());
    for (id, witness) in answer.witnesses.iter().enumerate() {
        let reference = match (&witness.location, &witness.source_sha256) {
            (Some(location), Some(digest)) => {
                let (_, range) =
                    address(&record, answer.generation, location, digest, &mut sources)?;
                Some(source_reference(&record, location, digest, range))
            }
            (None, None) => None,
            _ => {
                return Err(Error::Unavailable(
                    "native occurrence mapping has no matching original byte guard".into(),
                ));
            }
        };
        witnesses.push(RelationshipWitness {
            id,
            reference,
            role: witness.role.clone(),
            declared_type: witness.declared_type.clone(),
            instantiated_type: witness.instantiated_type.clone(),
            limitations: witness.limitations.clone(),
        });
    }
    let mut rows = Vec::with_capacity(answer.traversal.selected.len());
    for &id in &answer.traversal.selected {
        let node = nodes
            .get(id)
            .ok_or_else(|| Error::Unavailable("native graph selected an unknown node".into()))?;
        rows.push(SearchResult {
            source_relationship: None,
            source: "native_relationship".into(),
            kind: if node.entity.is_some() {
                "related_declaration"
            } else {
                "related_occurrence"
            }
            .into(),
            name: node.name.clone(),
            range: Some(ranges[id].clone()),
            context: String::new(),
            bytes: None,
            reference: Some(node.reference.clone()),
            entity: node.entity.clone(),
            captures: BTreeMap::new(),
            candidate_reference: None,
            refinement: None,
        });
    }
    let seeds = (0..intent.seeds.len())
        .map(|seed| {
            answer
                .traversal
                .reached
                .iter()
                .find(|node| node.distances.get(seed) == Some(&Some(0)))
                .map(|node| node.node)
                .ok_or_else(|| Error::Unavailable("native graph lost seed attribution".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let complete = answer.traversal.complete;
    let diagnostics = if complete || answer.traversal.stop == graph::Stop::Demand {
        Vec::new()
    } else {
        vec![QueryDiagnostic { phase: DiagnosticPhase::Context, code: "search.native_relationship_scope".into(), message: "relationship examination is incomplete within the declared domain/depth; missing nodes remain unknown".into(), source: Some("native_relationship".into()), detail: None }]
    };
    let source = SourceContext {
        id: "native_relationship".into(),
        path: project.into(),
        origin: Some(SourceKind::Working),
        provider: answer.session.backend.clone(),
        generation: Some(answer.generation.to_string()),
        searched_encoding: None,
        native: Some(NativeContext {
            session_invocation: record.invocation.as_str().into(),
            generation: answer.generation,
            analysis: answer.session,
        }),
    };
    Ok(GraphEvaluation {
        rows,
        sources: vec![source],
        complete,
        diagnostics,
        evidence: RelationshipEvidence {
            domain: RelationshipDomain::Native,
            seeds,
            relations: intent.relations.to_vec(),
            direction: intent.direction,
            seed_match: intent.seed_match,
            traversal: intent.traversal.clone(),
            depth: intent.depth,
            nodes,
            witnesses,
            graph: answer.traversal,
            limitations: answer.limitations,
        },
    })
}

fn source_reference(
    record: &crate::semantic::SessionRecord,
    location: &crate::semantic::NativeLocation,
    digest: &str,
    range: MatchRange,
) -> ResultReference {
    ResultReference::new(Locator::Source {
        path: location.path.clone().into(),
        source: SourceKind::Working,
        range: Some(range),
        guard: Some(SourceGuard {
            root: Some(record.configuration.project.clone().into()),
            sha256: Some(digest.into()),
            ..Default::default()
        }),
    })
}

/// Rehydrate only authenticated original byte ranges, caching source acquisition per response.
fn address(
    record: &crate::semantic::SessionRecord,
    generation: u64,
    location: &crate::semantic::NativeLocation,
    digest: &str,
    sources: &mut BTreeMap<std::path::PathBuf, SourceText>,
) -> Result<(ResultReference, MatchRange), Error> {
    if !sources.contains_key(&location.path) {
        sources.insert(
            location.path.clone(),
            SourceRequest::Working {
                path: location.path.clone(),
            }
            .acquire()
            .map_err(native)?,
        );
    }
    let source = &sources[&location.path];
    if source.identity().source_sha256() != digest
        || location.start > location.end
        || source.text().get(location.start..location.end).is_none()
    {
        return Err(Error::Stale(
            "native graph source or byte interval changed before guarded expansion".into(),
        ));
    }
    let text = source.text();
    let anchor = crate::source_read::semantic_anchor(text, location.start..location.end)
        .unwrap_or(location.start);
    let line_at = |offset: usize| text[..offset].bytes().filter(|byte| *byte == b'\n').count() + 1;
    let locator = Locator::Semantic {
        project: record.configuration.project.clone(),
        path: location.path.clone().into(),
        line: line_at(anchor),
        column: text[..anchor]
            .rsplit('\n')
            .next()
            .map_or(1, |line| line.chars().count() + 1),
        generation: Some(generation.to_string()),
        source_sha256: Some(digest.into()),
        candidate_fingerprint: None,
        session_invocation: Some(record.invocation.as_str().into()),
    };
    Ok((
        ResultReference::new(locator),
        MatchRange {
            byte_start: location.start,
            byte_end: location.end,
            line_start: line_at(location.start),
            line_end: line_at(
                location
                    .end
                    .saturating_sub(usize::from(location.end > location.start)),
            ),
        },
    ))
}
