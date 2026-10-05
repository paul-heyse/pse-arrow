// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native invariant declarations bound by the actual DataFusion session.
pub(crate) fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
pub(crate) fn table(name: &str) -> String {
    name.split('.')
        .map(identifier)
        .collect::<Vec<_>>()
        .join(".")
}
pub(crate) fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
pub(crate) fn columns(keys: &[&str], alias: &str) -> String {
    keys.iter()
        .map(|key| format!("{alias}.{}", identifier(key)))
        .collect::<Vec<_>>()
        .join(", ")
}
