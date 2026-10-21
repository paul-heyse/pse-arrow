DEFINE TABLE studies SCHEMAFULL; DEFINE FIELD retired ON studies TYPE bool DEFAULT false;
DEFINE TABLE analyses SCHEMAFULL; DEFINE FIELD retired ON analyses TYPE bool DEFAULT false;
DEFINE TABLE other SCHEMAFULL;
DEFINE TABLE obligations SCHEMAFULL;
DEFINE FIELD owner ON obligations TYPE option<record<studies|analyses>> REFERENCE ON DELETE REJECT;
INFO FOR TABLE obligations;
