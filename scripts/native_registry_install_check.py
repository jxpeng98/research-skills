#!/usr/bin/env python3
from __future__ import annotations

import runpy
import sys
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[1]
_TARGET = _REPO_ROOT / "tooling" / "scripts" / Path(__file__).name
for _import_root in (_TARGET.parent, _REPO_ROOT / "packages" / "python-qiongli" / "src", _REPO_ROOT):
    if str(_import_root) not in sys.path:
        sys.path.insert(0, str(_import_root))

if __name__ == "__main__":
    runpy.run_path(str(_TARGET), run_name="__main__")
else:
    _spec = spec_from_file_location(f"_qiongli_tooling_scripts_{Path(__file__).stem}", _TARGET)
    if _spec is None or _spec.loader is None:
        raise ImportError(f"Unable to load {_TARGET}")
    _module = module_from_spec(_spec)
    sys.modules[_spec.name] = _module
    _spec.loader.exec_module(_module)
    for _name, _value in vars(_module).items():
        if _name not in {"__name__", "__package__", "__loader__", "__spec__"}:
            globals()[_name] = _value
