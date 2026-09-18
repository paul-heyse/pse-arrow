// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Isolated tuple and graph-shape units; these do not run a compiler or Delta journey.
use super::*;
use datafusion::{
    common::TableReference,
    datasource::{MemTable, provider_as_source},
    logical_expr::LogicalPlanBuilder,
};
use pse_ids::{FixedBudget, SemanticId};
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::reference::{elements, math_context},
};
use pse_schema::model::ResultSpec;

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "native_tuple_selects_typed_members_and_preserves_empty_relations keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
async fn native_tuple_selects_typed_members_and_preserves_empty_relations() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let cancel = CancellationToken::new();
    let (session, _) = crate::passes::native_test::session(
        &registry,
        BTreeMap::new(),
        &FixedBudget::new(128 << 20),
        &cancel,
    )
    .unwrap();
    let mut spec = registry.algorithm("P3@1").unwrap().clone();
    spec.outputs = vec![
        ResultSpec {
            port: "values".into(),
            relation: "reference.elements".into(),
        },
        ResultSpec {
            port: "empty".into(),
            relation: "reference.math_context".into(),
        },
    ];
    let layout = Arc::new(layout::Layout::new(&registry, &spec).unwrap());
    let mut builder = elements::Row::builder(&registry, 1).unwrap();
    elements::Row::push(
        &mut builder,
        elements::Row {
            element_id: SemanticId::from_bytes([7; 16]),
            symbol: "C".into(),
            name: "carbon".into(),
            atomic_mass: 12.011,
        },
    )
    .unwrap();
    let values = elements::Row::finish(builder).unwrap();
    let empty =
        FieldCheckedBatch::concat(&registry, math_context::spec(&registry).unwrap(), &[]).unwrap();
    let tuple = layout
        .pack(
            crate::AlgorithmOutput {
                outputs: BTreeMap::from([
                    ("values".into(), values.clone()),
                    ("empty".into(), empty.clone()),
                ]),
                findings: vec![],
                derivations: vec![],
                plans: vec![],
            },
            &session,
            &cancel,
        )
        .unwrap();
    assert_eq!(
        layout.member("values", &tuple, &registry).unwrap().batch(),
        values.batch()
    );
    assert_eq!(
        layout.member("empty", &tuple, &registry).unwrap().batch(),
        empty.batch()
    );
    let provider = Arc::new(MemTable::try_new(tuple.schema(), vec![vec![tuple]]).unwrap());
    let name = TableReference::full("fixture", "native", "tuple");
    let session = session
        .with_provider(name.clone(), provider.clone(), &cancel)
        .unwrap();
    let tuple = LogicalPlanBuilder::scan(name, provider_as_source(provider), None)
        .unwrap()
        .build()
        .unwrap();
    let plans = layout.unpack_plans(&tuple, &registry).unwrap();
    for (port, expected) in [("values", values), ("empty", empty)] {
        let plan = plans[port].plan().clone();
        let completed = session
            .prepare_rule_plan(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let spec = registry.relation_by_id(expected.relation_id()).unwrap();
        let actual = completed
            .into_checked_relation(&registry, spec, &cancel)
            .unwrap();
        assert_eq!(actual.batch(), expected.batch());
    }
    let tuple = Arc::new(value::Tuple {
        plan: tuple,
        layout,
        identity: Arc::new(()),
    });
    let mut children = Vec::new();
    let mut arguments = Vec::new();
    for name in ["values", "empty"] {
        value::bind(
            Value {
                relation: plans[name].clone(),
                tuple: Some((Arc::clone(&tuple), name.to_owned())),
            },
            Some(name.to_owned()),
            &mut children,
            &mut arguments,
        );
    }
    assert_eq!(
        children.len(),
        1,
        "two results share one actual producing child"
    );
    assert_eq!(arguments[0].child, arguments[1].child);
}

#[derive(Debug)]
struct CopyValues {
    spec: pse_schema::model::AlgorithmSpec,
    calls: Arc<std::sync::atomic::AtomicUsize>,
    duplicate: bool,
}
impl Algorithm for CopyValues {
    fn spec(&self) -> &pse_schema::model::AlgorithmSpec {
        &self.spec
    }
    fn requires_physical(&self) -> bool {
        false
    }
    fn run<'a>(
        &'a self,
        ctx: &'a crate::AlgorithmContext<'a>,
        inputs: &'a crate::AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, std::result::Result<crate::AlgorithmOutput, CompilerError>>
    {
        Box::pin(async move {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let source = inputs
                .port("values")
                .unwrap()
                .as_ref()
                .unwrap()
                .relation()?
                .checked();
            let result = if self.duplicate {
                FieldCheckedBatch::concat_reserved(
                    ctx.registry,
                    elements::spec(ctx.registry)?,
                    &[source.clone(), source.clone()],
                    ctx.reserver,
                    ctx.cancel,
                )?
            } else {
                source.clone()
            };
            Ok(crate::AlgorithmOutput {
                outputs: BTreeMap::from([("values".into(), result)]),
                findings: vec![],
                derivations: vec![],
                plans: vec![],
            })
        })
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "native_requirements_gate_actual_arguments_and_results keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
async fn native_requirements_gate_actual_arguments_and_results() {
    use pse_relations::generated::authored::documents;
    use pse_schema::model::ArgumentSpec;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let mut declarations = pse_schema::RegistryBuilder::new();
    pse_schema::catalog::declare(&mut declarations);
    declarations.declare_algorithm(
        pse_schema::model::AlgorithmDecl::new(
            "CopyValues",
            "1",
            pse_schema::model::Determinism::Deterministic,
        )
        .inputs(vec![ArgumentSpec {
            consumption: pse_schema::model::algorithm::InputConsumption::Whole,
            port: "values".into(),
            relation: "reference.elements".into(),
            required: true,
        }])
        .outputs(vec![ResultSpec {
            port: "values".into(),
            relation: "reference.elements".into(),
        }])
        .conditions(
            vec!["reference.elements:unique:pk".into()],
            vec!["reference.elements:unique:pk".into()],
        ),
    );
    let registry = Arc::new(declarations.build().unwrap());
    let cancel = CancellationToken::new();
    for (duplicate_input, duplicate_output, expected_calls) in
        [(true, false, 0), (false, true, 1), (false, false, 1)]
    {
        let mut builder = elements::Row::builder(&registry, 2).unwrap();
        let row = elements::Row {
            element_id: SemanticId::from_bytes([7; 16]),
            symbol: "C".into(),
            name: "carbon".into(),
            atomic_mass: 12.011,
        };
        elements::Row::push(&mut builder, row.clone()).unwrap();
        if duplicate_input {
            elements::Row::push(&mut builder, row).unwrap();
        }
        let elements = elements::Row::finish(builder).unwrap();
        let documents =
            FieldCheckedBatch::concat(&registry, documents::spec(&registry).unwrap(), &[]).unwrap();
        let rows = BTreeMap::from([
            (elements::RELATION_KEY, elements),
            (documents::RELATION_KEY, documents),
        ]);
        let (session, _) = crate::passes::native_test::session(
            &registry,
            rows,
            &FixedBudget::new(128 << 20),
            &cancel,
        )
        .unwrap();
        let value = |key| {
            let name = session.table_reference(&key).unwrap();
            Value::from_relation(
                session
                    .relation_plan(&name.resolve("workspace", "reference"))
                    .unwrap(),
            )
        };
        let spec = registry.algorithm("CopyValues@1").unwrap().clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let algorithm = Arc::new(CopyValues {
            spec,
            calls: Arc::clone(&calls),
            duplicate: duplicate_output,
        });
        let result = plan(
            algorithm,
            BTreeMap::from([("values".into(), Some(value(elements::RELATION_KEY)))]),
            value(documents::RELATION_KEY),
            &session,
            &cancel,
        )
        .await
        .unwrap();
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "planning must not execute the finite algorithm"
        );
        let completed = session
            .prepare_rule_plan(result["values"].relation().plan().clone(), &cancel)
            .unwrap()
            .execute(&cancel)
            .await;
        assert_eq!(calls.load(Ordering::SeqCst), expected_calls);
        if duplicate_input || duplicate_output {
            assert!(completed.is_err());
        } else {
            assert_eq!(
                completed
                    .unwrap()
                    .batches()
                    .iter()
                    .map(|b| b.num_rows())
                    .sum::<usize>(),
                1
            );
            assert_artifact_siblings_share_one_invocation(
                &session,
                &result["values"],
                &calls,
                &cancel,
            )
            .await;
        }
    }
}

async fn assert_artifact_siblings_share_one_invocation(
    session: &SnapshotSession,
    value: &Value,
    calls: &std::sync::atomic::AtomicUsize,
    cancel: &CancellationToken,
) {
    use pse_catalog::artifact::{ArtifactPlan, RelationOutput};
    let output = value.relation();
    let artifact = ArtifactPlan::new(
        session.clone(),
        ["first", "second"]
            .into_iter()
            .map(|name| {
                (
                    TableReference::full("test", "outputs", name).resolve("test", "outputs"),
                    RelationOutput {
                        relation_id: output.relation_id(),
                        plan: output.plan().clone(),
                    },
                )
            })
            .collect(),
        cancel,
    )
    .unwrap();
    for expected in [2, 3] {
        let completed = artifact.execute_outputs(cancel).await.unwrap();
        assert_eq!(completed.len(), 2);
        let planning_work: Vec<_> = completed
            .values()
            .map(|value| value.observation().rules_fired().len())
            .collect();
        assert!(
            planning_work[0] > planning_work[1],
            "shared producers run native analysis and optimization once per invocation"
        );
        assert!(completed.values().all(|value| {
            value
                .batches()
                .iter()
                .map(|batch| batch.num_rows())
                .sum::<usize>()
                == 1
        }));
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::SeqCst),
            expected,
            "sibling outputs share only this invocation's actual cache node"
        );
    }
}
