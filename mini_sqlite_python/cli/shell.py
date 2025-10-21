"""Text-based interactive shell for the mini SQL engine."""

from __future__ import annotations

from core.engine import DatabaseEngine

PROMPT = "db> "
EXIT_COMMANDS = {"quit", "exit", ":q"}


def run_shell() -> None:
    """Launch a blocking REPL until the user types an exit command."""
    engine = DatabaseEngine()
    print("Welcome to the mini SQL shell. Type 'exit' to quit.")
    while True:
        try:
            query = input(PROMPT)
        except EOFError:
            print()
            break
        if not query:
            continue
        if query.strip().lower() in EXIT_COMMANDS:
            break
        for line in engine.execute(query):
            print(line)


if __name__ == "__main__":
    run_shell()
