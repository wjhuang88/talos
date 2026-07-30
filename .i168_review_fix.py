from __future__ import annotations

import sys
from pathlib import Path


def replace_once(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match in {path}, found {count}")
    file.write_text(text.replace(old, new, 1))


def append_before(path: str, marker: str, content: str, label: str) -> None:
    replace_once(path, marker, content + marker, label)


def apply_tests() -> None:
    openai = "crates/talos-provider/src/openai_sse.rs"
    replace_once(
        openai,
        '''        events
    }

    #[tokio::test]
    async fn openai_eof_after_partial_text_is_terminal_error() {
''',
        '''        events
    }

    fn terminal_error(events: &[AgentEvent]) -> &str {
        events
            .iter()
            .find_map(|event| match event {
                AgentEvent::Error { message } => Some(message.as_str()),
                _ => None,
            })
            .expect("terminal error")
    }

    #[tokio::test]
    async fn openai_eof_after_partial_text_is_terminal_error() {
''',
        "OpenAI terminal error helper",
    )
    replace_once(
        openai,
        '''    #[tokio::test]
    async fn openai_unknown_finish_reason_is_not_end_turn() {
        let events = parse_raw_body(
            b"data: {\\"choices\\":[{\\"index\\":0,\\"delta\\":{\\"content\\":\\"partial\\"},\\"finish_reason\\":\\"content_filter\\"}]}\\n\\n".to_vec(),
        )
        .await;
        assert!(events.iter().any(|event| matches!(event, AgentEvent::Error { message } if message.contains("content_filter"))));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }
''',
        '''    #[tokio::test]
    async fn openai_content_filter_uses_known_filtered_policy() {
        let events = parse_raw_body(
            b"data: {\\"choices\\":[{\\"index\\":0,\\"delta\\":{\\"content\\":\\"partial\\"},\\"finish_reason\\":\\"content_filter\\"}]}\\n\\n".to_vec(),
        )
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("filtered"));
        assert!(message.contains("content_filter"));
        assert!(!message.contains("unsupported provider finish_reason"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn openai_legacy_function_call_uses_known_policy() {
        let events = parse_raw_body(
            b"data: {\\"choices\\":[{\\"index\\":0,\\"delta\\":{\\"content\\":\\"partial\\"},\\"finish_reason\\":\\"function_call\\"}]}\\n\\n".to_vec(),
        )
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("legacy function_call"));
        assert!(message.contains("tool_calls"));
        assert!(!message.contains("unsupported provider finish_reason"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn openai_unknown_finish_reason_is_not_end_turn() {
        let events = parse_raw_body(
            b"data: {\\"choices\\":[{\\"index\\":0,\\"delta\\":{\\"content\\":\\"partial\\"},\\"finish_reason\\":\\"fixture_unknown_reason\\"}]}\\n\\n".to_vec(),
        )
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("unsupported provider finish_reason"));
        assert!(message.contains("fixture_unknown_reason"));
        assert!(!message.contains("content_filter"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }
''',
        "OpenAI known and unknown finish policy tests",
    )

    anthropic = "crates/talos-provider/src/anthropic_stream.rs"
    replace_once(
        anthropic,
        '''        events
    }

    fn text_event(text: &str) -> String {
''',
        '''        events
    }

    fn terminal_error(events: &[AgentEvent]) -> &str {
        events
            .iter()
            .find_map(|event| match event {
                AgentEvent::Error { message } => Some(message.as_str()),
                _ => None,
            })
            .expect("terminal error")
    }

    fn text_event(text: &str) -> String {
''',
        "Anthropic terminal error helper",
    )
    replace_once(
        anthropic,
        '''    #[tokio::test]
    async fn anthropic_unknown_stop_reason_is_not_end_turn() {
        let events = parse_body(format!(
            "{}{}",
            text_event("partial"),
            terminal_event("pause_turn")
        ))
        .await;
        assert!(events.iter().any(
            |event| matches!(event, AgentEvent::Error { message } if message.contains("pause_turn"))
        ));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }
''',
        '''    #[tokio::test]
    async fn anthropic_stop_sequence_is_explicit_completion() {
        let events = parse_body(format!(
            "{}{}",
            text_event("complete"),
            terminal_event("stop_sequence")
        ))
        .await;
        assert!(events.iter().any(|event| matches!(
            event,
            AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                ..
            }
        )));
        assert!(!events.iter().any(|event| matches!(event, AgentEvent::Error { .. })));
    }

    #[tokio::test]
    async fn anthropic_pause_turn_uses_known_paused_policy() {
        let events = parse_body(format!(
            "{}{}",
            text_event("partial"),
            terminal_event("pause_turn")
        ))
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("paused"));
        assert!(message.contains("pause_turn"));
        assert!(!message.contains("unsupported provider stop_reason"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn anthropic_refusal_uses_known_refusal_policy() {
        let events = parse_body(format!(
            "{}{}",
            text_event("partial"),
            terminal_event("refusal")
        ))
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("refused"));
        assert!(message.contains("refusal"));
        assert!(!message.contains("unsupported provider stop_reason"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }

    #[tokio::test]
    async fn anthropic_unknown_stop_reason_is_not_end_turn() {
        let events = parse_body(format!(
            "{}{}",
            text_event("partial"),
            terminal_event("fixture_unknown_reason")
        ))
        .await;
        let message = terminal_error(&events);
        assert!(message.contains("unsupported provider stop_reason"));
        assert!(message.contains("fixture_unknown_reason"));
        assert!(!message.contains("pause_turn"));
        assert!(!message.contains("refusal"));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnd { .. }))
        );
    }
''',
        "Anthropic known and unknown stop policy tests",
    )

    session = "crates/talos-session/tests/i168_terminal_diagnostic.rs"
    replace_once(
        session,
        'message: "unsupported provider finish_reason: content_filter".into(),',
        'message: "unsupported provider finish_reason: fixture_unknown_reason".into(),',
        "session unknown event uses synthetic reason",
    )
    replace_once(
        session,
        'assert_eq!(diagnostics[0].reason.as_deref(), Some("content_filter"));',
        'assert_eq!(diagnostics[0].reason.as_deref(), Some("fixture_unknown_reason"));',
        "session unknown reason assertion",
    )
    append_before(
        session,
        '''#[test]
fn terminal_diagnostic_is_excluded_from_messages_copy_export_and_provider_history() {
''',
        '''#[test]
fn known_provider_policies_remain_distinct_from_truly_unknown_reasons() {
    let known_cases = [
        (
            "provider response filtered by content policy (finish_reason=content_filter)",
            "content_filter",
        ),
        (
            "provider requested deprecated legacy function_call (finish_reason=function_call); use tool_calls",
            "legacy_function_call",
        ),
        (
            "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported",
            "pause_turn",
        ),
        (
            "provider refused request (stop_reason=refusal)",
            "refusal",
        ),
    ];

    for (index, (message, expected_reason)) in known_cases.into_iter().enumerate() {
        let terminal = AgentEvent::Error {
            message: message.into(),
        };
        let diagnostic = diagnostic("turn-known-policy", index as u32 + 1, &terminal);
        assert_eq!(diagnostic.outcome, ProviderTerminalOutcome::Error);
        assert_eq!(diagnostic.source, ProviderTerminalSource::ProviderError);
        assert_eq!(diagnostic.reason.as_deref(), Some(expected_reason));
    }

    let unknown = diagnostic(
        "turn-unknown-policy",
        1,
        &AgentEvent::Error {
            message: "unsupported provider stop_reason: fixture_unknown_reason".into(),
        },
    );
    assert_eq!(unknown.source, ProviderTerminalSource::UnsupportedReason);
    assert_eq!(unknown.reason.as_deref(), Some("fixture_unknown_reason"));

    let stop_sequence = diagnostic(
        "turn-stop-sequence",
        1,
        &AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        },
    );
    assert_eq!(stop_sequence.outcome, ProviderTerminalOutcome::Completed);
    assert_eq!(stop_sequence.source, ProviderTerminalSource::Explicit);
    assert_eq!(stop_sequence.reason, None);
}

''',
        "session known policy diagnostic tests",
    )

    cli_test = "crates/talos-cli/tests/i168_terminal.rs"
    Path(cli_test).write_text(
        Path(cli_test).read_text().rstrip()
        + '''

#[test]
fn terminal_cli_projection_preserves_known_provider_policy_causes() {
    for message in [
        "provider response filtered by content policy (finish_reason=content_filter)",
        "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported",
        "provider refused request (stop_reason=refusal)",
        "unsupported provider stop_reason: fixture_unknown_reason",
    ] {
        let mut engine = ConversationEngine::new("fixture-model".into(), "fixture-provider".into());
        engine.handle_turn_started();
        engine.handle_agent_event(&AgentEvent::TurnStart);
        engine.handle_agent_event(&AgentEvent::TextDelta {
            delta: "partial response".into(),
        });
        let outputs = engine.handle_agent_event(&AgentEvent::Error {
            message: message.into(),
        });

        assert!(outputs.iter().any(|output| matches!(
            output,
            UiOutput::Tip {
                text,
                kind: TipKind::Error,
            } if text == message
        )));
        assert!(outputs.iter().any(|output| matches!(
            output,
            UiOutput::Content(talos_conversation::ContentOutput::Block { text, .. })
                if text.contains(message)
        )));
        assert!(!engine.is_processing());
    }
}
'''
    )

    tui = "crates/talos-tui/src/app.rs"
    replace_once(
        tui,
        '''    #[test]
    fn terminal_processing_clear_status_reaches_tui_state() {
''',
        '''    #[test]
    fn terminal_known_provider_policy_tip_is_retained_by_tui() {
        for text in [
            "provider response filtered by content policy (finish_reason=content_filter)",
            "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported",
            "provider refused request (stop_reason=refusal)",
            "unsupported provider stop_reason: fixture_unknown_reason",
        ] {
            let mut tui = Tui::for_test(TuiState::new(), None);
            let should_exit = tui.handle_ui_output(UiOutput::Tip {
                text: text.into(),
                kind: TipKind::Error,
            });

            assert!(!should_exit);
            let tip = tui.state.tip.as_ref().expect("provider policy tip");
            assert_eq!(tip.kind, TipKind::Error);
            assert_eq!(tip.text, text);
        }
    }

    #[test]
    fn terminal_processing_clear_status_reaches_tui_state() {
''',
        "TUI known policy projection test",
    )

    Path("scripts/fixtures/i168_provider_terminal_fixture.py").write_text(FIXTURE_SERVER)
    Path("scripts/verify_i168_provider_terminal.sh").write_text(FIXTURE_RUNNER)


def apply_production() -> None:
    openai = "crates/talos-provider/src/openai_sse.rs"
    replace_once(
        openai,
        '''                let requested_stop_reason = match finish_reason.as_str() {
                    "stop" => StopReason::EndTurn,
                    "tool_calls" => StopReason::ToolUse,
                    "length" => StopReason::MaxTokens,
                    unknown => {
                        let _ = tx
                            .send(AgentEvent::Error {
                                message: format!(
                                    "unsupported provider finish_reason: {}",
                                    bounded_terminal_reason(unknown)
                                ),
                            })
                            .await;
                        return;
                    }
                };
''',
        '''                let requested_stop_reason = match finish_reason.as_str() {
                    "stop" => StopReason::EndTurn,
                    "tool_calls" => StopReason::ToolUse,
                    "length" => StopReason::MaxTokens,
                    "content_filter" => {
                        let _ = tx
                            .send(AgentEvent::Error {
                                message: "provider response filtered by content policy (finish_reason=content_filter)".into(),
                            })
                            .await;
                        return;
                    }
                    "function_call" => {
                        let _ = tx
                            .send(AgentEvent::Error {
                                message: "provider requested deprecated legacy function_call (finish_reason=function_call); use tool_calls".into(),
                            })
                            .await;
                        return;
                    }
                    unknown => {
                        let _ = tx
                            .send(AgentEvent::Error {
                                message: format!(
                                    "unsupported provider finish_reason: {}",
                                    bounded_terminal_reason(unknown)
                                ),
                            })
                            .await;
                        return;
                    }
                };
''',
        "OpenAI known finish reason policy",
    )

    anthropic = "crates/talos-provider/src/anthropic_stream.rs"
    replace_once(
        anthropic,
        '''    let stop_reason = match reason {
        "end_turn" => StopReason::EndTurn,
        "tool_use" => StopReason::ToolUse,
        "max_tokens" => StopReason::MaxTokens,
        unknown => {
            return Err(format!(
                "unsupported provider stop_reason: {}",
                bounded_terminal_reason(unknown)
            ));
        }
    };
''',
        '''    let stop_reason = match reason {
        "end_turn" | "stop_sequence" => StopReason::EndTurn,
        "tool_use" => StopReason::ToolUse,
        "max_tokens" => StopReason::MaxTokens,
        "pause_turn" => {
            return Err(
                "provider paused turn (stop_reason=pause_turn); automatic continuation is not supported"
                    .into(),
            );
        }
        "refusal" => {
            return Err("provider refused request (stop_reason=refusal)".into());
        }
        unknown => {
            return Err(format!(
                "unsupported provider stop_reason: {}",
                bounded_terminal_reason(unknown)
            ));
        }
    };
''',
        "Anthropic known stop reason policy",
    )

    diagnostic = "crates/talos-session/src/diagnostic.rs"
    replace_once(
        diagnostic,
        '''    for prefix in [
        "unsupported provider finish_reason:",
        "unsupported provider stop_reason:",
    ] {
''',
        '''    for (prefix, reason) in [
        (
            "provider response filtered by content policy",
            "content_filter",
        ),
        (
            "provider requested deprecated legacy function_call",
            "legacy_function_call",
        ),
        ("provider paused turn", "pause_turn"),
        ("provider refused request", "refusal"),
    ] {
        if message.starts_with(prefix) {
            return (ProviderTerminalSource::ProviderError, reason.into());
        }
    }
    for prefix in [
        "unsupported provider finish_reason:",
        "unsupported provider stop_reason:",
    ] {
''',
        "known provider policy diagnostic classification",
    )

    mode_print = "crates/talos-cli/src/mode_print.rs"
    replace_once(
        mode_print,
        '''    let mut stdout = io::stdout().lock();
    let mut terminal_notice = None;
''',
        '''    let mut stdout = io::stdout().lock();
    let mut terminal_notice = None;
    let mut saw_stdout = false;
    let mut stdout_line_open = false;
''',
        "print stdout state",
    )
    replace_once(
        mode_print,
        '''            } => {
                print!("{delta}");
                stdout.flush().context("failed to flush stdout")?;
            }
''',
        '''            } => {
                write!(stdout, "{delta}").context("failed to write stdout")?;
                stdout.flush().context("failed to flush stdout")?;
                if !delta.is_empty() {
                    saw_stdout = true;
                    stdout_line_open = !delta.ends_with('\n');
                }
            }
''',
        "print text delta writes",
    )
    replace_once(
        mode_print,
        '''                talos_core::session::TurnCompletionStatus::Success { .. } => {
                    if let Some(notice) = terminal_notice.take() {
                        eprintln!("{notice}");
                    }
                    println!();
                    return Ok(());
                }
''',
        '''                talos_core::session::TurnCompletionStatus::Success { .. } => {
                    if stdout_line_open || !saw_stdout {
                        writeln!(stdout).context("failed to finish stdout line")?;
                    }
                    stdout.flush().context("failed to flush stdout")?;
                    if let Some(notice) = terminal_notice.take() {
                        eprintln!("{notice}");
                    }
                    return Ok(());
                }
''',
        "print success terminal ordering",
    )
    replace_once(
        mode_print,
        '''                talos_core::session::TurnCompletionStatus::Error { message } => {
                    eprintln!("Error: {message}");
                    std::process::exit(1);
                }
''',
        '''                talos_core::session::TurnCompletionStatus::Error { message } => {
                    if stdout_line_open {
                        writeln!(stdout).context("failed to finish stdout line")?;
                        stdout.flush().context("failed to flush stdout")?;
                    }
                    eprintln!("Error: {message}");
                    std::process::exit(1);
                }
''',
        "print completed error ordering",
    )
    replace_once(
        mode_print,
        '''            SessionEvent::Error { message } => {
                eprintln!("Error: {message}");
                std::process::exit(1);
            }
''',
        '''            SessionEvent::Error { message } => {
                if stdout_line_open {
                    writeln!(stdout).context("failed to finish stdout line")?;
                    stdout.flush().context("failed to flush stdout")?;
                }
                eprintln!("Error: {message}");
                std::process::exit(1);
            }
''',
        "print session error ordering",
    )


FIXTURE_SERVER = r'''#!/usr/bin/env python3
"""Deterministic local SSE provider for I168 terminal-outcome verification."""

from __future__ import annotations

import argparse
import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

_COUNTS: dict[tuple[str, str], int] = {}
_LOCK = threading.Lock()
_LOG_FILE: Path


def _sse(event: str | None, payload: dict[str, Any]) -> bytes:
    prefix = f"event: {event}\n" if event else ""
    return f"{prefix}data: {json.dumps(payload, separators=(',', ':'))}\n\n".encode()


def _openai_text(text: str, finish_reason: str | None) -> bytes:
    return _sse(
        None,
        {
            "choices": [
                {
                    "index": 0,
                    "delta": {"content": text},
                    "finish_reason": finish_reason,
                }
            ]
        },
    )


def _openai_tool() -> bytes:
    return _sse(
        None,
        {
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "tool_calls": [
                            {
                                "index": 0,
                                "id": "call_fixture",
                                "type": "function",
                                "function": {
                                    "name": "git_status",
                                    "arguments": "{}",
                                },
                            }
                        ]
                    },
                    "finish_reason": "tool_calls",
                }
            ]
        },
    )


def _anthropic_text(text: str) -> bytes:
    return _sse(
        "content_block_delta",
        {"index": 0, "delta": {"type": "text_delta", "text": text}},
    )


def _anthropic_terminal(reason: str, stop_sequence: str | None = None) -> bytes:
    return _sse(
        "message_delta",
        {
            "delta": {"stop_reason": reason, "stop_sequence": stop_sequence},
            "usage": {"output_tokens": 1},
        },
    )


def _anthropic_tool() -> bytes:
    return b"".join(
        [
            _sse(
                "content_block_start",
                {
                    "index": 0,
                    "content_block": {
                        "type": "tool_use",
                        "id": "tool_fixture",
                        "name": "git_status",
                        "input": {},
                    },
                },
            ),
            _sse("content_block_stop", {"index": 0}),
            _anthropic_terminal("tool_use"),
        ]
    )


def _has_tool_result(protocol: str, messages: Any) -> bool:
    for message in messages if isinstance(messages, list) else []:
        if not isinstance(message, dict):
            continue
        if protocol == "openai" and message.get("role") == "tool":
            return True
        content = message.get("content")
        if isinstance(content, list) and any(
            isinstance(block, dict) and block.get("type") == "tool_result"
            for block in content
        ):
            return True
    return False


def _response(protocol: str, model: str, ordinal: int) -> tuple[list[bytes], int]:
    mode = model.split("-", 1)[1]
    if protocol == "openai":
        partial = _openai_text("fixture partial", None)
        table = {
            "normal": ([_openai_text("fixture normal", "stop")], 0),
            "max-tokens": ([_openai_text("fixture partial", "length")], 0),
            "content-filter": ([_openai_text("fixture partial", "content_filter")], 0),
            "legacy-function-call": ([_openai_text("fixture partial", "function_call")], 0),
            "unknown": ([_openai_text("fixture partial", "fixture_unknown_reason")], 0),
            "eof": ([partial], 0),
            "decode-error": ([partial, b"\xff"], 0),
            "transport-error": ([partial], 64),
        }
        if mode == "tool-continuation":
            return ([_openai_tool()], 0) if ordinal == 1 else ([partial], 0)
        return table[mode]

    partial = _anthropic_text("fixture partial")
    table = {
        "normal": ([_anthropic_text("fixture normal"), _anthropic_terminal("end_turn")], 0),
        "stop-sequence": (
            [_anthropic_text("fixture stop sequence"), _anthropic_terminal("stop_sequence", "END")],
            0,
        ),
        "max-tokens": ([partial, _anthropic_terminal("max_tokens")], 0),
        "pause-turn": ([partial, _anthropic_terminal("pause_turn")], 0),
        "refusal": ([partial, _anthropic_terminal("refusal")], 0),
        "unknown": ([partial, _anthropic_terminal("fixture_unknown_reason")], 0),
        "eof": ([partial], 0),
        "decode-error": ([partial, b"\xff"], 0),
        "transport-error": ([partial], 64),
    }
    if mode == "tool-continuation":
        return ([_anthropic_tool()], 0) if ordinal == 1 else ([partial], 0)
    return table[mode]


class FixtureHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, _format: str, *_args: Any) -> None:
        return

    def do_POST(self) -> None:  # noqa: N802
        length = int(self.headers.get("content-length", "0"))
        payload = json.loads(self.rfile.read(length) or b"{}")
        protocol = "openai" if self.path.endswith("/chat/completions") else "anthropic"
        model = str(payload.get("model", ""))
        key = (protocol, model)
        with _LOCK:
            ordinal = _COUNTS.get(key, 0) + 1
            _COUNTS[key] = ordinal
            record = {
                "protocol": protocol,
                "model": model,
                "response_ordinal": ordinal,
                "has_tool_result": _has_tool_result(protocol, payload.get("messages")),
            }
            with _LOG_FILE.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps(record, separators=(",", ":")) + "\n")

        segments, declared_extra = _response(protocol, model, ordinal)
        actual_length = sum(len(segment) for segment in segments)
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.send_header("Content-Length", str(actual_length + declared_extra))
        self.end_headers()
        for segment in segments:
            self.wfile.write(segment)
            self.wfile.flush()
            time.sleep(0.02)
        self.close_connection = True


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port-file", required=True, type=Path)
    parser.add_argument("--log-file", required=True, type=Path)
    args = parser.parse_args()

    global _LOG_FILE
    _LOG_FILE = args.log_file
    _LOG_FILE.write_text("", encoding="utf-8")
    server = ThreadingHTTPServer(("127.0.0.1", 0), FixtureHandler)
    args.port_file.write_text(str(server.server_address[1]), encoding="utf-8")
    server.serve_forever()


if __name__ == "__main__":
    main()
'''


FIXTURE_RUNNER = r'''#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="${1:-$repo_root/target/debug/talos}"
server="$repo_root/scripts/fixtures/i168_provider_terminal_fixture.py"
tmp="$(mktemp -d)"
port_file="$tmp/port"
request_log="$tmp/requests.jsonl"
server_pid=""
warning='Warning: response truncated because the provider reached the output token limit; partial response preserved.'

cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf "$tmp"
}
trap cleanup EXIT

python3 "$server" --port-file "$port_file" --log-file "$request_log" &
server_pid=$!
for _ in $(seq 1 100); do
  [[ -s "$port_file" ]] && break
  sleep 0.05
done
[[ -s "$port_file" ]] || { echo "fixture server failed to publish port" >&2; exit 1; }
port="$(cat "$port_file")"

assert_exact() {
  local path="$1"
  local expected="$2"
  python3 - "$path" "$expected" <<'PY'
from pathlib import Path
import sys
actual = Path(sys.argv[1]).read_text(encoding="utf-8")
expected = sys.argv[2]
if actual != expected:
    raise SystemExit(f"{sys.argv[1]} mismatch: expected {expected!r}, got {actual!r}")
PY
}

assert_empty() {
  [[ ! -s "$1" ]] || { echo "$1 expected empty" >&2; cat "$1" >&2; exit 1; }
}

assert_contains() {
  grep -Fqi "$2" "$1" || { echo "$1 missing: $2" >&2; cat "$1" >&2; exit 1; }
}

assert_not_contains() {
  if grep -Fqi "$2" "$1"; then
    echo "$1 unexpectedly contains: $2" >&2
    cat "$1" >&2
    exit 1
  fi
}

write_config() {
  local protocol="$1"
  local model="$2"
  local home="$3"
  local endpoint provider_protocol
  mkdir -p "$home/.talos"
  if [[ "$protocol" == "openai" ]]; then
    endpoint="http://127.0.0.1:$port/v1"
    provider_protocol="openai-chat"
  else
    endpoint="http://127.0.0.1:$port/v1/messages"
    provider_protocol="anthropic-messages"
  fi
  cat > "$home/.talos/config.toml" <<EOF
provider = "fixture"
model = "$model"

[providers.fixture]
protocol = "$provider_protocol"
base_url = "$endpoint"
api_key = "fixture-only"

[providers.fixture.timeout]
dispatch_timeout_secs = 5
first_packet_timeout_secs = 5
stream_idle_timeout_secs = 5
max_attempts = 1
backoff_base_ms = 1
backoff_max_ms = 1
EOF
}

printf 'protocol\tmode\texit\n'
case_index=0
run_case() {
  local protocol="$1"
  local mode="$2"
  local expected_exit="$3"
  local model="${protocol}-${mode}"
  local case_dir="$tmp/case-$case_index-$model"
  local home="$case_dir/home"
  LAST_OUT="$case_dir/stdout"
  LAST_ERR="$case_dir/stderr"
  case_index=$((case_index + 1))
  mkdir -p "$case_dir"
  write_config "$protocol" "$model" "$home"

  set +e
  HOME="$home" "$binary" --print --provider fixture --model "$model" --no-context \
    "I168 deterministic terminal fixture" >"$LAST_OUT" 2>"$LAST_ERR"
  local status=$?
  set -e
  printf '%s\t%s\t%s\n' "$protocol" "$mode" "$status"
  if [[ "$status" -ne "$expected_exit" ]]; then
    echo "fixture $model exit mismatch: expected $expected_exit, got $status" >&2
    cat "$LAST_OUT" >&2 || true
    cat "$LAST_ERR" >&2 || true
    exit 1
  fi
}

run_combined_max_tokens() {
  local protocol="$1"
  local model="${protocol}-max-tokens"
  local case_dir="$tmp/combined-$protocol"
  local home="$case_dir/home"
  local combined="$case_dir/combined"
  mkdir -p "$case_dir"
  write_config "$protocol" "$model" "$home"
  HOME="$home" "$binary" --print --provider fixture --model "$model" --no-context \
    "I168 combined stream fixture" >"$combined" 2>&1
  assert_exact "$combined" $'fixture partial\n'"$warning"$'\n'
}

run_case openai normal 0
assert_exact "$LAST_OUT" $'fixture normal\n'
assert_empty "$LAST_ERR"

run_case openai max-tokens 0
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_exact "$LAST_ERR" "$warning"$'\n'
[[ "$(grep -Foc "$warning" "$LAST_ERR")" -eq 1 ]]
assert_not_contains "$LAST_ERR" "Error:"
run_combined_max_tokens openai

run_case openai content-filter 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "filtered"
assert_contains "$LAST_ERR" "content_filter"
assert_not_contains "$LAST_ERR" "unsupported provider finish_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case openai legacy-function-call 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "legacy function_call"
assert_contains "$LAST_ERR" "tool_calls"
assert_not_contains "$LAST_ERR" "unsupported provider finish_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case openai unknown 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "unsupported provider finish_reason"
assert_contains "$LAST_ERR" "fixture_unknown_reason"
assert_not_contains "$LAST_ERR" "content_filter"
assert_not_contains "$LAST_ERR" "response truncated"

for mode in eof decode-error transport-error tool-continuation; do
  run_case openai "$mode" 1
  assert_exact "$LAST_OUT" $'fixture partial\n'
  assert_not_contains "$LAST_ERR" "response truncated"
done
assert_contains "$tmp/case-$((case_index-4))-openai-eof/stderr" "explicit terminal signal"
assert_contains "$tmp/case-$((case_index-3))-openai-decode-error/stderr" "decode error"
assert_contains "$tmp/case-$((case_index-2))-openai-transport-error/stderr" "transport read error"
assert_contains "$tmp/case-$((case_index-1))-openai-tool-continuation/stderr" "explicit terminal signal"

run_case anthropic normal 0
assert_exact "$LAST_OUT" $'fixture normal\n'
assert_empty "$LAST_ERR"

run_case anthropic stop-sequence 0
assert_exact "$LAST_OUT" $'fixture stop sequence\n'
assert_empty "$LAST_ERR"

run_case anthropic max-tokens 0
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_exact "$LAST_ERR" "$warning"$'\n'
[[ "$(grep -Foc "$warning" "$LAST_ERR")" -eq 1 ]]
assert_not_contains "$LAST_ERR" "Error:"
run_combined_max_tokens anthropic

run_case anthropic pause-turn 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "paused"
assert_contains "$LAST_ERR" "pause_turn"
assert_not_contains "$LAST_ERR" "unsupported provider stop_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case anthropic refusal 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "refused"
assert_contains "$LAST_ERR" "refusal"
assert_not_contains "$LAST_ERR" "unsupported provider stop_reason"
assert_not_contains "$LAST_ERR" "response truncated"

run_case anthropic unknown 1
assert_exact "$LAST_OUT" $'fixture partial\n'
assert_contains "$LAST_ERR" "unsupported provider stop_reason"
assert_contains "$LAST_ERR" "fixture_unknown_reason"
assert_not_contains "$LAST_ERR" "pause_turn"
assert_not_contains "$LAST_ERR" "refusal"
assert_not_contains "$LAST_ERR" "response truncated"

for mode in eof decode-error transport-error tool-continuation; do
  run_case anthropic "$mode" 1
  assert_exact "$LAST_OUT" $'fixture partial\n'
  assert_not_contains "$LAST_ERR" "response truncated"
done
assert_contains "$tmp/case-$((case_index-4))-anthropic-eof/stderr" "explicit terminal signal"
assert_contains "$tmp/case-$((case_index-3))-anthropic-decode-error/stderr" "decode error"
assert_contains "$tmp/case-$((case_index-2))-anthropic-transport-error/stderr" "transport read error"
assert_contains "$tmp/case-$((case_index-1))-anthropic-tool-continuation/stderr" "explicit terminal signal"

python3 - "$request_log" <<'PY'
import json
import sys

records = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
for protocol in ("openai", "anthropic"):
    model = f"{protocol}-tool-continuation"
    continuation = [
        record
        for record in records
        if record["model"] == model and record["response_ordinal"] >= 2
    ]
    if not continuation or not any(record["has_tool_result"] for record in continuation):
        raise SystemExit(f"{model}: continuation request did not preserve completed tool result")
    print(f"{protocol}\ttool-continuation-prefix\tcompleted_tool_result_seen=true")
PY

printf 'binary\t%s\n' "$binary"
printf 'commit\t%s\n' "$(git -C "$repo_root" rev-parse HEAD)"
'''


def main() -> None:
    if len(sys.argv) != 2 or sys.argv[1] not in {"tests", "production"}:
        raise SystemExit("usage: .i168_review_fix.py tests|production")
    if sys.argv[1] == "tests":
        apply_tests()
    else:
        apply_production()


if __name__ == "__main__":
    main()
