// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One lowering of explicit assertion truth into ordinary native selection queries.

use pse_schema::model::{RuleExpr, RulePlan};

pub(crate) fn candidates(plan: &RulePlan) -> Vec<(&'static str, RulePlan)> {
    match plan {
        RulePlan::Assert { input, predicate } => [
            ("true", RuleExpr::IsTrue(Box::new(predicate.clone()))),
            ("false", RuleExpr::IsFalse(Box::new(predicate.clone()))),
            ("unknown", RuleExpr::IsUnknown(Box::new(predicate.clone()))),
        ]
        .into_iter()
        .map(|(truth, predicate)| {
            (
                truth,
                RulePlan::Filter {
                    input: input.clone(),
                    predicate,
                },
            )
        })
        .collect(),
        RulePlan::Project { input, columns } => candidates(input)
            .into_iter()
            .map(|(truth, input)| {
                (
                    truth,
                    RulePlan::Project {
                        input: Box::new(input),
                        columns: columns.clone(),
                    },
                )
            })
            .collect(),
        _ => vec![("true", plan.clone())],
    }
}
