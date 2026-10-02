-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Immutable Plan 25e -> 25f transition (ADR-0148); prior history declarations stay unchanged.
-- Quiescent namespace admission and exact source verification precede this atomic step.
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.budget' AFTER 'validation.invariant';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.contract' AFTER 'authoring.budget';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.derived_write' AFTER 'authoring.contract';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.document_io' AFTER 'authoring.derived_write';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.missing_id' AFTER 'authoring.document_io';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.package_unresolved' AFTER 'authoring.missing_id';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.package_version_conflict' AFTER 'authoring.package_unresolved';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.rename_named' AFTER 'authoring.package_version_conflict';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.schema_version' AFTER 'authoring.rename_named';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.syntax' AFTER 'authoring.schema_version';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.unknown_key' AFTER 'authoring.syntax';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.unknown_row_key' AFTER 'authoring.unknown_key';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'authoring.unresolved_target' AFTER 'authoring.unknown_row_key';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'candidate.evaluation_failed' AFTER 'authoring.unresolved_target';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'compiler.cancelled' AFTER 'candidate.evaluation_failed';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'compiler.limit' AFTER 'compiler.cancelled';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'compiler.math' AFTER 'compiler.limit';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'compiler.missing' AFTER 'compiler.math';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'compiler.modeling' AFTER 'compiler.missing';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'compiler.structure' AFTER 'compiler.modeling';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'compiler.syntax' AFTER 'compiler.structure';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'domain.potential_evaluation_error' AFTER 'compiler.syntax';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'equation.canceling_terms' AFTER 'domain.potential_evaluation_error';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'equation.large_residual' AFTER 'equation.canceling_terms';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'equation.mismatched_term' AFTER 'equation.large_residual';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'equation.term_evaluation_failed' AFTER 'equation.mismatched_term';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'fit.candidate_validation' AFTER 'equation.term_evaluation_failed';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'fit.final_evaluation' AFTER 'fit.candidate_validation';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'fit.objective_overflow' AFTER 'fit.final_evaluation';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'fit.response_rank' AFTER 'fit.objective_overflow';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.analysis_inconclusive' AFTER 'fit.response_rank';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.condition_estimate' AFTER 'jacobian.analysis_inconclusive';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.extreme_column' AFTER 'jacobian.condition_estimate';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.extreme_entry' AFTER 'jacobian.extreme_column';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.extreme_row' AFTER 'jacobian.extreme_entry';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.numerical_rank_deficiency' AFTER 'jacobian.extreme_row';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.parallel_columns' AFTER 'jacobian.numerical_rank_deficiency';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'jacobian.parallel_rows' AFTER 'jacobian.parallel_columns';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.applicability' AFTER 'jacobian.parallel_rows';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.cancelled' AFTER 'math.applicability';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.coefficient_range' AFTER 'math.cancelled';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.contract' AFTER 'math.coefficient_range';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.domain' AFTER 'math.contract';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.evaluation' AFTER 'math.domain';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.library' AFTER 'math.evaluation';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.limit' AFTER 'math.library';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.native' AFTER 'math.limit';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.provider' AFTER 'math.native';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.quantity' AFTER 'math.provider';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.range' AFTER 'math.quantity';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'math.validity' AFTER 'math.range';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.budget' AFTER 'math.validity';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.cancelled' AFTER 'modeling.budget';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.capability' AFTER 'modeling.cancelled';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.invalid_model' AFTER 'modeling.capability';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.numerical' AFTER 'modeling.conditional_unit.admission.invalid_model';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.trial_rejected' AFTER 'modeling.conditional_unit.admission.numerical';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.contract' AFTER 'modeling.conditional_unit.admission.trial_rejected';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.diagnostic_samples' AFTER 'modeling.contract';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.diagnostics' AFTER 'modeling.diagnostic_samples';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.domain' AFTER 'modeling.diagnostics';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.domain.tightened' AFTER 'modeling.domain';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.dynamic.algebraic_reset.unsupported' AFTER 'modeling.domain.tightened';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.dynamic.inventory_transfer.event.unsupported' AFTER 'modeling.dynamic.algebraic_reset.unsupported';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.initialization.attempt_limit' AFTER 'modeling.dynamic.inventory_transfer.event.unsupported';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.initialization.incomplete' AFTER 'modeling.initialization.attempt_limit';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.initialization.minimum_step' AFTER 'modeling.initialization.incomplete';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.initialization.outcome' AFTER 'modeling.initialization.minimum_step';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.initialization.step_precision' AFTER 'modeling.initialization.outcome';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.nonlinear.attempt_limit' AFTER 'modeling.initialization.step_precision';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.nonlinear.cancelled' AFTER 'modeling.nonlinear.attempt_limit';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.nonlinear.initial_inconclusive' AFTER 'modeling.nonlinear.cancelled';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.nonlinear.time_limit' AFTER 'modeling.nonlinear.initial_inconclusive';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.objective' AFTER 'modeling.nonlinear.time_limit';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.provenance' AFTER 'modeling.objective';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.qualification.infeasibility_contradicted' AFTER 'modeling.provenance';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.qualification.rejected' AFTER 'modeling.qualification.infeasibility_contradicted';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.realization' AFTER 'modeling.qualification.rejected';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.staged.start' AFTER 'modeling.realization';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.study.initialization' AFTER 'modeling.staged.start';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.study.predecessor' AFTER 'modeling.study.initialization';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.study.procedure' AFTER 'modeling.study.predecessor';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.trajectory.incomplete' AFTER 'modeling.study.procedure';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.trajectory.rejected' AFTER 'modeling.trajectory.incomplete';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.cancelled' AFTER 'modeling.trajectory.rejected';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.contract' AFTER 'native.cancelled';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.internal' AFTER 'native.contract';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.limit' AFTER 'native.internal';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.numerical' AFTER 'native.limit';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.reuse' AFTER 'native.numerical';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.route_refused' AFTER 'native.reuse';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.structural' AFTER 'native.route_refused';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.unavailable' AFTER 'native.structural';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'native.unsupported' AFTER 'native.unavailable';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'objective.priority_optimization_incomplete' AFTER 'native.unsupported';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'shooting.candidate_validation' AFTER 'objective.priority_optimization_incomplete';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'shooting.evaluation' AFTER 'shooting.candidate_validation';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'structural.overdetermined' AFTER 'shooting.evaluation';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'structural.underdetermined' AFTER 'structural.overdetermined';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.fixed_zero' AFTER 'structural.underdetermined';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.large_value' AFTER 'variable.fixed_zero';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.missing_value' AFTER 'variable.large_value';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.near_lower_bound' AFTER 'variable.missing_value';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.near_upper_bound' AFTER 'variable.near_lower_bound';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.nonfinite' AFTER 'variable.near_upper_bound';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.only_in_inequalities' AFTER 'variable.nonfinite';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.outside_lower_bound' AFTER 'variable.only_in_inequalities';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.outside_upper_bound' AFTER 'variable.outside_lower_bound';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.small_value' AFTER 'variable.outside_upper_bound';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'variable.unused' AFTER 'variable.small_value';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.contract' AFTER 'variable.unused';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.ephemeral_publication' AFTER 'workflow.contract';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.export_expired' AFTER 'workflow.ephemeral_publication';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.job_payload_version' AFTER 'workflow.export_expired';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.legacy_workspace' AFTER 'workflow.job_payload_version';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.operations' AFTER 'workflow.legacy_workspace';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.publication_unresolved' AFTER 'workflow.operations';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.unclassified' AFTER 'workflow.publication_unresolved';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'quantity.unknown_id' AFTER 'workflow.unclassified';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'quantity.contract_mismatch' AFTER 'quantity.unknown_id';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'quantity.unit_conversion' AFTER 'quantity.contract_mismatch';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'quantity.incompatible' AFTER 'quantity.unit_conversion';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'quantity.nonfinite' AFTER 'quantity.incompatible';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.input' AFTER 'quantity.nonfinite';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.internal' AFTER 'workflow.input';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'workflow.panic' AFTER 'workflow.internal';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'diagnostic.aggregate' AFTER 'workflow.panic';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.binding.target' AFTER 'diagnostic.aggregate';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.binding.duplicate' AFTER 'study.binding.target';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.binding.physical' AFTER 'study.binding.duplicate';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.binding.revision' AFTER 'study.binding.physical';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.dependency.unusable' AFTER 'study.binding.revision';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.seed.unavailable' AFTER 'study.dependency.unusable';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.seed.incompatible' AFTER 'study.seed.unavailable';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.seed.internal' AFTER 'study.seed.incompatible';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.operation.unsupported' AFTER 'study.seed.internal';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.unsupported' AFTER 'study.operation.unsupported';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.cancelled' AFTER 'modeling.conditional_unit.admission.unsupported';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.resource_limit' AFTER 'modeling.conditional_unit.admission.cancelled';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.nonfinite' AFTER 'modeling.conditional_unit.admission.resource_limit';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.infrastructure' AFTER 'modeling.conditional_unit.admission.nonfinite';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.conflict' AFTER 'modeling.conditional_unit.admission.infrastructure';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.incompatible' AFTER 'modeling.conditional_unit.admission.conflict';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.internal' AFTER 'modeling.conditional_unit.admission.incompatible';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'modeling.conditional_unit.admission.inconclusive' AFTER 'modeling.conditional_unit.admission.internal';
ALTER TYPE pse_ops.diagnostic_code ADD VALUE 'study.policy.admission' AFTER 'modeling.conditional_unit.admission.inconclusive';
UPDATE pse_ops.schema_support_state SET ready=false WHERE history='operations';
ALTER TABLE pse_ops.study_point_members DROP CONSTRAINT study_point_members_point_fkey;
ALTER TABLE pse_ops.study_points DROP CONSTRAINT study_points_predecessor_fkey;
ALTER TABLE pse_ops.study_points DROP CONSTRAINT study_points_pkey;
ALTER TABLE pse_ops.study_points DROP CONSTRAINT study_points_job_id_key;
ALTER TABLE pse_ops.study_points DROP CONSTRAINT study_points_binding_key;
ALTER TABLE pse_ops.study_points RENAME TO study_points_25e;
CREATE TABLE pse_ops."study_points" (
    "study_id" pse_ops.study_id NOT NULL,
    "point_index" integer NOT NULL,
    "binding_hash" pse_ops.content_hash NOT NULL,
    "job_id" pse_ops.job_id NOT NULL,
    "state" pse_ops.study_point_state NOT NULL,
    "revision" bigint NOT NULL,
    "policy" jsonb NOT NULL,
    "outcome" jsonb NOT NULL,
    "updated_at" timestamptz NOT NULL,
    CONSTRAINT study_points_pkey PRIMARY KEY ("study_id", "point_index"),
    CONSTRAINT study_points_job_id_key UNIQUE ("job_id"),
    CONSTRAINT study_points_point_index_nonnegative_check CHECK ("point_index" >= 0),
    CONSTRAINT study_points_revision_nonnegative_check CHECK ("revision" >= 0)
);
-- No definitions, payloads, content identities, member rows or lifecycle values are rewritten.
-- Historical predecessor/content remain explicit unavailable evidence, never modern permission.
INSERT INTO pse_ops.study_points(study_id,point_index,binding_hash,job_id,state,revision,policy,outcome,updated_at)
SELECT study_id,point_index,binding_hash,job_id,state,0,
       jsonb_build_object('version',1,'kind','legacy_unavailable','predecessor',predecessor,'binding_hash',encode(binding_hash,'hex')),
       jsonb_build_object('version',1,'kind','legacy_unavailable','predecessor',predecessor,'binding_hash',encode(binding_hash,'hex')),
       updated_at
FROM pse_ops.study_points_25e;
DROP TABLE pse_ops.study_points_25e;
ALTER TABLE pse_ops.study_points ADD CONSTRAINT study_points_study_id_fkey
    FOREIGN KEY (study_id) REFERENCES pse_ops.studies(study_id);
ALTER TABLE pse_ops.study_points ADD CONSTRAINT study_points_job_id_fkey
    FOREIGN KEY (job_id) REFERENCES pse_ops.jobs(job_id);
ALTER TABLE pse_ops.study_point_members ADD CONSTRAINT study_point_members_point_fkey
    FOREIGN KEY (study_id,point_index) REFERENCES pse_ops.study_points(study_id,point_index);
CREATE INDEX study_points_pending_idx ON pse_ops.study_points(study_id,point_index)
    WHERE state='pending';
ALTER TABLE pse_ops.study_points ALTER COLUMN updated_at SET DEFAULT now();
UPDATE pse_ops.schema_support_state
SET source=CASE WHEN source='fresh' THEN 'fresh-v25e' ELSE '60293f46ba51830bb2a376d9554cae582e4f715234fa7163c38e372994de915b' END,
    target='13dd4f91810792475382a021425807a8ef42551bc2eaea71d6f9a8702009d00a',ready=true
WHERE history='operations';
COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 bb4d7025a28c62d4cef788de61277b31b4c1ae88fa6f0160d94dcc6f521cd759';
