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
updated = text.replace(old, new, 1)
footer_old = 'path.write_text(text)\n'
footer_new = '''old_recurring_event = "                        let _ = fired_tx.send(task_id_for_fire.clone());\\n"
new_recurring_event = (
    "                        let _ = fired_tx.send(TaskFireEvent::Delivered(\\n"
    "                            task_id_for_fire.clone(),\\n"
    "                        ));\\n"
)
if text.count(old_recurring_event) != 1:
    raise SystemExit("expected exactly one recurring fire event send")
text = text.replace(old_recurring_event, new_recurring_event, 1)

path.write_text(text)
'''
if updated.count(footer_old) != 1:
    raise SystemExit('expected one repair script footer')
path.write_text(updated.replace(footer_old, footer_new, 1))
