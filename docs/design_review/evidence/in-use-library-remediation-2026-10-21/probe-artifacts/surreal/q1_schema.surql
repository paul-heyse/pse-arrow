DEFINE TABLE attempts SCHEMAFULL;
DEFINE FIELD ident ON attempts TYPE string READONLY;
DEFINE FIELD kind ON attempts TYPE 'a' | 'b';
DEFINE FIELD opt ON attempts TYPE option<'x' | 'y'>;
DEFINE FIELD state ON attempts TYPE
    { phase: 'ingesting' }
  | { phase: 'closed', close_generation: int, manifest: string }
  | { phase: 'terminal', outcome: 'completed' | 'partial' | 'failed' | 'cancelled', completion: bytes }
  ASSERT $before = NONE
      OR $before.phase = $value.phase
      OR [$before.phase, $value.phase] IN [['ingesting','closed'], ['closed','terminal']];
DEFINE INDEX attempts_phase ON attempts FIELDS state.phase;
INFO FOR TABLE attempts;
