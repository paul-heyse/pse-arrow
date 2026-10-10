#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
[dependencies]
pse-ids = { path = "/home/paul/pse-arrow/crates/pse-ids" }
---
fn main() { println!("pse_ids linked: {}", std::any::type_name::<pse_ids::ContentHash>()); }
