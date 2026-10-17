-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
-- Empty results mean no reported error/loss statistic, not proof of capture completeness.
SELECT name, severity, source, value FROM stats
WHERE value != 0 AND severity IN ('error', 'data_loss')
ORDER BY severity, name;
