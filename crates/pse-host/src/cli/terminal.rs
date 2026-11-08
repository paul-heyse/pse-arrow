// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local presentation policy, separate from the original command outcome.
use crate::envelope::OutputFormat;
use std::io::{self, IsTerminal as _};

pub(super) struct Policy {
    pub(super) interactive: bool,
    pub(super) color: bool,
}
impl Policy {
    pub(super) fn stderr(format: OutputFormat) -> Self {
        let graphics = io::stderr().is_terminal()
            && std::env::var_os("NO_GRAPHICS").is_none()
            && !std::env::var_os("TERM").is_some_and(|value| value == "dumb");
        let color = !std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        Self::resolve(format, graphics, color)
    }

    fn resolve(format: OutputFormat, graphics: bool, color: bool) -> Self {
        let interactive = format == OutputFormat::Human && graphics;
        Self {
            interactive,
            color: interactive && color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_output_and_plain_human_never_admit_terminal_drawing() {
        for format in [OutputFormat::Json, OutputFormat::Ndjson] {
            let policy = Policy::resolve(format, true, true);
            assert!(!policy.interactive && !policy.color);
        }
        let plain = Policy::resolve(OutputFormat::Human, false, true);
        assert!(!plain.interactive && !plain.color);
        let monochrome = Policy::resolve(OutputFormat::Human, true, false);
        assert!(monochrome.interactive && !monochrome.color);
    }
}
