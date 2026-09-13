# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The IDAES 2.12.0 parity harness (blueprint §25, plan §5).

The only place in the repository that may import ``idaes``, and the only
coupling between this clean-room implementation and IDAES-PSE: behaviour is
read from IDAES, never code. Parity runs in its own environment
(``.venv-parity``, CPython 3.11-3.13, ``idaes-pse==2.12.0``, ``ipopt`` on
``PATH``) and is opt-in behind ``pytest --parity``.

Parity never skips. A missing solver or a wrong IDAES version fails the
pre-flight, because a green run that silently proved nothing is worse than a red
one: an ast-grep rule bans ``pytest.skip`` and ``pytest.importorskip`` under
this package.
"""
