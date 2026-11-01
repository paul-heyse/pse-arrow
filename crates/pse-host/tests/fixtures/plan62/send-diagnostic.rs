// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "compiler-input.rs"]
mod fixture;
fn require_send(_: impl Send) {}
pub fn selected_question() {
    require_send(fixture::suspended());
}
