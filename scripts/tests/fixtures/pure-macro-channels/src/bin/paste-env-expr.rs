// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

fn from_selected() {}
macro_rules! generated_selector {
    ($name:tt) => { paste::expr! { [<from_ env!($name)>]() } };
}
fn main() { generated_selector!("PSE_GUARDED_MACRO_PRESENT"); }
