-- invariant: reaction.conserves_declared_quantities
-- A reaction's coefficient-weighted composition does not sum to zero for a conserved quantity.
WITH contribution AS (
    -- the entry on the participant form, else the one on its species; a missing entry is zero
    SELECT DISTINCT ON (p.reaction, p.form, c.quantity)
           p.reaction, p.form, c.quantity, p.coefficient * c.value AS amount
    FROM tk.reaction_participant p
    JOIN tk.species_form f ON f.id = p.form
    JOIN tk.composition c ON c.entity IN (f.id, f.species)
    ORDER BY p.reaction, p.form, c.quantity, (c.entity = f.id) DESC
), counted AS (
    -- what a participant counts for a quantity, and, for an isotope, for its element as well
    SELECT reaction, quantity, amount FROM contribution
    UNION ALL
    SELECT c.reaction, q.of_element, c.amount
    FROM contribution c
    JOIN tk.isotope i ON i.id = c.quantity
    JOIN tk.conserved_quantity q ON q.id = i.id
    WHERE q.of_element IS NOT NULL
), balance AS (
    SELECT reaction, quantity, sum(amount) AS net FROM counted GROUP BY reaction, quantity
)
SELECT r.id, loc.locator, q.key AS quantity, b.net
FROM balance b
JOIN tk.reaction r ON r.id = b.reaction
JOIN tk.conserved_quantity q ON q.id = b.quantity
LEFT JOIN LATERAL (
    SELECT string_agg(i.locator, '; ' ORDER BY i.locator) AS locator
    FROM prov.record_origin o
    JOIN prov.import_record i ON i.id = o.import_record
    WHERE o.record = r.id
) loc ON true
WHERE abs(b.net) > 1e-9
