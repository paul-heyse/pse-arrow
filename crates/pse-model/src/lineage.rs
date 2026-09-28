// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Which model and which case a run's lineage, its solve rows and its numerical
//! requirements name (ADR-0115; Plan 22 B3b).
//!
//! `model` and `case` are declared identities without an owning relation: nothing mints
//! them. Every producer derives them from the identity it actually solved, and these are
//! the only such derivations. Each is an explicit conversion that keeps the bytes, so the
//! published `model_id` and `case_id` columns and every identity framed over a
//! requirement are unchanged by typing them.
//!
//! What the derivations say today, as the functions name it:
//! - a modeling solve or simulation names its **root instance** as both its model and
//!   its case ([`model_of_instance`], [`case_of_instance`]); an ordinary root instance has
//!   its root declaration's identity, while a fit experiment's instance has the
//!   experiment's;
//! - a fit names **itself** as both its model and its case ([`model_of_fit`],
//!   [`case_of_fit`]);
//! - modeling-sourced numerical requirements name the instance they were declared for,
//!   except an integrated simulation's, which name its **root declaration**
//!   ([`model_of_root`]), and an implicit block's trial hints, which name the **implicit
//!   stage** that solves the block ([`model_of_implicit_stage`]).
//!
//! So the model and the case of one lineage row are the same identity, and one model may
//! be named by different identities in different producers: a typed swap now fails to
//! compile, but the columns still carry the identity that was solved, not a separately
//! declared model or case.
//!
//! ```
//! use pse_model::generated::identities::{CaseId, InstanceId, ModelId};
//! use pse_model::lineage::{case_of_instance, model_of_instance};
//!
//! let root = InstanceId::from_bytes([7; 16]);
//! let (model, case): (ModelId, CaseId) = (model_of_instance(root), case_of_instance(root));
//! assert_eq!(model.as_id(), case.as_id());
//! ```
//!
//! ```compile_fail,E0308
//! use pse_model::generated::identities::{CaseId, InstanceId};
//! use pse_model::lineage::model_of_instance;
//!
//! // A model is not a case, even where both carry the same bytes.
//! let case: CaseId = model_of_instance(InstanceId::from_bytes([7; 16]));
//! ```

use crate::generated::identities::{CaseId, DeclarationId, FitId, InstanceId, ModelId};
use pse_ids::SemanticId;

/// The model a modeling solve or simulation of `instance` names: the instance itself.
#[must_use]
pub const fn model_of_instance(instance: InstanceId) -> ModelId {
    ModelId::from_id(instance.as_id())
}

/// The case a modeling solve or simulation of `instance` names: the instance itself.
#[must_use]
pub const fn case_of_instance(instance: InstanceId) -> CaseId {
    CaseId::from_id(instance.as_id())
}

/// The model an integrated simulation's numerical requirements name: its root
/// declaration, whichever instance of it was simulated.
#[must_use]
pub const fn model_of_root(root: DeclarationId) -> ModelId {
    ModelId::from_id(root.as_id())
}

/// The model a fit names, for its lineage and its parameter requirements: the fit.
#[must_use]
pub const fn model_of_fit(fit: FitId) -> ModelId {
    ModelId::from_id(fit.as_id())
}

/// The case a fit's lineage names: the fit.
#[must_use]
pub const fn case_of_fit(fit: FitId) -> CaseId {
    CaseId::from_id(fit.as_id())
}

/// The model an implicit block's trial-hint requirements name: the implicit stage that
/// solves the block, whose identity is the block's instance or a generated stage's.
#[must_use]
pub const fn model_of_implicit_stage(stage: SemanticId) -> ModelId {
    ModelId::from_id(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The derivations type the columns without moving a byte.
    #[test]
    fn lineage_derivations_keep_the_solved_identity() {
        let bytes = [0x5a; 16];
        let id = SemanticId::from_bytes(bytes);
        assert_eq!(model_of_instance(InstanceId::from_id(id)).as_id(), id);
        assert_eq!(case_of_instance(InstanceId::from_id(id)).as_id(), id);
        assert_eq!(model_of_root(DeclarationId::from_id(id)).as_id(), id);
        assert_eq!(model_of_fit(FitId::from_id(id)).as_id(), id);
        assert_eq!(case_of_fit(FitId::from_id(id)).as_id(), id);
        assert_eq!(model_of_implicit_stage(id).as_bytes(), &bytes);
    }
}
