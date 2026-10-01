-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Immutable Plan 25d -> 25e transition (ADR-0146); append a new version for later targets.
CREATE TABLE pse_ops."schema_support_state" (
    "history" text NOT NULL,
    "shared_version" integer NOT NULL,
    "source" text NOT NULL,
    "target" text NOT NULL,
    "ready" boolean NOT NULL,
    CONSTRAINT schema_support_state_pkey PRIMARY KEY ("history"),
    CONSTRAINT schema_support_state_history_known_check CHECK ("history" IN ('catalog', 'operations')),
    CONSTRAINT schema_support_state_shared_version_positive_check CHECK ("shared_version" > 0)
);
INSERT INTO pse_ops.schema_support_state VALUES ('catalog',1,'a660d5317c58a235a0717b2d24e808153d833d6bb41753ad850cf22217c3da0d','3aa8a326827f2b06cde68942b520e1480b0512b1112e91b15ba33f0fdff83927',true);
