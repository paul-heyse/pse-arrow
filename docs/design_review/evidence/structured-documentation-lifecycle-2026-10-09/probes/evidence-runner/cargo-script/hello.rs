#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
[dependencies]
---
fn main() { println!("hello from script, pkg={}", env!("CARGO_PKG_NAME")); }
