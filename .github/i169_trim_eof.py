from pathlib import Path

path = Path("docs/iterations/I169-batched-steering-turn.md")
path.write_text(path.read_text().rstrip() + "\n")
