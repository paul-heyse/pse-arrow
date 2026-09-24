// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Python attribute spellings are projections; declared wire names remain unchanged.
use std::collections::BTreeSet;

pub(super) fn fields<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let names = names.into_iter().collect::<Vec<_>>();
    let mut occupied = names
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    names
        .into_iter()
        .map(|name| {
            if !keyword(name) {
                return name.to_owned();
            }
            let mut identifier = format!("{name}_");
            while occupied.contains(&identifier) {
                identifier.push('_');
            }
            occupied.insert(identifier.clone());
            identifier
        })
        .collect()
}
fn keyword(name: &str) -> bool {
    matches!(
        name,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}
#[cfg(test)]
mod tests {
    use super::fields;
    #[test]
    fn keywords_avoid_every_declared_name_without_changing_legal_attributes() {
        assert_eq!(
            fields([
                "class", "class_", "class__", "from", "from_", "match", "type"
            ]),
            [
                "class___", "class_", "class__", "from__", "from_", "match", "type"
            ]
        );
        assert_eq!(
            fields(["class_", "class", "class__"]),
            ["class_", "class___", "class__"]
        );
    }
}
