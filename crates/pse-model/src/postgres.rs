// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reading a store row from its PostgreSQL composite value (ADR-0114 Outcome 25; Plan 22
//! X7). The generated `FromSql` of every `runtime.operational_*` row reads the binary
//! record its table's row type sends through [`Record`], which checks the field count and
//! names against the type the server described, then decodes each field with the field's
//! own type: typed ids accept their identity domains, enums their ENUM types.
//!
//! The registry's operational timestamps are microseconds since the Unix epoch (`ts_us`)
//! and its JSON documents are text, so `timestamptz` and `jsonb` fields have their own
//! readers. This is the value protocol only, never a driver.

use postgres_types::{FromSql, Kind, Type, WrongType};

type BoxError = Box<dyn std::error::Error + Sync + Send>;

/// Microseconds from the Unix epoch to PostgreSQL's (2000-01-01 00:00:00 UTC).
const POSTGRES_EPOCH_MICROS: i64 = 946_684_800_000_000;

/// The jsonb binary format version PostgreSQL sends.
const JSONB_VERSION: u8 = 1;

/// Whether `ty` is the composite row type `schema.name`.
pub fn composite(ty: &Type, schema: &str, name: &str) -> bool {
    ty.schema() == schema && ty.name() == name && matches!(ty.kind(), Kind::Composite(_))
}

/// A malformed or unexpected record.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct RecordError(String);

fn refuse(reason: String) -> BoxError {
    Box::new(RecordError(reason))
}

/// One binary composite value, read field by field in declaration order.
pub struct Record<'t, 'a> {
    ty: &'t Type,
    fields: std::slice::Iter<'t, postgres_types::Field>,
    raw: &'a [u8],
}

impl std::fmt::Debug for Record<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Record")
            .field("type", &self.ty.name())
            .field("remaining", &self.fields.len())
            .finish()
    }
}

fn split<'a>(raw: &mut &'a [u8], len: usize) -> Result<&'a [u8], BoxError> {
    if raw.len() < len {
        return Err(refuse(format!(
            "record truncated: {} bytes where {len} were expected",
            raw.len()
        )));
    }
    let (head, tail) = raw.split_at(len);
    *raw = tail;
    Ok(head)
}

fn int(raw: &mut &[u8]) -> Result<i32, BoxError> {
    let bytes: [u8; 4] = split(raw, 4)?
        .try_into()
        .map_err(|_| refuse("record word".to_owned()))?;
    Ok(i32::from_be_bytes(bytes))
}

impl<'t, 'a> Record<'t, 'a> {
    /// Start reading `raw` as a value of the composite type `ty`, whose fields must be
    /// exactly `names`, in order: a store table whose shape differs from the registry's
    /// row is refused rather than read positionally.
    ///
    /// # Errors
    /// A type that is not composite, a field list that differs from `names`, or a record
    /// whose field count differs.
    pub fn read(ty: &'t Type, raw: &'a [u8], names: &[&str]) -> Result<Self, BoxError> {
        let Kind::Composite(fields) = ty.kind() else {
            return Err(Box::new(WrongType::new::<Self>(ty.clone())));
        };
        if fields.len() != names.len()
            || fields.iter().zip(names).any(|(field, name)| field.name() != *name)
        {
            return Err(refuse(format!(
                "{}.{} has fields [{}]; the registry row has [{}]",
                ty.schema(),
                ty.name(),
                fields
                    .iter()
                    .map(postgres_types::Field::name)
                    .collect::<Vec<_>>()
                    .join(", "),
                names.join(", ")
            )));
        }
        let mut raw = raw;
        let count = int(&mut raw)?;
        if usize::try_from(count).ok() != Some(fields.len()) {
            return Err(refuse(format!(
                "{}.{} record has {count} fields; its type has {}",
                ty.schema(),
                ty.name(),
                fields.len()
            )));
        }
        Ok(Self {
            ty,
            fields: fields.iter(),
            raw,
        })
    }

    /// The next field's type and bytes; `None` bytes for SQL NULL.
    fn next(&mut self) -> Result<(&'t Type, &'t str, Option<&'a [u8]>), BoxError> {
        let field = self
            .fields
            .next()
            .ok_or_else(|| refuse(format!("{} record read past its last field", self.ty.name())))?;
        let oid = u32::from_be_bytes(
            split(&mut self.raw, 4)?
                .try_into()
                .map_err(|_| refuse("record oid".to_owned()))?,
        );
        if oid != field.type_().oid() {
            return Err(refuse(format!(
                "{}.{} arrived with type oid {oid}, not {}",
                self.ty.name(),
                field.name(),
                field.type_().oid()
            )));
        }
        let len = int(&mut self.raw)?;
        let bytes = match usize::try_from(len) {
            Ok(len) => Some(split(&mut self.raw, len)?),
            // -1 is SQL NULL.
            Err(_) => None,
        };
        Ok((field.type_(), field.name(), bytes))
    }

    /// The next field decoded by its own `FromSql` (`Option<T>` for a nullable column).
    ///
    /// # Errors
    /// A type `T` does not accept, NULL for a non-optional `T`, or a malformed value.
    pub fn value<T: FromSql<'a>>(&mut self) -> Result<T, BoxError> {
        let (ty, name, bytes) = self.next()?;
        if !T::accepts(ty) {
            return Err(refuse(format!(
                "{}.{name}: {}",
                self.ty.name(),
                WrongType::new::<T>(ty.clone())
            )));
        }
        T::from_sql_nullable(ty, bytes)
            .map_err(|error| refuse(format!("{}.{name}: {error}", self.ty.name())))
    }

    /// The next field, a `timestamptz`, as microseconds since the Unix epoch.
    ///
    /// # Errors
    /// NULL, another type, or a timestamp outside the representable range.
    pub fn micros(&mut self) -> Result<i64, BoxError> {
        self.opt_micros()?
            .ok_or_else(|| refuse(format!("{} timestamp is NULL", self.ty.name())))
    }

    /// The next field, a nullable `timestamptz`, as microseconds since the Unix epoch.
    ///
    /// # Errors
    /// Another type, or a timestamp outside the representable range.
    pub fn opt_micros(&mut self) -> Result<Option<i64>, BoxError> {
        let (ty, name, bytes) = self.next()?;
        if *ty != Type::TIMESTAMPTZ {
            return Err(refuse(format!("{}.{name} is {ty}, not timestamptz", self.ty.name())));
        }
        bytes
            .map(|bytes| {
                postgres_protocol::types::timestamp_from_sql(bytes)?
                    .checked_add(POSTGRES_EPOCH_MICROS)
                    .ok_or_else(|| refuse(format!("{}.{name} is out of range", self.ty.name())))
            })
            .transpose()
    }

    /// The next field, a `jsonb` document, as its JSON text.
    ///
    /// # Errors
    /// NULL, another type, an unknown jsonb version or text that is not UTF-8.
    pub fn json(&mut self) -> Result<String, BoxError> {
        self.opt_json()?
            .ok_or_else(|| refuse(format!("{} document is NULL", self.ty.name())))
    }

    /// The next field, a nullable `jsonb` document, as its JSON text.
    ///
    /// # Errors
    /// Another type, an unknown jsonb version or text that is not UTF-8.
    pub fn opt_json(&mut self) -> Result<Option<String>, BoxError> {
        let (ty, name, bytes) = self.next()?;
        if *ty != Type::JSONB {
            return Err(refuse(format!("{}.{name} is {ty}, not jsonb", self.ty.name())));
        }
        bytes
            .map(|bytes| match bytes.split_first() {
                Some((&JSONB_VERSION, text)) => Ok(std::str::from_utf8(text)?.to_owned()),
                _ => Err(refuse(format!(
                    "{}.{name} has an unknown jsonb version",
                    self.ty.name()
                ))),
            })
            .transpose()
    }

    /// Require that every field was read and no byte is left over.
    ///
    /// # Errors
    /// Unread fields or trailing bytes.
    pub fn finish(self) -> Result<(), BoxError> {
        if self.fields.len() != 0 || !self.raw.is_empty() {
            return Err(refuse(format!(
                "{} record has {} unread field(s) and {} trailing byte(s)",
                self.ty.name(),
                self.fields.len(),
                self.raw.len()
            )));
        }
        Ok(())
    }
}
