#!/usr/bin/env python3
"""Canonical Qiongli release-version parsing shared by release tooling."""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass, fields
import json
import re
import sys
from typing import Literal, Sequence


ReleaseLine = Literal["legacy-1x", "native-2x"]
ReleaseChannel = Literal["alpha", "beta", "stable"]

LEGACY_VERSION_SOURCE = "pyproject.toml"
NATIVE_VERSION_SOURCE = "packages/qiongli-native/Cargo.toml"

_NUMBER = r"(?:0|[1-9][0-9]*)"
_PRERELEASE_NUMBER = r"[1-9][0-9]*"
_VERSION_PATTERN = re.compile(
    rf"^(?:v)?"
    rf"(?P<major>{_NUMBER})\.(?P<minor>{_NUMBER})\.(?P<patch>{_NUMBER})"
    rf"(?:"
    rf"-(?P<semver_channel>alpha|beta)\.(?P<semver_number>{_PRERELEASE_NUMBER})"
    rf"|(?P<compact_channel>[aAbB])(?P<compact_number>{_PRERELEASE_NUMBER})"
    rf")?$"
)
_CHANNELS = ("alpha", "beta", "stable")


@dataclass(frozen=True, kw_only=True)
class ReleaseIdentity:
    product: str = "qiongli"
    release_line: ReleaseLine
    version: str
    repo_tag: str
    channel: ReleaseChannel
    prerelease_number: int | None
    package_version: str
    skill_version: str
    npm_version: str
    npm_dist_tag: str
    source_branch: str
    version_source: str
    is_prerelease: bool


def parse_release_version(
    raw: str, expected_channel: ReleaseChannel | str | None = None
) -> ReleaseIdentity:
    """Parse one supported version into its canonical cross-package identity.

    Versions before 2.0 remain on the legacy release line and support stable or
    beta releases. Version 2 uses the native release line and additionally
    supports alpha releases. Release-candidate, development, and build-metadata
    forms are deliberately rejected until their channel semantics are defined.
    """

    if expected_channel is not None and expected_channel not in _CHANNELS:
        raise ValueError(
            f"unsupported expected channel {expected_channel!r}; "
            "use alpha, beta, or stable"
        )

    value = raw.strip()
    match = _VERSION_PATTERN.fullmatch(value)
    if match is None:
        raise ValueError(
            "unsupported version format. Use stable `X.Y.Z`, beta "
            "`X.Y.ZbN` / `vX.Y.Z-beta.N`, or native alpha "
            "`2.Y.ZaN` / `v2.Y.Z-alpha.N`; numeric identifiers must not "
            "contain leading zeroes and prerelease N must start at 1"
        )

    major = int(match.group("major"))
    minor = int(match.group("minor"))
    patch = int(match.group("patch"))
    if major > 2:
        raise ValueError(
            f"unsupported release major {major}; "
            "only release lines before or at 2.x exist"
        )

    compact_channel = match.group("compact_channel")
    semantic_channel = match.group("semver_channel")
    if semantic_channel is not None:
        channel: ReleaseChannel = semantic_channel  # type: ignore[assignment]
        number = int(match.group("semver_number"))
    elif compact_channel is not None:
        channel = "alpha" if compact_channel.lower() == "a" else "beta"
        number = int(match.group("compact_number"))
    else:
        channel = "stable"
        number = None

    release_line: ReleaseLine = "native-2x" if major == 2 else "legacy-1x"
    if channel == "alpha" and release_line != "native-2x":
        raise ValueError("alpha releases are supported only on the native 2.x release line")
    if expected_channel is not None and channel != expected_channel:
        raise ValueError(
            f"release channel mismatch: version implies {channel!r}, "
            f"but {expected_channel!r} was expected"
        )

    base_version = f"{major}.{minor}.{patch}"
    if channel == "stable":
        version = base_version
        package_version = base_version
    else:
        assert number is not None
        version = f"{base_version}-{channel}.{number}"
        package_marker = "a" if channel == "alpha" else "b"
        package_version = f"{base_version}{package_marker}{number}"

    return ReleaseIdentity(
        release_line=release_line,
        version=version,
        repo_tag=f"v{version}",
        channel=channel,
        prerelease_number=number,
        package_version=package_version,
        skill_version=version,
        npm_version=version,
        npm_dist_tag="next" if channel != "stable" else "latest",
        source_branch="2.x"
        if release_line == "native-2x"
        else ("dev" if channel == "beta" else "primary"),
        version_source=NATIVE_VERSION_SOURCE
        if release_line == "native-2x"
        else LEGACY_VERSION_SOURCE,
        is_prerelease=channel != "stable",
    )


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Normalize a Qiongli release identity.")
    parser.add_argument("version")
    parser.add_argument("--expected-channel", choices=_CHANNELS)
    parser.add_argument(
        "--print-field",
        choices=tuple(field.name for field in fields(ReleaseIdentity)) + ("repo_version",),
        help="Print one field instead of the complete JSON identity.",
    )
    args = parser.parse_args(argv)

    identity = parse_release_version(args.version, args.expected_channel)
    if args.print_field:
        value = identity.repo_tag if args.print_field == "repo_version" else getattr(
            identity, args.print_field
        )
        if isinstance(value, bool) or value is None:
            print(json.dumps(value))
        else:
            print(value)
    else:
        print(json.dumps(asdict(identity), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as exc:
        print(f"[release-version] {exc}", file=sys.stderr)
        raise SystemExit(2)
