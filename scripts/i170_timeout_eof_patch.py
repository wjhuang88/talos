from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        count = text.count(old)
        if count != 1:
            raise SystemExit(f"{label}: expected one old form, found {count}")
        return text.replace(old, new, 1)
    if new not in text:
        raise SystemExit(f"{label}: neither old nor new form found")
    return text


bash_tool = Path("crates/talos-tools/src/bash_tool.rs")
text = bash_tool.read_text(encoding="utf-8")

if "let mut completed_status = None;" not in text:
    start_marker = "        let exit_status = loop {\n"
    end_marker = "        let exit_status = match exit_status {"
    if text.count(start_marker) != 1 or text.count(end_marker) != 1:
        raise SystemExit("unexpected shell loop markers")
    start = text.index(start_marker)
    end = text.index(end_marker, start)
    replacement = '''        let mut completed_status = None;
        let exit_status = loop {
            if completed_status.as_ref().is_some_and(Result::is_err)
                || (completed_status.is_some() && !stdout_open && !stderr_open)
            {
                break completed_status
                    .take()
                    .expect("completed status checked above");
            }

            tokio::select! {
                line_result = stdout_reader.next_line(), if stdout_open => {
                    match line_result {
                        Ok(Some(line)) => {
                            output.push_str(&line);
                            output.push('\\n');
                        }
                        Ok(None) => stdout_open = false,
                        Err(e) => {
                            output.push_str(&format!("[stdout error: {e}]\\n"));
                            stdout_open = false;
                        }
                    }
                }
                line_result = stderr_reader.next_line(), if stderr_open => {
                    match line_result {
                        Ok(Some(line)) => {
                            output.push_str(&line);
                            output.push('\\n');
                        }
                        Ok(None) => stderr_open = false,
                        Err(e) => {
                            output.push_str(&format!("[stderr error: {e}]\\n"));
                            stderr_open = false;
                        }
                    }
                }
                status = child.wait(), if completed_status.is_none() => {
                    completed_status = Some(status);
                }
                _ = &mut deadline => {
                    if completed_status.is_none() {
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                    }
                    // Preserve output already received before the absolute deadline. Descendants
                    // can inherit stdout/stderr handles and outlive the direct shell child, so
                    // waiting for EOF here would defeat the operation timeout.
                    output.push_str("[timeout]");
                    return ToolResult::error(output);
                }
            }
        };

'''
    text = text[:start] + replacement + text[end:]

marker = '''    #[tokio::test]
    async fn test_streaming_empty_output() {
'''
test = '''    #[tokio::test]
    async fn timeout_does_not_wait_for_descendant_pipe_eof() {
        let tool = BashTool::new(test_dir()).with_timeout(Duration::from_millis(300));
        #[cfg(windows)]
        let command = "$child = Start-Process powershell.exe -ArgumentList '-NoLogo','-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 10' -NoNewWindow -PassThru; Start-Sleep -Seconds 10";
        #[cfg(not(windows))]
        let command = "sleep 10 &";
        let started = std::time::Instant::now();

        let result = tool
            .execute(serde_json::json!({ "command": command }))
            .await;

        assert!(result.is_error);
        assert!(result.content.contains("[timeout]"));
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "absolute deadline waited for descendant-held pipe EOF"
        );
    }

'''
if "async fn timeout_does_not_wait_for_descendant_pipe_eof()" not in text:
    if text.count(marker) != 1:
        raise SystemExit("unexpected test insertion marker")
    text = text.replace(marker, test + marker, 1)

bash_tool.write_text(text, encoding="utf-8")

adr = Path("docs/decisions/057-windows-powershell-process-boundary.md")
text = adr.read_text(encoding="utf-8")
text = replace_once(
    text,
    "- At the deadline Talos kills and waits for the direct child, drains bounded remaining pipe output, appends `[timeout]`, and returns a tool error.",
    "- The same deadline covers direct-child completion and stdout/stderr closure. Talos completes normally only after the direct child and both pipes finish; otherwise it kills/waits the direct child when still running, preserves output already received, appends `[timeout]`, and returns without waiting for descendant-held pipe EOF.",
    "ADR timeout statement",
)
adr.write_text(text, encoding="utf-8")

review = Path("docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md")
text = review.read_text(encoding="utf-8")
text = replace_once(
    text,
    "At expiry, Talos kills and waits for the direct child, drains remaining pipes, emits `[timeout]`, and returns an error. The design does not guarantee descendant termination on Windows or Unix when a shell launches detached/grandchild work.",
    "The one deadline remains active until both the direct child and its stdout/stderr pipes finish. At expiry, Talos kills and waits for the direct child when still running, preserves output already received, emits `[timeout]`, and returns without waiting for pipe EOF. This prevents descendants that inherited the handles from extending the operation timeout while making no descendant-termination claim.",
    "security review timeout statement",
)
review.write_text(text, encoding="utf-8")
