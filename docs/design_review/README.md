# Design standard and reviews

The [core principles](design_principles/core/design-principles.md) organize review around
six architectural foundations. The [template](design_principles/core/design-review-template.md)
defines the review contract; [standard.toml](design_principles/standard.toml) selects its
versions and profiles. The [repository binding](design_principles/binding/pse-arrow.md)
provides local authority and scenario routes.

Start with the [design-change workflow](../authoritative_design/sections/design-change-workflow.md)
and the affected contract. Reviews are observations, not implementation certificates; adopted
findings have one disposition owner in the relevant plan. A review stays here while a finding
or a pending decision depends on it and then retires to Git history (ADR-0096); retained ADRs
cite retired reviews as `git:<commit>:<path>`.

`evidence/` keeps two pinned Arrow/DataFusion characterization probes that the
[capability maps](../capability-maps/README.md) cite for library behavior.
