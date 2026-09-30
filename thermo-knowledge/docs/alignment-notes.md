# Alignment notes: what the surveys require of the model

Working list for the alignment step (plan packet TK2). Each item is a fact a survey established
from pinned bytes and the change, if any, it requires of the declaration in `model/` and
`forms/`. An item is removed when the declaration holds it and a fixture proves it. This page is
a worklist, not a contract.

## Requires a model or meta-model change

No item is open. The declaration holds every change the surveys required of it, and each has a hard-case
fixture that proves it (`tests/test_hard_case_*.py`); a new survey adds a numbered row here
(`| # | Evidence | Survey | Required change |`) when it finds one.

## Held by the current declaration; the mapping must record the loss or assumption

| Evidence | Survey | How it is held |
|---|---|---|
| The equation a coefficient table belongs to is fixed only by a variable name or docstring; units are unstated or inconsistent across tables; one documented formula differs from the code | chemicals, thermo, clapeyron | a slot belongs to a form, so the mapping names the form per table; units are mapping assumptions recorded as loss; documentation-against-code differences are survey `[[discrepancy]]` records and the mapping follows what the code evaluates, saying so |
| The same parameter name has different meaning and unit in different tables (`bondvol`, `eps`, `kmat`) | clapeyron, thermopack, teqp | slots are form-local; nothing is mapped by name |
| A missing pair and a stored zero (or one) cannot be told apart | coolprop, thermo, aga8 | a stored value is mapped as `known`; the mapping rule records that the source cannot distinguish the two; a pair the source does not list produces no set |
| Computed values stored beside authored ones without a derivation record (states, ancillaries, generated estimation tables) | coolprop, chemicals | origin role `computed` or `estimated` with a `derivation` whose software or version is absent where the source does not record it |
| Enthalpy datum, temperature scale and gas constant are not stated | cantera, nasa_cea | `not_stated` members of the convention enums; a datum established numerically by the survey is a mapping assumption |
| Charge carried as a pseudo-element | cantera, nasa_cea | `composition` over `conserved_quantity` with the `charge` entity; the mapping converts the sign convention |
| One source row couples parameters of two forms (reducing parameters and the departure scale) | coolprop | the mapping emits two parameter sets from one import record |
| Identity is not a clean key: several entries share a CAS or InChIKey (ortho, para and normal hydrogen; site-scheme variants of water; conformers with different geometries) | thermopack, feos, sigma_profiles_pyscf | source entities stay distinct; spin isomers need curated decisions; site-scheme variants are one species under different parameterisations; conformer-level descriptors attach to the source entity's own record |
| A profile set is valid only with the model constants it was generated for | cosmosac_nist, sigma_profiles_pyscf | `dependency` between the profile parameterisation and the constants parameterisation |
| Fit provenance is not recoverable from the fitted database: only the best point is written, under generated symbol names with no link to datasets, and no covariance is kept | espei | a `fit` records what the source records; the missing links are `not stated`, not reconstructed |
| One wide component row feeds many incompatible models, and which model reads which column is decided by the consuming class; template rows carry placeholder values copied from water or methane, and a sentinel (an interaction value of one for ion pairs) stands for absence; most columns have no unit, range or citation | neqsim | the mapping splits one row into one parameter set per consuming form, declared per column group; placeholder and sentinel values are declared absent in `mapping.toml` with the evidence for each, and a row that is wholly template is `out_of_scope`, never data |
| Every equilibrium constant is relative to the basis its reaction is written on; the same species has different numbers under different master species, and one database ships two redox-basis variants | phreeqc | a `reaction` is its stoichiometry as written, so constants on different bases are constants of different reactions; nothing is re-based at import |
| The activity model is chosen by which keyword block a database contains, and defaults apply when a species has no ion-size parameters; site amounts and the electrostatic model are in the problem input, not in the database | phreeqc | the mapping declares the model assembly per database and records the source's block-presence rule as a selection fact; problem-input state is outside the knowledge base |
| Interaction rows are keyed by a sorted set of species names, so a repeated species loses its multiplicity; a later definition of a species or phase replaces an earlier one silently | phreeqc | subjects are role tuples, so the mapping must restore the multiplicity from the parameter kind; replaced definitions are both imported, distinguished by occurrence, with the source's last-wins rule recorded as a `selection_policy` fact |
| What a program evaluates differs from its data file (CEA replaces stated temperature ranges when compiling its library) | nasa_cea | the file is what is imported; the runtime behaviour is a survey `[[discrepancy]]` and matters for qualification against the program |
