# Authoring identity projection review addendum

## 1. Decision and scope

**Proposed.** This compact addendum reviews ADR-0059 and blueprint revision 11 against the design charter. Method: inspected blueprint §6.1 and §22.1, DocumentSection, registry document admission, EntityKind and template relation declarations. Scope is the decidability of identity projection; runtime correctness, publication and complete Wave 1 acceptance are not claimed. The existing review of Wave 1 corrections still governs those broader gates.

## 2. Authority and lifecycle map

DocumentSection owns the identity/name/owner projection; RelationSpec owns row fields and keys; EnumSpec owns EntityKind. Generated DTOs and schema documents are projections. Authored entity identities and names belong to the same candidate revision as their declarations.

## 3. Semantic contracts and invariants

Registry admission checks the sole semantic-ID key, text name, registered kind and owning foreign key. Loader admission checks actual owners, cycles, duplicate identities/names and explicit/named policy. Unnamed IDs remain explicit; absent entity metadata denotes a semantic record or relationship, never inferred registration.

## 4. Derivation and execution design

Resolve parent identities and names independently of document order; hydrate only declared context; decode exact generated rows; validate entity correspondence and references. Hashing follows those decisions and never replaces them.

## 5. Representative journeys

Two templates each declare a symbol named temperature: each symbol's explicitly declared template owner determines its qualified name. Reversing document order gives the same rows. A template_ports row keyed by (template_id, name) remains a relationship record and cannot accept an invented id alias. A missing owner stops loading with its source span and produces no published revision.

## 6. Acceptance gates

| Gate | Result | Evidence and scope |
|---|---|---|
| G1 Authority | Pass for proposed design | Each projection meaning resides in DocumentSection; generated DTOs own no separate fields. |
| G2 Semantic fidelity | Pass for proposed design | Identity, relationship keys, names and parent scope remain distinct. |
| G3 Validity | Pass for proposed design | Registry and loader boundaries have explicit negative outcomes. Runtime proof remains pending. |
| G4 Hidden behavior | Pass for proposed design | First-key and first-FK guesses are explicitly prohibited. |
| G5 Consistency and recovery | Not applicable here | Existing candidate/publication contract is unchanged; no new publication mechanism. |
| G6 Transformation and reuse | Pass for proposed design | Exact identity/registration checks precede fingerprints; complete projection rows are persisted. |
| G7 Truthful claims | Pass for proposed design | This addendum claims specifiability, not tested runtime behavior. |

## 7. Principle findings

| Finding | Principles | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| Baseline composite entity-registration rule is not representable | DM-06, DM-07, DM-24 | Blueprint §6.1 blanket registration versus template_ports composite key | An implementer invents IDs or drops part of a key | Register only declared entity identities | Composite alias negative and complete-key validation |
| Baseline nested naming scope was unspecified | DM-02, DM-09 | DocumentSection had no identity/name/owner projection | Equal local names collide or depend on arbitrary FK order | Explicit owner reference and actual parent name | Reversed-order nested naming and missing/cyclic owner negatives |

Applicability: authority, identity, validity and transformation groups apply directly. Solver performance, backend binding and distributed recovery are outside this metadata correction.

## 8. Alternatives and architectural leverage

| Alternative | Risk | Cost | Decision |
|---|---|---|---|
| Existing blanket PK registration | Composite keys cannot fit entity_id | Low apparent code, invalid meaning | Reject |
| Explicit DocumentSection projection | Metadata must be admitted and persisted | Four optional fields plus validation | Select; extends the existing mechanism |
| Simpler explicit IDs for all records | Safe IDs, but no defined nested naming or entity membership | Smaller loader but fails required named-package behavior | Insufficient alone |

## 9. Verification and measurement plan

**Proposed:** focused schema/authoring force-validate fixtures for actual IDs, aliases, names, parent closure, order independence, malformed mappings and source byte spans. A metadata mutation test establishes complete fingerprint input coverage, not semantic validity. No performance claim is made.

## 10. Exceptions and unresolved decisions

No SHOULD deviation is introduced. Formal ADR acceptance and complete runtime verification remain pending under the existing decision lifecycle and plan gates.

## 11. Decision and implementation changes

| Decision | Reason | Required implementation |
|---|---|---|
| Accept scoped proposed design | Identity and naming are unambiguous without duplicate row contracts | Land admitted metadata, typed projection rows, loader validation and named negative fixtures before claiming Tested |


Revision 14 addendum (**Proposed**, ADR-0053): inspected `template_symbols.indexed_by`, `template_domains` composite keys, and nameless `authored.domains`. The missing binding made indexed target membership undecidable. An explicit instance/domain-name/domain-ID relation closes that gap without inferring identity from coordinates. The narrower alternative of rejecting every indexed target fails the requested Wave 1 target scope.

| Additional gate | Result | Evidence and required fixture |
|---|---|---|
| G1/G2/G4 | Pass for proposed binding design | Authoritative domain/member rows are explicit; no hashed labels or implicit domain choice. |
| G3/G6/G7 | Pass for proposed design; runtime pending | Check actual owner, domain declaration, type and member values before emitting target IDs; mutation and ambiguity negatives must pass. |
| G5 | Not applicable to this addition | Existing unpublished candidate and atomic publication remain unchanged. |

| Finding | Principles | Evidence | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| Local domain names lacked actual instance bindings | DM-06, DM-07, DM-24 | Template names cannot identify nameless authored domain rows | A selector can silently bind a member from another domain | Explicit `instance_domain_bindings` | Missing/wrong owner/kind, wrong arity and ambiguous coordinate fixtures |

| Addendum decision | Scope |
|---|---|
| Accept scoped proposed design | Explicit bindings and exact member resolution; no runtime acceptance claim |


Revision 15 addendum (**Proposed**, ADR-0059): `template_guards`, `template_display` and nested submodel binding values carry DSL text but are not named entities. Expression scope therefore cannot be inferred from entity registration. An independent `expression_owner_column` on the existing DocumentSection supplies exactly the owning template FK. A shared binder must cover every AST path as lexical or resolved against actual identity/domain/declaration facts before P3 or rename.

| Gate | Proposed result | Required evidence |
|---|---|---|
| G1/G2/G4 | Pass | One explicit expression owner and one shared binding mechanism; no first-FK heuristic. |
| G3/G6/G7 | Pass for design, runtime pending | Unknown/ambiguous references, binder shadowing, missing ownership and incomplete rename inventory are negative fixtures. |
| G5 | Not applicable to this metadata addition | Existing atomic document/candidate publication contract remains. |

| Decision | Scope |
|---|---|
| Accept scoped proposed design | Explicit expression ownership and complete shared source binding; execution evidence remains required. |


### Revision 20 scoped review: complete normalized expression authority

**Proposed.** Method: inspected the 43-opcode table, math sink payloads, DocumentSection binding metadata, normalized projections, package headers and unit-set definitions. Finding: arithmetic nodes lacked source roots, predicate/equation payloads and composite declaration references; package unit selection was ambiguous. Impacted principles: DM-02, DM-06, DM-07, DM-09, DM-24, DM-59 and DM-60.

| Gate | Scoped finding and verdict |
|---|---|
| G1 Authority | Pass for proposed design: typed source keys and explicit roots refer to the same declared source rows; no second authored expression authority. |
| G2 Semantic fidelity | Pass: predicates, equation senses, branches and unresolved composite references retain exact meaning; numerical Opcode vocabulary is unchanged. |
| G3 Validity | Pass for specifiability: typed alternatives, complete references, acyclicity, original-source comparison and actual unit definitions are mandatory checks. Runtime tests pending. |
| G4 Hidden behavior | Pass: no first-row unit selection, fake IDs, placeholder Boolean nodes or digest certificates. |
| G5 Recovery | Existing atomic candidate/output publication applies unchanged. |
| G6 Derivation and reuse | Pass: derived unit/source IDs name derivations; actual complete inputs and row/payload validation control reuse. |
| G7 Evidence | Pass: this review claims a sound proposed contract, not completed runtime acceptance. |

Rejected alternatives were residual-only equations (lose senses/branches), overloaded numeric opcodes (changes compiled operator meaning), opaque syntax strings or hashes as normalized keys (cannot independently admit references), and guessed package unit sets (order-dependent physical meaning). The scoped design is accepted for implementation under the maintainer's existing authorization. Required tests are those listed in ADR-0054/0059 revision-20 confirmation; full compiler and lifecycle acceptance remain open.


Revision 23 scoped addendum (**Proposed**, ADR-0054): inspected Unit, UnitSet, convert_spec and the P3 FloatConst payload. A compound spelling lacked a source UnitId even though conversion coefficients require an actual unit definition. Accept scoped: persist the fully checked derived input representation, preserve actual component-unit dependencies, reject affine/datum factors, and use its derived ID only for naming. G1/G2/G4/G6 pass for this proposed representation; G3/G7 require compound conversion and unchanged-ID mutation fixtures. Atomic publication G5 is unchanged. Guessing an arbitrary source UnitId or treating expression-text hashing as unit validation is rejected.

### Revision 26 scoped addendum — field grammar

**Verdict: Accept-scoped. Evidence: Proposed.** Exact per-field grammar makes authoring interpretation an explicit registry declaration (DM-02, DM-06). Complete nested-leaf coverage and consistency across documents are semantic admission checks; registry fingerprints only identify admitted projections. Shared parser dispatch prevents loader/P1/P3 meaning drift. Required tests cover atomic guards, nested fields and missing/conflicting mappings; terminal Wave 1 gates remain required.
