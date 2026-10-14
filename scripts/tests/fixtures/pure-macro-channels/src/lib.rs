// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#[cfg(test)]
mod tests {
    use pest::Parser as _;

    paste::paste! { fn [<normal_ paste>]() -> usize { 3 } }
    paste::item! { fn [<normal_ item>]() -> usize { 7 } }

    #[derive(pest_derive::Parser)]
    #[grammar_inline = "digits = { SOI ~ ASCII_DIGIT+ ~ EOI }"]
    struct Digits;

    mod concatenated {
        #[derive(pest_derive::Parser)]
        #[grammar_inline = "start = { SOI ~ word ~ EOI }"]
        #[grammar_inline = "word = { ASCII_ALPHA+ }"]
        pub struct Words;
    }

    #[test]
    fn normal_paste_entries_and_inline_grammar_keep_library_semantics() {
        assert_eq!(normal_paste(), 3);
        assert_eq!(paste::expr! { [<normal_ item>]() }, 7);
        assert!(Digits::parse(Rule::digits, "1042").is_ok());
        assert!(Digits::parse(Rule::digits, "104x").is_err());
        assert!(concatenated::Words::parse(concatenated::Rule::start, "valid").is_ok());
        assert!(concatenated::Words::parse(concatenated::Rule::start, "123").is_err());
    }

    #[test]
    fn python_literals_preserve_nested_values_complex_bytes_and_refusals() {
        let source = "{'items': [True, None, (4,)], 'bytes': b'\\x41', 2-5j: {3}}";
        let value: py_literal::Value = source.parse().unwrap();
        let formatted = value.to_string();
        assert_eq!(formatted.parse::<py_literal::Value>().unwrap(), value);
        assert_eq!("b'\\x41'".parse::<py_literal::Value>().unwrap(), py_literal::Value::Bytes(vec![65]));
        for invalid in ["unknown", "[1,", "{'key':}", "1 + call()"] {
            assert!(invalid.parse::<py_literal::Value>().is_err(), "{invalid}");
        }
    }
}
