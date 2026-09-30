-- Hand-written physical layer, applied after the generated DDL (meta-model section 7).
--
-- Whether a form is qualified is a fact about recorded runs, not a declaration: this view lists,
-- for each form and contract output, the passing runs that were made against the form's current
-- text. A run was made against the current text when the hash it recorded equals the evaluation
-- hash that `meta.form_output` now holds for the output; editing the form's expressions changes
-- that hash, so runs made against an earlier text drop out of the view. A form with no row has
-- no passing run against its current text.
CREATE VIEW qual.form_qualification AS
SELECT
    f.name AS form,
    o.name AS output,
    r.key AS run,
    r.basis,
    s.key AS reference,
    r.points,
    r.relative_tolerance,
    r.worst_relative_deviation,
    encode(r.expression_hash, 'hex') AS expression_hash
FROM qual.qualification_run r
JOIN meta.form f ON f.id = r.form
JOIN meta.form_output o
    ON o.form = f.name AND decode(o.evaluation_hash, 'hex') = r.expression_hash
LEFT JOIN prov.source s ON s.id = r.reference
WHERE r.outcome = 'passed';

COMMENT ON VIEW qual.form_qualification IS
    'Passing qualification runs per form and contract output, made against the form''s current expression: the evidence that a form reproduces an independent answer. Derived; never declared.';
COMMENT ON COLUMN qual.form_qualification.form IS 'The form.';
COMMENT ON COLUMN qual.form_qualification.output IS 'The contract output the run compared.';
COMMENT ON COLUMN qual.form_qualification.run IS 'The run''s key.';
COMMENT ON COLUMN qual.form_qualification.basis IS 'What the run compared against.';
COMMENT ON COLUMN qual.form_qualification.reference IS 'The key of the library release or publication that supplied the reference answers.';
COMMENT ON COLUMN qual.form_qualification.points IS 'Points compared.';
COMMENT ON COLUMN qual.form_qualification.relative_tolerance IS 'Declared relative tolerance.';
COMMENT ON COLUMN qual.form_qualification.worst_relative_deviation IS 'Largest relative deviation observed.';
COMMENT ON COLUMN qual.form_qualification.expression_hash IS 'The evaluation hash the run recorded, in hex; equal to the output''s current one.';
