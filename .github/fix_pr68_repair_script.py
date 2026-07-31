from pathlib import Path

path = Path('.github/pr68_scheduler_repair.py')
text = path.read_text()
old = '''literal = re.compile(r"ScheduledTaskInfo \\{(?P<body>.*?)\\n(?P<indent>\\s*)\\}", re.S)
seen = 0

def add_delivery_error(match):
    global seen
    seen += 1
    body = match.group("body")
    indent = match.group("indent")
    if "delivery_error:" in body:
        return match.group(0)
    lines = body.splitlines()
    insert_at = None
    field_indent = None
    for index, line in enumerate(lines):
        if re.match(r"\\s*fire_at(?::|,)", line):
            insert_at = index + 1
            field_indent = line[: len(line) - len(line.lstrip())]
    if insert_at is None:
        raise SystemExit("ScheduledTaskInfo literal missing fire_at")
    lines.insert(insert_at, f"{field_indent}delivery_error: None,")
    return "ScheduledTaskInfo {" + "\\n".join(lines) + "\\n" + indent + "}"

text = literal.sub(add_delivery_error, text)
if seen < 4:
    raise SystemExit(f"expected at least four ScheduledTaskInfo occurrences, saw {seen}")
'''
new = '''literal = re.compile(r"ScheduledTaskInfo \\{(?P<body>.*?)\\n(?P<indent>\\s*)\\}", re.S)
modified = 0

def add_delivery_error(match):
    global modified
    body = match.group("body")
    indent = match.group("indent")
    if "delivery_error:" in body:
        return match.group(0)
    lines = body.splitlines()
    insert_at = None
    field_indent = None
    for index, line in enumerate(lines):
        if re.match(r"\\s*fire_at(?::|,)", line):
            insert_at = index + 1
            field_indent = line[: len(line) - len(line.lstrip())]
    if insert_at is None:
        return match.group(0)
    modified += 1
    lines.insert(insert_at, f"{field_indent}delivery_error: None,")
    return "ScheduledTaskInfo {" + "\\n".join(lines) + "\\n" + indent + "}"

text = literal.sub(add_delivery_error, text)
if modified < 4:
    raise SystemExit(f"expected at least four ScheduledTaskInfo literals, modified {modified}")
'''
if text.count(old) != 1:
    raise SystemExit('expected one structural matcher block')
path.write_text(text.replace(old, new, 1))
