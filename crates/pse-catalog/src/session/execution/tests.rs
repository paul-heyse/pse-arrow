// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Domain algorithms retain the actual caller and inherited policy after input rebinding.
use super::*;
use datafusion::{
    arrow::{array::RecordBatch, datatypes::DataType},
    common::ScalarValue,
    execution::{context::QueryPlanner, runtime_env::RuntimeEnv},
    logical_expr::{ColumnarValue, LogicalPlan, Volatility, create_udf},
    physical_plan::ExecutionPlan,
};
use std::sync::Mutex;

#[derive(Debug, Default)]
struct Capture(Mutex<Option<(Arc<NativeExecutionContext>, SessionState)>>);
#[async_trait::async_trait]
impl QueryPlanner for Capture {
    async fn create_physical_plan(
        &self,
        plan: &LogicalPlan,
        session: &dyn Session,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let services = NativeExecutionContext::from_session(session)?;
        let state = session
            .as_any()
            .downcast_ref::<SessionState>()
            .unwrap()
            .clone();
        *self.0.lock().unwrap() = Some((services, state));
        crate::session::planner::UnifiedPlanner::default()
            .create_physical_plan(plan, session)
            .await
    }
}

fn session(capture: Arc<Capture>, cancel: &CancellationToken) -> SnapshotSession {
    use crate::session::{ExecutionSettings, ThreadBudget};
    let original = create_udf(
        "actual_caller_function",
        vec![],
        DataType::Int64,
        Volatility::Volatile,
        Arc::new(|_| Ok(ColumnarValue::Scalar(ScalarValue::Int64(Some(731))))),
    );
    let builder = SessionStateBuilder::new_with_default_features()
        .with_query_planner(capture)
        .with_scalar_functions(vec![Arc::new(original)]);
    let mut policy = ProviderPolicy::new(
        pse_ids::SemanticId::NIL,
        ProviderScope::Schema("model".into(), "authored".into()),
    );
    policy
        .effects
        .remove(&pse_schema::model::provider::OperationEffect::Namespace);
    policy.max_bytes = Some(32 << 20);
    policy
        .required_settings
        .insert("datafusion.execution.batch_size".into(), "7".into());
    let factory = SessionFactory::from_builder(
        Arc::new(RuntimeEnv::default()),
        pse_ids::FixedBudget::new(128 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        "actual-caller",
        builder,
    )
    .unwrap()
    .with_policies(vec![policy])
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let spec = registry.relation("authored.packages").unwrap();
    let batch = RecordBatch::new_empty(Arc::new(
        pse_schema::arrow::relation_schema(&registry, spec).unwrap(),
    ));
    factory
        .candidate(BTreeMap::from([(spec.key, batch)]), registry, cancel)
        .unwrap()
        .with_purpose(OperationPurpose::Publish)
}

#[tokio::test]
async fn native_algorithms_keep_functions_configuration_scopes_and_resource_ceiling() {
    assert!(
        NativeExecutionContext::from_session(
            &SessionStateBuilder::new_with_default_features().build()
        )
        .is_err()
    );
    let cancel = CancellationToken::new();
    let capture = Arc::new(Capture::default());
    let outer = session(Arc::clone(&capture), &cancel);
    outer
        .sql("SELECT actual_caller_function()", &cancel)
        .await
        .unwrap();
    let (services, state) = capture.0.lock().unwrap().take().unwrap();
    assert!(
        services
            .admit_effects(&BTreeSet::from([
                pse_schema::model::provider::OperationEffect::Namespace,
            ]))
            .is_err()
    );
    services.require_settlement();
    assert!(services.must_settle());
    let next = NativeExecutionContext::execution_state(&state, &cancel);
    let next_services = NativeExecutionContext::from_session(&next).unwrap();
    assert!(!next_services.must_settle());
    assert!(!Arc::ptr_eq(&services, &next_services));
    assert!(Arc::ptr_eq(services.reserver(), next_services.reserver()));
    assert_eq!(state.config_options().execution.batch_size.get(), 7);
    let key = services
        .registry()
        .relation("authored.packages")
        .unwrap()
        .key;
    let child = outer.checked_input(&key).unwrap();
    let nested = services
        .candidate_roles(&state, BTreeMap::from([("actual_child".into(), child)]))
        .unwrap();
    assert_eq!(nested.effective_policy().unwrap().max_bytes, Some(32 << 20));
    assert!(
        services
            .reserver()
            .open("too-large-native-workspace")
            .try_grow(33 << 20)
            .is_err()
    );
    let values = nested
        .sql("SELECT actual_caller_function()", &cancel)
        .await
        .unwrap();
    assert_eq!(
        ScalarValue::try_from_array(values[0].column(0), 0).unwrap(),
        ScalarValue::Int64(Some(731))
    );
    let refused = nested
        .prepare_sql("CREATE TABLE workspace.other (id BIGINT)", &cancel)
        .await
        .unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("outside the effective purpose/policy"),
        "{refused}"
    );
    let foreign = SessionStateBuilder::new_with_default_features().build();
    assert!(services.candidate_roles(&foreign, BTreeMap::new()).is_err());
    cancel.cancel();
    assert!(services.candidate_roles(&state, BTreeMap::new()).is_err());
    capture.0.lock().unwrap().take();
}
