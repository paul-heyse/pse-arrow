// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Measurement selection from admitted typed attributes, never a second input model.
use crate::{
    CheckedPackage, DeclarationId, Result, Type, invalid,
    provenance::{Provenance, Reader},
    specialize::Value,
};
use pse_model::generated::enums::{ModelingDataFacet, ModelingUncertaintyKind};
use pse_quantity::{QuantityTypeId, scheme::Scheme};

/// Canonical measured value and difference-unit standard deviation with its exact origin.
#[derive(Clone, Copy, Debug)]
pub struct Measurement<'a> {
    /// The physical convention of the measured value, including datum and basis.
    pub quantity: QuantityTypeId,
    /// Missing remains explicit for withheld observations.
    pub value: Option<f64>,
    /// Standard deviation in the value's canonical difference unit.
    pub standard_deviation: Option<f64>,
    /// The origin admitted with the selected value attribute.
    pub provenance: &'a Provenance,
}
impl CheckedPackage {
    /// Select one measured attribute and its declared uncertainty, or one explicitly
    /// selected standard-deviation attribute. The scientific reader's taint rules apply.
    pub fn measurement(
        &self,
        root: DeclarationId,
        record: DeclarationId,
        attribute: &str,
        standard_deviation_attribute: Option<&str>,
    ) -> Result<Measurement<'_>> {
        let entity = self
            .record(record)
            .ok_or_else(|| invalid(record, "measurement record absent"))?;
        let provenance = self
            .attribute_provenance(record, attribute)
            .filter(|origin| origin.role.facets.contains(&ModelingDataFacet::Measured))
            .ok_or_else(|| invalid(record, "selected attribute requires a measured-role origin"))?;
        Reader::of(self, root).read(self, record, self.is_test_only(record), || {
            format!("measurement {}.{attribute}", record)
        })?;
        let value = entity
            .values
            .get(attribute)
            .ok_or_else(|| invalid(record, format!("measurement attribute {attribute} absent")))?;
        let declaration = self.kinds[&entity.kind]
            .attributes
            .iter()
            .find(|(name, _)| name == attribute)
            .map(|(_, id)| *id)
            .ok_or_else(|| invalid(record, "measurement attribute schema absent"))?;
        let ty = self
            .types
            .get(&declaration)
            .ok_or_else(|| invalid(record, "measurement type absent"))?;
        let ty = if let Type::Optional(element) = ty {
            element.as_ref()
        } else {
            ty
        };
        let Type::Quantity(scheme) = ty else {
            return Err(invalid(
                record,
                "a measurement value is a physical quantity",
            ));
        };
        let quantity = scheme
            .resolve_with_evidence(
                &self.quantities,
                &Default::default(),
                self.preconditions.as_ref(),
            )
            .map_err(|error| invalid(record, error.to_string()))?;
        let value = number(value, quantity, record, attribute)?;
        let uncertainty = entity.uncertainties.get(attribute);
        let standard_deviation = if let Some(name) = standard_deviation_attribute {
            if uncertainty.is_some() {
                return Err(invalid(
                    record,
                    "measurement uncertainty has two declarations",
                ));
            }
            let sigma = entity.values.get(name).ok_or_else(|| {
                invalid(
                    record,
                    format!("standard-deviation attribute {name} absent"),
                )
            })?;
            let difference = Scheme::Delta(Box::new(Scheme::Concrete(quantity)))
                .resolve_with_evidence(
                    &self.quantities,
                    &Default::default(),
                    self.preconditions.as_ref(),
                )
                .map_err(|error| invalid(record, error.to_string()))?;
            number(sigma, difference, record, name)?
        } else {
            match uncertainty {
                None => None,
                Some(uncertainty) => match uncertainty.kind {
                    ModelingUncertaintyKind::Standard => Some(uncertainty.magnitude),
                    ModelingUncertaintyKind::Relative => {
                        value.map(|value| value.abs() * uncertainty.magnitude)
                    }
                    ModelingUncertaintyKind::Bound => {
                        return Err(invalid(
                            record,
                            "an uncertainty bound is not a standard deviation",
                        ));
                    }
                },
            }
        };
        Ok(Measurement {
            quantity,
            value,
            standard_deviation,
            provenance,
        })
    }
}
fn number(
    value: &Value,
    quantity: QuantityTypeId,
    at: DeclarationId,
    name: &str,
) -> Result<Option<f64>> {
    match value {
        Value::Missing => Ok(None),
        Value::Number {
            bits,
            quantity: actual,
        } if *actual == quantity => Ok(Some(f64::from_bits(*bits))),
        _ => Err(invalid(
            at,
            format!(
                "measurement attribute {name} requires quantity {}, got {value:?}",
                quantity.as_id()
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        PhysicalScope, TypeContext, check,
        kernel_types::{physical, source},
    };
    #[test]
    fn measured_attributes_preserve_canonical_values_uncertainty_origin_and_taint() {
        let (registry, _) = physical();
        let context = TypeContext {
            quantities: &registry,
            preconditions: &pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
            scope: &PhysicalScope::default(),
        };
        let text = r#"package p {
            entity kind source provenance {attribute title:Text;}
            enum role {measured facets(measured), oracle facets(measured,test_only), fitted facets(requires_fit)}
            entity source s {title="measured"}
            entity source fit_receipt {title="qualified fit/run/source receipt"}
            entity kind sample {attribute value:Scalar?; attribute sigma:Scalar?;}
            entity sample a provenance(s,role.measured) {value=2 ± standard(0.1)}
            entity sample b provenance(s,role.measured) {value=4 ± relative(0.05)}
            entity sample c provenance(s,role.measured) {value=6,sigma=0.3}
            entity sample d provenance(s,role.oracle) {value=8 ± standard(0.4)}
            entity sample bounded provenance(s,role.measured) {value=8 ± bound(0.4)}
            entity sample fitted provenance(fit_receipt,role.fitted,lineage(fit fit_receipt)) {value=2,sigma=0.1}
            entity kind physical_sample {attribute value:Time?;attribute sigma:Scalar?;}
            entity physical_sample mismatch provenance(s,role.measured) {value=2{s},sigma=0.1}
            def D {} test F {}
        }"#;
        let package = check(&source(text), &context).unwrap();
        let root = package.names["p.D"];
        for (name, value, sigma) in [("a", 2., 0.1), ("b", 4., 0.2), ("c", 6., 0.3)] {
            let measured = package
                .measurement(
                    root,
                    package.names[&format!("p.{name}")],
                    "value",
                    (name == "c").then_some("sigma"),
                )
                .unwrap();
            assert_eq!(measured.value, Some(value));
            assert_eq!(measured.standard_deviation, Some(sigma));
            assert_eq!(measured.provenance.source, package.names["p.s"]);
        }
        assert!(
            package
                .measurement(root, package.names["p.a"], "value", Some("sigma"))
                .is_err()
        );
        let fitted = package.provenance(package.names["p.fitted"]).unwrap();
        assert_eq!(
            fitted.lineage,
            vec![(
                pse_model::generated::enums::ModelingLineageKind::Fit,
                package.names["p.fit_receipt"]
            )]
        );
        let refusal = package
            .measurement(root, package.names["p.mismatch"], "value", Some("sigma"))
            .unwrap_err()
            .to_string();
        assert!(
            refusal.contains("sigma") && refusal.contains("requires quantity"),
            "{refusal}"
        );
        assert!(
            package
                .measurement(root, package.names["p.bounded"], "value", None)
                .is_err()
        );
        assert!(
            package
                .measurement(root, package.names["p.d"], "value", None)
                .is_err()
        );
        assert!(
            package
                .measurement(package.names["p.F"], package.names["p.d"], "value", None)
                .is_ok()
        );
        assert!(
            check(
                &source(&text.replace(
                    "provenance(s,role.measured) {value=2",
                    "provenance(s,role.fitted) {value=2"
                )),
                &context
            )
            .is_err()
        );
    }
}
