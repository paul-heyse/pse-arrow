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
// Coordinates are constructed only for retained diagnostic samples.
// Field components are escaped once at preparation.
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
// Traversal stores bounded integer mappings, never one diagnostic path per value.
// Locations are materialized only for findings retained by the sample budget.
pub(super) enum Traversal {
    Root {
        start: usize,
        field: Arc<str>,
    },
    Range {
        parent: Arc<Self>,
        start: usize,
    },
    Select {
        parent: Arc<Self>,
        indices: Arc<[usize]>,
    },
    Field {
        parent: Arc<Self>,
        field: Arc<str>,
    },
    List {
        parent: Arc<Self>,
        entries: Vec<(usize, usize)>,
    },
}
impl Traversal {
    pub(super) fn row(&self, index: usize) -> usize {
        match self {
            Self::Root { start, .. } => start + index,
            Self::Range { parent, start } => parent.row(start + index),
            Self::Select { parent, indices } => parent.row(indices[index]),
            Self::Field { parent, .. } => parent.row(index),
            Self::List { parent, entries } => parent.row(entries[index].0),
        }
    }
    pub(super) fn location(&self, index: usize) -> Location {
        match self {
            Self::Root { start, field } => Location {
                row: start + index,
                path: PathCoordinate::field(None, field.clone()),
            },
            Self::Range { parent, start } => parent.location(start + index),
            Self::Select { parent, indices } => parent.location(indices[index]),
            Self::Field { parent, field } => {
                let mut location = parent.location(index);
                location.path = PathCoordinate::field(Some(location.path), field.clone());
                location
            }
            Self::List { parent, entries } => {
                let (row, ordinal) = entries[index];
                let mut location = parent.location(row);
                location.path = PathCoordinate::index(location.path, ordinal);
                location
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Selection {
    All,
    Range { start: usize, length: usize },
    Indices(Arc<[usize]>),
}
impl Selection {
    fn indices(indices: Vec<usize>, extent: usize) -> Self {
        if indices.len() == extent && indices.iter().copied().eq(0..extent) {
            Self::All
        } else if indices
            .windows(2)
            .all(|pair| pair[0].checked_add(1) == Some(pair[1]))
        {
            Self::Range {
                start: indices.first().copied().unwrap_or(0),
                length: indices.len(),
            }
        } else {
            Self::Indices(indices.into())
        }
    }
    fn apply(&self, array: &ArrayRef) -> Result<ArrayRef, RelationError> {
        match self {
            Self::All => Ok(array.clone()),
            Self::Range { start, length } => {
                if start
                    .checked_add(*length)
                    .is_none_or(|end| end > array.len())
                {
                    return Err(invalid("occurrence range outside array"));
                }
                Ok(array.slice(*start, *length))
            }
            Self::Indices(indices) => {
                let indices = indices
                    .iter()
                    .map(|index| {
                        u64::try_from(*index).map_err(|_| invalid("occurrence index overflow"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(take(
                    array.as_ref(),
                    &UInt64Array::from(indices),
                    Some(TakeOptions { check_bounds: true }),
                )?)
            }
        }
    }
    fn map(&self, parent: Arc<Traversal>) -> Arc<Traversal> {
        match self {
            Self::All => parent,
            Self::Range { start, .. } => Arc::new(Traversal::Range {
                parent,
                start: *start,
            }),
            Self::Indices(indices) => Arc::new(Traversal::Select {
                parent,
                indices: indices.clone(),
            }),
        }
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
        self.visit(
            array,
            Arc::new(Traversal::Root {
                start,
                field: self.path_name.clone(),
            }),
            findings,
        )
    }
    #[allow(
        clippy::too_many_lines,
        reason = "one bounded native container dispatch"
    )]
    fn visit(
        &self,
        array: &ArrayRef,
        traversal: Arc<Traversal>,
        findings: &mut Findings<'_>,
    ) -> Result<(), RelationError> {
        findings.cancel.checkpoint()?;
        let visibility = if let Some(nulls) = array
            .logical_nulls()
            .filter(|nulls| nulls.null_count() != 0)
        {
            let mut indices = Vec::with_capacity(array.len() - nulls.null_count());
            for index in 0..array.len() {
                if nulls.is_null(index) {
                    if !self.field.is_nullable() {
                        findings.add("nonnull", &self.field, array, index, &traversal)?;
                    }
                } else {
                    indices.push(index);
                }
            }
            Selection::indices(indices, array.len())
        } else {
            Selection::All
        };
        let traversal = visibility.map(traversal);
        let array = visibility.apply(array)?;
        if array.is_empty() {
            return Ok(());
        }
        // Encoded storage is selected through Arrow mappings, without scalar decoding.
        match self.field.data_type() {
            DataType::Dictionary(_, value) => {
                let decoded = arrow::compute::cast(&array, value)?;
                return self.children[0].visit(&decoded, traversal, findings);
            }
            DataType::RunEndEncoded(runs, _) => {
                macro_rules! run {
                    ($ty:ty) => {{
                        let run = array
                            .as_run_opt::<$ty>()
                            .ok_or_else(|| invalid("invalid run-end storage"))?;
                        let selected = (0..array.len())
                            .map(|index| run.get_physical_index(index))
                            .collect();
                        return self.children[0].visit(
                            &Selection::indices(selected, run.values().len())
                                .apply(run.values())?,
                            traversal,
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
                    let mut parents = Vec::new();
                    for index in 0..array.len() {
                        if union.type_id(index) == id {
                            offsets.push(union.value_offset(index));
                            parents.push(index);
                        }
                    }
                    if !offsets.is_empty() {
                        let parent =
                            Selection::indices(parents, array.len()).map(traversal.clone());
                        let child_traversal = Arc::new(Traversal::Field {
                            parent,
                            field: child.path_name.clone(),
                        });
                        child.visit(
                            &Selection::indices(offsets, union.child(id).len())
                                .apply(union.child(id))?,
                            child_traversal,
                            findings,
                        )?;
                    }
                }
                return Ok(());
            }
            _ => {}
        }
        if let Some(predicate) = &self.predicate {
            let batch = RecordBatch::try_new(self.schema.clone(), vec![array.clone()])?;
            let values = predicate
                .evaluate(&batch)
                .and_then(|value| value.into_array(array.len()))
                .map_err(engine)?;
            let flags = values
                .as_boolean_opt()
                .ok_or_else(|| invalid("local predicate is not Boolean"))?;
            for index in 0..array.len() {
                if flags.is_null(index) || !flags.value(index) {
                    findings.add("field-domain", &self.field, &array, index, &traversal)?;
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
                    let selection = if let (Some(alternative), Some(tags)) =
                        (&self.alternative, &tags)
                        && alternative.payloads().contains(child.field.name().as_str())
                    {
                        let tags = tags.as_string::<i32>();
                        Selection::indices(
                            (0..array.len())
                                .filter(|&index| {
                                    !tags.is_null(index)
                                        && alternative
                                            .arms
                                            .get(tags.value(index))
                                            .and_then(Option::as_deref)
                                            == Some(child.field.name())
                                })
                                .collect(),
                            array.len(),
                        )
                    } else {
                        Selection::All
                    };
                    let parent = selection.map(traversal.clone());
                    child.visit(
                        &selection.apply(values)?,
                        Arc::new(Traversal::Field {
                            parent,
                            field: child.path_name.clone(),
                        }),
                        findings,
                    )?;
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
                let mut entries = Vec::with_capacity(CHUNK);
                for (parent, range) in ranges.into_iter().enumerate() {
                    for (ordinal, index) in range.enumerate() {
                        indices.push(index);
                        entries.push((parent, ordinal));
                        if indices.len() == CHUNK {
                            let selected = Selection::indices(
                                std::mem::replace(&mut indices, Vec::with_capacity(CHUNK)),
                                values.len(),
                            );
                            let child_traversal = Arc::new(Traversal::List {
                                parent: traversal.clone(),
                                entries: std::mem::replace(&mut entries, Vec::with_capacity(CHUNK)),
                            });
                            self.children[0].visit(
                                &selected.apply(&values)?,
                                child_traversal,
                                findings,
                            )?;
                        }
                    }
                }
                if !indices.is_empty() {
                    self.children[0].visit(
                        &Selection::indices(indices, values.len()).apply(&values)?,
                        Arc::new(Traversal::List {
                            parent: traversal,
                            entries,
                        }),
                        findings,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
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

#[cfg(test)]
mod selection_unit {
    use super::*;
    use crate::native::arrow::array::Int64Array;
    #[test]
    fn whole_range_and_scattered_selection_preserve_values_nulls_and_mapping() {
        let array: ArrayRef = Arc::new(Int64Array::from(vec![Some(7), None, Some(9), Some(11)]));
        let all = Selection::indices(vec![0, 1, 2, 3], 4);
        assert!(matches!(all, Selection::All));
        assert!(Arc::ptr_eq(&array, &all.apply(&array).unwrap()));
        let range = Selection::indices(vec![1, 2], 4);
        assert!(matches!(
            range,
            Selection::Range {
                start: 1,
                length: 2
            }
        ));
        let scattered = Selection::indices(vec![2, 1, 2], 4);
        assert!(matches!(scattered, Selection::Indices(_)));
        assert_eq!(
            range
                .apply(&array)
                .unwrap()
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap(),
            &Int64Array::from(vec![None, Some(9)])
        );
        assert_eq!(
            scattered
                .apply(&array)
                .unwrap()
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap(),
            &Int64Array::from(vec![Some(9), None, Some(9)])
        );
        let root = Arc::new(Traversal::Root {
            start: 40,
            field: "a~1b".into(),
        });
        let selected = scattered.map(root);
        let nested = Traversal::List {
            parent: selected,
            entries: vec![(0, 3), (1, 0), (2, 1)],
        };
        assert_eq!(nested.row(0), 42);
        assert_eq!(nested.location(0).path.render(), "/a~1b/3");
        assert_eq!(nested.row(1), 41);
        assert_eq!(nested.location(1).path.render(), "/a~1b/0");
        assert!(Selection::indices(vec![4], 4).apply(&array).is_err());
    }
}
