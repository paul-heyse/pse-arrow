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
impl ::protovalidate_buffa::Validate for Payload {
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
impl ::protovalidate_buffa::Validate for __buffa::view::PayloadView<'_> {
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
impl ::protovalidate_buffa::Validate for UnsafeDivision {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    match {
                        let __left: i64 = (10i64 as i64);
                        let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                            self.divisor,
                        ));
                        __left.checked_div(__right)
                    } {
                        Some(value) => value,
                        None => {
                            __cel_evaluation_error
                                .set(
                                    Some("CEL numeric overflow, conversion or division error"),
                                );
                            0 as i64
                        }
                    }
                } > (0i64 as i64)))
                {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("unsafe.division"),
                            message: ::std::borrow::Cow::Borrowed(
                                "division must be positive",
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
impl ::protovalidate_buffa::Validate for __buffa::view::UnsafeDivisionView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    match {
                        let __left: i64 = (10i64 as i64);
                        let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                            self.divisor,
                        ));
                        __left.checked_div(__right)
                    } {
                        Some(value) => value,
                        None => {
                            __cel_evaluation_error
                                .set(
                                    Some("CEL numeric overflow, conversion or division error"),
                                );
                            0 as i64
                        }
                    }
                } > (0i64 as i64)))
                {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("unsafe.division"),
                            message: ::std::borrow::Cow::Borrowed(
                                "division must be positive",
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
impl ::protovalidate_buffa::Validate for MaskedAll {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let __prior = __cel_evaluation_error.take();
                    let __cel_range = ([(0i64 as i64), (1i64 as i64)].as_slice())
                        .to_owned();
                    let __outer = __prior.or(__cel_evaluation_error.take());
                    let mut __error = None;
                    let mut __value = !false;
                    for __cel_iter_x in (__cel_range.iter()) {
                        let __previous = __cel_evaluation_error.take();
                        __error = __error.or(__previous);
                        let __predicate = ({
                            match {
                                let __left: i64 = (10i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    *__cel_iter_x,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        } == (0i64 as i64));
                        let __predicate_error = __cel_evaluation_error.take();
                        if __predicate_error.is_none() && __predicate == false {
                            __value = false;
                            __error = None;
                            break;
                        }
                        __error = __error.or(__predicate_error);
                    }
                    __cel_evaluation_error.set(__outer.or(__error));
                    __value
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("masked.all"),
                            message: ::std::borrow::Cow::Borrowed(
                                "false dominates an element error",
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
impl ::protovalidate_buffa::Validate for __buffa::view::MaskedAllView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let __prior = __cel_evaluation_error.take();
                    let __cel_range = ([(0i64 as i64), (1i64 as i64)].as_slice())
                        .to_owned();
                    let __outer = __prior.or(__cel_evaluation_error.take());
                    let mut __error = None;
                    let mut __value = !false;
                    for __cel_iter_x in (__cel_range.iter()) {
                        let __previous = __cel_evaluation_error.take();
                        __error = __error.or(__previous);
                        let __predicate = ({
                            match {
                                let __left: i64 = (10i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    *__cel_iter_x,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        } == (0i64 as i64));
                        let __predicate_error = __cel_evaluation_error.take();
                        if __predicate_error.is_none() && __predicate == false {
                            __value = false;
                            __error = None;
                            break;
                        }
                        __error = __error.or(__predicate_error);
                    }
                    __cel_evaluation_error.set(__outer.or(__error));
                    __value
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("masked.all"),
                            message: ::std::borrow::Cow::Borrowed(
                                "false dominates an element error",
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
impl ::protovalidate_buffa::Validate for MaskedExists {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let __prior = __cel_evaluation_error.take();
                    let __cel_range = ([(0i64 as i64), (1i64 as i64)].as_slice())
                        .to_owned();
                    let __outer = __prior.or(__cel_evaluation_error.take());
                    let mut __error = None;
                    let mut __value = !true;
                    for __cel_iter_x in (__cel_range.iter()) {
                        let __previous = __cel_evaluation_error.take();
                        __error = __error.or(__previous);
                        let __predicate = ({
                            match {
                                let __left: i64 = (10i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    *__cel_iter_x,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        } == (10i64 as i64));
                        let __predicate_error = __cel_evaluation_error.take();
                        if __predicate_error.is_none() && __predicate == true {
                            __value = true;
                            __error = None;
                            break;
                        }
                        __error = __error.or(__predicate_error);
                    }
                    __cel_evaluation_error.set(__outer.or(__error));
                    __value
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("masked.exists"),
                            message: ::std::borrow::Cow::Borrowed(
                                "true dominates an element error",
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
impl ::protovalidate_buffa::Validate for __buffa::view::MaskedExistsView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let __prior = __cel_evaluation_error.take();
                    let __cel_range = ([(0i64 as i64), (1i64 as i64)].as_slice())
                        .to_owned();
                    let __outer = __prior.or(__cel_evaluation_error.take());
                    let mut __error = None;
                    let mut __value = !true;
                    for __cel_iter_x in (__cel_range.iter()) {
                        let __previous = __cel_evaluation_error.take();
                        __error = __error.or(__previous);
                        let __predicate = ({
                            match {
                                let __left: i64 = (10i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    *__cel_iter_x,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        } == (10i64 as i64));
                        let __predicate_error = __cel_evaluation_error.take();
                        if __predicate_error.is_none() && __predicate == true {
                            __value = true;
                            __error = None;
                            break;
                        }
                        __error = __error.or(__predicate_error);
                    }
                    __cel_evaluation_error.set(__outer.or(__error));
                    __value
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("masked.exists"),
                            message: ::std::borrow::Cow::Borrowed(
                                "true dominates an element error",
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
impl ::protovalidate_buffa::Validate for ErrorExistsOne {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let mut __cel_count: u32 = 0u32;
                    for __cel_iter_x in (([(0i64 as i64), (1i64 as i64)].as_slice())
                        .iter())
                    {
                        if ({
                            match {
                                let __left: i64 = (10i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    *__cel_iter_x,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        } == (10i64 as i64))
                        {
                            __cel_count = (__cel_count + 1).min(2);
                        }
                    }
                    __cel_count == 1
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("error.exists_one"),
                            message: ::std::borrow::Cow::Borrowed(
                                "exists_one retains errors",
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
impl ::protovalidate_buffa::Validate for __buffa::view::ErrorExistsOneView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let mut __cel_count: u32 = 0u32;
                    for __cel_iter_x in (([(0i64 as i64), (1i64 as i64)].as_slice())
                        .iter())
                    {
                        if ({
                            match {
                                let __left: i64 = (10i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    *__cel_iter_x,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        } == (10i64 as i64))
                        {
                            __cel_count = (__cel_count + 1).min(2);
                        }
                    }
                    __cel_count == 1
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("error.exists_one"),
                            message: ::std::borrow::Cow::Borrowed(
                                "exists_one retains errors",
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
impl ::protovalidate_buffa::Validate for RangeConstructionError {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let __prior = __cel_evaluation_error.take();
                    let __cel_range = ([
                        {
                            match {
                                let __left: i64 = (1i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    self.denom,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        },
                    ]
                        .as_slice())
                        .to_owned();
                    let __outer = __prior.or(__cel_evaluation_error.take());
                    let mut __error = None;
                    let mut __value = !true;
                    for __cel_iter_x in (__cel_range.iter()) {
                        let __previous = __cel_evaluation_error.take();
                        __error = __error.or(__previous);
                        let __predicate = true;
                        let __predicate_error = __cel_evaluation_error.take();
                        if __predicate_error.is_none() && __predicate == true {
                            __value = true;
                            __error = None;
                            break;
                        }
                        __error = __error.or(__predicate_error);
                    }
                    __cel_evaluation_error.set(__outer.or(__error));
                    __value
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("error.range"),
                            message: ::std::borrow::Cow::Borrowed(
                                "range construction is not masked by a matching element",
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
impl ::protovalidate_buffa::Validate for __buffa::view::RangeConstructionErrorView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        {
            let __cel_evaluation_error = ::core::cell::Cell::new(None::<&'static str>);
            let __cel_violation_start = violations.len();
            {
                if !(({
                    let __prior = __cel_evaluation_error.take();
                    let __cel_range = ([
                        {
                            match {
                                let __left: i64 = (1i64 as i64);
                                let __right: i64 = (::protovalidate_buffa::cel::CelScalar::cel_int(
                                    self.denom,
                                ));
                                __left.checked_div(__right)
                            } {
                                Some(value) => value,
                                None => {
                                    __cel_evaluation_error
                                        .set(
                                            Some("CEL numeric overflow, conversion or division error"),
                                        );
                                    0 as i64
                                }
                            }
                        },
                    ]
                        .as_slice())
                        .to_owned();
                    let __outer = __prior.or(__cel_evaluation_error.take());
                    let mut __error = None;
                    let mut __value = !true;
                    for __cel_iter_x in (__cel_range.iter()) {
                        let __previous = __cel_evaluation_error.take();
                        __error = __error.or(__previous);
                        let __predicate = true;
                        let __predicate_error = __cel_evaluation_error.take();
                        if __predicate_error.is_none() && __predicate == true {
                            __value = true;
                            __error = None;
                            break;
                        }
                        __error = __error.or(__predicate_error);
                    }
                    __cel_evaluation_error.set(__outer.or(__error));
                    __value
                })) {
                    violations
                        .push(::protovalidate_buffa::Violation {
                            field: ::protovalidate_buffa::FieldPath::default(),
                            rule: ::protovalidate_buffa::FieldPath::default(),
                            rule_id: ::std::borrow::Cow::Borrowed("error.range"),
                            message: ::std::borrow::Cow::Borrowed(
                                "range construction is not masked by a matching element",
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
impl ::protovalidate_buffa::Validate for TextExtentControls {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.text.chars().count() < 2usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                            2usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.text.chars().count() > 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                            3usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.text.len() < 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                        ::std::format!("value must be at least {} bytes", 3usize),
                    ),
                    for_key: false,
                });
        }
        if self.text.len() > 5usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                        ::std::format!("value must be at most {} bytes", 5usize),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.optional_text.as_deref() {
            if v.chars().count() < 2usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
            if v.chars().count() > 3usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
            if v.len() < 3usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
            if v.len() > 5usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
impl ::protovalidate_buffa::Validate for __buffa::view::TextExtentControlsView<'_> {
    fn validate(
        &self,
    ) -> ::core::result::Result<(), ::protovalidate_buffa::ValidationError> {
        let mut violations: ::std::vec::Vec<::protovalidate_buffa::Violation> = ::std::vec::Vec::new();
        if self.text.chars().count() < 2usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                            2usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.text.chars().count() > 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                            3usize,
                        ),
                    ),
                    for_key: false,
                });
        }
        if self.text.len() < 3usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                        ::std::format!("value must be at least {} bytes", 3usize),
                    ),
                    for_key: false,
                });
        }
        if self.text.len() > 5usize {
            violations
                .push(::protovalidate_buffa::Violation {
                    field: ::protovalidate_buffa::FieldPath {
                        elements: ::std::vec![
                            ::protovalidate_buffa::FieldPathElement {
                                field_number: Some(1i32),
                                field_name: Some(::std::borrow::Cow::Borrowed("text")),
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
                        ::std::format!("value must be at most {} bytes", 5usize),
                    ),
                    for_key: false,
                });
        }
        if let Some(v) = self.optional_text.as_deref() {
            if v.chars().count() < 2usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
            if v.chars().count() > 3usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
            if v.len() < 3usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
                        message: ::std::borrow::Cow::Borrowed(""),
                        for_key: false,
                    });
            }
            if v.len() > 5usize {
                violations
                    .push(::protovalidate_buffa::Violation {
                        field: ::protovalidate_buffa::FieldPath {
                            elements: ::std::vec![
                                ::protovalidate_buffa::FieldPathElement {
                                    field_number: Some(2i32),
                                    field_name: Some(
                                        ::std::borrow::Cow::Borrowed("optional_text"),
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
