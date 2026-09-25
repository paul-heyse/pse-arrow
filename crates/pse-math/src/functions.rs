// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

pub use pse_quantity::functions::{Function, Implementation, UnaryFunction};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_function_vocabulary_roundtrips_without_legacy_fallback() {
        for &function in Function::all() {
            assert_eq!(Function::parse(function.as_str()), Some(function));
        }
        for name in [
            "smooth_max",
            "smooth_min",
            "smooth_abs",
            "safe_sqrt",
            "safe_log",
            "tanh",
            "erf",
            "weighted_mean",
        ] {
            assert_eq!(Function::parse(name), None);
        }
        assert_eq!(
            Function::Convert.implementation(),
            Implementation::Unavailable
        );
        assert_eq!(
            Function::Broadcast.implementation(),
            Implementation::Unavailable
        );
    }
}
