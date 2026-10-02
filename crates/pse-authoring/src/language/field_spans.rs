// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Capture parser-owned string locations by their generated payload field path.
//! Allocation addresses are transient join keys during one parse, never source identity.
use crate::SourceSpan;
use serde::{
    Serialize,
    ser::{self, SerializeSeq, SerializeStruct},
};
use std::collections::BTreeMap;
#[derive(Debug)]
pub(super) struct CaptureError;
impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unsupported source span capture")
    }
}
impl std::error::Error for CaptureError {}
impl ser::Error for CaptureError {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self
    }
}
struct Capture<'a> {
    parsed: &'a BTreeMap<usize, (String, SourceSpan)>,
    output: &'a mut BTreeMap<String, SourceSpan>,
    path: String,
}
struct Fields<'a> {
    capture: Capture<'a>,
    index: usize,
}
pub(super) fn collect(
    value: &impl Serialize,
    parsed: &BTreeMap<usize, (String, SourceSpan)>,
) -> Result<BTreeMap<String, SourceSpan>, CaptureError> {
    let mut output = BTreeMap::new();
    value.serialize(Capture {
        parsed,
        output: &mut output,
        path: String::new(),
    })?;
    Ok(output)
}
macro_rules! scalar {
    ($($name:ident($ty:ty)),*) => { $(fn $name(self, _: $ty) -> Result<(), CaptureError> { Ok(()) })* };
}
impl<'a> ser::Serializer for Capture<'a> {
    type Ok = ();
    type Error = CaptureError;
    type SerializeSeq = Fields<'a>;
    type SerializeTuple = Fields<'a>;
    type SerializeTupleStruct = Fields<'a>;
    type SerializeTupleVariant = Fields<'a>;
    type SerializeMap = Fields<'a>;
    type SerializeStruct = Fields<'a>;
    type SerializeStructVariant = Fields<'a>;
    scalar!(
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char)
    );
    fn serialize_str(self, value: &str) -> Result<(), CaptureError> {
        if let Some((text, span)) = self.parsed.get(&(value.as_ptr() as usize))
            && text == value
        {
            self.output.insert(self.path, *span);
        }
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), CaptureError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), CaptureError> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), CaptureError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), CaptureError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), CaptureError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), CaptureError> {
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), CaptureError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        value: &T,
    ) -> Result<(), CaptureError> {
        value.serialize(self)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Fields<'a>, CaptureError> {
        Ok(Fields {
            capture: self,
            index: 0,
        })
    }
    fn serialize_tuple(self, n: usize) -> Result<Fields<'a>, CaptureError> {
        self.serialize_seq(Some(n))
    }
    fn serialize_tuple_struct(self, _: &'static str, n: usize) -> Result<Fields<'a>, CaptureError> {
        self.serialize_seq(Some(n))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        n: usize,
    ) -> Result<Fields<'a>, CaptureError> {
        self.serialize_seq(Some(n))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Fields<'a>, CaptureError> {
        Ok(Fields {
            capture: self,
            index: 0,
        })
    }
    fn serialize_struct(self, _: &'static str, n: usize) -> Result<Fields<'a>, CaptureError> {
        self.serialize_seq(Some(n))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        n: usize,
    ) -> Result<Fields<'a>, CaptureError> {
        self.serialize_seq(Some(n))
    }
}
impl Fields<'_> {
    fn field<T: ?Sized + Serialize>(&mut self, name: &str, value: &T) -> Result<(), CaptureError> {
        value.serialize(Capture {
            parsed: self.capture.parsed,
            output: self.capture.output,
            path: if self.capture.path.is_empty() {
                name.into()
            } else {
                format!("{}.{}", self.capture.path, name)
            },
        })
    }
}
impl SerializeSeq for Fields<'_> {
    type Ok = ();
    type Error = CaptureError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CaptureError> {
        let index = self.index;
        self.index += 1;
        self.field(&index.to_string(), value)
    }
    fn end(self) -> Result<(), CaptureError> {
        Ok(())
    }
}
impl SerializeStruct for Fields<'_> {
    type Ok = ();
    type Error = CaptureError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), CaptureError> {
        self.field(key, value)
    }
    fn end(self) -> Result<(), CaptureError> {
        Ok(())
    }
}
macro_rules! tuple {
    ($trait:ident, $method:ident) => {
        impl ser::$trait for Fields<'_> {
            type Ok = ();
            type Error = CaptureError;
            fn $method<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), CaptureError> {
                SerializeSeq::serialize_element(self, value)
            }
            fn end(self) -> Result<(), CaptureError> {
                Ok(())
            }
        }
    };
}
tuple!(SerializeTuple, serialize_element);
tuple!(SerializeTupleStruct, serialize_field);
tuple!(SerializeTupleVariant, serialize_field);
impl ser::SerializeStructVariant for Fields<'_> {
    type Ok = ();
    type Error = CaptureError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), CaptureError> {
        self.field(key, value)
    }
    fn end(self) -> Result<(), CaptureError> {
        Ok(())
    }
}
impl ser::SerializeMap for Fields<'_> {
    type Ok = ();
    type Error = CaptureError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<(), CaptureError> {
        Err(CaptureError)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<(), CaptureError> {
        Err(CaptureError)
    }
    fn end(self) -> Result<(), CaptureError> {
        Ok(())
    }
}
