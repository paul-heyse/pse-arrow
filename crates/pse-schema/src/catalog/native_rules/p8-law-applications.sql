-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

SELECT
  context.application_id,
  context.law_instance_decl_id,
  context.law_template_id,
  context.owner_instance_id,
  context.scope_id,
  context.product_id,
  binding.family AS law_family,
  context.quantity_type_id,
  context.basis_id,
  named_struct('kind', binding.subject_kind, 'total',
    CASE WHEN binding.subject_kind = 'total' THEN named_struct('phase', context.coordinates.phase) END,
    'energy',
    CASE WHEN binding.subject_kind = 'energy' THEN named_struct('phase', context.coordinates.phase) END,
    'momentum',
    CASE WHEN binding.subject_kind = 'momentum' THEN named_struct('phase', context.coordinates.phase) END,
    'species',
    CASE WHEN binding.subject_kind = 'species' THEN named_struct('member', context.coordinates.member, 'phase', context.coordinates.phase) END,
    'element',
    CASE WHEN binding.subject_kind = 'element' THEN named_struct('member', context.coordinates.member, 'phase', context.coordinates.phase) END,
    'phase_species',
    CASE WHEN binding.subject_kind = 'phase_species' THEN named_struct('member', context.coordinates.member, 'phase', context.coordinates.phase) END) AS subject,
  binding.source_family,
  binding.subject_projection,
  binding.expansion,
  context.derivation_id
FROM inferred.law_contexts AS context
JOIN reference.law_bindings AS binding
  ON context.law_template_id = binding.law_template_id
 AND context.balance_enum_id = binding.balance_enum_id
 AND context.balance_member = binding.balance_member
