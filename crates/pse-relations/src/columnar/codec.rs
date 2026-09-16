// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Direct codecs used by generated types; no row-shaped intermediate representation.

use arrow_array::builder::{
    ArrayBuilder, BooleanBuilder, FixedSizeBinaryBuilder, FixedSizeListBuilder, Float64Builder,
    Int16Builder, Int32Builder, Int64Builder, ListBuilder, StringBuilder,
    TimestampNanosecondBuilder, UInt8Builder, UInt16Builder, UInt32Builder, UInt64Builder,
};
use arrow_array::{
    Array, BooleanArray, FixedSizeBinaryArray, FixedSizeListArray, Float64Array, Int16Array,
    Int32Array, Int64Array, ListArray, StringArray, TimestampNanosecondArray, UInt8Array,
    UInt16Array, UInt32Array, UInt64Array,
};
use pse_ids::{ContentHash, SemanticId};

use super::{array, builder, mismatch, visible};
use crate::RelationError;

/// Storage conversion only. Generated push methods check logical values before append.
pub(crate) trait ArrowValue: Sized {
    fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError>;
    fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError>;
    fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError>;
}

macro_rules! scalar {
    ($value:ty, $builder:ty, $array:ty) => {
        impl ArrowValue for $value {
            fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
                builder::<$builder>(output)?.append_value(*self);
                Ok(())
            }
            fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
                builder::<$builder>(output)?.append_null();
                Ok(())
            }
            fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError> {
                visible(input, index)?;
                Ok(array::<$array>(input)?.value(index))
            }
        }
    };
}
scalar!(bool, BooleanBuilder, BooleanArray);
scalar!(i16, Int16Builder, Int16Array);
scalar!(i32, Int32Builder, Int32Array);
scalar!(u8, UInt8Builder, UInt8Array);
scalar!(u16, UInt16Builder, UInt16Array);
scalar!(u32, UInt32Builder, UInt32Array);
scalar!(u64, UInt64Builder, UInt64Array);
scalar!(f64, Float64Builder, Float64Array);

impl ArrowValue for i64 {
    fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        if output.as_any().is::<TimestampNanosecondBuilder>() {
            builder::<TimestampNanosecondBuilder>(output)?.append_value(*self);
        } else {
            builder::<Int64Builder>(output)?.append_value(*self);
        }
        Ok(())
    }
    fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        if output.as_any().is::<TimestampNanosecondBuilder>() {
            builder::<TimestampNanosecondBuilder>(output)?.append_null();
        } else {
            builder::<Int64Builder>(output)?.append_null();
        }
        Ok(())
    }
    fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError> {
        visible(input, index)?;
        if input.as_any().is::<TimestampNanosecondArray>() {
            Ok(array::<TimestampNanosecondArray>(input)?.value(index))
        } else {
            Ok(array::<Int64Array>(input)?.value(index))
        }
    }
}

impl ArrowValue for String {
    fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        append_string(output, Some(self))
    }
    fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        append_string(output, None)
    }
    fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError> {
        read_string(input, index).map(str::to_owned)
    }
}

macro_rules! identifier {
    ($value:ty) => {
        impl ArrowValue for $value {
            fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
                builder::<FixedSizeBinaryBuilder>(output)?.append_value(self.as_bytes())?;
                Ok(())
            }
            fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
                builder::<FixedSizeBinaryBuilder>(output)?.append_null();
                Ok(())
            }
            fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError> {
                visible(input, index)?;
                let bytes = array::<FixedSizeBinaryArray>(input)?
                    .value(index)
                    .try_into()
                    .map_err(|_| mismatch("the declared identifier width"))?;
                Ok(Self::from_bytes(bytes))
            }
        }
    };
}
identifier!(SemanticId);
identifier!(ContentHash);

impl<T: ArrowValue> ArrowValue for Option<T> {
    fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        match self {
            Some(value) => value.append(output),
            None => T::append_null(output),
        }
    }
    fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        T::append_null(output)
    }
    fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError> {
        if index >= input.len() {
            return Err(mismatch("a row within the array"));
        }
        if input.is_null(index) {
            Ok(None)
        } else {
            T::read(input, index).map(Some)
        }
    }
}

impl<T: ArrowValue> ArrowValue for Vec<T> {
    fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        let output = builder::<ListBuilder<Box<dyn ArrayBuilder>>>(output)?;
        let length = output
            .values()
            .len()
            .checked_add(self.len())
            .ok_or_else(|| mismatch("representable List offset"))?;
        i32::try_from(length).map_err(|_| mismatch("representable List offset"))?;
        for value in self {
            value.append(output.values().as_mut())?;
        }
        output.append(true);
        Ok(())
    }
    fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        builder::<ListBuilder<Box<dyn ArrayBuilder>>>(output)?.append(false);
        Ok(())
    }
    fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError> {
        visible(input, index)?;
        let input = array::<ListArray>(input)?;
        let offsets = input.value_offsets();
        let start =
            usize::try_from(offsets[index]).map_err(|_| mismatch("nonnegative List offset"))?;
        let end =
            usize::try_from(offsets[index + 1]).map_err(|_| mismatch("nonnegative List offset"))?;
        (start..end)
            .map(|index| T::read(input.values().as_ref(), index))
            .collect()
    }
}

impl<T: ArrowValue, const N: usize> ArrowValue for [T; N] {
    fn append(&self, output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        let output = builder::<FixedSizeListBuilder<Box<dyn ArrayBuilder>>>(output)?;
        if usize::try_from(output.value_length()).ok() != Some(N) {
            return Err(mismatch("the generated fixed-list width"));
        }
        for value in self {
            value.append(output.values().as_mut())?;
        }
        output.append(true);
        Ok(())
    }
    fn append_null(output: &mut dyn ArrayBuilder) -> Result<(), RelationError> {
        let output = builder::<FixedSizeListBuilder<Box<dyn ArrayBuilder>>>(output)?;
        if usize::try_from(output.value_length()).ok() != Some(N) {
            return Err(mismatch("the generated fixed-list width"));
        }
        for _ in 0..N {
            T::append_null(output.values().as_mut())?;
        }
        output.append(false);
        Ok(())
    }
    fn read(input: &dyn Array, index: usize) -> Result<Self, RelationError> {
        visible(input, index)?;
        let input = array::<FixedSizeListArray>(input)?;
        if usize::try_from(input.value_length()).ok() != Some(N) {
            return Err(mismatch("the generated fixed-list width"));
        }
        let start = usize::try_from(input.value_offset(index))
            .map_err(|_| mismatch("nonnegative fixed-list offset"))?;
        (start..start + N)
            .map(|index| T::read(input.values().as_ref(), index))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| mismatch("the generated fixed-list width"))
    }
}

pub(crate) fn append_string(
    output: &mut dyn ArrayBuilder,
    value: Option<&str>,
) -> Result<(), RelationError> {
    let output = builder::<StringBuilder>(output)?;
    match value {
        Some(value) => {
            let length = output
                .values_slice()
                .len()
                .checked_add(value.len())
                .ok_or_else(|| mismatch("representable Utf8 offset"))?;
            i32::try_from(length).map_err(|_| mismatch("representable Utf8 offset"))?;
            output.append_value(value);
        }
        None => output.append_null(),
    }
    Ok(())
}

pub(crate) fn read_string(input: &dyn Array, index: usize) -> Result<&str, RelationError> {
    visible(input, index)?;
    Ok(array::<StringArray>(input)?.value(index))
}
