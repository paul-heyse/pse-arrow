# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The staging stages: `tk read` (raw store to source-faithful Parquet) and `tk load-src`
(staged Parquet to `src_<id>` schemas), and the writer and manifest they share with readers."""
