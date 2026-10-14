// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

macro_rules! generated_grammar {
    ($path:tt) => {
        #[derive(pest_derive::Parser)]
        #[grammar = $path]
        struct GeneratedFileGrammar;
    };
}
generated_grammar!("must-not-be-read.pest");
fn main() {}
