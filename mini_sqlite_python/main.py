"""Entry point for the mini SQL project."""

from __future__ import annotations

import argparse

from cli.shell import run_shell
from web.server import run_server


def main() -> None:
    """Run the console shell or web UI based on CLI arguments."""
    parser = argparse.ArgumentParser(description="Educational mini SQL engine")
    parser.add_argument(
        "--web",
        action="store_true",
        help="Launch the web UI instead of the interactive shell.",
    )
    parser.add_argument("--host", default="127.0.0.1", help="Host for the web UI")
    parser.add_argument("--port", type=int, default=8000, help="Port for the web UI")
    args = parser.parse_args()

    if args.web:
        run_server(host=args.host, port=args.port)
    else:
        run_shell()


if __name__ == "__main__":
    main()
