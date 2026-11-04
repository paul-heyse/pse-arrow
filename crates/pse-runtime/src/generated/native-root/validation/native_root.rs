use super::*;
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ApplicabilityClaim {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(v) = self.id.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("id")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(v) = self.coverage.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(2i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("coverage")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if self.owner.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("owner")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .owner.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.evidence.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(5i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("evidence")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if self.form.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("form")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.form
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.call.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(7i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("call")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.call
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(inner) = self.layer.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(10i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("layer")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.basis.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(11i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("basis")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ApplicabilityClaimView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(v) = self.id.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("id")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(v) = self.coverage.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(2i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("coverage")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if self.owner.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("owner")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .owner.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.evidence.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(5i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("evidence")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if self.form.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("form")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.form
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.call.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(7i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("call")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.call
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(inner) = self.layer.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(10i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("layer")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.basis.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(11i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("basis")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ApplicabilityInput {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.quantity_type.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("quantity_type")),
                            field_type : Some(::protovalidate_buffa::FieldType::Bytes),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .quantity_type.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ApplicabilityInputView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.quantity_type.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("quantity_type")),
                            field_type : Some(::protovalidate_buffa::FieldType::Bytes),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .quantity_type.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ApplicabilityObservation {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.claim.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("claim")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(v) = self.instance.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(2i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("instance")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        for (idx, elem) in self.inputs.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("inputs")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.outcome.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(4i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("outcome")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.permissions.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(6i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("permissions"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::ApplicabilityObservationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.claim.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("claim")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(v) = self.instance.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(2i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("instance")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        for (idx, elem) in self.inputs.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("inputs")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.outcome.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(4i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("outcome")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.permissions.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(6i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("permissions"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ApplicabilityPermission {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("id")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.id
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.scope.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("scope")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .scope.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(inner) = self.target_kind.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("target_kind"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ApplicabilityPermissionView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("id")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.id
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.scope.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("scope")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .scope.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(inner) = self.target_kind.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("target_kind"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for Array16U8 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 16usize,
                            self.values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})", 16usize,
                            self.values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::Array16U8View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 16usize,
                            self.values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})", 16usize,
                            self.values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for Array2U64 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 2usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})", 2usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 2usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})", 2usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::Array2U64View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 2usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})", 2usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 2usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})", 2usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for Array3Array16U8 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})", 3usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})", 3usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.values.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("values")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::Array3Array16U8View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})", 3usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("values")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(18i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_items")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})", 3usize, self
                            .values.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.values.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("values")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for CapturedBuildInfo {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::CapturedBuildInfoView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticCode {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "host.io",
                "host.manager",
                "host.ownership",
                "host.deadline",
                "host.unknown_launch",
                "host.settlement",
                "host.environment",
                "host.stale_tools",
                "fixture.unprepared",
                "correctness.graph",
                "correctness.admission",
                "correctness.io",
                "authoring.parse",
                "authoring.parse.ambiguous_unary_power",
                "authoring.parse.budget",
                "authoring.parse.document_io",
                "authoring.parse.missing_id",
                "authoring.parse.nonfinite_number",
                "authoring.parse.syntax",
                "authoring.parse.unknown_key",
                "authoring.parse.unresolved_target",
                "authoring.pkg.unresolved",
                "authoring.pkg.version_conflict",
                "authoring.reference",
                "authoring.reference.contract",
                "authoring.reference.derived_write",
                "authoring.reference.rename_named",
                "authoring.reference.unknown_row_key",
                "capability.backend",
                "compile.discretization",
                "compile.feature",
                "compile.law",
                "compile.math",
                "compile.math.cyclic_expression",
                "compile.math.domain_violation_static",
                "compile.math.quantity_operation_unsupported",
                "compile.math.unit_inconsistent",
                "compile.property",
                "config.invalid",
                "internal.invariant",
                "kernel.unbound_parameter",
                "plan.initialization",
                "rule.float_key",
                "rule.head_schema_mismatch",
                "runtime.cancelled",
                "runtime.infrastructure",
                "runtime.resource_limit",
                "runtime.timeout",
                "schema.admission",
                "schema.arrow",
                "schema.codegen",
                "schema.contract_mismatch",
                "schema.duplicate_declaration",
                "schema.enum_member",
                "schema.extension_metadata",
                "schema.extension_type",
                "schema.fingerprint_mismatch",
                "schema.invalid_declaration",
                "schema.invalid_key",
                "schema.missing_granularity",
                "schema.missing_snapshot_class",
                "schema.nullability",
                "schema.ordinal_range",
                "schema.rule_float_key",
                "schema.rule_stratification",
                "schema.storage",
                "schema.unknown_metadata",
                "schema.unknown_reference",
                "schema.unknown_registry",
                "schema.unsupported_layout",
                "schema.version_mismatch",
                "solve.evaluation_error",
                "solve.infeasible",
                "solve.limit",
                "solve.locally_infeasible",
                "solve.solver_error",
                "solve.unbounded",
                "template.guard_undecidable",
                "user.model",
                "validation.invariant",
                "authoring.budget",
                "authoring.contract",
                "authoring.derived_write",
                "authoring.document_io",
                "authoring.missing_id",
                "authoring.package_unresolved",
                "authoring.package_version_conflict",
                "authoring.rename_named",
                "authoring.schema_version",
                "authoring.syntax",
                "authoring.unknown_key",
                "authoring.unknown_row_key",
                "authoring.unresolved_target",
                "candidate.evaluation_failed",
                "compiler.cancelled",
                "compiler.limit",
                "compiler.math",
                "compiler.missing",
                "compiler.modeling",
                "compiler.structure",
                "compiler.syntax",
                "domain.potential_evaluation_error",
                "equation.canceling_terms",
                "equation.large_residual",
                "equation.mismatched_term",
                "equation.term_evaluation_failed",
                "fit.candidate_validation",
                "fit.final_evaluation",
                "fit.objective_overflow",
                "fit.response_rank",
                "jacobian.analysis_inconclusive",
                "jacobian.condition_estimate",
                "jacobian.extreme_column",
                "jacobian.extreme_entry",
                "jacobian.extreme_row",
                "jacobian.numerical_rank_deficiency",
                "jacobian.parallel_columns",
                "jacobian.parallel_rows",
                "math.applicability",
                "math.cancelled",
                "math.coefficient_range",
                "math.contract",
                "math.domain",
                "math.evaluation",
                "math.library",
                "math.limit",
                "math.native",
                "math.provider",
                "math.quantity",
                "math.range",
                "math.validity",
                "modeling.budget",
                "modeling.cancelled",
                "modeling.capability",
                "modeling.conditional_unit.admission.invalid_model",
                "modeling.conditional_unit.admission.numerical",
                "modeling.conditional_unit.admission.trial_rejected",
                "modeling.contract",
                "modeling.diagnostic_samples",
                "modeling.diagnostics",
                "modeling.domain",
                "modeling.domain.tightened",
                "modeling.dynamic.algebraic_reset.unsupported",
                "modeling.dynamic.inventory_transfer.event.unsupported",
                "modeling.initialization.attempt_limit",
                "modeling.initialization.incomplete",
                "modeling.initialization.minimum_step",
                "modeling.initialization.outcome",
                "modeling.initialization.step_precision",
                "modeling.nonlinear.attempt_limit",
                "modeling.nonlinear.cancelled",
                "modeling.nonlinear.initial_inconclusive",
                "modeling.nonlinear.time_limit",
                "modeling.objective",
                "modeling.provenance",
                "modeling.qualification.infeasibility_contradicted",
                "modeling.qualification.rejected",
                "modeling.realization",
                "modeling.staged.start",
                "modeling.study.initialization",
                "modeling.study.predecessor",
                "modeling.study.procedure",
                "modeling.trajectory.incomplete",
                "modeling.trajectory.rejected",
                "native.cancelled",
                "native.contract",
                "native.internal",
                "native.limit",
                "native.numerical",
                "native.reuse",
                "native.route_refused",
                "native.structural",
                "native.unavailable",
                "native.unsupported",
                "objective.priority_optimization_incomplete",
                "shooting.candidate_validation",
                "shooting.evaluation",
                "structural.overdetermined",
                "structural.underdetermined",
                "variable.fixed_zero",
                "variable.large_value",
                "variable.missing_value",
                "variable.near_lower_bound",
                "variable.near_upper_bound",
                "variable.nonfinite",
                "variable.only_in_inequalities",
                "variable.outside_lower_bound",
                "variable.outside_upper_bound",
                "variable.small_value",
                "variable.unused",
                "workflow.contract",
                "workflow.ephemeral_publication",
                "workflow.export_expired",
                "workflow.job_payload_version",
                "workflow.legacy_workspace",
                "workflow.operations",
                "workflow.publication_unresolved",
                "workflow.unclassified",
                "quantity.unknown_id",
                "quantity.contract_mismatch",
                "quantity.unit_conversion",
                "quantity.incompatible",
                "quantity.nonfinite",
                "workflow.input",
                "workflow.internal",
                "workflow.panic",
                "diagnostic.aggregate",
                "study.binding.target",
                "study.binding.duplicate",
                "study.binding.physical",
                "study.binding.revision",
                "study.dependency.unusable",
                "study.seed.unavailable",
                "study.seed.incompatible",
                "study.seed.internal",
                "study.operation.unsupported",
                "modeling.conditional_unit.admission.unsupported",
                "modeling.conditional_unit.admission.cancelled",
                "modeling.conditional_unit.admission.resource_limit",
                "modeling.conditional_unit.admission.nonfinite",
                "modeling.conditional_unit.admission.infrastructure",
                "modeling.conditional_unit.admission.conflict",
                "modeling.conditional_unit.admission.incompatible",
                "modeling.conditional_unit.admission.internal",
                "modeling.conditional_unit.admission.inconclusive",
                "study.policy.admission",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::DiagnosticCodeView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "host.io",
                "host.manager",
                "host.ownership",
                "host.deadline",
                "host.unknown_launch",
                "host.settlement",
                "host.environment",
                "host.stale_tools",
                "fixture.unprepared",
                "correctness.graph",
                "correctness.admission",
                "correctness.io",
                "authoring.parse",
                "authoring.parse.ambiguous_unary_power",
                "authoring.parse.budget",
                "authoring.parse.document_io",
                "authoring.parse.missing_id",
                "authoring.parse.nonfinite_number",
                "authoring.parse.syntax",
                "authoring.parse.unknown_key",
                "authoring.parse.unresolved_target",
                "authoring.pkg.unresolved",
                "authoring.pkg.version_conflict",
                "authoring.reference",
                "authoring.reference.contract",
                "authoring.reference.derived_write",
                "authoring.reference.rename_named",
                "authoring.reference.unknown_row_key",
                "capability.backend",
                "compile.discretization",
                "compile.feature",
                "compile.law",
                "compile.math",
                "compile.math.cyclic_expression",
                "compile.math.domain_violation_static",
                "compile.math.quantity_operation_unsupported",
                "compile.math.unit_inconsistent",
                "compile.property",
                "config.invalid",
                "internal.invariant",
                "kernel.unbound_parameter",
                "plan.initialization",
                "rule.float_key",
                "rule.head_schema_mismatch",
                "runtime.cancelled",
                "runtime.infrastructure",
                "runtime.resource_limit",
                "runtime.timeout",
                "schema.admission",
                "schema.arrow",
                "schema.codegen",
                "schema.contract_mismatch",
                "schema.duplicate_declaration",
                "schema.enum_member",
                "schema.extension_metadata",
                "schema.extension_type",
                "schema.fingerprint_mismatch",
                "schema.invalid_declaration",
                "schema.invalid_key",
                "schema.missing_granularity",
                "schema.missing_snapshot_class",
                "schema.nullability",
                "schema.ordinal_range",
                "schema.rule_float_key",
                "schema.rule_stratification",
                "schema.storage",
                "schema.unknown_metadata",
                "schema.unknown_reference",
                "schema.unknown_registry",
                "schema.unsupported_layout",
                "schema.version_mismatch",
                "solve.evaluation_error",
                "solve.infeasible",
                "solve.limit",
                "solve.locally_infeasible",
                "solve.solver_error",
                "solve.unbounded",
                "template.guard_undecidable",
                "user.model",
                "validation.invariant",
                "authoring.budget",
                "authoring.contract",
                "authoring.derived_write",
                "authoring.document_io",
                "authoring.missing_id",
                "authoring.package_unresolved",
                "authoring.package_version_conflict",
                "authoring.rename_named",
                "authoring.schema_version",
                "authoring.syntax",
                "authoring.unknown_key",
                "authoring.unknown_row_key",
                "authoring.unresolved_target",
                "candidate.evaluation_failed",
                "compiler.cancelled",
                "compiler.limit",
                "compiler.math",
                "compiler.missing",
                "compiler.modeling",
                "compiler.structure",
                "compiler.syntax",
                "domain.potential_evaluation_error",
                "equation.canceling_terms",
                "equation.large_residual",
                "equation.mismatched_term",
                "equation.term_evaluation_failed",
                "fit.candidate_validation",
                "fit.final_evaluation",
                "fit.objective_overflow",
                "fit.response_rank",
                "jacobian.analysis_inconclusive",
                "jacobian.condition_estimate",
                "jacobian.extreme_column",
                "jacobian.extreme_entry",
                "jacobian.extreme_row",
                "jacobian.numerical_rank_deficiency",
                "jacobian.parallel_columns",
                "jacobian.parallel_rows",
                "math.applicability",
                "math.cancelled",
                "math.coefficient_range",
                "math.contract",
                "math.domain",
                "math.evaluation",
                "math.library",
                "math.limit",
                "math.native",
                "math.provider",
                "math.quantity",
                "math.range",
                "math.validity",
                "modeling.budget",
                "modeling.cancelled",
                "modeling.capability",
                "modeling.conditional_unit.admission.invalid_model",
                "modeling.conditional_unit.admission.numerical",
                "modeling.conditional_unit.admission.trial_rejected",
                "modeling.contract",
                "modeling.diagnostic_samples",
                "modeling.diagnostics",
                "modeling.domain",
                "modeling.domain.tightened",
                "modeling.dynamic.algebraic_reset.unsupported",
                "modeling.dynamic.inventory_transfer.event.unsupported",
                "modeling.initialization.attempt_limit",
                "modeling.initialization.incomplete",
                "modeling.initialization.minimum_step",
                "modeling.initialization.outcome",
                "modeling.initialization.step_precision",
                "modeling.nonlinear.attempt_limit",
                "modeling.nonlinear.cancelled",
                "modeling.nonlinear.initial_inconclusive",
                "modeling.nonlinear.time_limit",
                "modeling.objective",
                "modeling.provenance",
                "modeling.qualification.infeasibility_contradicted",
                "modeling.qualification.rejected",
                "modeling.realization",
                "modeling.staged.start",
                "modeling.study.initialization",
                "modeling.study.predecessor",
                "modeling.study.procedure",
                "modeling.trajectory.incomplete",
                "modeling.trajectory.rejected",
                "native.cancelled",
                "native.contract",
                "native.internal",
                "native.limit",
                "native.numerical",
                "native.reuse",
                "native.route_refused",
                "native.structural",
                "native.unavailable",
                "native.unsupported",
                "objective.priority_optimization_incomplete",
                "shooting.candidate_validation",
                "shooting.evaluation",
                "structural.overdetermined",
                "structural.underdetermined",
                "variable.fixed_zero",
                "variable.large_value",
                "variable.missing_value",
                "variable.near_lower_bound",
                "variable.near_upper_bound",
                "variable.nonfinite",
                "variable.only_in_inequalities",
                "variable.outside_lower_bound",
                "variable.outside_upper_bound",
                "variable.small_value",
                "variable.unused",
                "workflow.contract",
                "workflow.ephemeral_publication",
                "workflow.export_expired",
                "workflow.job_payload_version",
                "workflow.legacy_workspace",
                "workflow.operations",
                "workflow.publication_unresolved",
                "workflow.unclassified",
                "quantity.unknown_id",
                "quantity.contract_mismatch",
                "quantity.unit_conversion",
                "quantity.incompatible",
                "quantity.nonfinite",
                "workflow.input",
                "workflow.internal",
                "workflow.panic",
                "diagnostic.aggregate",
                "study.binding.target",
                "study.binding.duplicate",
                "study.binding.physical",
                "study.binding.revision",
                "study.dependency.unusable",
                "study.seed.unavailable",
                "study.seed.incompatible",
                "study.seed.internal",
                "study.operation.unsupported",
                "modeling.conditional_unit.admission.unsupported",
                "modeling.conditional_unit.admission.cancelled",
                "modeling.conditional_unit.admission.resource_limit",
                "modeling.conditional_unit.admission.nonfinite",
                "modeling.conditional_unit.admission.infrastructure",
                "modeling.conditional_unit.admission.conflict",
                "modeling.conditional_unit.admission.incompatible",
                "modeling.conditional_unit.admission.internal",
                "modeling.conditional_unit.admission.inconclusive",
                "study.policy.admission",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticNonfiniteObservation {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["nan", "positive_infinity", "negative_infinity"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::DiagnosticNonfiniteObservationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["nan", "positive_infinity", "negative_infinity"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticObservation {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::oneof::diagnostic_observation::Kind::Missing(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("missing")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::diagnostic_observation::Kind::Real(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("real")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::diagnostic_observation::Kind::Nonfinite(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(3i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("nonfinite")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::diagnostic_observation::Kind::Physical(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(4i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::diagnostic_observation::Kind::Integer(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(5i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("integer")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::diagnostic_observation::Kind::Boolean(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(6i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("boolean")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::diagnostic_observation::Kind::Text(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(7i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::diagnostic_observation::Kind::Contracts(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(8i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("contracts")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            None => {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                None, field_name :
                                Some(::std::borrow::Cow::Borrowed("kind")), field_type :
                                None, key_type : None, value_type : None, subscript : None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("required"),
                        message: ::std::borrow::Cow::Borrowed(
                            "at least one variant is required",
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::DiagnosticObservationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Missing(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("missing")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Real(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("real")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Nonfinite(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(3i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("nonfinite")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Physical(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(4i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Integer(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(5i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("integer")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Boolean(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(6i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("boolean")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Text(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(7i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::diagnostic_observation::Kind::Contracts(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(8i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("contracts")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            None => {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                None, field_name :
                                Some(::std::borrow::Cow::Borrowed("kind")), field_type :
                                None, key_type : None, value_type : None, subscript : None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("required"),
                        message: ::std::borrow::Cow::Borrowed(
                            "at least one variant is required",
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative1 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative1View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative2 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                let __cel_this = (self.value as u64);
                if !({
                    let __outer = __cel_evaluation_error.take();
                    let __left = ((__cel_this) <= (9218868437227405311u64 as u64));
                    let __left_error = __cel_evaluation_error.take();
                    let (__result, __error) = if __left_error.is_none() && __left == true
                    {
                        (__left, None)
                    } else {
                        let __right = {
                            let __outer = __cel_evaluation_error.take();
                            let __left = ((__cel_this)
                                >= (9223372036854775808u64 as u64));
                            let __left_error = __cel_evaluation_error.take();
                            let (__result, __error) = if __left_error.is_none()
                                && __left == false
                            {
                                (__left, None)
                            } else {
                                let __right = ((__cel_this)
                                    <= (18442240474082181119u64 as u64));
                                let __right_error = __cel_evaluation_error.take();
                                if __right_error.is_none() && __right == false {
                                    (__right, None)
                                } else {
                                    (__right, __left_error.or(__right_error))
                                }
                            };
                            __cel_evaluation_error.set(__outer.or(__error));
                            __result
                        };
                        let __right_error = __cel_evaluation_error.take();
                        if __right_error.is_none() && __right == true {
                            (__right, None)
                        } else {
                            (__right, __left_error.or(__right_error))
                        }
                    };
                    __cel_evaluation_error.set(__outer.or(__error));
                    __result
                }) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(1i32), field_name :
                                    Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Fixed64), key_type :
                                    None, value_type : None, subscript : None, },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(23), field_name :
                                    Some(::std::borrow::Cow::Borrowed("cel")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Message), key_type :
                                    None, value_type : None, subscript :
                                    Some(::protovalidate_buffa::Subscript::Index(0u64)), }
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("float64.finite"),
                            message: ::std::borrow::Cow::Borrowed(
                                "binary64 value must be finite",
                            ),
                            for_key: false,
                        });
                }
            }
            if let Some(reason) = __cel_evaluation_error.get() {
                violations.truncate(__cel_violation_start);
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath::default(),
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("__cel_runtime_error__"),
                        message: ::std::borrow::Cow::Borrowed(reason),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative2View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                let __cel_this = (self.value as u64);
                if !({
                    let __outer = __cel_evaluation_error.take();
                    let __left = ((__cel_this) <= (9218868437227405311u64 as u64));
                    let __left_error = __cel_evaluation_error.take();
                    let (__result, __error) = if __left_error.is_none() && __left == true
                    {
                        (__left, None)
                    } else {
                        let __right = {
                            let __outer = __cel_evaluation_error.take();
                            let __left = ((__cel_this)
                                >= (9223372036854775808u64 as u64));
                            let __left_error = __cel_evaluation_error.take();
                            let (__result, __error) = if __left_error.is_none()
                                && __left == false
                            {
                                (__left, None)
                            } else {
                                let __right = ((__cel_this)
                                    <= (18442240474082181119u64 as u64));
                                let __right_error = __cel_evaluation_error.take();
                                if __right_error.is_none() && __right == false {
                                    (__right, None)
                                } else {
                                    (__right, __left_error.or(__right_error))
                                }
                            };
                            __cel_evaluation_error.set(__outer.or(__error));
                            __result
                        };
                        let __right_error = __cel_evaluation_error.take();
                        if __right_error.is_none() && __right == true {
                            (__right, None)
                        } else {
                            (__right, __left_error.or(__right_error))
                        }
                    };
                    __cel_evaluation_error.set(__outer.or(__error));
                    __result
                }) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(1i32), field_name :
                                    Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Fixed64), key_type :
                                    None, value_type : None, subscript : None, },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(23), field_name :
                                    Some(::std::borrow::Cow::Borrowed("cel")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Message), key_type :
                                    None, value_type : None, subscript :
                                    Some(::protovalidate_buffa::Subscript::Index(0u64)), }
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("float64.finite"),
                            message: ::std::borrow::Cow::Borrowed(
                                "binary64 value must be finite",
                            ),
                            for_key: false,
                        });
                }
            }
            if let Some(reason) = __cel_evaluation_error.get() {
                violations.truncate(__cel_violation_start);
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath::default(),
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("__cel_runtime_error__"),
                        message: ::std::borrow::Cow::Borrowed(reason),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative3 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.value.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative3View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.value.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative4 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.value.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative4View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.value.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative5 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative5View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative6 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative6View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative7 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative7View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for diagnostic_observation::Alternative8 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        for (idx, elem) in self.value.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::diagnostic_observation::Alternative8View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        for (idx, elem) in self.value.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticOperandContract {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.quantity.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("quantity")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.indices.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(2i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("indices")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::DiagnosticOperandContractView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.quantity.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("quantity")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.indices.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(2i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("indices")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticOperationId {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.process_instance.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("process_instance"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::DiagnosticOperationIdView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.process_instance.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("process_instance"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticPhysicalObservation {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.quantity.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("quantity")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .quantity.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.unit.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("unit")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.unit
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.context.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("context")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .context.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                let __cel_this = (self.magnitude as u64);
                if !({
                    let __outer = __cel_evaluation_error.take();
                    let __left = ((__cel_this) <= (9218868437227405311u64 as u64));
                    let __left_error = __cel_evaluation_error.take();
                    let (__result, __error) = if __left_error.is_none() && __left == true
                    {
                        (__left, None)
                    } else {
                        let __right = {
                            let __outer = __cel_evaluation_error.take();
                            let __left = ((__cel_this)
                                >= (9223372036854775808u64 as u64));
                            let __left_error = __cel_evaluation_error.take();
                            let (__result, __error) = if __left_error.is_none()
                                && __left == false
                            {
                                (__left, None)
                            } else {
                                let __right = ((__cel_this)
                                    <= (18442240474082181119u64 as u64));
                                let __right_error = __cel_evaluation_error.take();
                                if __right_error.is_none() && __right == false {
                                    (__right, None)
                                } else {
                                    (__right, __left_error.or(__right_error))
                                }
                            };
                            __cel_evaluation_error.set(__outer.or(__error));
                            __result
                        };
                        let __right_error = __cel_evaluation_error.take();
                        if __right_error.is_none() && __right == true {
                            (__right, None)
                        } else {
                            (__right, __left_error.or(__right_error))
                        }
                    };
                    __cel_evaluation_error.set(__outer.or(__error));
                    __result
                }) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(1i32), field_name :
                                    Some(::std::borrow::Cow::Borrowed("magnitude")), field_type
                                    : Some(::protovalidate_buffa::FieldType::Fixed64), key_type
                                    : None, value_type : None, subscript : None, },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(23), field_name :
                                    Some(::std::borrow::Cow::Borrowed("cel")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Message), key_type :
                                    None, value_type : None, subscript :
                                    Some(::protovalidate_buffa::Subscript::Index(0u64)), }
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("float64.finite"),
                            message: ::std::borrow::Cow::Borrowed(
                                "binary64 value must be finite",
                            ),
                            for_key: false,
                        });
                }
            }
            if let Some(reason) = __cel_evaluation_error.get() {
                violations.truncate(__cel_violation_start);
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath::default(),
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("__cel_runtime_error__"),
                        message: ::std::borrow::Cow::Borrowed(reason),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::DiagnosticPhysicalObservationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.quantity.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("quantity")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .quantity.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.unit.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("unit")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.unit
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.context.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("context")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .context.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                let __cel_this = (self.magnitude as u64);
                if !({
                    let __outer = __cel_evaluation_error.take();
                    let __left = ((__cel_this) <= (9218868437227405311u64 as u64));
                    let __left_error = __cel_evaluation_error.take();
                    let (__result, __error) = if __left_error.is_none() && __left == true
                    {
                        (__left, None)
                    } else {
                        let __right = {
                            let __outer = __cel_evaluation_error.take();
                            let __left = ((__cel_this)
                                >= (9223372036854775808u64 as u64));
                            let __left_error = __cel_evaluation_error.take();
                            let (__result, __error) = if __left_error.is_none()
                                && __left == false
                            {
                                (__left, None)
                            } else {
                                let __right = ((__cel_this)
                                    <= (18442240474082181119u64 as u64));
                                let __right_error = __cel_evaluation_error.take();
                                if __right_error.is_none() && __right == false {
                                    (__right, None)
                                } else {
                                    (__right, __left_error.or(__right_error))
                                }
                            };
                            __cel_evaluation_error.set(__outer.or(__error));
                            __result
                        };
                        let __right_error = __cel_evaluation_error.take();
                        if __right_error.is_none() && __right == true {
                            (__right, None)
                        } else {
                            (__right, __left_error.or(__right_error))
                        }
                    };
                    __cel_evaluation_error.set(__outer.or(__error));
                    __result
                }) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(1i32), field_name :
                                    Some(::std::borrow::Cow::Borrowed("magnitude")), field_type
                                    : Some(::protovalidate_buffa::FieldType::Fixed64), key_type
                                    : None, value_type : None, subscript : None, },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(23), field_name :
                                    Some(::std::borrow::Cow::Borrowed("cel")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Message), key_type :
                                    None, value_type : None, subscript :
                                    Some(::protovalidate_buffa::Subscript::Index(0u64)), }
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("float64.finite"),
                            message: ::std::borrow::Cow::Borrowed(
                                "binary64 value must be finite",
                            ),
                            for_key: false,
                        });
                }
            }
            if let Some(reason) = __cel_evaluation_error.get() {
                violations.truncate(__cel_violation_start);
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath::default(),
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("__cel_runtime_error__"),
                        message: ::std::borrow::Cow::Borrowed(reason),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticRule {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "authoring.budget",
                "authoring.contract",
                "authoring.derived_write",
                "authoring.document_io",
                "authoring.missing_id",
                "authoring.package_unresolved",
                "authoring.package_version_conflict",
                "authoring.rename_named",
                "authoring.schema_version",
                "authoring.syntax",
                "authoring.unknown_key",
                "authoring.unknown_row_key",
                "authoring.unresolved_target",
                "candidate.evaluation_failed",
                "compiler.cancelled",
                "compiler.limit",
                "compiler.math",
                "compiler.missing",
                "compiler.modeling",
                "compiler.structure",
                "compiler.syntax",
                "domain.potential_evaluation_error",
                "equation.canceling_terms",
                "equation.large_residual",
                "equation.mismatched_term",
                "equation.term_evaluation_failed",
                "fit.candidate_validation",
                "fit.final_evaluation",
                "fit.objective_overflow",
                "fit.response_rank",
                "jacobian.analysis_inconclusive",
                "jacobian.condition_estimate",
                "jacobian.extreme_column",
                "jacobian.extreme_entry",
                "jacobian.extreme_row",
                "jacobian.numerical_rank_deficiency",
                "jacobian.parallel_columns",
                "jacobian.parallel_rows",
                "math.applicability",
                "math.cancelled",
                "math.coefficient_range",
                "math.contract",
                "math.domain",
                "math.evaluation",
                "math.library",
                "math.limit",
                "math.native",
                "math.provider",
                "math.quantity",
                "math.range",
                "math.validity",
                "modeling.budget",
                "modeling.cancelled",
                "modeling.capability",
                "modeling.conditional_unit.admission.invalid_model",
                "modeling.conditional_unit.admission.numerical",
                "modeling.conditional_unit.admission.trial_rejected",
                "modeling.contract",
                "modeling.diagnostic_samples",
                "modeling.diagnostics",
                "modeling.domain",
                "modeling.domain.tightened",
                "modeling.dynamic.algebraic_reset.unsupported",
                "modeling.dynamic.inventory_transfer.event.unsupported",
                "modeling.initialization.attempt_limit",
                "modeling.initialization.incomplete",
                "modeling.initialization.minimum_step",
                "modeling.initialization.outcome",
                "modeling.initialization.step_precision",
                "modeling.nonlinear.attempt_limit",
                "modeling.nonlinear.cancelled",
                "modeling.nonlinear.initial_inconclusive",
                "modeling.nonlinear.time_limit",
                "modeling.objective",
                "modeling.provenance",
                "modeling.qualification.infeasibility_contradicted",
                "modeling.qualification.rejected",
                "modeling.realization",
                "modeling.staged.start",
                "modeling.study.initialization",
                "modeling.study.predecessor",
                "modeling.study.procedure",
                "modeling.trajectory.incomplete",
                "modeling.trajectory.rejected",
                "native.cancelled",
                "native.contract",
                "native.internal",
                "native.limit",
                "native.numerical",
                "native.reuse",
                "native.route_refused",
                "native.structural",
                "native.unavailable",
                "native.unsupported",
                "objective.priority_optimization_incomplete",
                "shooting.candidate_validation",
                "shooting.evaluation",
                "structural.overdetermined",
                "structural.underdetermined",
                "variable.fixed_zero",
                "variable.large_value",
                "variable.missing_value",
                "variable.near_lower_bound",
                "variable.near_upper_bound",
                "variable.nonfinite",
                "variable.only_in_inequalities",
                "variable.outside_lower_bound",
                "variable.outside_upper_bound",
                "variable.small_value",
                "variable.unused",
                "workflow.contract",
                "workflow.ephemeral_publication",
                "workflow.export_expired",
                "workflow.job_payload_version",
                "workflow.legacy_workspace",
                "workflow.operations",
                "workflow.publication_unresolved",
                "workflow.unclassified",
                "quantity.unknown_id",
                "quantity.contract_mismatch",
                "quantity.unit_conversion",
                "quantity.incompatible",
                "quantity.nonfinite",
                "workflow.input",
                "workflow.internal",
                "workflow.panic",
                "diagnostic.aggregate",
                "study.binding.target",
                "study.binding.duplicate",
                "study.binding.physical",
                "study.binding.revision",
                "study.dependency.unusable",
                "study.seed.unavailable",
                "study.seed.incompatible",
                "study.seed.internal",
                "study.operation.unsupported",
                "modeling.conditional_unit.admission.unsupported",
                "modeling.conditional_unit.admission.cancelled",
                "modeling.conditional_unit.admission.resource_limit",
                "modeling.conditional_unit.admission.nonfinite",
                "modeling.conditional_unit.admission.infrastructure",
                "modeling.conditional_unit.admission.conflict",
                "modeling.conditional_unit.admission.incompatible",
                "modeling.conditional_unit.admission.internal",
                "modeling.conditional_unit.admission.inconclusive",
                "study.policy.admission",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::DiagnosticRuleView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "authoring.budget",
                "authoring.contract",
                "authoring.derived_write",
                "authoring.document_io",
                "authoring.missing_id",
                "authoring.package_unresolved",
                "authoring.package_version_conflict",
                "authoring.rename_named",
                "authoring.schema_version",
                "authoring.syntax",
                "authoring.unknown_key",
                "authoring.unknown_row_key",
                "authoring.unresolved_target",
                "candidate.evaluation_failed",
                "compiler.cancelled",
                "compiler.limit",
                "compiler.math",
                "compiler.missing",
                "compiler.modeling",
                "compiler.structure",
                "compiler.syntax",
                "domain.potential_evaluation_error",
                "equation.canceling_terms",
                "equation.large_residual",
                "equation.mismatched_term",
                "equation.term_evaluation_failed",
                "fit.candidate_validation",
                "fit.final_evaluation",
                "fit.objective_overflow",
                "fit.response_rank",
                "jacobian.analysis_inconclusive",
                "jacobian.condition_estimate",
                "jacobian.extreme_column",
                "jacobian.extreme_entry",
                "jacobian.extreme_row",
                "jacobian.numerical_rank_deficiency",
                "jacobian.parallel_columns",
                "jacobian.parallel_rows",
                "math.applicability",
                "math.cancelled",
                "math.coefficient_range",
                "math.contract",
                "math.domain",
                "math.evaluation",
                "math.library",
                "math.limit",
                "math.native",
                "math.provider",
                "math.quantity",
                "math.range",
                "math.validity",
                "modeling.budget",
                "modeling.cancelled",
                "modeling.capability",
                "modeling.conditional_unit.admission.invalid_model",
                "modeling.conditional_unit.admission.numerical",
                "modeling.conditional_unit.admission.trial_rejected",
                "modeling.contract",
                "modeling.diagnostic_samples",
                "modeling.diagnostics",
                "modeling.domain",
                "modeling.domain.tightened",
                "modeling.dynamic.algebraic_reset.unsupported",
                "modeling.dynamic.inventory_transfer.event.unsupported",
                "modeling.initialization.attempt_limit",
                "modeling.initialization.incomplete",
                "modeling.initialization.minimum_step",
                "modeling.initialization.outcome",
                "modeling.initialization.step_precision",
                "modeling.nonlinear.attempt_limit",
                "modeling.nonlinear.cancelled",
                "modeling.nonlinear.initial_inconclusive",
                "modeling.nonlinear.time_limit",
                "modeling.objective",
                "modeling.provenance",
                "modeling.qualification.infeasibility_contradicted",
                "modeling.qualification.rejected",
                "modeling.realization",
                "modeling.staged.start",
                "modeling.study.initialization",
                "modeling.study.predecessor",
                "modeling.study.procedure",
                "modeling.trajectory.incomplete",
                "modeling.trajectory.rejected",
                "native.cancelled",
                "native.contract",
                "native.internal",
                "native.limit",
                "native.numerical",
                "native.reuse",
                "native.route_refused",
                "native.structural",
                "native.unavailable",
                "native.unsupported",
                "objective.priority_optimization_incomplete",
                "shooting.candidate_validation",
                "shooting.evaluation",
                "structural.overdetermined",
                "structural.underdetermined",
                "variable.fixed_zero",
                "variable.large_value",
                "variable.missing_value",
                "variable.near_lower_bound",
                "variable.near_upper_bound",
                "variable.nonfinite",
                "variable.only_in_inequalities",
                "variable.outside_lower_bound",
                "variable.outside_upper_bound",
                "variable.small_value",
                "variable.unused",
                "workflow.contract",
                "workflow.ephemeral_publication",
                "workflow.export_expired",
                "workflow.job_payload_version",
                "workflow.legacy_workspace",
                "workflow.operations",
                "workflow.publication_unresolved",
                "workflow.unclassified",
                "quantity.unknown_id",
                "quantity.contract_mismatch",
                "quantity.unit_conversion",
                "quantity.incompatible",
                "quantity.nonfinite",
                "workflow.input",
                "workflow.internal",
                "workflow.panic",
                "diagnostic.aggregate",
                "study.binding.target",
                "study.binding.duplicate",
                "study.binding.physical",
                "study.binding.revision",
                "study.dependency.unusable",
                "study.seed.unavailable",
                "study.seed.incompatible",
                "study.seed.internal",
                "study.operation.unsupported",
                "modeling.conditional_unit.admission.unsupported",
                "modeling.conditional_unit.admission.cancelled",
                "modeling.conditional_unit.admission.resource_limit",
                "modeling.conditional_unit.admission.nonfinite",
                "modeling.conditional_unit.admission.infrastructure",
                "modeling.conditional_unit.admission.conflict",
                "modeling.conditional_unit.admission.incompatible",
                "modeling.conditional_unit.admission.internal",
                "modeling.conditional_unit.admission.inconclusive",
                "study.policy.admission",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticSeverity {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["error", "warning", "info"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::DiagnosticSeverityView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["error", "warning", "info"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for DiagnosticStage {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "workflow",
                "modeling",
                "routing",
                "native",
                "implicit",
                "presolve",
                "evaluation",
                "property",
                "fit",
                "shooting",
                "simulation",
                "initialization",
                "modeling.admission",
                "modeling.conditional_unit.admission",
                "modeling.diagnostics",
                "modeling.diagnostic_samples",
                "modeling.nonlinear",
                "modeling.qualification",
                "modeling.staged",
                "modeling.study",
                "modeling.trajectory",
                "modeling.dynamic",
                "dynamic-test",
                "fitting",
                "horizon",
                "compiler",
                "authoring",
                "quantity",
                "diagnostic",
                "binding",
                "study",
                "continuation",
                "study.admission",
                "study.policy",
                "study.binding",
                "applicability",
                "modeling-conservation-transfer",
                "modeling-event-reset",
                "nonlinear-explanation",
                "objective-levels",
                "shooting-trajectory-projection",
                "test",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::DiagnosticStageView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "workflow",
                "modeling",
                "routing",
                "native",
                "implicit",
                "presolve",
                "evaluation",
                "property",
                "fit",
                "shooting",
                "simulation",
                "initialization",
                "modeling.admission",
                "modeling.conditional_unit.admission",
                "modeling.diagnostics",
                "modeling.diagnostic_samples",
                "modeling.nonlinear",
                "modeling.qualification",
                "modeling.staged",
                "modeling.study",
                "modeling.trajectory",
                "modeling.dynamic",
                "dynamic-test",
                "fitting",
                "horizon",
                "compiler",
                "authoring",
                "quantity",
                "diagnostic",
                "binding",
                "study",
                "continuation",
                "study.admission",
                "study.policy",
                "study.binding",
                "applicability",
                "modeling-conservation-transfer",
                "modeling-event-reset",
                "nonlinear-explanation",
                "objective-levels",
                "shooting-trajectory-projection",
                "test",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for FileObservation {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.path.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("path")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.canonical.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("canonical")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::FileObservationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.path.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("path")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.canonical.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("canonical")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ImplicitOptions {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.time_limit.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(6i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("time_limit"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ImplicitOptionsView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.time_limit.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(6i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("time_limit"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ImplicitUnknown {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("id")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.id
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ImplicitUnknownView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("id")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.id
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for MathBodySpec {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.definition.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("definition")), field_type
                            : Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .definition.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.structure.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("structure")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .structure.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.physical.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("physical")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .physical.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.admissions.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("admissions")), field_type
                            : Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .admissions.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.policy.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("policy")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .policy.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::MathBodySpecView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.definition.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("definition")), field_type
                            : Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .definition.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.structure.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("structure")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .structure.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.physical.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("physical")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .physical.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.admissions.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("admissions")), field_type
                            : Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .admissions.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.policy.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("policy")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .policy.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for MathEvaluationLimits {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.derivative_components < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("derivative_components")),
                            field_type : Some(::protovalidate_buffa::FieldType::Uint64),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self
                            .derivative_components
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.operations < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("operations")), field_type
                            : Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self.operations
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.scratch_bytes < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("scratch_bytes")),
                            field_type : Some(::protovalidate_buffa::FieldType::Uint64),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self.scratch_bytes
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.provider_calls < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("provider_calls")),
                            field_type : Some(::protovalidate_buffa::FieldType::Uint64),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self.provider_calls
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::MathEvaluationLimitsView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.derivative_components < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("derivative_components")),
                            field_type : Some(::protovalidate_buffa::FieldType::Uint64),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self
                            .derivative_components
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.operations < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("operations")), field_type
                            : Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self.operations
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.scratch_bytes < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("scratch_bytes")),
                            field_type : Some(::protovalidate_buffa::FieldType::Uint64),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self.scratch_bytes
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.provider_calls < 1u64 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("provider_calls")),
                            field_type : Some(::protovalidate_buffa::FieldType::Uint64),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint64")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("gte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint64.gte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be >= {} (got {})", 1u64, self.provider_calls
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ModelBoundaryDiagnostic {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.code.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("code")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.causes.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(2i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("causes")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.class.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("class")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.severity.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(4i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("severity")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.stage.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(5i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("stage")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.rule.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(7i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("rule")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (key, value) in ((&self.observations)).iter() {
            if let Err(sub) = value.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(8i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("observations"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: Some(::protovalidate_buffa::FieldType::String),
                                            value_type: Some(::protovalidate_buffa::FieldType::Message),
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::StringKey(
                                                    ::std::borrow::Cow::Owned(
                                                        ::core::convert::AsRef::<str>::as_ref(key).to_owned(),
                                                    ),
                                                ),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.locations.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(9i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("locations")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.validity.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(10i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("validity")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.applicability.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(11i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("applicability"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ModelBoundaryDiagnosticView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let __pv_map_8 = ::protovalidate_buffa::__private::CanonicalMap::from_view(
            &self.observations,
        );
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.code.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("code")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.causes.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(2i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("causes")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.class.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("class")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.severity.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(4i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("severity")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.stage.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(5i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("stage")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.rule.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(7i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("rule")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (key, value) in ((&__pv_map_8)).iter() {
            if let Err(sub) = value.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(8i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("observations"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: Some(::protovalidate_buffa::FieldType::String),
                                            value_type: Some(::protovalidate_buffa::FieldType::Message),
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::StringKey(
                                                    ::std::borrow::Cow::Owned(
                                                        ::core::convert::AsRef::<str>::as_ref(key).to_owned(),
                                                    ),
                                                ),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.locations.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(9i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("locations")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(inner) = self.validity.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(10i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("validity")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.applicability.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(11i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("applicability"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ModelSourceLocation {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.source.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.revision.as_deref() {
            if v.len() != 32usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(2i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("revision")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 32usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(v) = self.document.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(3i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("document")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ModelSourceLocationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.source.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.revision.as_deref() {
            if v.len() != 32usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(2i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("revision")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 32usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(v) = self.document.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(3i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("document")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ModelValidityLineage {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.layer.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("layer")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.source.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.form.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(3i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("form")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ModelValidityLineageView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.layer.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("layer")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.source.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.form.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(3i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("form")), field_type :
                                Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(15i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(13i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("len")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})", 16usize, v.len()
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ModelingApplicabilityBasis {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["fitted", "recommended", "validated", "reported"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::ModelingApplicabilityBasisView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["fitted", "recommended", "validated", "reported"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ModelingApplicabilityOutcome {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "applicable",
                "outside_region",
                "unknown_evidence",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::ModelingApplicabilityOutcomeView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "applicable",
                "outside_region",
                "unknown_evidence",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ModelingPermissionTarget {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["records", "families"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::ModelingPermissionTargetView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["records", "families"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for ModelingValidityLayer {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["form", "data", "closure"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::ModelingValidityLayerView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["form", "data", "closure"];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativeBoundaryClass {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "invalid_model",
                "unsupported",
                "resource_limit",
                "trial_rejected",
                "nonfinite",
                "infrastructure",
                "cancelled",
                "conflict",
                "incompatible",
                "internal",
                "numerical",
                "inconclusive",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativeBoundaryClassView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "invalid_model",
                "unsupported",
                "resource_limit",
                "trial_rejected",
                "nonfinite",
                "infrastructure",
                "cancelled",
                "conflict",
                "incompatible",
                "internal",
                "numerical",
                "inconclusive",
            ];
            if !ALLOWED
                .iter()
                .any(|candidate| {
                    *candidate == ::core::convert::AsRef::<str>::as_ref(&self.value)
                })
            {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(14i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("string")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(10i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("in")), field_type :
                                Some(::protovalidate_buffa::FieldType::String), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]", ALLOWED.join(", ")
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativePathBuf {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.platform.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("platform")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.platform.is_set() {
            if let Some(inner) = self.platform.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("platform")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativePathBufView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.platform.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("platform")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.platform.is_set() {
            if let Some(inner) = self.platform.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("platform")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativeRootBindings {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.instance.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("instance")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .instance.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.checked_members.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(2i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("checked_members"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.checked_sources.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("checked_sources"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.occurrences.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(4i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("occurrences"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativeRootBindingsView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.instance.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("instance")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .instance.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.checked_members.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(2i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("checked_members"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.checked_sources.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(3i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("checked_sources"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        for (idx, elem) in self.occurrences.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(4i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("occurrences"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for native_root_bindings::Field2Entry {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("key")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.key
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.value.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("value")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .value.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::native_root_bindings::Field2EntryView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("key")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.key
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.value.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("value")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .value.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for native_root_bindings::Field3Entry {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("key")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.key
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.value.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("value")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .value.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::native_root_bindings::Field3EntryView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("key")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.key
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.value.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("value")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .value.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for native_root_bindings::Field4Entry {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("key")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.key
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.value.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("value")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.value.is_set() {
            if let Some(inner) = self.value.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::native_root_bindings::Field4EntryView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("key")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self.key
                            .len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.value.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("value")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.value.is_set() {
            if let Some(inner) = self.value.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativeRootHello {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.diagnostic_operation.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("diagnostic_operation"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.attempt.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("attempt")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .attempt.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.executable.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("executable")), field_type
                            : Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.executable.is_set() {
            if let Some(inner) = self.executable.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(6i32),
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("executable"),
                                                ),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if !self.build.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(7i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("build")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.build.is_set() {
            if let Some(inner) = self.build.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(7i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("build")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if self.native_build.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(8i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("native_build")),
                            field_type : Some(::protovalidate_buffa::FieldType::Bytes),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .native_build.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativeRootHelloView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.diagnostic_operation.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("diagnostic_operation"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.attempt.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("attempt")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .attempt.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.executable.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("executable")), field_type
                            : Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.executable.is_set() {
            if let Some(inner) = self.executable.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(6i32),
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("executable"),
                                                ),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if !self.build.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(7i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("build")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.build.is_set() {
            if let Some(inner) = self.build.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(7i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("build")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if self.native_build.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(8i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("native_build")),
                            field_type : Some(::protovalidate_buffa::FieldType::Bytes),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .native_build.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativeRootInput {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.body.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("body")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 1usize,
                            self.body.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.body.len() > 16777216usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("body")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            16777216usize, self.body.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.spec.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("spec")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.spec.is_set() {
            if let Some(inner) = self.spec.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("spec")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if !self.source.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.source.is_set() {
            if let Some(inner) = self.source.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(3i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("source")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if self.registry.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("registry")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 1usize,
                            self.registry.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.registry.len() > 16777216usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("registry")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            16777216usize, self.registry.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.registry_digest.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("registry_digest")),
                            field_type : Some(::protovalidate_buffa::FieldType::Bytes),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .registry_digest.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.unknowns.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(6i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("unknowns")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if !self.options.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(9i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("options")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.options.is_set() {
            if let Some(inner) = self.options.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(9i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("options")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if !self.evaluation.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(10i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("evaluation")), field_type
                            : Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.evaluation.is_set() {
            if let Some(inner) = self.evaluation.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(10i32),
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("evaluation"),
                                                ),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativeRootInputView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.body.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("body")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 1usize,
                            self.body.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.body.len() > 16777216usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("body")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            16777216usize, self.body.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.spec.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("spec")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.spec.is_set() {
            if let Some(inner) = self.spec.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("spec")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if !self.source.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.source.is_set() {
            if let Some(inner) = self.source.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(3i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("source")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if self.registry.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("registry")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 1usize,
                            self.registry.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.registry.len() > 16777216usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("registry")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            16777216usize, self.registry.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.registry_digest.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("registry_digest")),
                            field_type : Some(::protovalidate_buffa::FieldType::Bytes),
                            key_type : None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .registry_digest.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.unknowns.iter().enumerate() {
            if let Err(sub) = elem.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(6i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("unknowns")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: Some(
                                                ::protovalidate_buffa::Subscript::Index(idx as u64),
                                            ),
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if !self.options.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(9i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("options")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.options.is_set() {
            if let Some(inner) = self.options.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(9i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("options")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if !self.evaluation.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(10i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("evaluation")), field_type
                            : Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.evaluation.is_set() {
            if let Some(inner) = self.evaluation.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(10i32),
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("evaluation"),
                                                ),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativeRootOutcome {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::oneof::native_root_outcome::Kind::Proposal(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("proposal")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::native_root_outcome::Kind::Refused(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("refused")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            None => {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                None, field_name :
                                Some(::std::borrow::Cow::Borrowed("kind")), field_type :
                                None, key_type : None, value_type : None, subscript : None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("required"),
                        message: ::std::borrow::Cow::Borrowed(
                            "at least one variant is required",
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativeRootOutcomeView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::view::oneof::native_root_outcome::Kind::Proposal(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("proposal")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::native_root_outcome::Kind::Refused(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("refused")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            None => {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                None, field_name :
                                Some(::std::borrow::Cow::Borrowed("kind")), field_type :
                                None, key_type : None, value_type : None, subscript : None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("required"),
                        message: ::std::borrow::Cow::Borrowed(
                            "at least one variant is required",
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for native_root_outcome::Alternative1 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::native_root_outcome::Alternative1View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for native_root_outcome::Alternative2 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.value.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::native_root_outcome::Alternative2View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.value.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativeRootRequest {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.diagnostic_parent.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("diagnostic_parent"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.attempt.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("attempt")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .attempt.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.execution.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("execution")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .execution.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.request.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("request")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .request.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(inner) = self.remaining.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(5i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("remaining")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.source.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 1usize,
                            self.source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.source.len() > 16777216usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            16777216usize, self.source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            if let ::core::option::Option::Some(__cel_inner) = self.remaining.as_option()
            {
                if !({
                    let __outer = __cel_evaluation_error.take();
                    let __left = ((::protovalidate_buffa::cel::CelScalar::cel_uint(
                        __cel_inner.secs,
                    )) > (0u64 as u64));
                    let __left_error = __cel_evaluation_error.take();
                    let (__result, __error) = if __left_error.is_none() && __left == true
                    {
                        (__left, None)
                    } else {
                        let __right = ((::protovalidate_buffa::cel::CelScalar::cel_uint(
                            __cel_inner.nanos,
                        )) > (0u64 as u64));
                        let __right_error = __cel_evaluation_error.take();
                        if __right_error.is_none() && __right == true {
                            (__right, None)
                        } else {
                            (__right, __left_error.or(__right_error))
                        }
                    };
                    __cel_evaluation_error.set(__outer.or(__error));
                    __result
                }) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(5i32), field_name :
                                    Some(::std::borrow::Cow::Borrowed("remaining")), field_type
                                    : Some(::protovalidate_buffa::FieldType::Message), key_type
                                    : None, value_type : None, subscript : None, },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(23), field_name :
                                    Some(::std::borrow::Cow::Borrowed("cel")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Message), key_type :
                                    None, value_type : None, subscript :
                                    Some(::protovalidate_buffa::Subscript::Index(0u64)), }
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("duration.positive"),
                            message: ::std::borrow::Cow::Borrowed(
                                "a positive duration is required",
                            ),
                            for_key: false,
                        });
                }
            }
            if let Some(reason) = __cel_evaluation_error.get() {
                violations.truncate(__cel_violation_start);
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath::default(),
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("__cel_runtime_error__"),
                        message: ::std::borrow::Cow::Borrowed(reason),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativeRootRequestView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.diagnostic_parent.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(1i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("diagnostic_parent"),
                                            ),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.attempt.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("attempt")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .attempt.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.execution.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("execution")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .execution.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.request.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("request")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .request.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(inner) = self.remaining.as_option() {
            if let Err(sub) = inner.validate() {
                violations
                    .extend(
                        sub
                            .violations
                            .into_iter()
                            .map(|mut v| {
                                v.field
                                    .elements
                                    .insert(
                                        0,
                                        ::protovalidate_buffa::FieldPathElement {
                                            field_number: Some(5i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("remaining")),
                                            field_type: Some(::protovalidate_buffa::FieldType::Message),
                                            key_type: None,
                                            value_type: None,
                                            subscript: None,
                                        },
                                    );
                                v
                            }),
                    );
            }
        }
        if self.source.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("min_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})", 1usize,
                            self.source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.source.len() > 16777216usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(6i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("source")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("max_len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            16777216usize, self.source.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            if let ::core::option::Option::Some(__cel_inner) = self.remaining.as_option()
            {
                if !({
                    let __outer = __cel_evaluation_error.take();
                    let __left = ((::protovalidate_buffa::cel::CelScalar::cel_uint(
                        __cel_inner.secs,
                    )) > (0u64 as u64));
                    let __left_error = __cel_evaluation_error.take();
                    let (__result, __error) = if __left_error.is_none() && __left == true
                    {
                        (__left, None)
                    } else {
                        let __right = ((::protovalidate_buffa::cel::CelScalar::cel_uint(
                            __cel_inner.nanos,
                        )) > (0u64 as u64));
                        let __right_error = __cel_evaluation_error.take();
                        if __right_error.is_none() && __right == true {
                            (__right, None)
                        } else {
                            (__right, __left_error.or(__right_error))
                        }
                    };
                    __cel_evaluation_error.set(__outer.or(__error));
                    __result
                }) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(5i32), field_name :
                                    Some(::std::borrow::Cow::Borrowed("remaining")), field_type
                                    : Some(::protovalidate_buffa::FieldType::Message), key_type
                                    : None, value_type : None, subscript : None, },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement { field_number :
                                    Some(23), field_name :
                                    Some(::std::borrow::Cow::Borrowed("cel")), field_type :
                                    Some(::protovalidate_buffa::FieldType::Message), key_type :
                                    None, value_type : None, subscript :
                                    Some(::protovalidate_buffa::Subscript::Index(0u64)), }
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("duration.positive"),
                            message: ::std::borrow::Cow::Borrowed(
                                "a positive duration is required",
                            ),
                            for_key: false,
                        });
                }
            }
            if let Some(reason) = __cel_evaluation_error.get() {
                violations.truncate(__cel_violation_start);
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath::default(),
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("__cel_runtime_error__"),
                        message: ::std::borrow::Cow::Borrowed(reason),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NativeRootResponse {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.attempt.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("attempt")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .attempt.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.execution.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("execution")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .execution.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.request.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("request")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .request.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.result.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("result")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.result.is_set() {
            if let Some(inner) = self.result.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(4i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("result")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NativeRootResponseView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.attempt.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(1i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("attempt")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 16usize, self
                            .attempt.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.execution.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("execution")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .execution.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.request.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("request")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bytes), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(15i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("bytes")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(13i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("len")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint64), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})", 32usize, self
                            .request.len()
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.result.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(4i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("result")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(25i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("required")), field_type :
                            Some(::protovalidate_buffa::FieldType::Bool), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.result.is_set() {
            if let Some(inner) = self.result.as_option() {
                if let Err(sub) = inner.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(4i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("result")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for NonnegativeDuration {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.nanos > 999999999u32 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("nanos")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint32")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("lte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint32.lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be <= {} (got {})", 999999999u32, self.nanos
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::NonnegativeDurationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.nanos > 999999999u32 {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(2i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("nanos")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(5i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("uint32")), field_type :
                            Some(::protovalidate_buffa::FieldType::Message), key_type :
                            None, value_type : None, subscript : None, },
                            ::protovalidate_buffa::FieldPathElement { field_number :
                            Some(3i32), field_name :
                            Some(::std::borrow::Cow::Borrowed("lte")), field_type :
                            Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                            None, value_type : None, subscript : None, },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint32.lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be <= {} (got {})", 999999999u32, self.nanos
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for PlatformPathBytes {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::oneof::platform_path_bytes::Kind::Unix(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("unix")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::oneof::platform_path_bytes::Kind::Windows(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("windows")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            None => {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                None, field_name :
                                Some(::std::borrow::Cow::Borrowed("kind")), field_type :
                                None, key_type : None, value_type : None, subscript : None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("required"),
                        message: ::std::borrow::Cow::Borrowed(
                            "at least one variant is required",
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for __buffa::view::PlatformPathBytesView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::view::oneof::platform_path_bytes::Kind::Unix(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(1i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("unix")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            Some(__buffa::view::oneof::platform_path_bytes::Kind::Windows(v)) => {
                if let Err(sub) = v.validate() {
                    violations
                        .extend(
                            sub
                                .violations
                                .into_iter()
                                .map(|mut v| {
                                    v.field
                                        .elements
                                        .insert(
                                            0,
                                            ::protovalidate_buffa::FieldPathElement {
                                                field_number: Some(2i32),
                                                field_name: Some(::std::borrow::Cow::Borrowed("windows")),
                                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                                key_type: None,
                                                value_type: None,
                                                subscript: None,
                                            },
                                        );
                                    v
                                }),
                        );
                }
            }
            None => {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                None, field_name :
                                Some(::std::borrow::Cow::Borrowed("kind")), field_type :
                                None, key_type : None, value_type : None, subscript : None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath::default(),
                        rule_id: ::std::borrow::Cow::Borrowed("required"),
                        message: ::std::borrow::Cow::Borrowed(
                            "at least one variant is required",
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for platform_path_bytes::Alternative1 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::platform_path_bytes::Alternative1View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate for platform_path_bytes::Alternative2 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        for (idx, elem) in self.value.iter().enumerate() {
            if *elem > 65535u32 {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                                None, value_type : None, subscript :
                                Some(::protovalidate_buffa::Subscript::Index(idx as u64)),
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(18i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(4i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("items")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(5i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("uint32")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(3i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("lte")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("uint32.lte"),
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
#[allow(
    clippy::all,
    unused_mut,
    unused_variables,
    unused_parens,
    dead_code,
    unreachable_patterns,
    reason = "protovalidate-buffa generated validators — codegen emits uniform scaffolding regardless of which rules apply"
)]
impl ::protovalidate_buffa::Validate
for __buffa::view::platform_path_bytes::Alternative2View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        for (idx, elem) in self.value.iter().enumerate() {
            if *elem > 65535u32 {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(1i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("value")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                                None, value_type : None, subscript :
                                Some(::protovalidate_buffa::Subscript::Index(idx as u64)),
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(18i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("repeated")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(4i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("items")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(5i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("uint32")), field_type :
                                Some(::protovalidate_buffa::FieldType::Message), key_type :
                                None, value_type : None, subscript : None, },
                                ::protovalidate_buffa::FieldPathElement { field_number :
                                Some(3i32), field_name :
                                Some(::std::borrow::Cow::Borrowed("lte")), field_type :
                                Some(::protovalidate_buffa::FieldType::Uint32), key_type :
                                None, value_type : None, subscript : None, },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("uint32.lte"),
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
        }
        if let Some(index) = violations
            .iter()
            .position(|v| v.rule_id == "__cel_runtime_error__")
        {
            let v = violations.swap_remove(index);
            return ::core::result::Result::Err(::protovalidate_buffa::ValidationError {
                runtime_error: ::core::option::Option::Some(v.message.into_owned()),
                ..::core::default::Default::default()
            });
        }
        if violations.is_empty() {
            Ok(())
        } else {
            Err(::protovalidate_buffa::ValidationError {
                violations,
                ..::core::default::Default::default()
            })
        }
    }
}
