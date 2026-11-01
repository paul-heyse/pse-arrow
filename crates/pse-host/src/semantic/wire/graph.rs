// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked typed graph mappings; no JSON transcoding of known graph semantics.
use super::*;
use crate::investigation::{Direction, Relationship, graph};

fn relation_to_pb(value: Relationship) -> pb::Relationship {
    match value {
        Relationship::Definition => pb::Relationship::Definition,
        Relationship::References => pb::Relationship::References,
        Relationship::Calls => pb::Relationship::Calls,
        Relationship::Implementations => pb::Relationship::Implementations,
        Relationship::Contains => pb::Relationship::RelationContains,
        Relationship::ParameterType => pb::Relationship::ParameterType,
        Relationship::ReturnType => pb::Relationship::ReturnType,
        Relationship::FieldType => pb::Relationship::FieldType,
        Relationship::Uses => pb::Relationship::Uses,
    }
}
fn relation_from_pb(value: buffa::EnumValue<pb::Relationship>) -> Result<Relationship> {
    Ok(
        match value
            .as_known()
            .ok_or_else(|| malformed("graph.relation"))?
        {
            pb::Relationship::Definition => Relationship::Definition,
            pb::Relationship::References => Relationship::References,
            pb::Relationship::Calls => Relationship::Calls,
            pb::Relationship::Implementations => Relationship::Implementations,
            pb::Relationship::RelationContains => Relationship::Contains,
            pb::Relationship::ParameterType => Relationship::ParameterType,
            pb::Relationship::ReturnType => Relationship::ReturnType,
            pb::Relationship::FieldType => Relationship::FieldType,
            pb::Relationship::Uses => Relationship::Uses,
        },
    )
}
pub(super) fn query_to_pb(value: &GraphQuery) -> Result<pb::NativeGraphQuery> {
    let seeds = value
        .seeds
        .iter()
        .map(|seed| {
            let request = request_to_pb(&Request::Query(seed.clone()))?;
            match required(request.operation, "graph.seed")? {
                pb::native_request::Operation::Query(value) => Ok(*value),
                _ => Err(malformed("graph.seed")),
            }
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(pb::NativeGraphQuery {
        seeds,
        relations: value
            .relations
            .iter()
            .copied()
            .map(relation_to_pb)
            .map(Into::into)
            .collect(),
        direction: match value.direction {
            Direction::Outgoing => pb::Direction::Outgoing,
            Direction::Incoming => pb::Direction::Incoming,
        }
        .into(),
        traversal_paths: value.traversal_paths.iter().map(|p| path(p)).collect(),
        output_paths: value.output_paths.iter().map(|p| path(p)).collect(),
        depth: number(value.depth),
        max_nodes: number(value.max_nodes),
        max_edges: number(value.max_edges),
        seed_match: match value.seed_match {
            graph::SeedMatch::Any => pb::SeedMatch::AnySeed,
            graph::SeedMatch::All => pb::SeedMatch::AllSeeds,
        }
        .into(),
        exists: value.exists,
        ..Default::default()
    })
}
pub(super) fn query_from_pb(value: pb::NativeGraphQuery) -> Result<GraphQuery> {
    if value.seeds.is_empty()
        || value.relations.is_empty()
        || value.depth == 0
        || value.max_nodes < value.seeds.len() as u64
        || value.max_edges == 0
    {
        return Err(malformed(
            "graph nonempty seeds/relations and positive bounds",
        ));
    }
    let seeds = value
        .seeds
        .into_iter()
        .map(|seed| {
            match request_from_pb(pb::NativeRequest {
                operation: Some(pb::native_request::Operation::Query(Box::new(seed))),
                ..Default::default()
            })? {
                Request::Query(value) if matches!(value.kind, Kind::Explain) => Ok(value),
                _ => Err(malformed("graph seeds require Explain positions")),
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let relations = value
        .relations
        .into_iter()
        .map(relation_from_pb)
        .collect::<Result<Vec<_>>>()?;
    if relations
        .iter()
        .enumerate()
        .any(|(index, relation)| relations[..index].contains(relation))
    {
        return Err(malformed("graph relation kinds must be unique"));
    }
    Ok(GraphQuery {
        seeds,
        relations,
        direction: match value
            .direction
            .as_known()
            .ok_or_else(|| malformed("graph.direction"))?
        {
            pb::Direction::Outgoing => Direction::Outgoing,
            pb::Direction::Incoming => Direction::Incoming,
        },
        traversal_paths: value.traversal_paths.into_iter().map(from_path).collect(),
        output_paths: value.output_paths.into_iter().map(from_path).collect(),
        depth: size(value.depth, "graph.depth")?,
        max_nodes: size(value.max_nodes, "graph.max_nodes")?,
        max_edges: size(value.max_edges, "graph.max_edges")?,
        seed_match: match value
            .seed_match
            .as_known()
            .ok_or_else(|| malformed("graph.seed_match"))?
        {
            pb::SeedMatch::AnySeed => graph::SeedMatch::Any,
            pb::SeedMatch::AllSeeds => graph::SeedMatch::All,
        },
        exists: value.exists,
    })
}
fn role_to_pb(role: GraphRole) -> pb::NativeGraphRole {
    use pb::native_graph_role::Kind as K;
    let mut value = pb::NativeGraphRole::default();
    value.kind = match role {
        GraphRole::Containment => K::Containment,
        GraphRole::Parameter { index } => {
            value.index = Some(number(index));
            K::Parameter
        }
        GraphRole::Return { async_output } => {
            value.async_output = Some(async_output);
            K::Return
        }
        GraphRole::Field { index, name } => {
            value.index = Some(number(index));
            value.name = name;
            K::Field
        }
        GraphRole::Reference {
            read,
            write,
            import,
            test,
        } => {
            value.read = read;
            value.write = write;
            value.import = import;
            value.test = test;
            K::Reference
        }
        GraphRole::Navigation => K::Navigation,
    }
    .into();
    value
}
fn role_from_pb(value: pb::NativeGraphRole) -> Result<GraphRole> {
    use pb::native_graph_role::Kind as K;
    let kind = value
        .kind
        .as_known()
        .ok_or_else(|| malformed("graph.role.kind"))?;
    let positional = matches!(kind, K::Parameter | K::Field);
    if value.index.is_some() != positional
        || (value.name.is_some() && kind != K::Field)
        || value.async_output.is_some() != (kind == K::Return)
        || ((value.read.is_some()
            || value.write.is_some()
            || value.import.is_some()
            || value.test.is_some())
            && kind != K::Reference)
    {
        return Err(malformed("graph.role alternatives"));
    }
    Ok(match kind {
        K::Containment => GraphRole::Containment,
        K::Parameter => GraphRole::Parameter {
            index: size(
                required(value.index, "graph.role.index")?,
                "graph.role.index",
            )?,
        },
        K::Return => GraphRole::Return {
            async_output: required(value.async_output, "graph.role.async_output")?,
        },
        K::Field => GraphRole::Field {
            index: size(
                required(value.index, "graph.role.index")?,
                "graph.role.index",
            )?,
            name: value.name,
        },
        K::Reference => GraphRole::Reference {
            read: value.read,
            write: value.write,
            import: value.import,
            test: value.test,
        },
        K::Navigation => GraphRole::Navigation,
    })
}
pub(super) fn answer_to_pb(value: GraphAnswer) -> Result<pb::NativeGraphAnswer> {
    use pb::native_graph_traversal::Stop as S;
    let t = value.traversal;
    Ok(pb::NativeGraphAnswer {
        session: analysis_to_pb(value.session)?.into(),
        generation: value.generation,
        nodes: value
            .nodes
            .into_iter()
            .map(|node| pb::NativeGraphNode {
                location: native_location_to_pb(node.location).into(),
                name: node.name,
                source_sha256: node.source_sha256,
                declaration: node.declaration,
                ..Default::default()
            })
            .collect(),
        witnesses: value
            .witnesses
            .into_iter()
            .map(|w| pb::NativeGraphWitness {
                location: w.location.map(native_location_to_pb).into(),
                source_sha256: w.source_sha256,
                role: role_to_pb(w.role).into(),
                declared_type: w.declared_type,
                instantiated_type: w.instantiated_type,
                limitations: w.limitations,
                ..Default::default()
            })
            .collect(),
        traversal: pb::NativeGraphTraversal {
            reached: t
                .reached
                .into_iter()
                .map(|r| pb::NativeGraphReached {
                    node: number(r.node),
                    distances: r
                        .distances
                        .into_iter()
                        .map(|distance| pb::NativeGraphDistance {
                            distance: distance.map(number),
                            ..Default::default()
                        })
                        .collect(),
                    ..Default::default()
                })
                .collect(),
            selected: t.selected.into_iter().map(number).collect(),
            edges: t
                .edges
                .into_iter()
                .map(|e| pb::NativeGraphEdge {
                    source: number(e.source),
                    target: number(e.target),
                    relation: relation_to_pb(e.relation).into(),
                    witness: number(e.witness),
                    ..Default::default()
                })
                .collect(),
            frontier: t.frontier.into_iter().map(number).collect(),
            complete: t.complete,
            stop: match t.stop {
                graph::Stop::Complete => S::Complete,
                graph::Stop::Nodes => S::Nodes,
                graph::Stop::Edges => S::Edges,
                graph::Stop::Provider => S::Provider,
                graph::Stop::Cancelled => S::Cancelled,
                graph::Stop::Demand => S::Demand,
            }
            .into(),
            examined_edges: number(t.examined_edges),
            ..Default::default()
        }
        .into(),
        limitations: value.limitations,
        ..Default::default()
    })
}
pub(in crate::semantic) fn answer_from_pb(mut value: pb::NativeGraphAnswer) -> Result<GraphAnswer> {
    use pb::native_graph_traversal::Stop as S;
    let t = required(value.traversal.take(), "graph.traversal")?;
    let answer = GraphAnswer {
        session: analysis_from_pb(required(value.session.take(), "graph.session")?)?,
        generation: value.generation,
        nodes: value
            .nodes
            .into_iter()
            .map(|mut n| {
                Ok(GraphNode {
                    location: native_location_from_pb(required(
                        n.location.take(),
                        "graph.node.location",
                    )?)?,
                    name: n.name,
                    source_sha256: n.source_sha256,
                    declaration: n.declaration,
                })
            })
            .collect::<Result<_>>()?,
        witnesses: value
            .witnesses
            .into_iter()
            .map(|mut w| {
                if w.location.is_set() != w.source_sha256.is_some() {
                    return Err(malformed("graph.witness location guard"));
                }
                Ok(GraphWitness {
                    location: w.location.take().map(native_location_from_pb).transpose()?,
                    source_sha256: w.source_sha256,
                    role: role_from_pb(required(w.role.take(), "graph.witness.role")?)?,
                    declared_type: w.declared_type,
                    instantiated_type: w.instantiated_type,
                    limitations: w.limitations,
                })
            })
            .collect::<Result<_>>()?,
        traversal: graph::TraversalResult {
            reached: t
                .reached
                .into_iter()
                .map(|r| {
                    Ok(graph::Reached {
                        node: size(r.node, "graph.reached.node")?,
                        distances: r
                            .distances
                            .into_iter()
                            .map(|d| d.distance.map(|d| size(d, "graph.distance")).transpose())
                            .collect::<Result<_>>()?,
                    })
                })
                .collect::<Result<_>>()?,
            selected: t
                .selected
                .into_iter()
                .map(|n| size(n, "graph.selected"))
                .collect::<Result<_>>()?,
            edges: t
                .edges
                .into_iter()
                .map(|e| {
                    Ok(graph::Edge {
                        source: size(e.source, "graph.edge.source")?,
                        target: size(e.target, "graph.edge.target")?,
                        relation: relation_from_pb(e.relation)?,
                        witness: size(e.witness, "graph.edge.witness")?,
                    })
                })
                .collect::<Result<_>>()?,
            frontier: t
                .frontier
                .into_iter()
                .map(|n| size(n, "graph.frontier"))
                .collect::<Result<_>>()?,
            complete: t.complete,
            stop: match t.stop.as_known().ok_or_else(|| malformed("graph.stop"))? {
                S::Complete => graph::Stop::Complete,
                S::Nodes => graph::Stop::Nodes,
                S::Edges => graph::Stop::Edges,
                S::Provider => graph::Stop::Provider,
                S::Cancelled => graph::Stop::Cancelled,
                S::Demand => graph::Stop::Demand,
            },
            examined_edges: size(t.examined_edges, "graph.examined_edges")?,
        },
        limitations: value.limitations,
    };
    let digest =
        |value: &str| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    if answer.nodes.iter().any(|n| !digest(&n.source_sha256))
        || answer
            .witnesses
            .iter()
            .any(|w| w.source_sha256.as_ref().is_some_and(|value| !digest(value)))
        || answer
            .traversal
            .reached
            .iter()
            .any(|n| n.node >= answer.nodes.len())
        || answer
            .traversal
            .selected
            .iter()
            .chain(&answer.traversal.frontier)
            .any(|&n| n >= answer.nodes.len())
        || answer.traversal.edges.iter().any(|e| {
            e.source >= answer.nodes.len()
                || e.target >= answer.nodes.len()
                || e.witness >= answer.witnesses.len()
        })
    {
        return Err(malformed("graph indexes and source guards"));
    }
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    fn context() -> AnalysisContext {
        AnalysisContext {
            backend: "native-fixture".into(),
            configuration: Configuration {
                backend: Backend::NativeSnapshot,
                project: "/fixture".into(),
                features: Vec::new(),
                no_default_features: false,
                target: None,
                build_macros: false,
                test_context: false,
            },
            position_encoding: "utf-8".into(),
            backend_status: None,
            preparation: "prepared fixture".into(),
        }
    }
    fn location() -> NativeLocation {
        NativeLocation {
            path: "/fixture/lib.rs".into(),
            start: 3,
            end: 7,
        }
    }
    fn answer() -> GraphAnswer {
        GraphAnswer {
            session: context(),
            generation: 1,
            nodes: vec![GraphNode {
                declaration: true,
                location: location(),
                name: Some("Policy".into()),
                source_sha256: "a".repeat(64),
            }],
            witnesses: vec![GraphWitness {
                location: Some(location()),
                source_sha256: Some("a".repeat(64)),
                role: GraphRole::Parameter { index: 2 },
                declared_type: Some("Policy".into()),
                instantiated_type: None,
                limitations: Vec::new(),
            }],
            traversal: graph::TraversalResult {
                reached: vec![graph::Reached {
                    node: 0,
                    distances: vec![Some(0), Some(2)],
                }],
                selected: vec![0],
                edges: vec![graph::Edge {
                    source: 0,
                    target: 0,
                    relation: Relationship::ParameterType,
                    witness: 0,
                }],
                frontier: vec![0],
                complete: true,
                stop: graph::Stop::Complete,
                examined_edges: 1,
            },
            limitations: Vec::new(),
        }
    }
    fn query() -> GraphQuery {
        GraphQuery {
            seeds: vec![Query {
                kind: Kind::Explain,
                file: "/fixture/lib.rs".into(),
                line: 1,
                column: 4,
                if_source: Some("a".repeat(64)),
                if_generation: Some(1),
                if_candidate: None,
                limit: 20,
                end_line: None,
            }],
            relations: vec![Relationship::ParameterType, Relationship::FieldType],
            direction: Direction::Incoming,
            traversal_paths: vec!["/fixture/lib.rs".into()],
            output_paths: Vec::new(),
            depth: 2,
            max_nodes: 20,
            max_edges: 100,
            seed_match: graph::SeedMatch::All,
            exists: false,
        }
    }
    #[test]
    fn normalized_graph_wire_preserves_guards_roles_and_rejects_unknown_semantics() {
        let wire = query_to_pb(&query()).unwrap();
        let wire =
            pb::NativeGraphQuery::decode_from_slice(wire.encode_to_vec().as_slice()).unwrap();
        let restored = query_from_pb(wire.clone()).unwrap();
        assert_eq!(
            restored.seeds[0].if_source.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(
            restored.relations,
            vec![Relationship::ParameterType, Relationship::FieldType]
        );
        assert_eq!(restored.seed_match, graph::SeedMatch::All);
        let mut invalid = wire.clone();
        invalid.seeds.clear();
        assert!(query_from_pb(invalid).is_err());
        let mut invalid = wire.clone();
        invalid.relations[0] = 999i32.into();
        assert!(query_from_pb(invalid).is_err());
        let mut invalid = wire.clone();
        invalid.direction = 999i32.into();
        assert!(query_from_pb(invalid).is_err());
        let mut invalid = wire.clone();
        invalid.seed_match = 999i32.into();
        assert!(query_from_pb(invalid).is_err());
        let mut invalid = wire;
        invalid.max_nodes = 0;
        assert!(query_from_pb(invalid).is_err());
        let wire = answer_to_pb(answer()).unwrap();
        let restored = answer_from_pb(
            pb::NativeGraphAnswer::decode_from_slice(wire.encode_to_vec().as_slice()).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            restored.witnesses[0].role,
            GraphRole::Parameter { index: 2 }
        ));
        assert_eq!(
            restored.traversal.reached[0].distances,
            vec![Some(0), Some(2)]
        );
        let mut invalid = wire.clone();
        invalid.witnesses[0].source_sha256 = None;
        assert!(answer_from_pb(invalid).is_err());
        let mut invalid = wire.clone();
        invalid.witnesses[0].role.as_option_mut().unwrap().index = None;
        assert!(answer_from_pb(invalid).is_err());
        let mut invalid = wire.clone();
        invalid.traversal.as_option_mut().unwrap().edges[0].target = 9;
        assert!(answer_from_pb(invalid).is_err());
        let mut invalid = wire.clone();
        invalid.traversal.as_option_mut().unwrap().edges[0].witness = 9;
        assert!(answer_from_pb(invalid).is_err());
        let mut invalid = wire;
        invalid.nodes[0].source_sha256 = "unguarded".into();
        assert!(answer_from_pb(invalid).is_err());
    }
}
