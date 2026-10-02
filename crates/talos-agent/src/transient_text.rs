//! Observer-only filtering of explicitly declared model-private tokens.
//!
//! This module does not discover tokens by syntax and never edits the provider
//! transcript. Its caller supplies concrete tokens from tool-owned metadata.

use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct PrivateTokens {
    tokens: Vec<String>,
    trie: Vec<TrieNode>,
}

#[derive(Clone, Default)]
struct TrieNode {
    children: HashMap<u8, usize>,
    terminal: bool,
}

impl PrivateTokens {
    pub(crate) fn extend(&mut self, tokens: impl IntoIterator<Item = String>) {
        for token in tokens {
            if !token.is_empty() && !self.tokens.contains(&token) {
                self.insert(&token);
                self.tokens.push(token);
            }
        }
        // Prefer a complete longer token to its shorter prefix.
        self.tokens
            .sort_by_key(|token| std::cmp::Reverse(token.len()));
    }

    fn insert(&mut self, token: &str) {
        if self.trie.is_empty() {
            self.trie.push(TrieNode::default());
        }
        let mut node = 0;
        for byte in token.bytes() {
            let next = if let Some(next) = self.trie[node].children.get(&byte) {
                *next
            } else {
                let next = self.trie.len();
                self.trie.push(TrieNode::default());
                self.trie[node].children.insert(byte, next);
                next
            };
            node = next;
        }
        self.trie[node].terminal = true;
    }

    pub(crate) fn project(&self, text: &str) -> String {
        let mut pending = text.to_owned();
        self.drain(&mut pending, true)
    }

    pub(crate) fn project_value(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(text) => *text = self.project(text),
            serde_json::Value::Array(values) => {
                for value in values {
                    self.project_value(value);
                }
            }
            serde_json::Value::Object(values) => {
                for value in values.values_mut() {
                    self.project_value(value);
                }
            }
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
            }
        }
    }

    /// Holds only a suffix that could still become a private token. Callers
    /// keep separate buffers for text and thinking, and flush before TurnEnd.
    pub(crate) fn push(&self, pending: &mut String, delta: &str) -> String {
        pending.push_str(delta);
        self.drain(pending, false)
    }

    pub(crate) fn finish(&self, pending: &mut String) -> String {
        self.drain(pending, true)
    }

    fn drain(&self, pending: &mut String, final_chunk: bool) -> String {
        let mut output = String::new();
        let mut consumed = 0;
        while consumed < pending.len() {
            let rest = &pending[consumed..];
            let (matched, partial) = self.match_prefix(rest, final_chunk);
            if partial {
                break;
            }
            if matched > 0 {
                output.push_str("[private]");
                consumed += matched;
            } else if let Some(ch) = rest.chars().next() {
                output.push(ch);
                consumed += ch.len_utf8();
            }
        }
        pending.drain(..consumed);
        output
    }

    fn match_prefix(&self, rest: &str, final_chunk: bool) -> (usize, bool) {
        if self.trie.is_empty() {
            return (0, false);
        }
        let mut node = 0;
        let mut matched = 0;
        let mut traversed = 0;
        for byte in rest.bytes() {
            let Some(next) = self.trie[node].children.get(&byte) else {
                break;
            };
            node = *next;
            traversed += 1;
            if self.trie[node].terminal {
                matched = traversed;
            }
        }
        let partial =
            !final_chunk && traversed == rest.len() && !self.trie[node].children.is_empty();
        (matched, partial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_utf8_split_matches_whole_text_projection() {
        let mut tokens = PrivateTokens::default();
        tokens.extend(["[snapshot:s1]", "1:aa", "1:aa|"].map(str::to_owned));
        let input = "中文 [snapshot:s1] 与 1:aa| 内容；1:ab| 保留";
        let expected = "中文 [private] 与 [private] 内容；1:ab| 保留";
        assert_eq!(tokens.project(input), expected);
        for split in 0..=input.len() {
            if !input.is_char_boundary(split) {
                continue;
            }
            let mut pending = String::new();
            let mut output = tokens.push(&mut pending, &input[..split]);
            output.push_str(&tokens.push(&mut pending, &input[split..]));
            output.push_str(&tokens.finish(&mut pending));
            assert_eq!(output, expected, "split {split}");
            assert!(pending.is_empty());
        }
    }

    #[test]
    fn character_chunks_do_not_release_a_private_prefix() {
        let mut tokens = PrivateTokens::default();
        tokens.extend(["[snapshot:s1]".to_owned()]);
        let mut pending = String::new();
        for ch in "[snapshot:s1".chars() {
            assert!(tokens.push(&mut pending, &ch.to_string()).is_empty());
        }
        assert_eq!(tokens.push(&mut pending, "]"), "[private]");
        assert!(tokens.finish(&mut pending).is_empty());
    }

    #[test]
    fn ordinary_text_and_incomplete_prefixes_are_preserved() {
        let mut tokens = PrivateTokens::default();
        tokens.extend([String::new(), "[snapshot:s1]".to_owned()]);
        for input in ["中文🙂", "1:aa|", "[snapshot:s2]", "a [snap"] {
            let mut pending = String::new();
            let mut output = tokens.push(&mut pending, input);
            output.push_str(&tokens.finish(&mut pending));
            assert_eq!(output, input);
        }
    }

    #[test]
    fn nested_tool_arguments_hide_handles_without_changing_other_values() {
        let mut tokens = PrivateTokens::default();
        tokens.extend(["sdeadbeef1".to_owned()]);
        let mut value = serde_json::json!({
            "arguments": ["handle sdeadbeef1", {"body": "中文 sdeadbeef1"}],
            "enabled": true,
            "count": 2,
            "other": "sdeadbeef2"
        });
        tokens.project_value(&mut value);
        assert_eq!(
            value,
            serde_json::json!({
                "arguments": ["handle [private]", {"body": "中文 [private]"}],
                "enabled": true,
                "count": 2,
                "other": "sdeadbeef2"
            })
        );
    }

    #[test]
    fn matching_does_not_reprocess_replacement_text() {
        let mut tokens = PrivateTokens::default();
        tokens.extend(["secret", "private"].map(str::to_owned));
        assert_eq!(tokens.project("secretprivate"), "[private][private]");
    }
}
