#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from bridges.providers.research_collab import (
    GEMINI_ACP_COMMAND_ENV,
    BROKER_BACKEND_CMD_ENV,
    BROKER_TOKEN_ENV,
    CommandJSONBackend,
    GeminiACPBackend,
    GeminiCLIBackend,
    build_default_broker_backend,
    create_broker_server,
)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run a local Gemini session broker for research-skills.",
    )
    parser.add_argument("--host", default="127.0.0.1", help="Bind host (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=8767, help="Bind port (default: 8767)")
    parser.add_argument(
        "--token",
        default=os.environ.get(BROKER_TOKEN_ENV, ""),
        help=f"Optional bearer token; defaults to {BROKER_TOKEN_ENV}",
    )
    parser.add_argument(
        "--backend",
        choices=["auto", "acp", "cli", "command"],
        default="auto",
        help="Broker backend mode.",
    )
    parser.add_argument(
        "--acp-cmd",
        default=os.environ.get(GEMINI_ACP_COMMAND_ENV, ""),
        help=f"ACP backend command; defaults to {GEMINI_ACP_COMMAND_ENV} or 'gemini --acp'",
    )
    parser.add_argument(
        "--backend-cmd",
        default=os.environ.get(BROKER_BACKEND_CMD_ENV, ""),
        help=f"JSON command backend; defaults to {BROKER_BACKEND_CMD_ENV}",
    )
    args = parser.parse_args()

    if args.backend == "command":
        backend_cmd = str(args.backend_cmd or "").strip()
        if not backend_cmd:
            raise SystemExit("--backend command requires --backend-cmd or RESEARCH_GEMINI_BROKER_BACKEND_CMD")
        backend = CommandJSONBackend(backend_cmd)
    elif args.backend == "acp":
        backend = GeminiACPBackend(command=str(args.acp_cmd or "").strip() or None)
    elif args.backend == "cli":
        backend = GeminiCLIBackend()
    else:
        backend = build_default_broker_backend()

    server = create_broker_server(
        host=args.host,
        port=args.port,
        token=str(args.token or "").strip() or None,
        backend=backend,
    )
    address = server.server_address
    print(
        json.dumps(
            {
                "status": "ok",
                "summary": "Gemini session broker started.",
                "data": {
                    "host": address[0],
                    "port": address[1],
                    "backend": backend.name,
                    "token_protected": bool(str(args.token or "").strip()),
                },
            },
            ensure_ascii=False,
        ),
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
