// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrowed protobuf preflight. No prost Value or owned string is allocated
//! until the original endpoint's cumulative occurrence envelope admits it.
use super::request::ResponseLimits;

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Varint,
    Fixed64,
    String,
    Blob,
    FixedUuid,
    Message(&'static str),
}
#[derive(Default)]
pub(super) struct Counts {
    wire: usize,
    nodes: usize,
    items: usize,
    strings: usize,
    blobs: usize,
}
impl Counts {
    pub(super) fn check(
        &mut self,
        bytes: &[u8],
        schema: Option<&str>,
        limits: ResponseLimits,
    ) -> Result<(), &'static str> {
        self.wire = self
            .wire
            .checked_add(bytes.len())
            .ok_or("response wire count overflow")?;
        if self.wire > limits.wire_bytes {
            return Err("aggregate gRPC response exceeds wire envelope");
        }
        self.message(
            bytes,
            schema.ok_or("unrecognized gRPC response recipient")?,
            0,
            limits,
        )
    }
    fn own_nodes(&mut self, count: usize, limits: ResponseLimits) -> Result<(), &'static str> {
        self.nodes = self
            .nodes
            .checked_add(count)
            .ok_or("response node count overflow")?;
        if self.nodes > limits.nodes {
            return Err("gRPC response exceeds node envelope");
        }
        Ok(())
    }
    fn message(
        &mut self,
        mut bytes: &[u8],
        schema: &str,
        depth: usize,
        limits: ResponseLimits,
    ) -> Result<(), &'static str> {
        if depth > limits.depth {
            return Err("gRPC response exceeds depth envelope");
        }
        // Charge the owned message once. Its ordinary field tags and scalar
        // fields are inline storage, not another owned Value/key allocation.
        // Repeated non-message fields additionally allocate vector slots.
        self.own_nodes(1, limits)?;
        let mut items = 0usize;
        while !bytes.is_empty() {
            let key = varint(&mut bytes)?;
            let tag = u32::try_from(key >> 3).map_err(|_| "invalid protobuf tag")?;
            if tag == 0 {
                return Err("zero protobuf tag");
            }
            let shape = super::wire_schema::field(schema, tag);
            if let Some((kind, true)) = shape {
                if !matches!(kind, Kind::Message(_)) {
                    self.own_nodes(1, limits)?;
                }
                items = items.checked_add(1).ok_or("response item count overflow")?;
                self.items = self
                    .items
                    .checked_add(1)
                    .ok_or("response item count overflow")?;
                if items > limits.container_items || self.items > limits.nodes {
                    return Err("gRPC response exceeds container envelope");
                }
            }
            match key & 7 {
                0 => {
                    varint(&mut bytes)?;
                }
                1 => {
                    take(&mut bytes, 8)?;
                }
                5 => {
                    take(&mut bytes, 4)?;
                }
                2 => {
                    let len = usize::try_from(varint(&mut bytes)?)
                        .map_err(|_| "protobuf length overflow")?;
                    let value = take(&mut bytes, len)?;
                    if let Some((kind, true)) = shape {
                        let count = match kind {
                            Kind::Fixed64 if len % 8 == 0 => len / 8,
                            Kind::Varint => {
                                let mut packed = value;
                                let mut count = 0usize;
                                while !packed.is_empty() {
                                    varint(&mut packed)?;
                                    count = count
                                        .checked_add(1)
                                        .ok_or("packed protobuf item overflow")?;
                                }
                                count
                            }
                            Kind::Fixed64 => return Err("malformed packed protobuf scalar"),
                            _ => 0,
                        };
                        if matches!(kind, Kind::Fixed64 | Kind::Varint) {
                            self.own_nodes(count.saturating_sub(1), limits)?;
                        }
                        items = items
                            .checked_add(count.saturating_sub(1))
                            .ok_or("response item count overflow")?;
                        self.items = self
                            .items
                            .checked_add(count.saturating_sub(1))
                            .ok_or("response item count overflow")?;
                        if items > limits.container_items || self.items > limits.nodes {
                            return Err("gRPC response exceeds packed container envelope");
                        }
                    }
                    match shape.map(|(kind, _)| kind) {
                        Some(Kind::Message(nested)) => {
                            self.message(value, nested, depth + 1, limits)?
                        }
                        Some(Kind::String) => {
                            self.strings = self
                                .strings
                                .checked_add(len)
                                .ok_or("response string count overflow")?;
                            if len > limits.string_length || self.strings > limits.string_bytes {
                                return Err("gRPC response exceeds string envelope");
                            }
                            std::str::from_utf8(value).map_err(|_| "invalid protobuf UTF8")?;
                        }
                        Some(Kind::FixedUuid) => {
                            // Native protocol identities are fixed envelopes,
                            // not application blob payloads. Nodes and encoded
                            // bytes still charge every occurrence.
                            if len != 16 {
                                return Err("invalid native UUID length");
                            }
                        }
                        Some(Kind::Blob) => {
                            self.blobs = self
                                .blobs
                                .checked_add(len)
                                .ok_or("response blob count overflow")?;
                            if len > limits.blob_length || self.blobs > limits.blob_bytes {
                                return Err("gRPC response exceeds blob envelope");
                            }
                        }
                        // Unknown fields are bounded by encoded length; prost skips
                        // them without building a nested owned representation.
                        _ => {}
                    }
                }
                _ => return Err("unsupported protobuf wire type"),
            }
        }
        Ok(())
    }
}
fn take<'a>(bytes: &mut &'a [u8], len: usize) -> Result<&'a [u8], &'static str> {
    let (value, rest) = bytes
        .split_at_checked(len)
        .ok_or("truncated protobuf field")?;
    *bytes = rest;
    Ok(value)
}
fn varint(bytes: &mut &[u8]) -> Result<u64, &'static str> {
    let mut value = 0u64;
    for shift in (0..70).step_by(7) {
        let byte = *take(bytes, 1)?.first().ok_or("truncated protobuf varint")?;
        if shift == 63 && byte > 1 {
            return Err("protobuf varint overflow");
        }
        value |= u64::from(byte & 127) << shift;
        if byte & 128 == 0 {
            return Ok(value);
        }
    }
    Err("protobuf varint overflow")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> ResponseLimits {
        ResponseLimits {
            wire_bytes: 1024,
            nodes: 32,
            depth: 8,
            container_items: 4,
            string_bytes: 8,
            string_length: 8,
            blob_bytes: 8,
            blob_length: 8,
        }
    }
    #[test]
    fn cumulative_frames_and_occurrences_are_bounded_before_decode() {
        let string = [0x3a, 4, b'a', b'b', b'c', b'd'];
        let mut counts = Counts::default();
        assert!(counts.check(&string, Some("Value"), limits()).is_ok());
        assert!(counts.check(&string, Some("Value"), limits()).is_ok());
        assert!(counts.check(&string, Some("Value"), limits()).is_err());
    }
    #[test]
    fn empty_repeated_strings_and_packed_scalars_still_charge_owned_slots() {
        let mut envelope = limits();
        envelope.nodes = 2;
        // A scalar bool is one Value allocation; inline tag/value bytes do
        // not manufacture a second allocation.
        assert!(
            Counts::default()
                .check(&[0x18, 1], Some("Value"), envelope)
                .is_ok()
        );
        // Empty strings consume no byte allowance but each vector slot owns
        // String storage; two slots plus the message exceed two nodes.
        assert!(
            Counts::default()
                .check(&[0x22, 0, 0x22, 0], Some("ServerCapabilities"), envelope)
                .is_err()
        );
        // Packed enum values likewise own individual inline vector slots.
        assert!(
            Counts::default()
                .check(&[0x22, 2, 1, 1], Some("QueryRequest"), envelope)
                .is_err()
        );
    }
    #[test]
    fn malformed_and_excessively_nested_values_are_refused() {
        assert!(
            Counts::default()
                .check(&[0x3a, 10], Some("Value"), limits())
                .is_err()
        );
        assert!(
            Counts::default()
                .check(&[0x3a, 1, 255], Some("Value"), limits())
                .is_err()
        );
        let mut value = vec![0x20, 1];
        for _ in 0..10 {
            let mut array = vec![10, value.len() as u8];
            array.extend(value);
            value = vec![0xa2, 1, array.len() as u8];
            value.extend(array);
        }
        assert!(
            Counts::default()
                .check(&value, Some("Value"), limits())
                .is_err()
        );
    }
}
