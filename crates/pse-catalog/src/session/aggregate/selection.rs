// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! MIN/MAX select an actual input value, so its field meaning follows that input.

use datafusion::{
    arrow::datatypes::{DataType, FieldRef},
    common::{DataFusionError, Result},
    functions_aggregate::min_max::{max_udaf, min_udaf},
    logical_expr::{
        Accumulator, AggregateUDF, AggregateUDFImpl, Expr, GroupsAccumulator, ReversedUDAF,
        Signature,
        function::{AccumulatorArgs, StateFieldsArgs},
        utils::AggregateOrderSensitivity,
    },
};
use std::sync::{Arc, LazyLock};

static MIN: LazyLock<Arc<AggregateUDF>> = LazyLock::new(|| selection("pse_min", min_udaf()));
static MAX: LazyLock<Arc<AggregateUDF>> = LazyLock::new(|| selection("pse_max", max_udaf()));

fn selection(name: &'static str, native: Arc<AggregateUDF>) -> Arc<AggregateUDF> {
    Arc::new(AggregateUDF::from(Selection { name, native }))
}

pub(super) fn functions() -> [Arc<AggregateUDF>; 2] {
    [Arc::clone(&MIN), Arc::clone(&MAX)]
}

/// Native MIN retaining the actual selected argument's semantic field.
pub fn min(value: Expr) -> Expr {
    MIN.call(vec![value])
}

/// Native MAX retaining the actual selected argument's semantic field.
pub fn max(value: Expr) -> Expr {
    MAX.call(vec![value])
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Selection {
    name: &'static str,
    native: Arc<AggregateUDF>,
}

impl AggregateUDFImpl for Selection {
    fn name(&self) -> &str {
        self.name
    }
    fn signature(&self) -> &Signature {
        self.native.signature()
    }
    fn coerce_types(&self, args: &[DataType]) -> Result<Vec<DataType>> {
        self.native.coerce_types(args)
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        self.native.return_type(args)
    }
    fn return_field(&self, args: &[FieldRef]) -> Result<FieldRef> {
        let [source] = args else {
            return Err(DataFusionError::Plan(format!(
                "{} requires one actual input field",
                self.name
            )));
        };
        let native = self.native.return_field(args)?;
        if native.data_type() != source.data_type() {
            return Err(DataFusionError::Plan(format!(
                "{} changed the selected input storage",
                self.name
            )));
        }
        // Empty/all-null groups produce NULL even for a nonnull input field.
        Ok(Arc::new(
            source
                .as_ref()
                .clone()
                .with_name(self.name)
                .with_nullable(true),
        ))
    }
    fn state_fields(&self, args: StateFieldsArgs<'_>) -> Result<Vec<FieldRef>> {
        self.native.state_fields(args)
    }
    fn accumulator(&self, args: AccumulatorArgs<'_>) -> Result<Box<dyn Accumulator>> {
        self.native.accumulator(args)
    }
    fn groups_accumulator_supported(&self, args: AccumulatorArgs<'_>) -> bool {
        self.native.groups_accumulator_supported(args)
    }
    fn create_groups_accumulator(
        &self,
        args: AccumulatorArgs<'_>,
    ) -> Result<Box<dyn GroupsAccumulator>> {
        self.native.create_groups_accumulator(args)
    }
    fn create_sliding_accumulator(
        &self,
        args: AccumulatorArgs<'_>,
    ) -> Result<Box<dyn Accumulator>> {
        self.native.create_sliding_accumulator(args)
    }
    fn order_sensitivity(&self) -> AggregateOrderSensitivity {
        self.native.order_sensitivity()
    }
    fn reverse_expr(&self) -> ReversedUDAF {
        ReversedUDAF::Identical
    }
    fn supports_null_handling_clause(&self) -> bool {
        self.native.supports_null_handling_clause()
    }
}
