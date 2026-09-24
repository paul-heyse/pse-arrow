// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! MIN/MAX select an actual input value, so its field meaning follows that input.

use datafusion::{
    arrow::datatypes::{DataType, FieldRef},
    common::{DataFusionError, Result},
    functions_aggregate::min_max::{max_udaf, min_udaf},
    logical_expr::{
        Accumulator, AggregateUDF, AggregateUDFImpl, GroupsAccumulator, ReversedUDAF, Signature,
        function::{AccumulatorArgs, StateFieldsArgs},
    },
};
use std::sync::Arc;

fn selection(name: &'static str, native: Arc<AggregateUDF>) -> Arc<AggregateUDF> {
    Arc::new(AggregateUDF::from(Selection {
        name,
        origin: native.clone(),
        native,
    }))
}

pub(super) fn adapt(function: &Arc<AggregateUDF>) -> Option<Arc<AggregateUDF>> {
    if function.as_ref() == min_udaf().as_ref() {
        Some(selection("min", Arc::clone(function)))
    } else if function.as_ref() == max_udaf().as_ref() {
        Some(selection("max", Arc::clone(function)))
    } else {
        None
    }
}
pub(super) fn native(function: &AggregateUDF) -> Option<&Arc<AggregateUDF>> {
    function
        .inner()
        .downcast_ref::<Selection>()
        .map(|value| &value.origin)
}

crate::session::native_hooks::native_identity!(Selection);

#[derive(Debug)]
struct Selection {
    origin: Arc<AggregateUDF>,
    name: &'static str,
    native: Arc<AggregateUDF>,
}

impl AggregateUDFImpl for Selection {
    crate::session::native_hooks::aggregate_hooks!();

    fn name(&self) -> &str {
        self.name
    }
    fn signature(&self) -> &Signature {
        self.native.signature()
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
    fn with_beneficial_ordering(
        self: Arc<Self>,
        beneficial: bool,
    ) -> Result<Option<Arc<dyn AggregateUDFImpl>>> {
        Ok(self
            .native
            .as_ref()
            .clone()
            .with_beneficial_ordering(beneficial)?
            .map(|native| {
                let adapted: Arc<dyn AggregateUDFImpl> = Arc::new(Self {
                    name: self.name,
                    origin: self.origin.clone(),
                    native: Arc::new(native),
                });
                adapted
            }))
    }
    fn reverse_expr(&self) -> ReversedUDAF {
        match self.native.inner().reverse_expr() {
            ReversedUDAF::Reversed(native) => {
                ReversedUDAF::Reversed(Arc::new(AggregateUDF::from(Self {
                    name: self.name,
                    origin: self.origin.clone(),
                    native,
                })))
            }
            other => other,
        }
    }
}
