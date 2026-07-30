from pathlib import Path

paths = [
    Path("docs/backlog/active/ARCH-034-R01-tool-registration-composition.md"),
    Path("docs/iterations/I158-tool-registration-composition.md"),
    Path("docs/iterations/README.md"),
]

for path in paths:
    text = path.read_text()
    count = text.count("43a63e30")
    if count == 0:
        raise SystemExit(f"{path}: expected stale baseline")
    path.write_text(text.replace("43a63e30", "e539537d"))
