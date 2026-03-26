from __future__ import annotations

import os
import shlex
import sys


def split_command(command: str) -> list[str]:
    parts = shlex.split(command, posix=os.name != "nt")
    if os.name == "nt":
        return [_strip_wrapping_quotes(part) for part in parts]
    return parts


def current_python_command(*args: str) -> str:
    return format_command(sys.executable, *args)


def format_command(*parts: str) -> str:
    if os.name == "nt":
        return " ".join(_quote_windows_arg(part) for part in parts)
    return " ".join(shlex.quote(part) for part in parts)


def _strip_wrapping_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


def _quote_windows_arg(value: str) -> str:
    if not value:
        return '""'
    if any(char in value for char in ' \t"'):
        escaped = value.replace('"', '\\"')
        return f'"{escaped}"'
    return value
