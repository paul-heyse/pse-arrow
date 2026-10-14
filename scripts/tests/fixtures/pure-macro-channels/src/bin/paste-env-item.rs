// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

macro_rules! generated_selector {
    ($name:tt) => { paste::item! { fn [<from_ env!($name)>]() {} } };
}
generated_selector!("PSE_GUARDED_MACRO_PRESENT");
fn main() {}
