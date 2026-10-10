#!/usr/bin/env python3
"""Tokenize captured synthetic Auto-review message content with tiktoken 0.12.0.

Measurement-only dependency; never added to Talos or Cargo.lock. Counts message
content, not provider chat framing or billed usage. Encodings are representative,
not a claim about every provider/model. Run accept_auto_locale.py --capture-dir first.
"""
import argparse
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture_dir", type=Path)
    args = parser.parse_args()
    import tiktoken
    assert tiktoken.__version__ == "0.12.0", "use the recorded measurement version"
    guidance = ("The contextual request contains a bounded locale hint. Write effect_summary "
                "and decision_points in that locale when possible; the locale is "
                "presentation-only and never changes the decision.")
    captures = sorted(args.capture_dir.glob("*.json"))
    assert captures, "no captured production requests"
    for path in captures:
        request = json.loads(path.read_text())
        messages = request["messages"]
        assert len(messages) == 2
        system, user = (message["content"] for message in messages)
        assert guidance in system, "locale instruction changed; update measurement boundary"
        prefix, raw = user.split("\n", 1)
        payload = json.loads(raw)
        serialize = lambda value: json.dumps(value, ensure_ascii=False, separators=(",", ":"))
        assert serialize(payload) == raw, "preserve the production serialization exactly"
        locale = payload.pop("locale")
        without_hint = prefix + "\n" + serialize(payload)
        without_guidance = system.replace(guidance, "", 1)
        for name in ["cl100k_base", "o200k_base"]:
            encoding = tiktoken.get_encoding(name)
            count = lambda content: len(encoding.encode(content))
            current = count(system) + count(user)
            field_baseline = count(system) + count(without_hint)
            locale_baseline = count(without_guidance) + count(without_hint)
            print(json.dumps({
                "capture": path.name, "locale": locale, "encoding": name,
                "current_content_tokens": current,
                "hint_token_delta": current - field_baseline,
                "hint_and_guidance_token_delta": current - locale_baseline,
                "hint_byte_delta": len(user.encode()) - len(without_hint.encode()),
            }))


if __name__ == "__main__":
    main()
