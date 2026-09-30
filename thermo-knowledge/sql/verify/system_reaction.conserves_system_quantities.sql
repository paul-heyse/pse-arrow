-- invariant: system_reaction.conserves_system_quantities
-- A reaction of a chemical system does not conserve a conserved quantity the system declares: the coefficient-weighted sum of its participants' composition entries for the quantity is not zero.
WITH contribution AS (
    -- the entry on the participant form, else the one on its species; a missing entry is zero
    SELECT DISTINCT ON (p.reaction, p.form, c.quantity)
           p.reaction, p.form, c.quantity, p.coefficient * c.value AS amount
    FROM tk.reaction_participant p
    JOIN tk.species_form f ON f.id = p.form
    JOIN tk.composition c ON c.entity IN (f.id, f.species)
    ORDER BY p.reaction, p.form, c.quantity, (c.entity = f.id) DESC
), balance AS (
    SELECT reaction, quantity, sum(amount) AS net FROM contribution GROUP BY reaction, quantity
)
SELECT sr.id, loc.locator, sr.system, sr.reaction, q.key AS quantity, b.net
FROM tk.system_reaction sr
JOIN tk.system_conserves sc ON sc.system = sr.system
JOIN tk.conserved_quantity q ON q.id = sc.quantity
JOIN balance b ON b.reaction = sr.reaction AND b.quantity = sc.quantity
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = sr.reaction
) loc ON true
WHERE abs(b.net) > 1e-9
