"""mdBook 0.5 preprocessor: same rewrite applied to every chapter."""
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from typed import rewrite
if len(sys.argv) > 1 and sys.argv[1] == "supports":
    sys.exit(0)
context, book = json.load(sys.stdin)
def visit(items):
    for item in items:
        ch = item.get("Chapter") if isinstance(item, dict) else None
        if ch:
            ch["content"] = rewrite(ch["content"])
            visit(ch.get("sub_items", []))
visit(book["items"])
json.dump(book, sys.stdout)
