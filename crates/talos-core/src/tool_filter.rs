//! Streaming tool call detector shared by provider and TUI.
//!
//! Processes text chunks and separates normal text from tool call blocks
//! delimited by `<tool_call>...</tool_call>` (or ```` ```json-tool ``` ````).

pub struct ToolSyntaxFilter {
    buffer: String,
    in_tool_block: bool,
    pending: String,
    just_entered_tool_block: bool,
    just_closed_tool_block: bool,
    completed_tool_call: Option<String>,
}

pub struct ToolFilterOutput {
    /// Clean text to display (tool call syntax stripped).
    pub text: String,
    /// `<tool_call>` opening tag detected this chunk.
    pub tool_call_started: bool,
    /// `</tool_call>` closing tag detected, full JSON content available.
    pub tool_call_completed: Option<String>,
}

const START_MARKERS: &[&str] = &["<tool_call>", "<toolcall>", "```json-tool"];
const END_MARKERS: &[&str] = &["</tool_call>", "```"];

fn is_prefix_of_any_marker(text: &str) -> bool {
    START_MARKERS.iter().any(|m| m.starts_with(text)) && !text.is_empty()
}

fn find_start_marker(text: &str) -> Option<usize> {
    let mut earliest = None;
    for marker in START_MARKERS {
        if let Some(pos) = text.find(marker)
            && earliest.is_none_or(|e| pos < e)
        {
            earliest = Some(pos);
        }
    }
    earliest
}

fn strip_markers(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    for m in START_MARKERS {
        if s.starts_with(m) {
            s = s[m.len()..].to_string();
        }
    }
    for m in END_MARKERS {
        if s.ends_with(m) {
            let end = s.len() - m.len();
            s.truncate(end);
        }
    }
    s.trim().to_string()
}

impl ToolSyntaxFilter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_tool_block: false,
            pending: String::new(),
            just_entered_tool_block: false,
            just_closed_tool_block: false,
            completed_tool_call: None,
        }
    }

    pub fn push_chunk(&mut self, chunk: &str) -> ToolFilterOutput {
        self.pending.push_str(chunk);
        let text = self.drain_pending();
        let started = self.just_entered_tool_block;
        let completed = self.completed_tool_call.take();
        self.just_entered_tool_block = false;
        self.just_closed_tool_block = false;
        ToolFilterOutput {
            text,
            tool_call_started: started,
            tool_call_completed: completed,
        }
    }

    fn drain_pending(&mut self) -> String {
        let mut output = String::new();

        while !self.pending.is_empty() {
            if self.in_tool_block {
                let mut found_end: Option<(usize, usize)> = None;
                for marker in END_MARKERS {
                    if let Some(pos) = self.pending.find(marker)
                        && found_end.is_none_or(|(ep, _)| pos < ep)
                    {
                        found_end = Some((pos, marker.len()));
                    }
                }

                if let Some((pos, marker_len)) = found_end {
                    self.buffer.push_str(&self.pending[..pos + marker_len]);
                    let raw = std::mem::take(&mut self.buffer);
                    self.completed_tool_call = Some(strip_markers(&raw));
                    self.pending = self.pending[pos + marker_len..].to_string();
                    self.in_tool_block = false;
                    self.just_closed_tool_block = true;
                } else {
                    self.buffer.push_str(&self.pending);
                    self.pending.clear();
                }
            } else {
                let search_text = &self.pending;

                if let Some(pos) = find_start_marker(search_text) {
                    output.push_str(&search_text[..pos]);

                    let after = &search_text[pos..];
                    let matched_marker = START_MARKERS.iter().find(|m| after.starts_with(*m));

                    if let Some(marker) = matched_marker {
                        self.buffer.push_str(marker);
                        self.pending = search_text[pos + marker.len()..].to_string();
                        self.in_tool_block = true;
                        self.just_entered_tool_block = true;
                    } else {
                        let max_partial = START_MARKERS
                            .iter()
                            .filter_map(|m| {
                                if m.starts_with(after) || after.starts_with(m) {
                                    None
                                } else {
                                    let prefix_len = after.len().min(m.len());
                                    if m[..prefix_len] == after[..prefix_len] {
                                        Some(prefix_len)
                                    } else {
                                        None
                                    }
                                }
                            })
                            .max()
                            .unwrap_or(0);

                        if max_partial > 0 {
                            output.push_str(&search_text[..pos]);
                            self.pending = after.to_string();
                            return output;
                        } else {
                            output.push_str(after);
                            self.pending.clear();
                        }
                    }
                } else {
                    let mut hold_back = 0;
                    let len = self.pending.len();
                    for i in (1..=len.min(20)).rev() {
                        let split = len - i;
                        if !self.pending.is_char_boundary(split) {
                            continue;
                        }
                        let suffix = &self.pending[split..];
                        if is_prefix_of_any_marker(suffix) {
                            hold_back = i;
                            break;
                        }
                    }

                    if hold_back > 0 {
                        let split = len - hold_back;
                        output.push_str(&self.pending[..split]);
                        self.pending = self.pending[split..].to_string();
                        return output;
                    } else {
                        output.push_str(&self.pending);
                        self.pending.clear();
                    }
                }
            }
        }

        output
    }

    pub fn finish(&mut self) -> String {
        if self.in_tool_block {
            String::new()
        } else {
            let drained = self.pending.clone();
            self.pending.clear();
            drained
        }
    }

    pub fn is_in_tool_block(&self) -> bool {
        self.in_tool_block
    }
}

impl Default for ToolSyntaxFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_text_passes_through() {
        let mut f = ToolSyntaxFilter::new();
        let out = f.push_chunk("hello world");
        assert_eq!(out.text, "hello world");
        assert!(!out.tool_call_started);
        assert!(out.tool_call_completed.is_none());
    }

    #[test]
    fn detects_tool_call_start() {
        let mut f = ToolSyntaxFilter::new();
        let out = f.push_chunk("text before <tool_call>");
        assert_eq!(out.text, "text before ");
        assert!(out.tool_call_started);
        assert!(out.tool_call_completed.is_none());
    }

    #[test]
    fn detects_tool_call_complete() {
        let mut f = ToolSyntaxFilter::new();
        let out = f.push_chunk("<tool_call>{\"name\":\"write\"}</tool_call>");
        assert!(out.tool_call_started);
        let completed = out.tool_call_completed.expect("should complete");
        assert!(completed.contains("\"write\""));
    }

    #[test]
    fn streams_tool_call_across_chunks() {
        let mut f = ToolSyntaxFilter::new();

        let out = f.push_chunk("<tool_call>");
        assert!(out.tool_call_started);
        assert!(out.tool_call_completed.is_none());

        let out = f.push_chunk("{\"name\":\"write\",\"args\":");
        assert!(out.text.is_empty());

        let out = f.push_chunk("{\"path\":\"a.txt\"}}</tool_call>");
        let completed = out.tool_call_completed.expect("should complete");
        assert!(completed.contains("write"));
        assert!(completed.contains("a.txt"));
    }

    #[test]
    fn strips_tool_call_from_visible_text() {
        let mut f = ToolSyntaxFilter::new();
        let out = f.push_chunk("hello <tool_call>{}</tool_call> world");
        assert_eq!(out.text, "hello  world");
    }

    #[test]
    fn holds_back_partial_marker() {
        let mut f = ToolSyntaxFilter::new();
        let out = f.push_chunk("hello <tool_");
        assert_eq!(out.text, "hello ");
        assert!(!out.tool_call_started);

        let out = f.push_chunk("call>{}</tool_call>");
        assert!(out.tool_call_started);
        assert!(out.tool_call_completed.is_some());
    }
}
