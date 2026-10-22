// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit hard guards have no selectable evidence permission.
use super::*;
use crate::envelope::Guard;
#[derive(Debug)]
pub(super) struct PreparedGuard {
    pub(super) inspection: Guard,
    pub(super) root: crate::resolved::PredicateIx,
}
impl Engine<'_, '_> {
    pub(super) fn specialize_guards(
        &mut self,
        instance: InstanceId,
        function: DeclarationId,
        contract: &crate::Function,
        statics: &Environment,
    ) -> Result<Vec<PreparedGuard>> {
        let mut specialized = Vec::new();
        for guard in &contract.envelopes {
            let mut predicate =
                self.rewrite_predicate(instance, &guard.predicate, statics, &[function])?;
            predicate.inspection.strip_spans();
            let reads = crate::envelope::Reads {
                sets: statics
                    .get(&guard.carrier)
                    .and_then(|value| self.p.set_identity(value))
                    .into_iter()
                    .collect(),
                variables: guard
                    .arguments
                    .iter()
                    .filter_map(|argument| {
                        contract
                            .arguments
                            .iter()
                            .position(|(name, _)| name == argument)
                            .map(|position| position as u32)
                    })
                    .collect(),
            };
            specialized.push(PreparedGuard {
                root: predicate.root,
                inspection: Guard {
                    predicate: predicate.inspection,
                    reads,
                    ..guard.clone()
                },
            });
        }
        Ok(specialized)
    }
}
