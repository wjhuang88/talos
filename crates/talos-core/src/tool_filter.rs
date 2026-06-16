//! Streaming filter that suppresses tool call syntax from visible text output.

pub struct ToolSyntaxFilter {
    buffer: String,
    in_tool_block: bool,
    pending: String,
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

impl ToolSyntaxFilter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            in_tool_block: false,
            pending: String::new(),
        }
    }

    pub fn push_chunk(&mut self, chunk: &str) -> String {
        self.pending.push_str(chunk);
        self.drain_pending()
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
                    self.pending = self.pending[pos + marker_len..].to_string();
                    self.in_tool_block = false;
                    self.buffer.clear();
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
}

impl Default for ToolSyntaxFilter {
    fn default() -> Self {
        Self::new()
    }
}
