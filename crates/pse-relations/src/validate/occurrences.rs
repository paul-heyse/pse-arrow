// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded native occurrence selection. Kernels see visible values only, before evaluation.
use super::{
    RelationError,
    prepared::{Findings, engine},
};
use crate::native::{
    arrow::{
        array::{
            Array, ArrayRef, RecordBatch, UInt64Array,
            cast::AsArray,
            types::{Int16Type, Int32Type, Int64Type},
        },
        compute::{TakeOptions, take},
        datatypes::{DataType, FieldRef, Schema},
    },
    common::{Column, DFSchema},
    logical_expr::Expr,
    physical_expr::PhysicalExpr,
};
use pse_schema::{Registry, model::TaggedAlternative};
use std::sync::Arc;

pub(super) const CHUNK: usize = 4096;
#[derive(Debug)]
pub(super) struct Node {
    field: FieldRef,
    path_name: Arc<str>,
    schema: Arc<Schema>,
    predicate: Option<Arc<dyn PhysicalExpr>>,
    children: Vec<Self>,
    alternative: Option<TaggedAlternative>,
}
#[derive(Clone)]
pub(super) struct Location {
    pub row: usize,
    pub path: Arc<PathCoordinate>,
}
// Persistent compact coordinates keep only active traversal paths alive. No
// successful value renders a string; field components are escaped at preparation.
pub(super) struct PathCoordinate {
    parent: Option<Arc<Self>>,
    segment: PathSegment,
}
enum PathSegment {
    Field(Arc<str>),
    Index(usize),
}
impl PathCoordinate {
    fn field(parent: Option<Arc<Self>>, name: Arc<str>) -> Arc<Self> {
        Arc::new(Self {
            parent,
            segment: PathSegment::Field(name),
        })
    }
    fn index(parent: Arc<Self>, index: usize) -> Arc<Self> {
        Arc::new(Self {
            parent: Some(parent),
            segment: PathSegment::Index(index),
        })
    }
    pub(super) fn render(&self) -> String {
        let mut parts = vec![&self.segment];
        let mut current = self.parent.as_deref();
        while let Some(parent) = current {
            parts.push(&parent.segment);
            current = parent.parent.as_deref();
        }
        let mut result = String::new();
        for part in parts.into_iter().rev() {
            result.push('/');
            match part {
                PathSegment::Field(name) => result.push_str(name),
                PathSegment::Index(index) => result.push_str(&index.to_string()),
            }
        }
        result
    }
}
impl Node {
    pub(super) fn prepare(
        registry: &Registry,
        state: &dyn super::planner::ValidationPlanner,
        field: FieldRef,
    ) -> Result<Self, RelationError> {
        let children = match field.data_type() {
            DataType::Dictionary(_, value) => vec![Arc::new(
                field
                    .as_ref()
                    .clone()
                    .with_data_type(value.as_ref().clone()),
            )],
            DataType::RunEndEncoded(_, child)
            | DataType::List(child)
            | DataType::LargeList(child)
            | DataType::ListView(child)
            | DataType::LargeListView(child)
            | DataType::FixedSizeList(child, _)
            | DataType::Map(child, _) => vec![Arc::clone(child)],
            DataType::Struct(fields) => fields.to_vec(),
            DataType::Union(fields, _) => {
                fields.iter().map(|(_, field)| Arc::clone(field)).collect()
            }
            _ => vec![],
        }
        .into_iter()
        .map(|child| Self::prepare(registry, state, child))
        .collect::<Result<_, _>>()?;
        let predicate = if matches!(
            field.data_type(),
            DataType::Dictionary(..)
                | DataType::RunEndEncoded(..)
                | DataType::Union(..)
                | DataType::Null
        ) {
            None
        } else {
            let schema =
                DFSchema::try_from(Schema::new(vec![Arc::clone(&field)])).map_err(engine)?;
            let expression = super::predicates::field_local(
                registry,
                &field,
                Expr::Column(Column::from_name(field.name())),
            )
            .map_err(engine)?;
            Some(super::prepared::prepare_expression(
                state, expression, &schema,
            )?)
        };
        let alternative = TaggedAlternative::from_field(&field)?;
        Ok(Self {
            path_name: pointer(field.name()).into(),
            schema: Arc::new(Schema::new(vec![field.clone()])),
            field,
            predicate,
            children,
            alternative,
        })
    }
    pub(super) fn root(
        &self,
        array: &ArrayRef,
        start: usize,
        findings: &mut Findings<'_>,
    ) -> Result<(), RelationError> {
        let path = PathCoordinate::field(None, self.path_name.clone());
        let locations = (start..start + array.len())
            .map(|row| Location {
                row,
                path: path.clone(),
            })
            .collect();
        self.visit(array, locations, findings)
    }
    #[allow(
        clippy::too_many_lines,
        reason = "one bounded native container dispatch"
    )]
    fn visit(
        &self,
        array: &ArrayRef,
        locations: Vec<Location>,
        findings: &mut Findings<'_>,
    ) -> Result<(), RelationError> {
        findings.cancel.checkpoint()?;
        let nulls = array.logical_nulls();
        let mut indices = Vec::with_capacity(array.len());
        let mut visible = Vec::with_capacity(array.len());
        for (index, location) in locations.into_iter().enumerate() {
            if nulls.as_ref().is_some_and(|nulls| nulls.is_null(index)) {
                if !self.field.is_nullable() {
                    findings.add("nonnull", &self.field, array, index, &location)?;
                }
            } else {
                indices.push(index);
                visible.push(location);
            }
        }
        if indices.is_empty() {
            return Ok(());
        }
        // Encoded containers are resolved through native child selection, not scalar decoding.
        match self.field.data_type() {
            DataType::Dictionary(_, value) => {
                let decoded = arrow::compute::cast(&array, value)?;
                return self.children[0].visit(&gather(&decoded, &indices)?, visible, findings);
            }
            DataType::RunEndEncoded(runs, _) => {
                macro_rules! run {
                    ($ty:ty) => {{
                        let run = array
                            .as_run_opt::<$ty>()
                            .ok_or_else(|| invalid("invalid run-end storage"))?;
                        let selected = indices
                            .iter()
                            .map(|index| run.get_physical_index(*index))
                            .collect::<Vec<_>>();
                        return self.children[0].visit(
                            &gather(run.values(), &selected)?,
                            visible,
                            findings,
                        );
                    }};
                }
                match runs.data_type() {
                    DataType::Int16 => run!(Int16Type),
                    DataType::Int32 => run!(Int32Type),
                    DataType::Int64 => run!(Int64Type),
                    _ => return Err(invalid("invalid run-end type")),
                }
            }
            DataType::Union(fields, _) => {
                let union = array
                    .as_union_opt()
                    .ok_or_else(|| invalid("invalid union storage"))?;
                for ((id, _), child) in fields.iter().zip(&self.children) {
                    let mut offsets = Vec::new();
                    let mut places = Vec::new();
                    for (&index, place) in indices.iter().zip(&visible) {
                        if union.type_id(index) == id {
                            offsets.push(union.value_offset(index));
                            places.push(Location {
                                row: place.row,
                                path: PathCoordinate::field(
                                    Some(place.path.clone()),
                                    child.path_name.clone(),
                                ),
                            });
                        }
                    }
                    if !offsets.is_empty() {
                        child.visit(&gather(union.child(id), &offsets)?, places, findings)?;
                    }
                }
                return Ok(());
            }
            _ => {}
        }
        let array = gather(array, &indices)?;
        if let Some(predicate) = &self.predicate {
            let schema = Arc::clone(&self.schema);
            let batch = RecordBatch::try_new(schema, vec![Arc::clone(&array)])?;
            let values = predicate
                .evaluate(&batch)
                .and_then(|value| value.into_array(array.len()))
                .map_err(engine)?;
            let flags = values
                .as_boolean_opt()
                .ok_or_else(|| invalid("local predicate is not Boolean"))?;
            for (index, place) in visible.iter().enumerate() {
                if flags.is_null(index) || !flags.value(index) {
                    findings.add("field-domain", &self.field, &array, index, place)?;
                }
            }
        }
        match self.field.data_type() {
            DataType::Struct(_) => {
                let structure = array
                    .as_struct_opt()
                    .ok_or_else(|| invalid("invalid struct storage"))?;
                let tags = self
                    .alternative
                    .as_ref()
                    .map(|alternative| {
                        let values = structure
                            .column_by_name(&alternative.discriminator)
                            .ok_or_else(|| invalid("alternative discriminator absent"))?;
                        arrow::compute::cast(values, &DataType::Utf8).map_err(RelationError::from)
                    })
                    .transpose()?;
                for (child, values) in self.children.iter().zip(structure.columns()) {
                    let mut indices = Vec::new();
                    let mut places = Vec::new();
                    for (index, place) in visible.iter().enumerate() {
                        if let (Some(alternative), Some(tags)) = (&self.alternative, &tags)
                            && alternative.payloads().contains(child.field.name().as_str())
                        {
                            let tags = tags.as_string::<i32>();
                            if tags.is_null(index)
                                || alternative
                                    .arms
                                    .get(tags.value(index))
                                    .and_then(Option::as_deref)
                                    != Some(child.field.name())
                            {
                                continue;
                            }
                        }
                        indices.push(index);
                        places.push(Location {
                            row: place.row,
                            path: PathCoordinate::field(
                                Some(place.path.clone()),
                                child.path_name.clone(),
                            ),
                        });
                    }
                    if !indices.is_empty() {
                        child.visit(&gather(values, &indices)?, places, findings)?;
                    }
                }
            }
            DataType::List(_)
            | DataType::LargeList(_)
            | DataType::ListView(_)
            | DataType::LargeListView(_)
            | DataType::FixedSizeList(..)
            | DataType::Map(..) => {
                let (values, ranges) = ranges(&array)?;
                let mut indices = Vec::with_capacity(CHUNK);
                let mut places = Vec::with_capacity(CHUNK);
                for (place, range) in visible.iter().zip(ranges) {
                    for (ordinal, index) in range.enumerate() {
                        indices.push(index);
                        places.push(Location {
                            row: place.row,
                            path: PathCoordinate::index(place.path.clone(), ordinal),
                        });
                        if indices.len() == CHUNK {
                            self.children[0].visit(
                                &gather(&values, &indices)?,
                                std::mem::take(&mut places),
                                findings,
                            )?;
                            indices.clear();
                        }
                    }
                }
                if !indices.is_empty() {
                    self.children[0].visit(&gather(&values, &indices)?, places, findings)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
fn gather(array: &ArrayRef, indices: &[usize]) -> Result<ArrayRef, RelationError> {
    if indices.len() == array.len() && indices.iter().copied().eq(0..array.len()) {
        return Ok(Arc::clone(array));
    }
    let indices = indices
        .iter()
        .map(|index| u64::try_from(*index).map_err(|_| invalid("occurrence index overflow")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(take(
        array.as_ref(),
        &UInt64Array::from(indices),
        Some(TakeOptions { check_bounds: true }),
    )?)
}
fn ranges(array: &ArrayRef) -> Result<(ArrayRef, Vec<std::ops::Range<usize>>), RelationError> {
    macro_rules! offsets {
        ($array:expr) => {{
            let array = $array;
            let ranges = array
                .value_offsets()
                .windows(2)
                .map(|pair| Ok(index(pair[0])?..index(pair[1])?))
                .collect::<Result<_, RelationError>>()?;
            (Arc::clone(array.values()), ranges)
        }};
    }
    macro_rules! views {
        ($array:expr) => {{
            let array = $array;
            let ranges = array
                .value_offsets()
                .iter()
                .zip(array.value_sizes())
                .map(|(start, size)| {
                    let start = index(*start)?;
                    Ok(start
                        ..start
                            .checked_add(index(*size)?)
                            .ok_or_else(|| invalid("list offset overflow"))?)
                })
                .collect::<Result<_, RelationError>>()?;
            (Arc::clone(array.values()), ranges)
        }};
    }
    Ok(match array.data_type() {
        DataType::List(_) => offsets!(array.as_list::<i32>()),
        DataType::LargeList(_) => offsets!(array.as_list::<i64>()),
        DataType::ListView(_) => views!(array.as_list_view::<i32>()),
        DataType::LargeListView(_) => views!(array.as_list_view::<i64>()),
        DataType::FixedSizeList(_, width) => {
            let values = array.as_fixed_size_list();
            let width = index(*width)?;
            (
                Arc::clone(values.values()),
                (0..array.len())
                    .map(|row| {
                        let start = index(values.value_offset(row))?;
                        Ok(start..start + width)
                    })
                    .collect::<Result<_, RelationError>>()?,
            )
        }
        DataType::Map(..) => {
            let values = array.as_map();
            let entries: ArrayRef = Arc::new(values.entries().clone());
            (
                entries,
                values
                    .value_offsets()
                    .windows(2)
                    .map(|pair| Ok(index(pair[0])?..index(pair[1])?))
                    .collect::<Result<_, RelationError>>()?,
            )
        }
        _ => return Err(invalid("occurrence is not a list or map")),
    })
}
fn index(value: impl TryInto<usize>) -> Result<usize, RelationError> {
    value
        .try_into()
        .map_err(|_| invalid("negative or oversized occurrence offset"))
}
fn pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
fn invalid(reason: &str) -> RelationError {
    super::mismatch("native occurrence", reason)
}
