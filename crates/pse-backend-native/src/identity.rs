// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Settings identity derived from serde. Every serialized field, variant and float bit
//! pattern is framed, so a field added to a settings type changes identity without a
//! hand-written field list (F09). This is identity only; it is not a wire format.
use crate::ProblemError;
use pse_ids::{ContentHash, FramedHasher};
use serde::ser::{self, Serialize};

/// Frame `value` into `hasher`. Floats keep their exact bits, including signed zero,
/// infinities and NaN payloads; sequences, maps and structs carry explicit ends.
///
/// # Errors
/// A custom serializer refused the value.
pub fn frame<T: Serialize + ?Sized>(
    hasher: &mut FramedHasher,
    value: &T,
) -> Result<(), ProblemError> {
    value
        .serialize(Framer(hasher))
        .map_err(|e| ProblemError::Internal(format!("settings identity: {}", e.0)))
}

/// Content identity of one serializable value under `context`.
///
/// # Errors
/// A custom serializer refused the value.
pub fn of<T: Serialize + ?Sized>(
    context: &'static str,
    value: &T,
) -> Result<ContentHash, ProblemError> {
    let mut hasher = FramedHasher::new(context);
    frame(&mut hasher, value)?;
    Ok(hasher.finish_hash())
}

#[derive(Debug)]
struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}

struct Framer<'a>(&'a mut FramedHasher);
struct Compound<'a>(&'a mut FramedHasher);

impl<'a> ser::Serializer for Framer<'a> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.0.str("bool").bool(v);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.0.str("i64").part(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        self.0.str("i128").part(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.0.str("u64").u64(v);
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        self.0.str("u128").part(&v.to_le_bytes());
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.0.str("f32").u32(v.to_bits());
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        self.0.str("f64").u64(v.to_bits());
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.0.str("char").u32(u32::from(v));
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.0.str("str").str(v);
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.0.str("bytes").part(v);
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Error> {
        self.0.str("none");
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Error> {
        self.0.str("some");
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        self.0.str("unit");
        Ok(())
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Error> {
        self.0.str("unit_struct").str(name);
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.0.str("unit_variant").str(name).str(variant);
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.0.str("newtype_struct").str(name);
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.0.str("newtype_variant").str(name).str(variant);
        value.serialize(self)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Compound<'a>, Error> {
        self.0.str("seq");
        Ok(Compound(self.0))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Compound<'a>, Error> {
        self.0.str("tuple");
        Ok(Compound(self.0))
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.0.str("tuple_struct").str(name);
        Ok(Compound(self.0))
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.0.str("tuple_variant").str(name).str(variant);
        Ok(Compound(self.0))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Compound<'a>, Error> {
        self.0.str("map");
        Ok(Compound(self.0))
    }
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Compound<'a>, Error> {
        self.0.str("struct").str(name);
        Ok(Compound(self.0))
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.0.str("struct_variant").str(name).str(variant);
        Ok(Compound(self.0))
    }
}
impl Compound<'_> {
    fn element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(Framer(self.0))
    }
    fn end(self) -> Result<(), Error> {
        self.0.str("end");
        Ok(())
    }
}
impl ser::SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Error> {
        Compound::end(self)
    }
}
impl ser::SerializeTuple for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Error> {
        Compound::end(self)
    }
}
impl ser::SerializeTupleStruct for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Error> {
        Compound::end(self)
    }
}
impl ser::SerializeTupleVariant for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }
    fn end(self) -> Result<(), Error> {
        Compound::end(self)
    }
}
impl ser::SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        self.0.str("key");
        self.element(key)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.0.str("value");
        self.element(value)
    }
    fn end(self) -> Result<(), Error> {
        Compound::end(self)
    }
}
impl ser::SerializeStruct for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.0.str("field").str(key);
        self.element(value)
    }
    fn skip_field(&mut self, key: &'static str) -> Result<(), Error> {
        self.0.str("skipped").str(key);
        Ok(())
    }
    fn end(self) -> Result<(), Error> {
        Compound::end(self)
    }
}
impl ser::SerializeStructVariant for Compound<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.0.str("field").str(key);
        self.element(value)
    }
    fn skip_field(&mut self, key: &'static str) -> Result<(), Error> {
        self.0.str("skipped").str(key);
        Ok(())
    }
    fn end(self) -> Result<(), Error> {
        Compound::end(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(serde::Serialize)]
    struct Probe {
        value: f64,
        items: Vec<Option<u32>>,
        tag: Option<&'static str>,
    }
    #[test]
    fn identity_frames_every_field_and_exact_float_bits() {
        let base = Probe {
            value: 0.0,
            items: vec![Some(1), None],
            tag: None,
        };
        let key = of("pse.test.identity.v1", &base).unwrap();
        assert_eq!(key, of("pse.test.identity.v1", &base).unwrap());
        for changed in [
            Probe {
                value: -0.0,
                ..base_clone(&base)
            },
            Probe {
                items: vec![None, Some(1)],
                ..base_clone(&base)
            },
            Probe {
                items: vec![Some(1), None, None],
                ..base_clone(&base)
            },
            Probe {
                tag: Some(""),
                ..base_clone(&base)
            },
        ] {
            assert_ne!(key, of("pse.test.identity.v1", &changed).unwrap());
        }
        // A sequence boundary cannot move between adjacent fields.
        let split = (vec![1_u32, 2], vec![3_u32]);
        let moved = (vec![1_u32], vec![2_u32, 3]);
        assert_ne!(
            of("pse.test.identity.v1", &split).unwrap(),
            of("pse.test.identity.v1", &moved).unwrap()
        );
        assert_ne!(
            of("pse.test.identity.v1", &f64::INFINITY).unwrap(),
            of("pse.test.identity.v1", &f64::NAN).unwrap()
        );
    }
    fn base_clone(p: &Probe) -> Probe {
        Probe {
            value: p.value,
            items: p.items.clone(),
            tag: p.tag,
        }
    }
}
