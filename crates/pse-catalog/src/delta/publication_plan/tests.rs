// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Publication constructors retain native shared producers without expanding them.
use super::*;
use crate::session::contract::ExecutionContract;
use datafusion::{arrow::datatypes::DataType, common::DFSchema, logical_expr::EmptyRelation};
use pse_relations::generated::enums::PublicationKind;
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, FieldContract, Namespace, RelationDecl, SnapshotClass, provider::OperationEffect,
    },
};

#[test]
fn declared_publication_preserves_deep_shared_inputs_without_running_them() {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_publications(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Shared publication input",
        )
        .pk(&["id"])
        .columns(vec![FieldContract::key(
            "id",
            FieldContract::native(DataType::Int64),
            "Key",
        )]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let spec = registry.relation("authored.values").unwrap();
    let mut input = LogicalPlan::EmptyRelation(EmptyRelation {
        produce_one_row: false,
        schema: Arc::new(
            DFSchema::try_from(pse_schema::arrow::relation_schema(&registry, spec).unwrap())
                .unwrap(),
        ),
    });
    for _ in 0..40 {
        let child = Arc::new(ExecutionContract::plan(
            input,
            None,
            [OperationEffect::Read].into_iter().collect(),
        ));
        input = LogicalPlan::Union(Union {
            schema: Arc::clone(child.schema()),
            inputs: vec![Arc::clone(&child), child],
        });
    }
    let directory = tempfile::tempdir().unwrap();
    let location = url::Url::from_directory_path(directory.path()).unwrap();
    for width in [1, 3] {
        let members = (0..width)
            .map(|index| {
                Member::Write(MemberWrite {
                    reference: ResolvedTableReference {
                        catalog: "artifact".into(),
                        schema: "authored".into(),
                        table: format!("values_{index}").into(),
                    },
                    relation_id: spec.id,
                    table: deltalake::DeltaTableBuilder::from_url(
                        location.join(&format!("values_{index}/")).unwrap(),
                    )
                    .unwrap()
                    .build()
                    .unwrap(),
                    input: input.clone(),
                })
            })
            .collect();
        let id = SemanticId::from_bytes([1; 16]);
        let header = publications::Row {
            workspace_id: id,
            publication_id: id,
            parent_publication_id: None,
            attempt_id: id,
            kind: PublicationKind::Relations,
            inputs: vec![],
            members: vec![],
        };
        let publication = plan(
            location.join("control/").unwrap(),
            header,
            members,
            Arc::clone(&registry),
        )
        .unwrap();
        assert_eq!(publication.schema().field(0).name(), "version");
    }
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}
