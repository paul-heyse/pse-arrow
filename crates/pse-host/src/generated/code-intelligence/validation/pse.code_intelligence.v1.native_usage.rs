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
impl ::protovalidate_buffa::Validate for NativeAnalysisBackend {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::oneof::native_analysis_backend::Kind::Lsp(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("lsp")),
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
            Some(__buffa::oneof::native_analysis_backend::Kind::NativeSnapshot(v)) => {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("native_snapshot"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::NativeAnalysisBackendView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        match &self.kind {
            Some(__buffa::view::oneof::native_analysis_backend::Kind::Lsp(v)) => {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("lsp")),
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
            Some(
                __buffa::view::oneof::native_analysis_backend::Kind::NativeSnapshot(v),
            ) => {
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
                                                field_name: Some(
                                                    ::std::borrow::Cow::Borrowed("native_snapshot"),
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
impl ::protovalidate_buffa::Validate for native_analysis_backend::Alternative1 {
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
for __buffa::view::native_analysis_backend::Alternative1View<'_> {
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
impl ::protovalidate_buffa::Validate for native_analysis_backend::Alternative2 {
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
for __buffa::view::native_analysis_backend::Alternative2View<'_> {
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
impl ::protovalidate_buffa::Validate for NativeAnalysisContext {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.configuration.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("configuration"),
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
        if self.configuration.is_set() {
            if let Some(inner) = self.configuration.as_option() {
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
                                                    ::std::borrow::Cow::Borrowed("configuration"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::NativeAnalysisContextView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.configuration.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(2i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("configuration"),
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
        if self.configuration.is_set() {
            if let Some(inner) = self.configuration.as_option() {
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
                                                    ::std::borrow::Cow::Borrowed("configuration"),
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
impl ::protovalidate_buffa::Validate for NativeByteLocation {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.path.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("path")),
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
        if self.path.is_set() {
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
impl ::protovalidate_buffa::Validate for __buffa::view::NativeByteLocationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.path.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("path")),
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
        if self.path.is_set() {
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
impl ::protovalidate_buffa::Validate for NativeConfiguration {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.backend.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("backend")),
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
        if let Some(inner) = self.project.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("project")),
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
impl ::protovalidate_buffa::Validate for __buffa::view::NativeConfigurationView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.backend.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("backend")),
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
        if let Some(inner) = self.project.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("project")),
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
impl ::protovalidate_buffa::Validate for NativeDependencyReference {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.definition.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("definition"),
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
        if self.definition.is_set() {
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
                                                field_number: Some(1i32),
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
        }
        for (idx, elem) in self.profiles.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("profiles")),
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
        if !self.occurrence.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("occurrence"),
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
        if self.occurrence.is_set() {
            if let Some(inner) = self.occurrence.as_option() {
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
                                                    ::std::borrow::Cow::Borrowed("occurrence"),
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
impl ::protovalidate_buffa::Validate
for __buffa::view::NativeDependencyReferenceView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.definition.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("definition"),
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
        if self.definition.is_set() {
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
                                                field_number: Some(1i32),
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
        }
        for (idx, elem) in self.profiles.iter().enumerate() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("profiles")),
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
        if !self.occurrence.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(3i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("occurrence"),
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
        if self.occurrence.is_set() {
            if let Some(inner) = self.occurrence.as_option() {
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
                                                    ::std::borrow::Cow::Borrowed("occurrence"),
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
impl ::protovalidate_buffa::Validate for NativeDependencyUsageAnswer {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.session.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("session")),
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
        if self.session.is_set() {
            if let Some(inner) = self.session.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("session")),
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
        for (idx, elem) in self.references.iter().enumerate() {
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
                                                ::std::borrow::Cow::Borrowed("references"),
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
for __buffa::view::NativeDependencyUsageAnswerView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if !self.session.is_set() {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("session")),
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
        if self.session.is_set() {
            if let Some(inner) = self.session.as_option() {
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
                                                field_name: Some(::std::borrow::Cow::Borrowed("session")),
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
        for (idx, elem) in self.references.iter().enumerate() {
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
                                                ::std::borrow::Cow::Borrowed("references"),
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
impl ::protovalidate_buffa::Validate for NativeDependencyUsageQuery {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.package_ids.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("package_ids"),
                                ),
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
                            self.package_ids.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.package_ids.len() > 4294967295usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("package_ids"),
                                ),
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
                            4294967295usize,
                            self.package_ids.len(),
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
for __buffa::view::NativeDependencyUsageQueryView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.package_ids.len() < 1usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("package_ids"),
                                ),
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
                            self.package_ids.len(),
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.package_ids.len() > 4294967295usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(
                                    ::std::borrow::Cow::Borrowed("package_ids"),
                                ),
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
                            4294967295usize,
                            self.package_ids.len(),
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
impl ::protovalidate_buffa::Validate for NativeLibraryProfile {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.manifest.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("manifest")),
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
        if let Some(inner) = self.source_root.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("source_root"),
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
        if let Some(inner) = self.library_root.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("library_root"),
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
        {
            let mut seen = ::std::collections::HashSet::with_capacity(
                self.features.len(),
            );
            let dup = self.features.iter().any(|item| !seen.insert(item));
            if dup {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(5i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("features")),
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
                                    field_number: Some(18i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(3i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("unique")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("repeated.unique"),
                        message: ::std::borrow::Cow::Borrowed(
                            "repeated value must contain unique items",
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
impl ::protovalidate_buffa::Validate for __buffa::view::NativeLibraryProfileView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if let Some(inner) = self.manifest.as_option() {
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
                                            field_name: Some(::std::borrow::Cow::Borrowed("manifest")),
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
        if let Some(inner) = self.source_root.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("source_root"),
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
        if let Some(inner) = self.library_root.as_option() {
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
                                                ::std::borrow::Cow::Borrowed("library_root"),
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
        {
            let mut seen = ::std::collections::HashSet::with_capacity(
                self.features.len(),
            );
            let dup = self.features.iter().any(|item| !seen.insert(item));
            if dup {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(5i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("features")),
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
                                    field_number: Some(18i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("repeated")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(3i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("unique")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Bool),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                            ],
                        },
                        rule_id: ::std::borrow::Cow::Borrowed("repeated.unique"),
                        message: ::std::borrow::Cow::Borrowed(
                            "repeated value must contain unique items",
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
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("platform")),
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
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("platform")),
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                    key_type: None,
                                    value_type: None,
                                    subscript: Some(
                                        ::protovalidate_buffa::Subscript::Index(idx as u64),
                                    ),
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
                                    field_number: Some(4i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("items")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(5i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("uint32")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(3i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("lte")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
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
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(1i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("value")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                    key_type: None,
                                    value_type: None,
                                    subscript: Some(
                                        ::protovalidate_buffa::Subscript::Index(idx as u64),
                                    ),
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
                                    field_number: Some(4i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("items")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(5i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("uint32")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Message),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(3i32),
                                    field_name: Some(::std::borrow::Cow::Borrowed("lte")),
                                    field_type: Some(::protovalidate_buffa::FieldType::Uint32),
                                    key_type: None,
                                    value_type: None,
                                    subscript: None,
                                },
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
