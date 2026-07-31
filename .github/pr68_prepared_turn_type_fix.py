from pathlib import Path

path = Path('.github/pr68_prepared_turn_repair.py')
text = path.read_text()
old = "    '                    result = agent.preview_request(message, history) => result,\\n',\n    '                    result = async { Ok(agent.preview_prepared_session_turn(&prepared)) } => result,\\n',\n"
new = "    '                    result = agent.preview_request(message, history) => result,\\n',\n    '                    result = async { Ok::<Option<String>, AgentError>(agent.preview_prepared_session_turn(&prepared)) } => result,\\n',\n"
if old in text:
    text = text.replace(old, new, 1)
elif 'Ok::<Option<String>, AgentError>(agent.preview_prepared_session_turn(&prepared))' not in text:
    raise SystemExit('expected prepared preview replacement block')
path.write_text(text)
