-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse

DEFINE TABLE pse_control_transaction_probe SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD marker ON pse_control_transaction_probe TYPE string;
DEFINE API '/pse-fixture/control/success' FOR post PERMISSIONS FULL
    MIDDLEWARE api::res::body('json') THEN {
        CREATE pse_control_transaction_probe:success SET marker='committed-success';
        RETURN {status:200,body:{marker:'committed-success'}};
    };
DEFINE API '/pse-fixture/control/refusal' FOR post PERMISSIONS FULL
    MIDDLEWARE api::res::body('json') THEN {
        CREATE pse_control_transaction_probe:refusal SET marker='committed-refusal';
        RETURN {status:409,body:{marker:'committed-refusal'}};
    };
DEFINE API '/pse-fixture/control/failure' FOR post PERMISSIONS FULL
    MIDDLEWARE api::res::body('json') THEN {
        CREATE pse_control_transaction_probe:failure SET marker='uncommitted-failure';
        THROW 'private control after-write failure witness';
    };
