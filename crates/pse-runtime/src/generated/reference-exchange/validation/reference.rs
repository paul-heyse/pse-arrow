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
impl ::protovalidate_buffa::Validate for Array8PhysicalRatio {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 8usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("values")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            8usize,
                            self.values.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 8usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("values")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            8usize,
                            self.values.len(),
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
impl ::protovalidate_buffa::Validate for __buffa::view::Array8PhysicalRatioView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.values.len() < 8usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("values")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            8usize,
                            self.values.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.values.len() > 8usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("values")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            8usize,
                            self.values.len(),
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
impl ::protovalidate_buffa::Validate for BasisId {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for __buffa::view::BasisIdView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for BasisKind {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "molar",
                "mass",
                "volume",
                "energy",
                "standard_volume",
                "dimensionless",
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for __buffa::view::BasisKindView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "molar",
                "mass",
                "volume",
                "energy",
                "standard_volume",
                "dimensionless",
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for CompositionBasis {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "mole_fraction",
                "mass_fraction",
                "volume_fraction",
                "molality",
                "molarity",
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for __buffa::view::CompositionBasisView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "mole_fraction",
                "mass_fraction",
                "volume_fraction",
                "molality",
                "molarity",
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for DocumentVersion1 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if (self.value) < (1u32) || (self.value) > (1u32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("uint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.value),
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
impl ::protovalidate_buffa::Validate for __buffa::view::DocumentVersion1View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if (self.value) < (1u32) || (self.value) > (1u32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("uint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.value),
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
impl ::protovalidate_buffa::Validate for EntityKindId {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for __buffa::view::EntityKindIdView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for KindDefinition {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        for (idx, elem) in self.monomial.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("monomial")),
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
        if let Some(inner) = self.canonical_unit.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("canonical_unit"),
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
                                            field_number: Some(3i32),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        if let Some(inner) = self.scale_kind.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("scale_kind"),
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
        if let Some(inner) = self.subject_kind.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("subject_kind"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::KindDefinitionView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        for (idx, elem) in self.monomial.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("monomial")),
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
        if let Some(inner) = self.canonical_unit.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("canonical_unit"),
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
                                            field_number: Some(3i32),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        if let Some(inner) = self.scale_kind.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("scale_kind"),
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
        if let Some(inner) = self.subject_kind.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("subject_kind"),
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
impl ::protovalidate_buffa::Validate for KindFactor {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
        if let Some(inner) = self.exponent.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("exponent")),
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
impl ::protovalidate_buffa::Validate for __buffa::view::KindFactorView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
        if let Some(inner) = self.exponent.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("exponent")),
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
impl ::protovalidate_buffa::Validate for PhysicalBasis {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
        if let Some(inner) = self.composition_basis.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("composition_basis"),
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
        if let Some(inner) = self.rate_basis.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("rate_basis"),
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
        if let Some(inner) = self.reference_conditions.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_conditions"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::PhysicalBasisView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
        if let Some(inner) = self.composition_basis.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("composition_basis"),
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
        if let Some(inner) = self.rate_basis.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("rate_basis"),
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
        if let Some(inner) = self.reference_conditions.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_conditions"),
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
impl ::protovalidate_buffa::Validate for PhysicalDimensionVector {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.exponents.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("exponents")),
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
impl ::protovalidate_buffa::Validate for __buffa::view::PhysicalDimensionVectorView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.exponents.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("exponents")),
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
impl ::protovalidate_buffa::Validate for PhysicalEntityKind {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
impl ::protovalidate_buffa::Validate for __buffa::view::PhysicalEntityKindView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
impl ::protovalidate_buffa::Validate for PhysicalQuantityDescription {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.quantity.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("quantity")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.quantity.is_set() {
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
        }
        if !self.quantity_kind.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("quantity_kind"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.quantity_kind.is_set() {
            if let Some(inner) = self.quantity_kind.as_option() {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("quantity_kind"),
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
        if !self.canonical_unit.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("canonical_unit"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.canonical_unit.is_set() {
            if let Some(inner) = self.canonical_unit.as_option() {
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
                                                    ::std::borrow::Cow::Borrowed("canonical_unit"),
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
        if !self.representation_unit.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("representation_unit"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.representation_unit.is_set() {
            if let Some(inner) = self.representation_unit.as_option() {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("representation_unit"),
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
                                            field_number: Some(7i32),
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
        if let Some(inner) = self.basis_reference_state.as_option() {
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
                                            field_number: Some(8i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("basis_reference_state"),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        for (idx, elem) in self.shape.iter().enumerate() {
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
                                            field_number: Some(10i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("shape")),
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
        if let Some(inner) = self.subject.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("subject")),
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
for __buffa::view::PhysicalQuantityDescriptionView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.quantity.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("quantity")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.quantity.is_set() {
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
        }
        if !self.quantity_kind.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("quantity_kind"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.quantity_kind.is_set() {
            if let Some(inner) = self.quantity_kind.as_option() {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("quantity_kind"),
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
        if !self.canonical_unit.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("canonical_unit"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.canonical_unit.is_set() {
            if let Some(inner) = self.canonical_unit.as_option() {
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
                                                    ::std::borrow::Cow::Borrowed("canonical_unit"),
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
        if !self.representation_unit.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("representation_unit"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.representation_unit.is_set() {
            if let Some(inner) = self.representation_unit.as_option() {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("representation_unit"),
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
                                            field_number: Some(7i32),
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
        if let Some(inner) = self.basis_reference_state.as_option() {
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
                                            field_number: Some(8i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("basis_reference_state"),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        for (idx, elem) in self.shape.iter().enumerate() {
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
                                            field_number: Some(10i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("shape")),
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
        if let Some(inner) = self.subject.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("subject")),
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
impl ::protovalidate_buffa::Validate for PhysicalQuantityKind {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.dimension.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("dimension")),
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
        if let Some(inner) = self.addition_kind.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("addition_kind"),
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
        if let Some(inner) = self.category.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("category")),
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
        if let Some(inner) = self.definition.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("definition"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::PhysicalQuantityKindView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.dimension.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("dimension")),
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
        if let Some(inner) = self.addition_kind.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("addition_kind"),
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
        if let Some(inner) = self.category.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("category")),
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
        if let Some(inner) = self.definition.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("definition"),
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
impl ::protovalidate_buffa::Validate for PhysicalRatio {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if (self.num) < (-32768i32) || (self.num) > (32767i32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("num")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(7i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("sint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.num),
                    ),
                    for_key: false,
                });
        }
        if (self.den) < (-32768i32) || (self.den) > (32767i32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("den")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(7i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("sint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.den),
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
impl ::protovalidate_buffa::Validate for __buffa::view::PhysicalRatioView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if (self.num) < (-32768i32) || (self.num) > (32767i32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("num")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(7i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("sint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.num),
                    ),
                    for_key: false,
                });
        }
        if (self.den) < (-32768i32) || (self.den) > (32767i32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("den")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(7i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Sint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("sint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.den),
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
impl ::protovalidate_buffa::Validate for PhysicalUnit {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.dimension.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("dimension")),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        if let Some(inner) = self.definition.as_option() {
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
                                            field_number: Some(8i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("definition"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::PhysicalUnitView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.dimension.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("dimension")),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        if let Some(inner) = self.definition.as_option() {
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
                                            field_number: Some(8i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("definition"),
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
impl ::protovalidate_buffa::Validate for physical_unit::Field8Value {
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
for __buffa::view::physical_unit::Field8ValueView<'_> {
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
impl ::protovalidate_buffa::Validate for QuantityAdditionKind {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["additive", "origin_sensitive"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for __buffa::view::QuantityAdditionKindView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["additive", "origin_sensitive"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for QuantityKindCategory {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["count", "indicator"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for __buffa::view::QuantityKindCategoryView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["count", "indicator"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for QuantityKindId {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for __buffa::view::QuantityKindIdView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for QuantityType {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.key.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("key")),
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
        if let Some(inner) = self.canonical_unit.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("canonical_unit"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::QuantityTypeView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.key.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("key")),
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
        if let Some(inner) = self.canonical_unit.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("canonical_unit"),
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
impl ::protovalidate_buffa::Validate for QuantityTypeId {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for __buffa::view::QuantityTypeIdView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for QuantityTypeKey {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
                                            field_number: Some(2i32),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        if let Some(inner) = self.scale_kind.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("scale_kind"),
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
        for (idx, elem) in self.shape.iter().enumerate() {
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
                                            field_number: Some(5i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("shape")),
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
        if let Some(inner) = self.subject_kind.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("subject_kind"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::QuantityTypeKeyView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
                                            field_number: Some(2i32),
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
        if let Some(inner) = self.reference_state.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("reference_state"),
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
        if let Some(inner) = self.scale_kind.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("scale_kind"),
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
        for (idx, elem) in self.shape.iter().enumerate() {
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
                                            field_number: Some(5i32),
                                            field_name: Some(::std::borrow::Cow::Borrowed("shape")),
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
        if let Some(inner) = self.subject_kind.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("subject_kind"),
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
impl ::protovalidate_buffa::Validate for RateBasis {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["per_volume", "per_mass", "per_area"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for __buffa::view::RateBasisView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["per_volume", "per_mass", "per_area"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for ReferenceBlock {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.content.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("content")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.content.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.schema.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("schema")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.schema.len(),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceBlockView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.content.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("content")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.content.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.schema.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("schema")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.schema.len(),
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
impl ::protovalidate_buffa::Validate for ReferenceCaseContract {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.version.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("version")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.version.is_set() {
            if let Some(inner) = self.version.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("version")),
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
        if self.case_name.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("case_name")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.case_name.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("case_name")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 1024usize),
                    ),
                    for_key: false,
                });
        }
        if !self.meaning.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.meaning.is_set() {
            if let Some(inner) = self.meaning.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
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
        if self.observables.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("observables"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.observables.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.observables.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("observables"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.observables.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.observables.iter().enumerate() {
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
                                                ::std::borrow::Cow::Borrowed("observables"),
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
        if self.specification.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("specification"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.specification.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.specification.iter().enumerate() {
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
                                            field_number: Some(5i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("specification"),
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
        if self.artifacts.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(6i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("artifacts")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.artifacts.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.artifacts.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(6i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("artifacts")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            1024usize,
                            self.artifacts.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.artifacts.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("artifacts")),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceCaseContractView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.version.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("version")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.version.is_set() {
            if let Some(inner) = self.version.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("version")),
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
        if self.case_name.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("case_name")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.case_name.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("case_name")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 1024usize),
                    ),
                    for_key: false,
                });
        }
        if !self.meaning.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.meaning.is_set() {
            if let Some(inner) = self.meaning.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
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
        if self.observables.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("observables"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.observables.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.observables.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("observables"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.observables.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.observables.iter().enumerate() {
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
                                                ::std::borrow::Cow::Borrowed("observables"),
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
        if self.specification.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("specification"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.specification.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.specification.iter().enumerate() {
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
                                            field_number: Some(5i32),
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("specification"),
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
        if self.artifacts.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(6i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("artifacts")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.artifacts.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.artifacts.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(6i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("artifacts")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            1024usize,
                            self.artifacts.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.artifacts.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("artifacts")),
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
impl ::protovalidate_buffa::Validate for ReferenceCondition {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.quantity_type.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("quantity_type"),
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
        if let Some(inner) = self.unit.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("unit")),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceConditionView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.quantity_type.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("quantity_type"),
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
        if let Some(inner) = self.unit.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("unit")),
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
impl ::protovalidate_buffa::Validate for ReferenceEnvelope {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if (self.version) < (1u32) || (self.version) > (1u32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("version")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("uint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.version),
                    ),
                    for_key: false,
                });
        }
        if !self.meaning.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.meaning.is_set() {
            if let Some(inner) = self.meaning.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
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
        if self.required.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.required.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.required.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.required.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.outcome.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("outcome")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.outcome.is_set() {
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceEnvelopeView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if (self.version) < (1u32) || (self.version) > (1u32) {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("version")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("uint32")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("gte")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("uint32.gte_lte"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value out of range (got {})", self.version),
                    ),
                    for_key: false,
                });
        }
        if !self.meaning.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.meaning.is_set() {
            if let Some(inner) = self.meaning.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("meaning")),
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
        if self.required.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.required.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.required.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.required.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if !self.outcome.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("outcome")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.outcome.is_set() {
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
impl ::protovalidate_buffa::Validate for ReferenceFixedInput {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.key.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 1024usize),
                    ),
                    for_key: false,
                });
        }
        if !self.physical.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.physical.is_set() {
            if let Some(inner) = self.physical.as_option() {
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceFixedInputView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.key.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.key.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 1024usize),
                    ),
                    for_key: false,
                });
        }
        if !self.physical.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.physical.is_set() {
            if let Some(inner) = self.physical.as_option() {
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
impl ::protovalidate_buffa::Validate for ReferenceInputArtifact {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.path.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("path")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.path.len() > 4096usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("path")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 4096usize),
                    ),
                    for_key: false,
                });
        }
        if self.sha256.len() < 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sha256")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})",
                            32usize,
                            self.sha256.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.sha256.len() > 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sha256")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            32usize,
                            self.sha256.len(),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceInputArtifactView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.path.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("path")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.path.len() > 4096usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("path")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 4096usize),
                    ),
                    for_key: false,
                });
        }
        if self.sha256.len() < 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sha256")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} bytes (got {})",
                            32usize,
                            self.sha256.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.sha256.len() > 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("sha256")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} bytes (got {})",
                            32usize,
                            self.sha256.len(),
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
impl ::protovalidate_buffa::Validate for ReferenceMeaning {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.case.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("case")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.case.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.source.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("source")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.source.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.formulation.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("formulation"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.formulation.len(),
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
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.physical.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.configuration.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("configuration"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.configuration.len(),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceMeaningView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.case.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("case")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.case.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.source.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("source")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.source.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.formulation.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("formulation"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.formulation.len(),
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
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.physical.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.configuration.len() != 32usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("configuration"),
                                ),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            32usize,
                            self.configuration.len(),
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
impl ::protovalidate_buffa::Validate for ReferenceObservable {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.id.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.quantity.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("quantity")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.quantity.len(),
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
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("unit")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.unit.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.basis.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(4i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("basis")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(15i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(13i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})",
                                16usize,
                                v.len(),
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(v) = self.datum.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(5i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("datum")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(15i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(13i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})",
                                16usize,
                                v.len(),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceObservableView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.id.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.quantity.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("quantity")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.quantity.len(),
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
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("unit")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.unit.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.basis.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(4i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("basis")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(15i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(13i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})",
                                16usize,
                                v.len(),
                            ),
                        ),
                        for_key: false,
                    });
            }
        }
        if let Some(v) = self.datum.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(5i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("datum")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(15i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(13i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})",
                                16usize,
                                v.len(),
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
impl ::protovalidate_buffa::Validate for ReferenceObservableContract {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.output.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("output")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.output.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.key.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.key.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 1024usize),
                    ),
                    for_key: false,
                });
        }
        if !self.physical.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.physical.is_set() {
            if let Some(inner) = self.physical.as_option() {
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
for __buffa::view::ReferenceObservableContractView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.output.len() != 16usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("output")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(15i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(13i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value must be exactly {} bytes (got {})",
                            16usize,
                            self.output.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.key.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(4i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at least {} bytes", 1usize),
                    ),
                    for_key: false,
                });
        }
        if self.key.len() > 1024usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("key")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(5i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_bytes")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_bytes"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!("value must be at most {} bytes", 1024usize),
                    ),
                    for_key: false,
                });
        }
        if !self.physical.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("physical")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.physical.is_set() {
            if let Some(inner) = self.physical.as_option() {
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
impl ::protovalidate_buffa::Validate for ReferenceOutcome {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::oneof::reference_outcome::Kind::Completed(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("completed")),
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
            Some(__buffa::oneof::reference_outcome::Kind::Failed(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("failed")),
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
            Some(__buffa::oneof::reference_outcome::Kind::Unavailable(v)) => {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("unavailable"),
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
            Some(__buffa::oneof::reference_outcome::Kind::NotRun(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("not_run")),
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: None,
                                    field_name: Some(::std::borrow::Cow::Borrowed("kind")),
                                    field_type: None,
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceOutcomeView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::view::oneof::reference_outcome::Kind::Completed(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("completed")),
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
            Some(__buffa::view::oneof::reference_outcome::Kind::Failed(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("failed")),
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
            Some(__buffa::view::oneof::reference_outcome::Kind::Unavailable(v)) => {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("unavailable"),
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
            Some(__buffa::view::oneof::reference_outcome::Kind::NotRun(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("not_run")),
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: None,
                                    field_name: Some(::std::borrow::Cow::Borrowed("kind")),
                                    field_type: None,
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
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
impl ::protovalidate_buffa::Validate for reference_outcome::Alternative1 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.outputs.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("outputs")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.outputs.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.outputs.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("outputs")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.outputs.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.outputs.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("outputs")),
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
        for (idx, elem) in self.blocks.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("blocks")),
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
for __buffa::view::reference_outcome::Alternative1View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.outputs.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("outputs")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.min_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at least {} items (got {})",
                            1usize,
                            self.outputs.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.outputs.len() > 65536usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("outputs")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(18i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_items")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("repeated.max_items"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "list must contain at most {} items (got {})",
                            65536usize,
                            self.outputs.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        for (idx, elem) in self.outputs.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("outputs")),
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
        for (idx, elem) in self.blocks.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("blocks")),
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
impl ::protovalidate_buffa::Validate for reference_outcome::Alternative2 {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.code.chars().count() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("code")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} characters",
                            1usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.code.chars().count() > 256usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("code")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} characters",
                            256usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.stage.chars().count() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("stage")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} characters",
                            1usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.stage.chars().count() > 256usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("stage")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} characters",
                            256usize,
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
for __buffa::view::reference_outcome::Alternative2View<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.code.chars().count() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("code")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} characters",
                            1usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.code.chars().count() > 256usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("code")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} characters",
                            256usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.stage.chars().count() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("stage")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("min_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.min_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at least {} characters",
                            1usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.stage.chars().count() > 256usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("stage")),
                                field_type: Some(::protovalidate_buffa::FieldType::String),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(14i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                field_type: Some(::protovalidate_buffa::FieldType::Message),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("max_len")),
                                field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("string.max_len"),
                    message: ::std::borrow::Cow::Owned(
                        ::std::format!(
                            "value length must be at most {} characters",
                            256usize,
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
impl ::protovalidate_buffa::Validate for reference_outcome::Alternative3 {
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
for __buffa::view::reference_outcome::Alternative3View<'_> {
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
impl ::protovalidate_buffa::Validate for reference_outcome::Alternative4 {
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
for __buffa::view::reference_outcome::Alternative4View<'_> {
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
impl ::protovalidate_buffa::Validate for ReferenceState {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
        if let Some(inner) = self.temperature.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("temperature"),
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
        if let Some(inner) = self.pressure.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("pressure")),
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
        if let Some(v) = self.subject.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(7i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("subject")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(15i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(13i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})",
                                16usize,
                                v.len(),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceStateView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.id.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("id")),
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
        if let Some(inner) = self.kind.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("kind")),
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
        if let Some(inner) = self.temperature.as_option() {
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
                                            field_name: Some(
                                                ::std::borrow::Cow::Borrowed("temperature"),
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
        if let Some(inner) = self.pressure.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("pressure")),
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
        if let Some(v) = self.subject.as_deref() {
            if v.len() != 16usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(7i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("subject")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(15i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(13i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be exactly {} bytes (got {})",
                                16usize,
                                v.len(),
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
impl ::protovalidate_buffa::Validate for ReferenceStateId {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceStateIdView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for ReferenceStateKind {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "elemental_at_conditions",
                "compound_at_conditions",
                "ideal_gas_at_conditions",
                "custom",
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ReferenceStateKindView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &[
                "elemental_at_conditions",
                "compound_at_conditions",
                "ideal_gas_at_conditions",
                "custom",
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for ScaleKind {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["point", "difference"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for __buffa::view::ScaleKindView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            const ALLOWED: &[&str] = &["point", "difference"];
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(14i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("string")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(10i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("in")),
                                    field_type: Some(::protovalidate_buffa::FieldType::String),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("string.in"),
                        message: ::std::borrow::Cow::Owned(
                            ::std::format!(
                                "value must be one of [{}]",
                                ALLOWED.join(", "),
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
impl ::protovalidate_buffa::Validate for UnitFactor {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.unit.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("unit")),
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
        if let Some(inner) = self.exponent.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("exponent")),
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
impl ::protovalidate_buffa::Validate for __buffa::view::UnitFactorView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.unit.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("unit")),
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
        if let Some(inner) = self.exponent.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("exponent")),
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
impl ::protovalidate_buffa::Validate for UnitId {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
impl ::protovalidate_buffa::Validate for __buffa::view::UnitIdView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.id.is_none() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(25i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("required")),
                                field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                key_type: None,
                                value_type: None,
                                subscript: None,
                            },
                        ],
                    },
                    rule_id: ::std::borrow::Cow::Borrowed("required"),
                    message: ::std::borrow::Cow::Borrowed("value is required"),
                    for_key: false,
                });
        }
        if self.id.is_some() {
            if let Some(v) = self.id.as_deref() {
                if v.len() != 16usize {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(1i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("id")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Bytes),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule: ::protovalidate_buffa::FieldPath {
                                elements: ::std::vec![
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(15i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("bytes")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Message),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                    ::protovalidate_buffa::FieldPathElement {
                                        field_number: Some(13i32),
                                        field_name: Some(::std::borrow::Cow::Borrowed("len")),
                                        field_type: Some(::protovalidate_buffa::FieldType::Uint64),
                                        key_type: None,
                                        value_type: None,
                                        subscript: None,
                                    },
                                ],
                            },
                            rule_id: ::std::borrow::Cow::Borrowed("bytes.len"),
                            message: ::std::borrow::Cow::Owned(
                                ::std::format!(
                                    "value must be exactly {} bytes (got {})",
                                    16usize,
                                    v.len(),
                                ),
                            ),
                            for_key: false,
                        });
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
