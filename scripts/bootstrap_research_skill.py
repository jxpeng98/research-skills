from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from research_skills.universal_installer import PROFILE_CHOICES, TARGET_CHOICES, InstallOptions, install


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Cross-platform local installer for research-skills with partial/full profiles."
    )
    parser.add_argument(
        "--profile",
        choices=PROFILE_CHOICES,
        default="partial",
        help="Install preset: partial (assets only) or full (assets + shell CLI when supported + doctor).",
    )
    parser.add_argument(
        "--target",
        choices=TARGET_CHOICES,
        default="all",
        help="Install target (default: all).",
    )
    parser.add_argument(
        "--mode",
        choices=("copy", "link"),
        default="copy",
        help="Install mode (default: copy).",
    )
    parser.add_argument(
        "--project-dir",
        default=str(Path.cwd()),
        help="Project directory for workflow integration (default: current dir).",
    )
    parser.add_argument("--overwrite", action="store_true", help="Overwrite existing installed files.")
    parser.add_argument("--install-cli", action="store_true", help="Force shell CLI installation.")
    parser.add_argument("--no-cli", action="store_true", help="Skip shell CLI installation.")
    parser.add_argument("--doctor", action="store_true", help="Force doctor after install.")
    parser.add_argument("--no-doctor", action="store_true", help="Skip doctor even in full profile.")
    parser.add_argument("--cli-dir", help="Directory for shell CLI binaries.")
    parser.add_argument("--dry-run", action="store_true", help="Show install actions without writing files.")
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    install_cli = None
    if args.install_cli:
        install_cli = True
    if args.no_cli:
        install_cli = False

    doctor = None
    if args.doctor:
        doctor = True
    if args.no_doctor:
        doctor = False

    options = InstallOptions(
        repo_root=REPO_ROOT,
        project_dir=Path(args.project_dir),
        target=args.target,
        mode=args.mode,
        overwrite=args.overwrite,
        install_cli=install_cli,
        cli_dir=Path(args.cli_dir) if args.cli_dir else None,
        doctor=doctor,
        dry_run=args.dry_run,
        profile=args.profile,
    )
    return install(options)


if __name__ == "__main__":
    raise SystemExit(main())
