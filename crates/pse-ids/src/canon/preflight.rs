// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Checked construction bounds, derived from admitted source values and live stages.

use super::normalize::{cast, enum_value};
use crate::{
    CancellationToken, CanonError, CanonicalContract, Envelope, EnvelopeBound, FieldPath, Layout,
};
use arrow_array::{
    Array, BinaryArray, FixedSizeListArray, ListArray, RecordBatch, StringArray, StructArray,
};
use arrow_schema::{DataType, Field};
use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) struct Estimate {
    pub rows: usize,
    pub normalized: usize,
    pub reservation: usize,
    pub stream_limit: usize,
}
struct Count {
    len: usize,
    nulls: usize,
    variable: usize,
    children: Vec<Self>,
}
impl Count {
    fn new(layout: &Layout) -> Self {
        Self {
            len: 0,
            nulls: 0,
            variable: 0,
            children: match layout {
                Layout::List(inner) | Layout::FixedSizeList(inner, _) => vec![Self::new(inner)],
                Layout::Struct(children) => children
                    .iter()
                    .map(|(_, layout)| Self::new(layout))
                    .collect(),
                _ => Vec::new(),
            },
        }
    }
}
fn overflow() -> CanonError {
    CanonError::Envelope {
        what: EnvelopeBound::Bytes,
        limit: Envelope::PHASE1.max_normalized_bytes,
        actual: u64::MAX,
    }
}
pub(super) fn add(one: usize, two: usize) -> Result<usize, CanonError> {
    one.checked_add(two).ok_or_else(overflow)
}
fn mul(one: usize, two: usize) -> Result<usize, CanonError> {
    one.checked_mul(two).ok_or_else(overflow)
}
fn bits(len: usize) -> Result<usize, CanonError> {
    Ok(add(len, 7)? / 8)
}
fn round(bytes: usize) -> Result<usize, CanonError> {
    Ok(add(bytes, 63)? & !63)
}
fn bound(what: EnvelopeBound, actual: usize, limit: u64) -> Result<(), CanonError> {
    let actual = u64::try_from(actual).map_err(|_| overflow())?;
    if actual > limit {
        Err(CanonError::Envelope {
            what,
            limit,
            actual,
        })
    } else {
        Ok(())
    }
}

pub(super) fn control_bytes(contract: &CanonicalContract) -> Result<usize, CanonError> {
    let mut bytes = 1024usize;
    for (key, value) in contract.schema.metadata() {
        bytes = add(bytes, add(key.len(), value.len())?)?;
    }
    for field in contract.schema.fields() {
        bytes = add(bytes, field_bytes(field)?)?;
    }
    // Preflight counters, reconstructed layout/domain maps and schema traversal strings.
    mul(bytes, 8)
}
fn field_bytes(field: &Field) -> Result<usize, CanonError> {
    let mut bytes = add(256, field.name().len())?;
    for (key, value) in field.metadata() {
        bytes = add(bytes, add(key.len(), value.len())?)?;
    }
    match field.data_type() {
        DataType::List(child) | DataType::FixedSizeList(child, _) => {
            bytes = add(bytes, field_bytes(child)?)?;
        }
        DataType::Struct(children) => {
            for child in children {
                bytes = add(bytes, field_bytes(child)?)?;
            }
        }
        _ => {}
    }
    Ok(bytes)
}

pub(super) fn estimate(
    contract: &CanonicalContract,
    batches: &[RecordBatch],
    envelope: Envelope,
    cancel: &CancellationToken,
) -> Result<Estimate, CanonError> {
    Envelope::lowered(envelope.max_rows, envelope.max_normalized_bytes)?;
    validate_contract(contract)?;
    let mut counts = contract.layouts.iter().map(Count::new).collect::<Vec<_>>();
    let mut rows = 0usize;
    let mut input = 0usize;
    for batch in batches {
        cancel.checkpoint()?;
        contract.check_batch_schema(&batch.schema())?;
        rows = add(rows, batch.num_rows())?;
        bound(EnvelopeBound::Rows, rows, envelope.max_rows)?;
        input = add(input, crate::owned_buffer::retained_buffer_bytes(batch)?)?;
        for (index, ((array, field), (layout, count))) in batch
            .columns()
            .iter()
            .zip(contract.schema.fields())
            .zip(contract.layouts.iter().zip(&mut counts))
            .enumerate()
        {
            array.to_data().validate_full()?;
            for row in 0..array.len() {
                measure(
                    count,
                    field,
                    layout,
                    array.as_ref(),
                    row,
                    false,
                    &MeasureContext {
                        path: FieldPath::root().child(index),
                        cancel,
                    },
                )?;
            }
        }
    }
    let mut normalized = 0usize;
    let mut padded = 0usize;
    let mut slots = 0usize;
    let mut variable = 0usize;
    for (count, layout) in counts.iter().zip(&contract.layouts) {
        let summary = sizes(count, layout)?;
        normalized = add(normalized, summary.0)?;
        padded = add(padded, summary.1)?;
        slots = add(slots, summary.2)?;
        variable = add(variable, summary.3)?;
    }
    bound(
        EnvelopeBound::Bytes,
        normalized,
        envelope.max_normalized_bytes,
    )?;
    let control = control_bytes(contract)?;
    // Live regions: the borrowed input claim, concatenation and sorted copy (3 input
    // extents); primitive Option staging plus normalized buffers (3 padded extents);
    // both finished IPC streams and their amortized Vec/write staging (6 extents).
    // RowConverter's variable blocks have at most two payload bytes per source byte;
    // 32 bytes per scalar slot also covers row offsets, sentinels and selection vectors.
    // Twenty-eight control extents cover both FlatBuffer schemas, metadata rows,
    // bounded stream capacities and the complete preimage.
    let reservation = add(
        add(mul(input, 3)?, mul(padded, 9)?)?,
        add(
            add(mul(variable, 2)?, mul(slots, 32)?)?,
            add(mul(rows, 32)?, mul(control, 28)?)?,
        )?,
    )?;
    if reservation > isize::MAX.unsigned_abs() {
        return Err(overflow());
    }
    let stream_limit = add(padded, mul(control, 4)?)?;
    Ok(Estimate {
        rows,
        normalized,
        reservation,
        stream_limit,
    })
}

fn validate_contract(contract: &CanonicalContract) -> Result<(), CanonError> {
    if contract.primary_key.is_empty() {
        return Err(CanonError::InvalidKey {
            column: String::new(),
            reason: "canonical ordering requires a non-empty primary key".to_owned(),
        });
    }
    if contract.layouts.len() != contract.schema.fields().len() {
        return Err(CanonError::Internal(
            "contract layout count differs from schema".to_owned(),
        ));
    }
    let mut domains = BTreeMap::new();
    for (index, layout) in contract.layouts.iter().enumerate() {
        domains_for(layout, &FieldPath::root().child(index), &mut domains)?;
    }
    let names = contract
        .primary_key
        .iter()
        .map(|index| {
            contract
                .schema
                .fields()
                .get(*index)
                .map(|field| field.name().as_str())
                .ok_or_else(|| CanonError::InvalidKey {
                    column: index.to_string(),
                    reason: "key ordinal is outside schema".to_owned(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let rebuilt = CanonicalContract::try_new(
        contract.relation_id,
        contract.schema_version,
        contract.registry_fingerprint,
        Arc::clone(&contract.schema),
        &names,
        &domains,
    )?;
    if rebuilt.layouts != contract.layouts {
        return Err(CanonError::Internal(
            "contract layouts differ from declared storage".to_owned(),
        ));
    }
    for index in &contract.primary_key {
        if floating(&contract.layouts[*index]) {
            return Err(CanonError::InvalidKey {
                column: contract.schema.field(*index).name().clone(),
                reason: "floating descendants cannot form a canonical key".to_owned(),
            });
        }
    }
    Ok(())
}
fn floating(layout: &Layout) -> bool {
    match layout {
        Layout::Float32 | Layout::Float64 => true,
        Layout::List(inner) | Layout::FixedSizeList(inner, _) => floating(inner),
        Layout::Struct(children) => children.iter().any(|(_, layout)| floating(layout)),
        _ => false,
    }
}
fn domains_for(
    layout: &Layout,
    path: &FieldPath,
    out: &mut BTreeMap<FieldPath, Arc<[String]>>,
) -> Result<(), CanonError> {
    match layout {
        Layout::Enum { members, .. } => {
            if members.is_empty()
                || members
                    .iter()
                    .enumerate()
                    .any(|(index, value)| members[..index].contains(value))
            {
                return Err(CanonError::UnsupportedLayout {
                    path: path.clone(),
                    what: "empty or duplicate enum domain".to_owned(),
                });
            }
            out.insert(path.clone(), Arc::clone(members));
        }
        Layout::List(inner) => domains_for(inner, &path.child(0), out)?,
        Layout::FixedSizeList(inner, width) if *width > 0 => {
            domains_for(inner, &path.child(0), out)?;
        }
        Layout::FixedSizeList(_, _) | Layout::FixedSizeBinary(..) => {
            if matches!(layout,Layout::FixedSizeBinary(width) if *width>0) {
            } else {
                return Err(CanonError::UnsupportedLayout {
                    path: path.clone(),
                    what: "non-positive fixed width".to_owned(),
                });
            }
        }
        Layout::Struct(children) => {
            for (index, (_, layout)) in children.iter().enumerate() {
                domains_for(layout, &path.child(index), out)?;
            }
        }
        _ => {}
    }
    Ok(())
}

struct MeasureContext<'a> {
    path: FieldPath,
    cancel: &'a CancellationToken,
}
impl MeasureContext<'_> {
    fn child(&self, index: usize) -> Self {
        Self {
            path: self.path.child(index),
            cancel: self.cancel,
        }
    }
}
fn begin_value(
    count: &mut Count,
    field: &Field,
    array: &dyn Array,
    row: usize,
    hidden: bool,
    context: &MeasureContext<'_>,
) -> Result<bool, CanonError> {
    if count.len.is_multiple_of(4096) {
        context.cancel.checkpoint()?;
    }
    count.len = add(count.len, 1)?;
    let absent = !hidden && array.is_null(row);
    if absent && !field.is_nullable() {
        return Err(CanonError::ContractMismatch {
            path: context.path.clone(),
            expected: "non-null visible value".to_owned(),
            actual: format!("null in row {row}"),
        });
    }
    count.nulls = add(count.nulls, usize::from(absent))?;
    let masked = hidden || absent;
    Ok(masked)
}

fn measure(
    count: &mut Count,
    field: &Field,
    layout: &Layout,
    array: &dyn Array,
    row: usize,
    hidden: bool,
    context: &MeasureContext<'_>,
) -> Result<(), CanonError> {
    let masked = begin_value(count, field, array, row, hidden, context)?;
    match layout {
        Layout::Utf8 if !masked => {
            count.variable = add(count.variable, cast::<StringArray>(array)?.value(row).len())?;
        }
        Layout::Binary if !masked => {
            count.variable = add(count.variable, cast::<BinaryArray>(array)?.value(row).len())?;
        }
        Layout::Enum { key, members } if !masked => {
            let value = enum_value(array, *key, row)?.ok_or_else(|| CanonError::EnumMember {
                path: context.path.clone(),
                value: "<null dictionary member>".to_owned(),
            })?;
            if !members.iter().any(|member| member == value) {
                return Err(CanonError::EnumMember {
                    path: context.path.clone(),
                    value: value.to_owned(),
                });
            }
            count.variable = add(count.variable, value.len())?;
        }
        Layout::List(inner) => {
            if !masked {
                let list = cast::<ListArray>(array)?;
                let DataType::List(child) = field.data_type() else {
                    return Err(overflow());
                };
                let start = usize::try_from(list.value_offsets()[row]).map_err(|_| overflow())?;
                let end = usize::try_from(list.value_offsets()[row + 1]).map_err(|_| overflow())?;
                for index in start..end {
                    measure(
                        &mut count.children[0],
                        child,
                        inner,
                        list.values().as_ref(),
                        index,
                        false,
                        &context.child(0),
                    )?;
                }
            }
        }
        Layout::FixedSizeList(inner, width) => {
            let list = cast::<FixedSizeListArray>(array)?;
            let DataType::FixedSizeList(child, _) = field.data_type() else {
                return Err(overflow());
            };
            let width = usize::try_from(*width).map_err(|_| overflow())?;
            let start = mul(row, width)?;
            for index in start..add(start, width)? {
                measure(
                    &mut count.children[0],
                    child,
                    inner,
                    list.values().as_ref(),
                    index,
                    masked,
                    &context.child(0),
                )?;
            }
        }
        Layout::Struct(layouts) => {
            let structure = cast::<StructArray>(array)?;
            let DataType::Struct(fields) = field.data_type() else {
                return Err(overflow());
            };
            for (index, (((_, layout), child), array)) in layouts
                .iter()
                .zip(fields)
                .zip(structure.columns())
                .enumerate()
            {
                measure(
                    &mut count.children[index],
                    child,
                    layout,
                    array.as_ref(),
                    row,
                    masked,
                    &context.child(index),
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn sizes(count: &Count, layout: &Layout) -> Result<(usize, usize, usize, usize), CanonError> {
    let mut raw = 0usize;
    let mut padded = 0usize;
    let mut slots = count.len;
    let mut variable = count.variable;
    let mut buffer = |bytes| -> Result<(), CanonError> {
        raw = add(raw, bytes)?;
        padded = add(padded, round(bytes)?)?;
        Ok(())
    };
    if count.nulls > 0 {
        buffer(bits(count.len)?)?;
    }
    match layout {
        Layout::Boolean => buffer(bits(count.len)?)?,
        Layout::Int8 | Layout::UInt8 => buffer(count.len)?,
        Layout::Int16 | Layout::UInt16 => buffer(mul(count.len, 2)?)?,
        Layout::Int32 | Layout::UInt32 | Layout::Float32 => buffer(mul(count.len, 4)?)?,
        Layout::Int64 | Layout::UInt64 | Layout::Float64 | Layout::TimestampNsUtc => {
            buffer(mul(count.len, 8)?)?;
        }
        Layout::FixedSizeBinary(width) => buffer(mul(
            count.len,
            usize::try_from(*width).map_err(|_| overflow())?,
        )?)?,
        Layout::Utf8 | Layout::Binary | Layout::Enum { .. } => {
            bound(EnvelopeBound::Offset, count.variable, i32::MAX as u64)?;
            buffer(mul(add(count.len, 1)?, 4)?)?;
            buffer(count.variable)?;
        }
        Layout::List(_) => {
            bound(
                EnvelopeBound::Offset,
                count.children[0].len,
                i32::MAX as u64,
            )?;
            buffer(mul(add(count.len, 1)?, 4)?)?;
        }
        Layout::FixedSizeList(..) | Layout::Struct(_) => {}
    }
    let children: Vec<&Layout> = match layout {
        Layout::List(inner) | Layout::FixedSizeList(inner, _) => vec![inner],
        Layout::Struct(children) => children.iter().map(|(_, layout)| layout).collect(),
        _ => vec![],
    };
    for (count, layout) in count.children.iter().zip(children) {
        let child = sizes(count, layout)?;
        raw = add(raw, child.0)?;
        padded = add(padded, child.1)?;
        slots = add(slots, child.2)?;
        variable = add(variable, child.3)?;
    }
    Ok((raw, padded, slots, variable))
}
