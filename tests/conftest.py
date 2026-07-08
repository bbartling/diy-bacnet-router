"""Test setup: pin the config directory to the repo's config/.

This makes the unit tests independent of how the package was installed
(editable vs. copied into site-packages), since ``app.config`` resolves its
config directory relative to the installed module by default.
"""

import os
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
os.environ.setdefault("RUSTY_GATEWAY_CONFIG_DIR", str(_REPO_ROOT / "config"))
