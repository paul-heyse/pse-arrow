// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual controller/estimator mapping intents retain their existing tuple documents.

pse_model::projection_tuple_struct! {
    /// One controller path whose solved value drives the selected plant input.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ControllerMove {
        projection: { name: "ControllerMove", rust_path: crate::workflow::ControllerMove, version: 1,
            direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Controller case path whose value is applied.
        pub path: String => { tag: 1, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
        /// Driven plant input index.
        pub input: usize => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_horizon_intents_preserve_original_tuple_bytes_and_document_hash() {
        let output = pse_ids::SemanticId::from_bytes([7; 16]);
        let original = (
            vec![("move".to_owned(), 2usize)],
            Some(vec![("binding".to_owned(), "next".to_owned())]),
            vec![(output, vec!["old".to_owned(), "new".to_owned()])],
        );
        let current = (
            vec![ControllerMove {
                path: "move".into(),
                input: 2,
            }],
            Some(vec![ControllerPrediction {
                binding: "binding".into(),
                predictor: "next".into(),
            }]),
            vec![EstimatorMeasurement {
                output,
                paths: vec!["old".into(), "new".into()],
            }],
        );
        assert_eq!(
            serde_json::to_vec(&current).unwrap(),
            serde_json::to_vec(&original).unwrap()
        );
        assert_eq!(
            pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &current).unwrap(),
            pse_ids::document::of(pse_ids::Frame::DurableStudyRequestV5, &original).unwrap(),
        );
        let restored: (
            Vec<ControllerMove>,
            Option<Vec<ControllerPrediction>>,
            Vec<EstimatorMeasurement>,
        ) = serde_json::from_slice(&serde_json::to_vec(&original).unwrap()).unwrap();
        assert_eq!(restored, current);
    }
}
pse_model::projection_tuple_struct! {
    /// One controller signal binding and the case path predicting its next value.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ControllerPrediction {
        projection: { name: "ControllerPrediction", rust_path: crate::workflow::ControllerPrediction, version: 1,
            direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Existing measured or estimated controller binding.
        pub binding: String => { tag: 1, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
        /// Controller case path predicting that binding one period ahead.
        pub predictor: String => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    }
}
pse_model::projection_tuple_struct! {
    /// One original plant output and its ordered estimator case paths over the window.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct EstimatorMeasurement {
        projection: { name: "EstimatorMeasurement", rust_path: crate::workflow::EstimatorMeasurement, version: 1,
            direction: pse_model::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Existing measured plant contract output.
        pub output: pse_ids::SemanticId => { tag: 1, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
        /// Oldest-to-newest case paths, including the current window endpoint.
        pub paths: Vec<String> => { tag: 2, direction: pse_model::boundary_projection::Direction::Both, rules: [] };
    }
}
