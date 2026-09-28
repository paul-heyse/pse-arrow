// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Which model, case, instance and fit a run's lineage, its solve rows and its numerical
//! requirements name (ADR-0115; Plan 22 B3b, B3c).
//!
//! The registry column documents of `runtime.run_lineage`, `runtime.solve_runs` and
//! `authored.numerical_requirements` own the meaning; these are its only derivations.
//!
//! - **Model** (`model_id`, a `declaration`): the specialized definition, that is the root
//!   declaration a specialization instantiates (a definition, a case or a test; a preset
//!   root names the preset). Every producer names one model by the same id: a modeling
//!   solve or simulation, a steady or transient fit experiment, an integrated
//!   simulation's requirements and an implicit block's trial hints all name the root of
//!   the specialization they belong to.
//! - **Case** (`case_id`, a `declaration`): the root, when the root is a `case` or a
//!   `test` declaration (a test is a case with an oracle; both carry a fixture); absent
//!   otherwise.
//! - **Instance** (`instance_id`): what the root became. An ordinary root instance has
//!   the root declaration's identity; a fit experiment prepares its case under its own
//!   instance. An implicit block's trial hints name the block's instance, and a generated
//!   rate system, which is no authored instance, names none.
//! - **Fit** (`fit_id`): the fit whose rows these are. A fit names the one model (and
//!   case) all its experiments specialize and no instance; when its experiments
//!   specialize different definitions, no single model exists and it names none.
//!
//! ```
//! use pse_model::generated::enums::ModelingDeclarationKind as Kind;
//! use pse_model::generated::identities::{DeclarationId, FitId, InstanceId};
//! use pse_model::lineage::{Fitted, Solved};
//!
//! let root = DeclarationId::from_bytes([7; 16]);
//! let solved = Solved::new(root, Kind::Case, InstanceId::from_bytes([7; 16]));
//! assert_eq!((solved.model(), solved.case()), (root, Some(root)));
//! let experiment = Solved::new(root, Kind::Case, InstanceId::from_bytes([8; 16]));
//! let fit = Fitted::new(FitId::from_bytes([9; 16]), [solved, experiment]);
//! assert_eq!(fit.lineage().model_id, Some(root));
//! assert_eq!(fit.lineage().instance_id, None);
//! ```
//!
//! A root is a declaration and the instance it becomes is not, even where both carry the
//! same bytes:
//!
//! ```compile_fail,E0308
//! use pse_model::generated::enums::ModelingDeclarationKind as Kind;
//! use pse_model::generated::identities::DeclarationId;
//! use pse_model::lineage::Solved;
//!
//! let root = DeclarationId::from_bytes([7; 16]);
//! let _ = Solved::new(root, Kind::Definition, root);
//! ```

use crate::generated::enums::ModelingDeclarationKind;
use crate::generated::identities::{DeclarationId, FitId, InstanceId};

/// The four lineage columns a row carries, as the registry declares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Lineage {
    /// `model_id`: the specialized definition.
    pub model_id: Option<DeclarationId>,
    /// `case_id`: the case declaration, when the root is a case.
    pub case_id: Option<DeclarationId>,
    /// `instance_id`: the instance the rows' preparation declared them for.
    pub instance_id: Option<InstanceId>,
    /// `fit_id`: the fit, for a fit's rows.
    pub fit_id: Option<FitId>,
}

/// One specialization that was solved, simulated or prepared as a fit experiment: its
/// model, its case and the instance its root became.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Solved {
    model: DeclarationId,
    case: Option<DeclarationId>,
    instance: InstanceId,
}

impl Solved {
    /// The specialization of `root`, a declaration of `kind`, as `instance`.
    #[must_use]
    pub const fn new(root: DeclarationId, kind: ModelingDeclarationKind, instance: InstanceId) -> Self {
        let case = match kind {
            ModelingDeclarationKind::Case | ModelingDeclarationKind::Test => Some(root),
            _ => None,
        };
        Self {
            model: root,
            case,
            instance,
        }
    }

    /// The specialized definition: the root declaration.
    #[must_use]
    pub const fn model(&self) -> DeclarationId {
        self.model
    }

    /// The case declaration, when the root is a case or a test.
    #[must_use]
    pub const fn case(&self) -> Option<DeclarationId> {
        self.case
    }

    /// The instance the root became.
    #[must_use]
    pub const fn instance(&self) -> InstanceId {
        self.instance
    }

    /// What this specialization's lineage row, solve rows and requirements name.
    #[must_use]
    pub const fn lineage(&self) -> Lineage {
        Lineage {
            model_id: Some(self.model),
            case_id: self.case,
            instance_id: Some(self.instance),
            fit_id: None,
        }
    }

    /// What the trial-hint requirements of an implicit stage inside this specialization
    /// name: this model and case, and the stage's block instance (`None` for a generated
    /// rate system, which is no authored instance).
    #[must_use]
    pub const fn stage(&self, block: Option<InstanceId>) -> Lineage {
        Lineage {
            model_id: Some(self.model),
            case_id: self.case,
            instance_id: block,
            fit_id: None,
        }
    }
}

/// A fit and the model and case its experiments share.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fitted {
    fit: FitId,
    model: Option<DeclarationId>,
    case: Option<DeclarationId>,
}

impl Fitted {
    /// `fit` over its experiments' specializations: it names their model and case when
    /// every experiment specializes the same root, and none otherwise.
    #[must_use]
    pub fn new(fit: FitId, experiments: impl IntoIterator<Item = Solved>) -> Self {
        let mut shared: Option<Option<(DeclarationId, Option<DeclarationId>)>> = None;
        for experiment in experiments {
            let this = (experiment.model, experiment.case);
            shared = Some(match shared {
                None => Some(this),
                Some(Some(first)) if first == this => Some(first),
                Some(_) => None,
            });
        }
        let (model, case) = shared
            .flatten()
            .map_or((None, None), |(model, case)| (Some(model), case));
        Self { fit, model, case }
    }

    /// The fit.
    #[must_use]
    pub const fn fit(&self) -> FitId {
        self.fit
    }

    /// What the fit's lineage row and its parameter requirements name.
    #[must_use]
    pub const fn lineage(&self) -> Lineage {
        Lineage {
            model_id: self.model,
            case_id: self.case,
            instance_id: None,
            fit_id: Some(self.fit),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated::enums::ModelingDeclarationKind as Kind;

    fn declaration(byte: u8) -> DeclarationId {
        DeclarationId::from_bytes([byte; 16])
    }

    fn instance(byte: u8) -> InstanceId {
        InstanceId::from_bytes([byte; 16])
    }

    /// A root names itself as the model; only a case or a test names a case.
    #[test]
    fn a_root_is_the_model_and_a_case_root_the_case() {
        for (kind, case) in [
            (Kind::Definition, None),
            (Kind::Preset, None),
            (Kind::Case, Some(declaration(1))),
            (Kind::Test, Some(declaration(1))),
        ] {
            let solved = Solved::new(declaration(1), kind, instance(2));
            assert_eq!(
                solved.lineage(),
                Lineage {
                    model_id: Some(declaration(1)),
                    case_id: case,
                    instance_id: Some(instance(2)),
                    fit_id: None,
                },
                "{kind:?}"
            );
            assert_eq!(solved.stage(None).instance_id, None);
            assert_eq!(solved.stage(Some(instance(3))).model_id, Some(declaration(1)));
        }
    }

    /// A fit names its experiments' model when they share one, and none otherwise.
    #[test]
    fn a_fit_names_the_model_its_experiments_share() {
        let fit = FitId::from_bytes([9; 16]);
        let one = Solved::new(declaration(1), Kind::Test, instance(2));
        let other_instance = Solved::new(declaration(1), Kind::Test, instance(3));
        let shared = Fitted::new(fit, [one, other_instance]).lineage();
        assert_eq!(
            shared,
            Lineage {
                model_id: Some(declaration(1)),
                case_id: Some(declaration(1)),
                instance_id: None,
                fit_id: Some(fit),
            }
        );
        let other_model = Solved::new(declaration(4), Kind::Test, instance(5));
        let spanning = Fitted::new(fit, [one, other_model]).lineage();
        assert_eq!((spanning.model_id, spanning.case_id), (None, None));
        assert_eq!(spanning.fit_id, Some(fit));
        assert_eq!(Fitted::new(fit, []).lineage().model_id, None);
    }
}
