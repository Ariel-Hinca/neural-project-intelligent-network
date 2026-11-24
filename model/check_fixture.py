"""Load the example into a temporary database and check it holds together."""

import sqlite3
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def main() -> None:
    db = sqlite3.connect(":memory:")
    db.executescript((ROOT / "schema.sql").read_text(encoding="utf-8"))
    db.executescript((ROOT / "fixture.sql").read_text(encoding="utf-8"))

    nodes = db.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]
    edges = db.execute("SELECT COUNT(*) FROM edges").fetchone()[0]
    if nodes != 7 or edges != 6:
        raise SystemExit(f"expected 7 nodes and 6 links, got {nodes} and {edges}")

    linked = db.execute(
        """
        SELECT from_node.name, edges.relation, to_node.name
        FROM edges
        JOIN nodes AS from_node ON from_node.id = edges.from_id
        JOIN nodes AS to_node ON to_node.id = edges.to_id
        WHERE from_node.id = 'repo:npin' AND to_node.id = 'rule:secrets'
        """
    ).fetchone()
    if linked != ("neural-project-intelligent-network", "contains", "npin-secrets"):
        raise SystemExit(f"example link missing or wrong: {linked}")

    try:
        db.execute(
            "INSERT INTO nodes (id, kind, name) VALUES ('bad', 'NotAKind', 'nope')"
        )
    except sqlite3.IntegrityError:
        pass
    else:
        raise SystemExit("database accepted a kind that is not on the list")

    print("fixture ok: 7 things, 6 links")


if __name__ == "__main__":
    main()
