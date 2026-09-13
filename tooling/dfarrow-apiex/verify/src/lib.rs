//! Compile gate for V1 — Extension-Point Contracts (DataFusion 55.0.0).
//!
//! Each module implements one extension point using exactly the required-method
//! signatures published in `dfarrow55_extension_point_contracts_2026-09-13.md`.
//! A published signature that does not match the pinned source breaks this build.
#![allow(dead_code, unused_variables)]

// `Any` is intentionally NOT imported: `as_any` is not a member of these
// traits in 55.0.0, though it is in most tutorials and in the narrative corpus.
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion_common::{Result, Statistics};

// --- 1) TableProvider -------------------------------------------------------
mod table_provider {
    use super::*;
    use datafusion_expr::{Expr, TableType};
    use datafusion_physical_plan::ExecutionPlan;
    use datafusion_session::{Session, TableProvider};

    #[derive(Debug)]
    pub struct MyTable {
        schema: SchemaRef,
    }

    #[async_trait::async_trait]
    impl TableProvider for MyTable {
        fn schema(&self) -> SchemaRef {
            Arc::clone(&self.schema)
        }
        fn table_type(&self) -> TableType {
            TableType::Base
        }
        async fn scan(
            &self,
            state: &dyn Session,
            projection: Option<&Vec<usize>>,
            filters: &[Expr],
            limit: Option<usize>,
        ) -> Result<Arc<dyn ExecutionPlan>> {
            unimplemented!("compile gate only")
        }
    }
}

// --- 2) ScalarUDFImpl -------------------------------------------------------
mod scalar_udf {
    use super::*;
    use datafusion_expr::{
        ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
    };

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct MyScalar {
        signature: Signature,
    }

    impl ScalarUDFImpl for MyScalar {
        fn name(&self) -> &str {
            "my_scalar"
        }
        fn signature(&self) -> &Signature {
            &self.signature
        }
        fn return_type(&self, arg_types: &[DataType]) -> Result<DataType> {
            Ok(DataType::Int64)
        }
        fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
            unimplemented!("compile gate only")
        }
    }

    // Volatility is a registration-time *value*, invisible to static extraction;
    // this pins that the variants named in the registry catalog exist.
    const _: [Volatility; 3] = [Volatility::Immutable, Volatility::Stable, Volatility::Volatile];
}

// --- 3) AggregateUDFImpl ----------------------------------------------------
mod aggregate_udf {
    use super::*;
    use datafusion_expr::function::AccumulatorArgs;
    use datafusion_expr::{Accumulator, AggregateUDFImpl, Signature};

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct MyAgg {
        signature: Signature,
    }

    impl AggregateUDFImpl for MyAgg {
        fn name(&self) -> &str {
            "my_agg"
        }
        fn signature(&self) -> &Signature {
            &self.signature
        }
        fn return_type(&self, arg_types: &[DataType]) -> Result<DataType> {
            Ok(DataType::Int64)
        }
        fn accumulator(&self, acc_args: AccumulatorArgs<'_>) -> Result<Box<dyn Accumulator>> {
            unimplemented!("compile gate only")
        }
    }
}

// --- 4) OptimizerRule -------------------------------------------------------
mod optimizer_rule {
    use super::*;
    use datafusion_common::tree_node::Transformed;
    use datafusion_expr::LogicalPlan;
    use datafusion_optimizer::{OptimizerConfig, OptimizerRule};

    #[derive(Debug, Default)]
    pub struct MyRule;

    impl OptimizerRule for MyRule {
        fn name(&self) -> &str {
            "my_rule"
        }
        fn rewrite(
            &self,
            plan: LogicalPlan,
            config: &dyn OptimizerConfig,
        ) -> Result<Transformed<LogicalPlan>> {
            Ok(Transformed::no(plan))
        }
    }
}

// --- 5) CatalogProvider -----------------------------------------------------
mod catalog_provider {
    use super::*;
    use datafusion_session::{CatalogProvider, SchemaProvider};

    #[derive(Debug)]
    pub struct MyCatalog;

    impl CatalogProvider for MyCatalog {
        fn schema_names(&self) -> Vec<String> {
            vec![]
        }
        fn schema(&self, name: &str) -> Option<Arc<dyn SchemaProvider>> {
            None
        }
    }
}

// --- 6) ExecutionPlan -------------------------------------------------------
mod execution_plan {
    use super::*;
    use datafusion_common::tree_node::TreeNodeRecursion;
    use datafusion_execution::TaskContext;
    use datafusion_physical_expr::PhysicalExpr;
    use datafusion_physical_plan::{
        DisplayAs, DisplayFormatType, ExecutionPlan, PlanProperties, SendableRecordBatchStream,
    };

    #[derive(Debug)]
    pub struct MyExec {
        props: Arc<PlanProperties>,
    }

    impl DisplayAs for MyExec {
        fn fmt_as(&self, t: DisplayFormatType, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "MyExec")
        }
    }

    impl ExecutionPlan for MyExec {
        fn name(&self) -> &str {
            "MyExec"
        }
        fn properties(&self) -> &Arc<PlanProperties> {
            &self.props
        }
        // Required in 55.0.0 (added by the 55 planner work; see §40A).
        fn apply_expressions(
            &self,
            f: &mut dyn FnMut(&Arc<dyn PhysicalExpr>) -> Result<TreeNodeRecursion>,
        ) -> Result<TreeNodeRecursion> {
            Ok(TreeNodeRecursion::Continue)
        }
        fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
            vec![]
        }
        fn with_new_children(
            self: Arc<Self>,
            children: Vec<Arc<dyn ExecutionPlan>>,
        ) -> Result<Arc<dyn ExecutionPlan>> {
            Ok(self)
        }
        fn execute(
            &self,
            partition: usize,
            context: Arc<TaskContext>,
        ) -> Result<SendableRecordBatchStream> {
            unimplemented!("compile gate only")
        }
    }
}

/// Keeps the shared imports live so an accidental import-only regression is caught.
pub fn anchor() -> (SchemaRef, Statistics) {
    let schema: SchemaRef = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, true)]));
    let stats = Statistics::new_unknown(&schema);
    (schema, stats)
}
