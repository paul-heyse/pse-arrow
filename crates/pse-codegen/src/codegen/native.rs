// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One native field traversal; target languages own rendering and boundary policy.
use crate::{SchemaError, model::FieldContract};
use arrow_schema::{DataType, Field};

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Domain,
    Storage,
}
pub(super) trait Policy {
    type Value;
    fn child_stem(&self, field: &FieldContract, stem: &str, child: &Field) -> String {
        let suffix = match field.field().data_type() {
            DataType::List(_)
            | DataType::LargeList(_)
            | DataType::ListView(_)
            | DataType::LargeListView(_)
            | DataType::FixedSizeList(..) => "Item".into(),
            _ => super::rust::types::pascal(child.name()),
        };
        format!("{stem}{suffix}")
    }
    fn leaf(
        &mut self,
        field: &FieldContract,
        stem: &str,
        mode: Mode,
    ) -> Result<Option<Self::Value>, SchemaError>;
    fn container(
        &mut self,
        field: &FieldContract,
        stem: &str,
        children: Vec<Self::Value>,
    ) -> Result<Self::Value, SchemaError>;
    fn nullable(&mut self, value: Self::Value, nullable: bool) -> Self::Value;
    fn decorate(
        &mut self,
        _field: &FieldContract,
        value: Self::Value,
    ) -> Result<Self::Value, SchemaError> {
        Ok(value)
    }
}
pub(super) fn render<P: Policy>(
    policy: &mut P,
    field: &FieldContract,
    stem: &str,
    mode: Mode,
) -> Result<P::Value, SchemaError> {
    let child_mode = if field.extension().is_some() {
        Mode::Storage
    } else {
        mode
    };
    let value = if let Some(value) = policy.leaf(field, stem, mode)? {
        value
    } else {
        let children = pse_columnar::native_field::children(field.field().data_type())
            .into_iter()
            .map(|child| {
                let child_stem = policy.child_stem(field, stem, child);
                let value = render(
                    policy,
                    &FieldContract::from_field(child.clone()),
                    &child_stem,
                    child_mode,
                )?;
                Ok(policy.nullable(value, child.is_nullable()))
            })
            .collect::<Result<Vec<_>, SchemaError>>()?;
        policy.container(field, stem, children)?
    };
    policy.decorate(field, value)
}
pub(super) fn storage<P: Policy>(
    policy: &mut P,
    kind: &DataType,
    stem: &str,
) -> Result<P::Value, SchemaError> {
    render(
        policy,
        &FieldContract::from_field(Field::new(stem, kind.clone(), false)),
        stem,
        Mode::Storage,
    )
}
