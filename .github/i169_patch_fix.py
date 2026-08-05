from pathlib import Path

path = Path(".github/i169_reconciliation_windows_patch.py")
text = path.read_text()
old = '''def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one match in {path}, found {count}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1))
'''
new = '''def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    expected = 2 if "HTTP/1.1 302 Found" in old else 1
    if count != expected:
        raise RuntimeError(
            f"expected {expected} match(es) in {path}, found {count}: {old[:120]!r}"
        )
    path.write_text(text.replace(old, new, expected))
'''
if text.count(old) != 1:
    raise RuntimeError("patch helper boundary changed")
path.write_text(text.replace(old, new, 1))
