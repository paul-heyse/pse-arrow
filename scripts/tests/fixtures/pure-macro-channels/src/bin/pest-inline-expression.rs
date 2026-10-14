// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#[derive(pest_derive::Parser)]
#[grammar_inline = include_str!("must-not-be-read.pest")]
struct FileExpressionGrammar;
fn main() {}
