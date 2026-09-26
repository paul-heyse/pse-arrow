# THERMO-VALIDATION-009 v0.1

Start with [thermodynamics_blueprint_validation_v0_1.md](thermodynamics_blueprint_validation_v0_1.md).

## What executed

The new deliverable audit passed 49 structural checks. Thirty bounded source-extract/host numerical probes; 1,120 bounded sequential publication schedules; 128 guard combinations; eight deliberately unsafe controls; and reruns of the inherited 26/43 reference tests and 35/42 structural checks. Full native workers: six BLOCKED, zero executed. No independent real-fluid or complete flowsheet validation.

## Files

- `thermodynamics_blueprint_validation_v0_1_register.json`: review/evidence management register, not a runtime data schema.
- `fixtures.json`: exact numerical fixtures and tolerances.
- `source_manifest.json`: pins, original blob identifiers, modifications and documentation sources.
- `scripts/extracted_chedl_kernels.py`: two small MIT-noticed source extracts, not the full packages.
- `scripts/run_numerical_probes.py`: authored numerical harness using SciPy/NumPy.
- `scripts/check_publication_model.py`: bounded sequential model and negative controls.
- `scripts/run_native_probes.py`: blocked here; optional package worker branches are untested.
- `scripts/validate_validation_package.py`: artifact and trace audit.
- `results/`: actual outputs and logs.
- `baselines/`: unchanged Step-8 bundle and its predecessor artifacts.

## Reproduction

The recorded Python/NumPy/SciPy versions are in `results/numerical_probes.json`. Run from the bundle root:

```bash
python scripts/run_numerical_probes.py
python scripts/check_publication_model.py
python scripts/run_native_probes.py
python scripts/validate_validation_package.py
```

The native runner does not install dependencies. Its missing-dependency exit status is deliberately nonzero. Changing dependency versions or fixtures requires a new evidence record, not reinterpreting this bundle's results.

## Decision

Proceed with implementation design of the semantic/behavioral core and the documented refinements. Do not infer production backend approval, real-fluid accuracy, complete scenario conformance, or native thread/failure safety from this bounded work.
