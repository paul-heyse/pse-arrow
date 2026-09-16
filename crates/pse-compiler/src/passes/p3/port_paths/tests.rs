// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use crate::passes::native_rows::keyed_rows;
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_ids::{FixedBudget, MemoryReserver};
use pse_relations::columnar::FieldCheckedBatch;
use pse_relations::generated::{
    enums::{Direction, PortKind},
    provenance::algorithm_source_occurrences,
};
use std::{num::NonZeroUsize, sync::Arc};

fn session() -> SnapshotSession {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let reserver: Arc<dyn MemoryReserver> = FixedBudget::new(128 << 20);
    let one = NonZeroUsize::new(1).unwrap();
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        reserver,
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
        native_engine_profile(),
    )
    .unwrap()
    .candidate_checked(BTreeMap::new(), registry, &CancellationToken::new())
    .unwrap()
}

fn source(paths: &[(&str, &str)]) -> FieldCheckedBatch {
    let mut builder = template_ports::Builder::new().unwrap();
    for (name, path) in paths {
        builder
            .push(template_ports::Row {
                template_id: SemanticId::from_bytes([41; 16]),
                name: (*name).to_owned(),
                kind: PortKind::Material,
                direction: Direction::Inlet,
                bound_to: (*path).to_owned(),
                guard_id: None,
                doc: "Actual source syntax fixture".to_owned(),
            })
            .unwrap();
    }
    builder.finish().unwrap()
}

async fn selected(
    input: FieldCheckedBatch,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> (Vec<(template_ports::Row, SourceKey)>, AlgorithmInputs) {
    let role = "port-syntax";
    let session = session
        .with_checked_role_inputs(BTreeMap::from([(role.to_owned(), input)]), cancel)
        .unwrap();
    let mut arguments = AlgorithmInputs::new(session.reserver(), "port-syntax-test");
    let rows = keyed_rows::<template_ports::Row>(
        &mut arguments,
        session.scan_role(role).unwrap(),
        &session,
        session.registry(),
        cancel,
    )
    .await
    .unwrap();
    (
        rows.into_iter()
            .map(|row| {
                (
                    row.row,
                    SourceKey {
                        relation: template_ports::RELATION_KEY,
                        port: role.to_owned(),
                        key: row.key,
                    },
                )
            })
            .collect(),
        arguments,
    )
}

#[tokio::test]
async fn port_grammar_captures_actual_selected_keys_and_source_roles() {
    let session = session();
    let cancel = CancellationToken::new();
    let (ports, _arguments) = selected(
        source(&[("feed", "reactor.state"), ("own", "self")]),
        &session,
        &cancel,
    )
    .await;
    let keys = ports
        .iter()
        .map(|(row, key)| (row.name.clone(), key.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut output = OutputRows::new(session.registry(), session.reserver(), &cancel).unwrap();
    parse(ports, &mut output, &cancel).unwrap();
    let output = output.finish().unwrap();
    let lengths = port_binding_lengths::View::from_checked(
        &output.columns[&port_binding_lengths::RELATION_KEY],
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(
        lengths
            .iter()
            .map(|row| (row.name.as_str(), row.length))
            .collect::<Vec<_>>(),
        vec![("feed", 2), ("own", 0)]
    );
    let steps =
        port_binding_steps::View::from_checked(&output.columns[&port_binding_steps::RELATION_KEY])
            .unwrap()
            .rows()
            .unwrap();
    assert_eq!(
        steps
            .iter()
            .map(|row| (row.name.as_str(), row.position, row.child_name.as_str()))
            .collect::<Vec<_>>(),
        vec![("feed", 0, "reactor"), ("feed", 1, "state")]
    );
    // This tests syntax-to-column correspondence. Shared NativeInput tests separately
    // exercise source membership and derivation construction against published inputs.
    let occurrences = algorithm_source_occurrences::View::from_checked(&output.occurrences)
        .unwrap()
        .rows()
        .unwrap();
    assert_eq!(occurrences.len(), 4);
    for row in occurrences {
        let ordinal = usize::try_from(row.constructed_row_ordinal).unwrap();
        let name = if row.output_relation_id == port_binding_lengths::RELATION_ID {
            &lengths[ordinal].name
        } else {
            assert_eq!(row.output_relation_id, port_binding_steps::RELATION_ID);
            &steps[ordinal].name
        };
        assert_eq!(row.source_relation_id, template_ports::RELATION_ID);
        assert_eq!(row.source_key, keys[name].key);
        assert_eq!(row.source_port, keys[name].port);
    }
}

#[tokio::test]
async fn malformed_port_grammar_and_cancellation_fail_construction() {
    let session = session();
    let cancel = CancellationToken::new();
    for path in ["", "state..child", ".state", "state[0]", "state.2child"] {
        let (ports, _arguments) = selected(
            source(&[("first", "self"), ("bad", path)]),
            &session,
            &cancel,
        )
        .await;
        let mut output = OutputRows::new(session.registry(), session.reserver(), &cancel).unwrap();
        assert!(parse(ports, &mut output, &cancel).is_err());
    }
    let (ports, _arguments) = selected(source(&[("feed", "state")]), &session, &cancel).await;
    let mut output = OutputRows::new(session.registry(), session.reserver(), &cancel).unwrap();
    cancel.cancel();
    assert!(parse(ports, &mut output, &cancel).is_err());
}
