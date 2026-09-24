// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native accumulators retain their algorithms, state protocols and size accounting.
use super::{restore, restore_scalar};
use datafusion::{
    arrow::{
        array::{ArrayRef, BooleanArray},
        datatypes::FieldRef,
    },
    common::{Result, ScalarValue},
    logical_expr::{Accumulator, EmitTo, GroupsAccumulator},
};

#[derive(Debug)]
pub(super) struct Single {
    pub(super) native: Box<dyn Accumulator>,
    pub(super) field: FieldRef,
}
impl Accumulator for Single {
    fn update_batch(&mut self, values: &[ArrayRef]) -> Result<()> {
        self.native.update_batch(values)
    }
    fn evaluate(&mut self) -> Result<ScalarValue> {
        restore_scalar(self.native.evaluate()?, &self.field)
    }
    fn size(&self) -> usize {
        self.native.size() + size_of::<Self>()
    }
    fn state(&mut self) -> Result<Vec<ScalarValue>> {
        let mut states = self.native.state()?;
        let state = states
            .first_mut()
            .ok_or_else(|| super::invalid("native value state absent"))?;
        *state = restore_scalar(state.clone(), &self.field)?;
        Ok(states)
    }
    fn merge_batch(&mut self, states: &[ArrayRef]) -> Result<()> {
        self.native.merge_batch(states)
    }
    fn retract_batch(&mut self, values: &[ArrayRef]) -> Result<()> {
        self.native.retract_batch(values)
    }
    fn supports_retract_batch(&self) -> bool {
        self.native.supports_retract_batch()
    }
}

pub(super) struct Groups {
    pub(super) native: Box<dyn GroupsAccumulator>,
    pub(super) field: FieldRef,
}
impl GroupsAccumulator for Groups {
    fn update_batch(
        &mut self,
        values: &[ArrayRef],
        indices: &[usize],
        filter: Option<&BooleanArray>,
        count: usize,
    ) -> Result<()> {
        self.native.update_batch(values, indices, filter, count)
    }
    fn evaluate(&mut self, emit: EmitTo) -> Result<ArrayRef> {
        restore(&self.native.evaluate(emit)?, &self.field)
    }
    fn state(&mut self, emit: EmitTo) -> Result<Vec<ArrayRef>> {
        restore_states(self.native.state(emit)?, &self.field)
    }
    fn merge_batch(&mut self, values: &[ArrayRef], indices: &[usize], count: usize) -> Result<()> {
        self.native.merge_batch(values, indices, count)
    }
    fn convert_to_state(
        &self,
        values: &[ArrayRef],
        filter: Option<&BooleanArray>,
    ) -> Result<Vec<ArrayRef>> {
        restore_states(self.native.convert_to_state(values, filter)?, &self.field)
    }
    fn size(&self) -> usize {
        self.native.size() + size_of::<Self>()
    }
}
fn restore_states(mut states: Vec<ArrayRef>, field: &FieldRef) -> Result<Vec<ArrayRef>> {
    let state = states
        .first_mut()
        .ok_or_else(|| super::invalid("native value state absent"))?;
    *state = restore(state, field)?;
    Ok(states)
}
