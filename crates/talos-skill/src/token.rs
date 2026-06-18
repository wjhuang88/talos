/// Estimates the token count for a given text.
///
/// Uses a simple heuristic: ~4 characters per token for English text,
/// ~2 characters per token for CJK (Chinese, Japanese, Korean) text.
/// This is a rough approximation suitable for budget planning, not a
/// substitute for the actual tokenizer.
pub fn estimate_tokens(text: &str) -> usize {
    let mut cjk_count = 0usize;
    let mut total_chars = 0usize;

    for ch in text.chars() {
        total_chars += 1;
        if matches!(
            ch as u32,
            0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0xF900..=0xFAFF
                | 0x3040..=0x309F | 0x30A0..=0x30FF | 0xAC00..=0xD7AF
        ) {
            cjk_count += 1;
        }
    }

    let english_chars = total_chars.saturating_sub(cjk_count);
    english_chars.div_ceil(4) + cjk_count.div_ceil(2)
}
