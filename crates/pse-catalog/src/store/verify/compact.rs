// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Allocation preflight for the pinned Parquet parser, not a semantic decoder.
//!
//! The compact protocol is specified at
//! <https://github.com/apache/thrift/blob/master/doc/specs/thrift-compact-protocol.md>.
//! parquet 59.3.0 `read_thrift_vec` allocates from collection counts before reading an
//! element; `schema_from_array_helper` does likewise for `num_children`. Check these
//! extents against actual encoded bytes and the flattened schema before the library
//! allocates. The library still decodes every schema, column, statistic and value.

use super::admission;
use crate::CatalogError;

const MAX_DEPTH: usize = 64;

pub(super) fn preflight(bytes: &[u8]) -> Result<usize, CatalogError> {
    let mut scan = Scan {
        bytes,
        at: 0,
        items: 0,
        children: Vec::new(),
        saw_schema: false,
    };
    scan.structure(0, Scope::Footer)?;
    if scan.at != bytes.len() {
        return Err(refused("trailing bytes after compact footer"));
    }
    if !scan.saw_schema || scan.children.is_empty() {
        return Err(refused("flattened schema is absent"));
    }
    let mut pending = Vec::new();
    for (index, children) in scan.children.iter().copied().enumerate() {
        if index != 0 {
            while pending.last() == Some(&0) {
                pending.pop();
            }
            let parent = pending
                .last_mut()
                .ok_or_else(|| refused("flattened schema has excess elements"))?;
            *parent -= 1;
        }
        if children != 0 {
            pending.push(children);
        }
        if pending.len() > MAX_DEPTH {
            return Err(refused("flattened schema exceeds supported nesting depth"));
        }
    }
    if pending.iter().any(|left| *left != 0) {
        return Err(refused("schema child count exceeds actual schema elements"));
    }
    scan.items
        .checked_mul(1024)
        .and_then(|items| {
            bytes
                .len()
                .checked_mul(32)
                .and_then(|bytes| items.checked_add(bytes))
        })
        .ok_or_else(super::super::encode::overflow)
}

#[derive(Clone, Copy)]
enum Scope {
    Footer,
    SchemaList,
    SchemaElement(usize),
    Other,
}
struct Scan<'a> {
    bytes: &'a [u8],
    at: usize,
    items: usize,
    children: Vec<usize>,
    saw_schema: bool,
}
impl Scan<'_> {
    fn byte(&mut self) -> Result<u8, CatalogError> {
        let byte = *self
            .bytes
            .get(self.at)
            .ok_or_else(|| refused("truncated compact value"))?;
        self.at += 1;
        Ok(byte)
    }
    fn varint(&mut self) -> Result<u64, CatalogError> {
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = self.byte()?;
            if shift == 63 && byte > 1 {
                return Err(refused("compact integer overflow"));
            }
            value |= u64::from(byte & 127) << shift;
            if byte & 128 == 0 {
                return Ok(value);
            }
        }
        Err(refused("oversized compact integer"))
    }
    fn skip(&mut self, len: usize) -> Result<(), CatalogError> {
        self.at = self
            .at
            .checked_add(len)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| refused("declared binary length exceeds actual footer bytes"))?;
        Ok(())
    }
    fn count(&self, n: u64) -> Result<usize, CatalogError> {
        let n = usize::try_from(n)
            .map_err(|_| refused("collection count exceeds addressable extent"))?;
        if n > self.bytes.len() - self.at || n > i32::MAX as usize {
            return Err(refused(
                "collection count exceeds actual remaining encoded bytes",
            ));
        }
        Ok(n)
    }
    fn structure(&mut self, depth: usize, scope: Scope) -> Result<(), CatalogError> {
        if depth > MAX_DEPTH {
            return Err(refused("compact nesting exceeds supported depth"));
        }
        let mut field = 0i64;
        let mut saw_children = false;
        loop {
            let header = self.byte()?;
            if header == 0 {
                return Ok(());
            }
            let kind = header & 15;
            field = if header >> 4 == 0 {
                zigzag(self.varint()?)
            } else {
                field
                    .checked_add(i64::from(header >> 4))
                    .ok_or_else(|| refused("field id overflow"))?
            };
            let child_scope = if matches!(scope, Scope::Footer) && field == 2 {
                if self.saw_schema || kind != 9 {
                    return Err(refused("invalid or duplicate schema collection"));
                }
                self.saw_schema = true;
                Scope::SchemaList
            } else {
                Scope::Other
            };
            if let Scope::SchemaElement(index) = scope
                && field == 5
            {
                if saw_children || kind != 5 {
                    return Err(refused("invalid or duplicate schema child count"));
                }
                saw_children = true;
                let count = usize::try_from(zigzag(self.varint()?))
                    .map_err(|_| refused("negative schema child count"))?;
                if count > self.children.len() {
                    return Err(refused(
                        "schema child count exceeds actual schema collection",
                    ));
                }
                self.children[index] = count;
            } else {
                self.value(kind, true, depth + 1, child_scope)?;
            }
        }
    }
    fn value(
        &mut self,
        kind: u8,
        inline: bool,
        depth: usize,
        scope: Scope,
    ) -> Result<(), CatalogError> {
        if depth > MAX_DEPTH {
            return Err(refused("compact nesting exceeds supported depth"));
        }
        self.items = self
            .items
            .checked_add(1)
            .ok_or_else(super::super::encode::overflow)?;
        match kind {
            1 | 2 => {
                if !inline && !matches!(self.byte()?, 1 | 2) {
                    return Err(refused("invalid collection boolean"));
                }
            }
            3 => self.skip(1)?,
            4..=6 => {
                self.varint()?;
            }
            7 => self.skip(8)?,
            8 => {
                let len = usize::try_from(self.varint()?)
                    .map_err(|_| refused("binary extent overflow"))?;
                self.skip(len)?;
            }
            9 | 10 => self.list(depth, scope)?,
            11 => {
                let encoded = self.varint()?;
                let count = self.count(
                    encoded
                        .checked_mul(2)
                        .ok_or_else(super::super::encode::overflow)?,
                )? / 2;
                if count != 0 {
                    let types = self.byte()?;
                    for _ in 0..count {
                        self.value(types >> 4, false, depth + 1, Scope::Other)?;
                        self.value(types & 15, false, depth + 1, Scope::Other)?;
                    }
                }
            }
            12 => self.structure(depth, scope)?,
            _ => return Err(refused("unsupported compact type")),
        }
        Ok(())
    }
    fn list(&mut self, depth: usize, scope: Scope) -> Result<(), CatalogError> {
        let header = self.byte()?;
        let encoded = if header >> 4 == 15 {
            self.varint()?
        } else {
            u64::from(header >> 4)
        };
        let count = self.count(encoded)?;
        let schema = matches!(scope, Scope::SchemaList);
        if schema {
            if header & 15 != 12 {
                return Err(refused("schema collection is not structs"));
            }
            self.children.resize(count, 0);
        }
        for index in 0..count {
            self.value(
                header & 15,
                false,
                depth + 1,
                if schema {
                    Scope::SchemaElement(index)
                } else {
                    Scope::Other
                },
            )?;
        }
        Ok(())
    }
}
fn zigzag(value: u64) -> i64 {
    i64::from_ne_bytes((value >> 1).to_ne_bytes()) ^ -i64::try_from(value & 1).unwrap_or(0)
}
fn refused(reason: &str) -> CatalogError {
    admission("Parquet allocation preflight", reason)
}

#[cfg(test)]
mod tests {
    use super::preflight;
    #[test]
    fn huge_collections_and_schema_counts_in_tiny_footers_fail_before_library_allocation() {
        // Footer field 2 is list<struct>. A huge declared list has no item bytes.
        assert!(preflight(&[0x29, 0xfc, 0xff, 0xff, 0xff, 0xff, 7, 0]).is_err());
        // One actual root schema struct claims 127 children; no child elements exist.
        assert!(preflight(&[0x29, 0x1c, 0x55, 0xfe, 1, 0, 0]).is_err());
        assert!(preflight(&[0x29, 0x1c, 0, 0]).is_ok());
    }
}
