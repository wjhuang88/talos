use super::*;
use crate::scrollback_input::{
    build_input_text, composer_content_width, composer_scroll_offset, cursor_line_col_with_width,
    input_line_count_with_width,
};
use crate::{inline_terminal::ViewportComponent, scrollback::InputComponent};
use ratatui::{buffer::Buffer, layout::Rect};

#[test]
fn input_line_count_with_width_counts_content_rows() {
    assert_eq!(input_line_count_with_width(&"a".repeat(80), 20), 4);
    assert_eq!(input_line_count_with_width("你好你好你好你好你好", 5), 5);
    assert_eq!(input_line_count_with_width("aaaa\nbbbbbbbb", 4), 3);
    assert_eq!(input_line_count_with_width("", 20), 1);
    assert_eq!(input_line_count_with_width("abc", 0), 1);
    assert_eq!(input_line_count_with_width(&"a".repeat(20), 20), 1);
}

#[test]
fn cursor_line_col_with_width_tracks_wrapped_cursor_position() {
    assert_eq!(cursor_line_col_with_width("abc", 80), (0, 3));
    assert_eq!(cursor_line_col_with_width(&"a".repeat(25), 20), (1, 5));
    assert_eq!(cursor_line_col_with_width(&"a".repeat(20), 20), (1, 0));
    assert_eq!(cursor_line_col_with_width("你好你", 4), (1, 2));
    assert_eq!(cursor_line_col_with_width("你好你好", 4), (2, 0));
    assert_eq!(cursor_line_col_with_width("aaaa\nbbbbb", 4), (2, 1));
    assert_eq!(cursor_line_col_with_width("", 20), (0, 0));
    assert_eq!(cursor_line_col_with_width("abc", 0), (0, 0));
}

#[test]
fn composer_content_width_reserves_prefix_columns() {
    assert_eq!(composer_content_width(80), 76);
    assert_eq!(composer_content_width(3), 1);
    assert_eq!(composer_content_width(0), 1);
}

#[test]
fn input_component_exact_boundary_keeps_last_cell_visible() {
    let mut state = TuiState::new();
    state.input_buffer = "a".repeat(76);
    state.cursor_pos = state.input_buffer.chars().count();
    let area = Rect::new(0, 0, 80, 2);
    let mut buffer = Buffer::empty(area);
    let mut frame = crate::inline_terminal::InlineFrame::new(area, &mut buffer);

    InputComponent {
        state: &state,
        max_height: crate::scrollback::MAX_COMPOSER_LINES,
    }
    .render(&mut frame, area);

    assert_eq!(buffer[(78, 0)].symbol(), "a");
    assert_eq!(buffer[(79, 0)].symbol(), " ");
}

#[test]
fn height_hint_cap_uses_wrapped_content_and_cursor_spill_row() {
    let mut state = TuiState::new();
    state.input_buffer = "one\ntwo\nthree".to_string();
    state.cursor_pos = state.input_buffer.chars().count();
    assert_eq!(
        InputComponent {
            state: &state,
            max_height: crate::scrollback::MAX_COMPOSER_LINES
        }
        .height_hint(80),
        3
    );

    state.input_buffer = std::iter::repeat_n("line", 25)
        .collect::<Vec<_>>()
        .join("\n");
    state.cursor_pos = state.input_buffer.chars().count();
    assert_eq!(
        InputComponent {
            state: &state,
            max_height: crate::scrollback::MAX_COMPOSER_LINES
        }
        .height_hint(80),
        10
    );

    state.input_buffer = "a".repeat(20);
    state.cursor_pos = state.input_buffer.chars().count();
    assert_eq!(
        InputComponent {
            state: &state,
            max_height: crate::scrollback::MAX_COMPOSER_LINES
        }
        .height_hint(23),
        2
    );
}

#[test]
fn composer_scroll_offset_keeps_cursor_in_capped_window() {
    let buffer = std::iter::repeat_n("line", 25)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(composer_scroll_offset(&buffer, &buffer, 80, 10), 15);

    let short_buffer = std::iter::repeat_n("line", 5)
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        composer_scroll_offset(&short_buffer, &short_buffer, 80, 10),
        0
    );

    let cursor_at_row_three = "line\n".repeat(3);
    assert_eq!(
        composer_scroll_offset(&cursor_at_row_three, &buffer, 80, 10),
        3
    );

    let boundary = "a".repeat(20);
    assert_eq!(composer_scroll_offset(&boundary, &boundary, 20, 10), 0);

    let max_boundary = "a".repeat(200);
    assert_eq!(
        composer_scroll_offset(&max_boundary, &max_boundary, 20, 10),
        1
    );
}

#[test]
fn build_input_text_wraps_long_buffer_at_content_width() {
    let mut state = TuiState::new();
    state.input_buffer = "a".repeat(200);
    state.cursor_pos = state.input_buffer.chars().count();

    let text = build_input_text(&state, 77);

    assert_eq!(text.lines.len(), 3);
    assert_eq!(cursor_line_col_with_width(&state.input_buffer, 77), (2, 46));
}

#[test]
fn build_input_text_shows_last_ten_visual_lines() {
    let mut state = TuiState::new();
    state.input_buffer = (0..25)
        .map(|line| format!("line-{line}"))
        .collect::<Vec<_>>()
        .join("\n");
    state.cursor_pos = state.input_buffer.chars().count();

    let text = build_input_text(&state, 77);

    assert_eq!(text.lines.len(), 10);
    assert_eq!(text.lines[0].to_string(), "   line-15");
    assert_eq!(text.lines[9].to_string(), "   line-24");
}

#[test]
fn build_input_text_respects_compressed_composer_height() {
    let mut state = TuiState::new();
    state.input_buffer = (0..5)
        .map(|line| format!("line-{line}"))
        .collect::<Vec<_>>()
        .join("\n");
    state.cursor_pos = state.input_buffer.chars().count();

    let text = crate::scrollback_input::build_input_text_with_max_height(&state, 77, 2);

    assert_eq!(text.lines.len(), 2);
    assert_eq!(text.lines[0].to_string(), "   line-3");
    assert_eq!(text.lines[1].to_string(), "   line-4");
    assert_eq!(
        composer_scroll_offset(&state.input_buffer, &state.input_buffer, 77, 2),
        3
    );
}

#[test]
fn build_input_text_keeps_short_buffer_on_one_line() {
    let mut state = TuiState::new();
    state.input_buffer = "abcdefghij".to_string();
    state.cursor_pos = state.input_buffer.chars().count();

    let text = build_input_text(&state, 77);

    assert_eq!(text.lines.len(), 1);
    assert_eq!(text.lines[0].to_string(), " > abcdefghij");
    assert_eq!(
        composer_scroll_offset(&state.input_buffer, &state.input_buffer, 77, 10),
        0
    );
}

#[test]
fn cursor_placement_wrap_and_scroll_math_stays_in_visible_window() {
    let wrapped = "a".repeat(25);
    assert_eq!(cursor_line_col_with_width(&wrapped, 20), (1, 5));
    assert_eq!(composer_scroll_offset(&wrapped, &wrapped, 20, 10), 0);

    let scrolled = std::iter::repeat_n("line", 30)
        .collect::<Vec<_>>()
        .join("\n");
    let (cursor_row, _) = cursor_line_col_with_width(&scrolled, 20);
    let offset = composer_scroll_offset(&scrolled, &scrolled, 20, 10);
    assert_eq!(offset, 20);
    assert_eq!(cursor_row.saturating_sub(offset), 9);
}

#[test]
fn credential_input_collects_pasted_text_and_submits() {
    let mut state = TuiState::new();

    state.open_credential_input("openai", None, false, None);
    state.credential_append_str("sk-test-key\n");

    let response = state.credential_submit().expect("credential response");
    assert_eq!(response.provider, "openai");
    assert_eq!(response.api_key, "sk-test-key");
    assert_eq!(response.model_id, None);
    assert!(!response.connect_mode);
    assert!(response.base_url.is_none());
    assert!(!state.slash_menu.is_open);
}

#[test]
fn empty_credential_submit_closes_without_response() {
    let mut state = TuiState::new();

    state.open_credential_input("openai", Some("gpt-4.1"), false, None);

    assert!(state.credential_submit().is_none());
    assert!(!state.slash_menu.is_open);
}

// ── /connect credential (standard provider key-only, custom provider URL) ──────────────

#[test]
fn connect_mode_standard_provider_submits_without_base_url_field() {
    let mut state = TuiState::new();
    state.open_credential_input(
        "groq",
        None,
        true,
        Some("https://api.groq.com/openai/v1".to_string()),
    );

    state.credential_append_str("gsk-test-key");
    let response = state
        .credential_submit()
        .expect("standard provider should submit after API key");

    assert_eq!(response.provider, "groq");
    assert_eq!(response.api_key, "gsk-test-key");
    assert_eq!(
        response.base_url.as_deref(),
        Some("https://api.groq.com/openai/v1")
    );
    assert!(!state.slash_menu.is_open);
}

#[test]
fn connect_mode_custom_provider_first_submit_advances_to_base_url_field() {
    let mut state = TuiState::new();
    state.open_credential_input("custom-gw", None, true, None);

    state.credential_append_str("custom-key");
    let response = state.credential_submit();

    assert!(
        response.is_none(),
        "custom provider must collect base URL before submit"
    );
    assert!(
        state.slash_menu.is_open,
        "panel must stay open for base_url"
    );
    assert_eq!(
        state.slash_menu.credential_field,
        crate::state::CredentialField::BaseUrl
    );
    assert_eq!(state.slash_menu.credential_buffer, "custom-key");
}

#[test]
fn connect_mode_custom_provider_second_submit_returns_typed_base_url() {
    let mut state = TuiState::new();
    state.open_credential_input("custom-gw", None, true, None);

    state.credential_append_str("custom-key");
    state.credential_submit();
    state.credential_append_str("https://custom.example/v1");
    let response = state
        .credential_submit()
        .expect("second submit must return response");

    assert_eq!(response.provider, "custom-gw");
    assert_eq!(response.api_key, "custom-key");
    assert!(response.connect_mode);
    assert_eq!(
        response.base_url.as_deref(),
        Some("https://custom.example/v1")
    );
    assert!(!state.slash_menu.is_open);
}

#[test]
fn connect_mode_custom_provider_empty_base_url_stays_open() {
    let mut state = TuiState::new();
    state.open_credential_input("custom-gw", None, true, None);

    state.credential_append_str("custom-key");
    state.credential_submit();
    let response = state.credential_submit();

    assert!(response.is_none());
    assert!(state.slash_menu.is_open);
    assert_eq!(
        state.slash_menu.credential_field,
        crate::state::CredentialField::BaseUrl
    );
}

#[test]
fn connect_mode_empty_api_key_cancels_without_advancing() {
    let mut state = TuiState::new();
    state.open_credential_input("groq", None, true, None);

    let response = state.credential_submit();

    assert!(response.is_none());
    assert!(
        !state.slash_menu.is_open,
        "empty API key in connect_mode must cancel, not advance"
    );
}

#[test]
fn non_connect_mode_ignores_base_url_and_submits_single_phase() {
    let mut state = TuiState::new();
    state.open_credential_input("anthropic", None, false, None);

    state.credential_append_str("sk-ant-test");
    let response = state
        .credential_submit()
        .expect("non-connect mode must submit on first Enter");

    assert_eq!(response.api_key, "sk-ant-test");
    assert!(response.base_url.is_none());
    assert!(!response.connect_mode);
}

#[test]
fn credential_append_and_backspace_route_to_active_field() {
    let mut state = TuiState::new();
    state.open_credential_input("groq", None, true, None);

    state.credential_append_str("abc");
    state.credential_backspace();
    assert_eq!(state.slash_menu.credential_buffer, "ab");
    assert!(state.slash_menu.base_url_buffer.is_empty());

    state.credential_append_str("x");
    state.credential_submit(); // advance to BaseUrl
    state.credential_append_str("https://x.example");
    state.credential_backspace();

    assert_eq!(state.slash_menu.credential_buffer, "abx");
    assert_eq!(state.slash_menu.base_url_buffer, "https://x.exampl");
}

#[test]
fn paste_is_ignored_while_approval_is_visible() {
    let mut state = TuiState::new();

    state.activate_approval("write", "file edit");
    state.input_paste("secret");

    assert_eq!(state.input_buffer, "");
}

#[test]
fn paste_updates_filter_when_picker_is_visible() {
    let mut state = TuiState::new();
    let data = ModelPickerData {
        recent: vec![],
        ready_models: vec![],
        setup_providers: vec![],
    };

    state.open_model_picker(&data);
    state.input_paste("filter_text");

    assert_eq!(state.input_buffer, "filter_text");
}

#[test]
fn paste_still_updates_slash_query_and_composer() {
    let mut state = TuiState::new();

    state.input_paste("hello");
    assert_eq!(state.input_buffer, "hello");

    state.input_clear();
    state.open_slash_menu(talos_conversation::command_registry());
    state.input_paste("model");

    assert_eq!(state.input_buffer, "/model");
    // TUI-033: /model is DirectExecution — Enter sends bare "/model" and
    // clears the composer instead of filling it with "/model ".
    let action = state.accept_selected_panel_item();
    assert_eq!(
        action,
        crate::state::PanelAction::SendMessage("/model".to_string())
    );
    assert!(state.input_buffer.is_empty());
}

#[test]
fn approval_state_preserves_full_multibyte_arguments() {
    let cmd = "gh issue create --title \"feat: write 和 edit 工具应显示内容输出\" --label bug";
    let state = BottomPanelState::open_approval("bash", cmd);
    if let PanelKind::Approval { arguments, .. } = state.kind.as_ref().unwrap() {
        assert_eq!(arguments, cmd);
    }
}

#[test]
fn approval_truncation_short_string_unchanged() {
    let state = BottomPanelState::open_approval("bash", "ls -la");
    if let PanelKind::Approval { arguments, .. } = state.kind.as_ref().unwrap() {
        assert_eq!(arguments, "ls -la");
    }
}

// ── MC106: Group-aware search filtering ─────────────────────────────

fn model_item(id: &str, provider: &str, is_current: bool) -> talos_conversation::ModelPickerItem {
    talos_conversation::ModelPickerItem {
        command: "/model".to_string(),
        model_id: id.to_string(),
        provider: provider.to_string(),
        label: format!("{id} {provider}"),
        context_limit: Some(100_000),
        pricing: None,
        authenticated: true,
        is_current,
        variants: vec![],
        variant: None,
    }
}

fn sample_model_picker_data() -> ModelPickerData {
    ModelPickerData {
        recent: vec![],
        ready_models: vec![
            model_item("claude-sonnet-4-5", "anthropic", true),
            model_item("claude-opus-4-1", "anthropic", false),
            model_item("gpt-4o", "openai", false),
            model_item("o3", "openai", false),
        ],
        setup_providers: vec![],
    }
}

fn connect_item(
    provider: &str,
    name: &str,
    has_credential: bool,
) -> talos_conversation::ConnectPickerItem {
    talos_conversation::ConnectPickerItem {
        provider: provider.to_string(),
        name: name.to_string(),
        model_count: 3,
        api_base_url: None,
        has_credential,
        doc_url: None,
    }
}

fn sample_connect_picker_data() -> talos_conversation::ConnectPickerData {
    talos_conversation::ConnectPickerData {
        connected: vec![connect_item("anthropic", "Anthropic", true)],
        available: vec![
            connect_item("openai", "OpenAI", false),
            connect_item("groq", "Groq", false),
        ],
    }
}

#[test]
fn model_picker_level1_search_matches_provider_names() {
    let data = sample_model_picker_data();
    let menu = BottomPanelState::open_model_picker(&data);

    let indices = menu.filtered_indices("openai");
    let visible_labels: Vec<&str> = indices
        .iter()
        .map(|&i| menu.items[i].label.as_str())
        .collect();

    assert!(
        visible_labels.contains(&"openai"),
        "openai provider row must be visible when querying 'openai': {visible_labels:?}"
    );
    assert!(
        !visible_labels.contains(&"anthropic"),
        "non-matching anthropic provider must be hidden: {visible_labels:?}"
    );
}

#[test]
fn model_picker_level2_navigation_skips_headers_and_filtered_items() {
    let data = sample_model_picker_data();
    let mut menu = BottomPanelState::open_model_list("openai", &data);

    let navigable: Vec<usize> = menu
        .filtered_indices("")
        .into_iter()
        .filter(|&i| menu.items[i].action != PanelItemAction::Header)
        .collect();
    assert_eq!(navigable.len(), 2, "openai should have 2 models");

    menu.selected_index = navigable[0];
    menu.select_next("");
    assert_ne!(
        menu.items[menu.selected_index].action,
        PanelItemAction::Header,
        "select_next must never land on a Header"
    );
    assert_ne!(
        menu.selected_index, navigable[0],
        "select_next must move to the next model"
    );

    menu.select_next("");
    assert!(
        menu.items[menu.selected_index].label.contains("gpt-4o")
            || menu.items[menu.selected_index].label.contains("o3"),
        "wrapped selection must stay within openai models, got {:?}",
        menu.items[menu.selected_index].label
    );
}

#[test]
fn model_picker_search_no_match_hides_all_groups() {
    let data = sample_model_picker_data();
    let menu = BottomPanelState::open_model_picker(&data);

    let indices = menu.filtered_indices("zzz-nonexistent");
    assert!(indices.is_empty(), "no groups should match: {indices:?}");
}

#[test]
fn model_picker_empty_query_shows_everything() {
    let data = sample_model_picker_data();
    let menu = BottomPanelState::open_model_picker(&data);

    let indices = menu.filtered_indices("");
    assert_eq!(indices.len(), menu.items.len());
}

#[test]
fn model_picker_level2_navigation_within_filtered_set() {
    let data = sample_model_picker_data();
    let mut menu = BottomPanelState::open_model_list("openai", &data);

    menu.selected_index = menu
        .filtered_indices("openai")
        .into_iter()
        .find(|&i| menu.items[i].action != PanelItemAction::Header)
        .unwrap_or(0);

    let first_selection = menu.selected_index;
    assert!(menu.items[first_selection].action != PanelItemAction::Header);

    menu.select_next("openai");
    assert_ne!(
        menu.items[menu.selected_index].action,
        PanelItemAction::Header,
        "select_next must never land on a Header"
    );
    assert_ne!(
        menu.selected_index, first_selection,
        "select_next must move within the filtered openai model set"
    );

    menu.select_next("openai");
    let after_wrap = menu.selected_index;
    assert!(
        menu.items[after_wrap].label.contains("gpt-4o")
            || menu.items[after_wrap].label.contains("o3"),
        "wrapped selection must stay within openai models, got {:?}",
        menu.items[after_wrap].label
    );
}

#[test]
fn model_picker_select_next_prev_never_select_header() {
    let data = sample_model_picker_data();
    let mut menu = BottomPanelState::open_model_picker(&data);

    for _ in 0..(menu.items.len() * 2) {
        menu.select_next("");
        assert_ne!(
            menu.items[menu.selected_index].action,
            PanelItemAction::Header
        );
    }
    for _ in 0..(menu.items.len() * 2) {
        menu.select_prev("");
        assert_ne!(
            menu.items[menu.selected_index].action,
            PanelItemAction::Header
        );
    }
}

#[test]
fn model_picker_level1_lists_providers_not_models() {
    let data = sample_model_picker_data();
    let menu = BottomPanelState::open_model_picker(&data);

    let open_model_list_count = menu
        .items
        .iter()
        .filter(|i| {
            matches!(
                &i.action,
                PanelItemAction::OpenModelList { .. } | PanelItemAction::SwitchModel { .. }
            )
        })
        .count();
    assert!(
        open_model_list_count > 0,
        "Level 1 must contain provider or recent rows"
    );

    let providers: Vec<&str> = menu
        .items
        .iter()
        .filter_map(|i| match &i.action {
            PanelItemAction::OpenModelList { provider } => Some(provider.as_str()),
            _ => None,
        })
        .collect();
    assert!(providers.contains(&"openai"), "openai provider must appear");
    assert!(
        providers.contains(&"anthropic"),
        "anthropic provider must appear"
    );
}

#[test]
fn connect_picker_search_matches_provider_group() {
    let data = sample_connect_picker_data();
    let menu = BottomPanelState::open_connect_picker(&data);

    let indices = menu.filtered_indices("groq");
    let labels: Vec<&str> = indices
        .iter()
        .map(|&i| menu.items[i].label.as_str())
        .collect();

    assert!(
        labels.contains(&"Available"),
        "matching group header must show: {labels:?}"
    );
    assert!(labels.iter().any(|l| l.contains("Groq")), "{labels:?}");
    assert!(
        !labels.iter().any(|l| l.contains("OpenAI")),
        "non-matching sibling in same group must be hidden: {labels:?}"
    );
    assert!(
        !labels.contains(&"Connected"),
        "non-matching Connected group must be hidden entirely: {labels:?}"
    );
}

#[test]
fn connect_picker_is_picker_and_supports_filtering() {
    let data = sample_connect_picker_data();
    let menu = BottomPanelState::open_connect_picker(&data);
    assert!(menu.is_picker());
}

#[test]
fn reset_selection_for_query_lands_on_first_navigable_match() {
    let data = sample_model_picker_data();
    let mut menu = BottomPanelState::open_model_list("openai", &data);

    menu.reset_selection_for_query("openai");
    assert_ne!(
        menu.items[menu.selected_index].action,
        PanelItemAction::Header
    );
    assert!(
        menu.items[menu.selected_index].label.contains("gpt-4o")
            || menu.items[menu.selected_index].label.contains("o3")
    );
}

#[test]
fn reset_selection_for_query_falls_back_to_zero_when_nothing_matches() {
    let data = sample_model_picker_data();
    let mut menu = BottomPanelState::open_model_picker(&data);

    menu.reset_selection_for_query("zzz-nonexistent");
    assert_eq!(menu.selected_index, 0);
}

#[test]
fn tuistate_panel_query_uses_raw_buffer_for_pickers() {
    let mut state = TuiState::new();
    let data = sample_model_picker_data();
    state.open_model_picker(&data);
    state.input_append_char('g');
    state.input_append_char('p');
    state.input_append_char('t');

    assert_eq!(state.panel_query(), "gpt");
}

#[test]
fn tuistate_panel_query_strips_slash_for_slash_menu() {
    let mut state = TuiState::new();
    state.open_slash_menu(talos_conversation::command_registry());
    state.append_slash_query_char('m');
    state.append_slash_query_char('o');

    assert_eq!(state.panel_query(), "mo");
}

#[test]
fn input_append_char_newline_inserts_multiline_buffer() {
    let mut state = TuiState::new();
    state.input_append_char('h');
    state.input_append_char('i');
    state.input_append_char('\n');
    state.input_append_char('t');
    state.input_append_char('h');
    state.input_append_char('e');
    state.input_append_char('r');
    state.input_append_char('e');
    assert_eq!(state.input_buffer, "hi\nthere");
    assert_eq!(state.cursor_pos, 8);
}

#[test]
fn input_append_char_newline_at_cursor_mid_buffer() {
    let mut state = TuiState::new();
    state.input_append_str("hello");
    state.cursor_pos = 2;
    state.input_append_char('\n');
    assert_eq!(state.input_buffer, "he\nllo");
    assert_eq!(state.cursor_pos, 3);
}

// ── TUI-030 composer input history tests ──────────────────────────────────

#[test]
fn history_prev_navigates_to_newest_then_oldest() {
    let mut state = TuiState::new();
    state.input_submit(); // empty, no record
    state.input_append_str("hello");
    state.input_submit();
    state.input_append_str("world");
    state.input_submit();

    assert_eq!(state.input_history, vec!["hello", "world"]);

    // First Up → newest entry
    state.history_prev();
    assert_eq!(state.input_buffer, "world");
    assert_eq!(state.history_cursor, Some(1));

    // Second Up → oldest entry
    state.history_prev();
    assert_eq!(state.input_buffer, "hello");
    assert_eq!(state.history_cursor, Some(0));

    // Third Up at oldest → stays
    state.history_prev();
    assert_eq!(state.input_buffer, "hello");
    assert_eq!(state.history_cursor, Some(0));
}

#[test]
fn history_next_restores_exact_draft() {
    let mut state = TuiState::new();
    state.input_append_str("first");
    state.input_submit();

    // Type a draft, then navigate
    state.input_append_str("my draft");
    state.history_prev();
    assert_eq!(state.input_buffer, "first");
    assert_eq!(state.draft_input, "my draft");

    // Down past newest restores draft
    state.history_next();
    assert_eq!(state.input_buffer, "my draft");
    assert!(state.history_cursor.is_none());
    assert!(state.draft_input.is_empty());
}

#[test]
fn history_next_at_draft_does_nothing() {
    let mut state = TuiState::new();
    state.input_append_str("entry");
    state.input_submit();
    state.history_next(); // no-op when already at draft
    assert!(state.history_cursor.is_none());
}

#[test]
fn history_prev_empty_history_does_nothing() {
    let mut state = TuiState::new();
    state.history_prev();
    assert!(state.input_buffer.is_empty());
    assert!(state.history_cursor.is_none());
}

#[test]
fn history_dedup_consecutive_duplicates() {
    let mut state = TuiState::new();
    state.input_append_str("same");
    state.input_submit();
    state.input_append_str("same");
    state.input_submit();
    state.input_append_str("different");
    state.input_submit();
    state.input_append_str("different");
    state.input_submit();

    assert_eq!(state.input_history, vec!["same", "different"]);
}

#[test]
fn history_submit_resets_cursor() {
    let mut state = TuiState::new();
    state.input_append_str("entry");
    state.input_submit();

    // Navigate to history, then submit
    state.history_prev();
    assert!(state.history_cursor.is_some());
    state.input_submit();
    assert!(state.history_cursor.is_none());
    assert!(state.draft_input.is_empty());
}

#[test]
fn history_navigation_roundtrip_preserves_draft() {
    let mut state = TuiState::new();
    state.input_append_str("a");
    state.input_submit();
    state.input_append_str("b");
    state.input_submit();
    state.input_append_str("c");
    state.input_submit();

    // Type a multiline draft
    state.input_append_str("line one\nline two");
    state.history_prev(); // → "c"
    state.history_prev(); // → "b"
    state.history_prev(); // → "a"
    state.history_next(); // → "b"
    state.history_next(); // → "c"
    state.history_next(); // → draft

    assert_eq!(state.input_buffer, "line one\nline two");
    assert!(state.history_cursor.is_none());
}

#[test]
fn history_non_consecutive_duplicates_kept() {
    let mut state = TuiState::new();
    state.input_append_str("x");
    state.input_submit();
    state.input_append_str("y");
    state.input_submit();
    state.input_append_str("x");
    state.input_submit();

    // Non-consecutive "x" is kept
    assert_eq!(state.input_history, vec!["x", "y", "x"]);
}

#[test]
fn history_load_sets_cursor_to_end() {
    let mut state = TuiState::new();
    state.input_append_str("hello world");
    state.input_submit();

    state.history_prev();
    assert_eq!(state.cursor_pos, "hello world".chars().count());
}

// ── I145 / TUI-026 queue preview tests ─────────────────────────────────────

use talos_conversation::{SteeringQueueEntry, SteeringQueueSnapshot};

fn snap(entries: &[&str], total: usize) -> SteeringQueueSnapshot {
    let omitted = total.saturating_sub(entries.len());
    SteeringQueueSnapshot {
        entries: entries
            .iter()
            .map(|t| SteeringQueueEntry {
                text: t.to_string(),
                truncated: false,
            })
            .collect(),
        total_count: total,
        omitted_count: omitted,
    }
}

#[test]
fn queue_preview_height_zero_when_empty() {
    let c = crate::scrollback::QueuePreviewComponent {
        snapshot: None,
        followup_count: 0,
        max_rows: 6,
    };
    assert_eq!(c.height_hint(80), 0);
}

#[test]
fn queue_preview_height_respects_six_row_cap() {
    let entries: Vec<String> = (0..8).map(|i| format!("msg{i}")).collect();
    let refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let s = snap(&refs, 8);
    let c = crate::scrollback::QueuePreviewComponent {
        snapshot: Some(&s),
        followup_count: 0,
        max_rows: 6,
    };
    let h = c.height_hint(80);
    assert!(h <= 6, "height_hint must be <= 6, got {h}");
}

#[test]
fn queue_preview_height_shows_hidden_count_for_8_entries() {
    let entries: Vec<String> = (0..8).map(|i| format!("msg{i}")).collect();
    let refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let s = snap(&refs, 8);
    let c = crate::scrollback::QueuePreviewComponent {
        snapshot: Some(&s),
        followup_count: 0,
        max_rows: 6,
    };
    let h = c.height_hint(80);
    // 1 header + 4 entries + 1 summary (+3 more) = 6
    assert_eq!(
        h, 6,
        "8 entries at max_rows=6 should produce header + 4 entries + summary"
    );
}

#[test]
fn queue_preview_height_compresses_on_narrow_max_rows() {
    let entries: Vec<String> = (0..8).map(|i| format!("msg{i}")).collect();
    let refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let s = snap(&refs, 8);
    let c = crate::scrollback::QueuePreviewComponent {
        snapshot: Some(&s),
        followup_count: 0,
        max_rows: 3,
    };
    let h = c.height_hint(80);
    assert!(h <= 3, "max_rows=3 must cap height at 3, got {h}");
}

#[test]
fn queue_preview_hidden_count_covers_unrendered_entries() {
    let entries: Vec<String> = (0..8).map(|i| format!("msg{i}")).collect();
    let refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let s = snap(&refs, 8);
    let c = crate::scrollback::QueuePreviewComponent {
        snapshot: Some(&s),
        followup_count: 0,
        max_rows: 6,
    };
    let plan = c.plan();
    assert_eq!(
        plan.entries_to_show, 4,
        "should show 4 entries (reserving 1 for summary)"
    );
    assert_eq!(plan.hidden_count, 4, "8 total - 4 shown = 4 hidden");
    assert!(plan.show_summary, "must show +4 more summary");
}

#[test]
fn queue_preview_no_summary_when_all_fit() {
    let s = snap(&["a", "b"], 2);
    let c = crate::scrollback::QueuePreviewComponent {
        snapshot: Some(&s),
        followup_count: 0,
        max_rows: 6,
    };
    let plan = c.plan();
    assert_eq!(plan.entries_to_show, 2);
    assert_eq!(plan.hidden_count, 0);
    assert!(!plan.show_summary);
    assert_eq!(plan.total_rows, 3); // 1 header + 2 entries
}

#[test]
fn normalize_single_line_replaces_newlines() {
    assert_eq!(
        crate::scrollback::normalize_single_line("hello\nworld"),
        "hello ⏎ world"
    );
    assert_eq!(
        crate::scrollback::normalize_single_line("a\rb\r\nc"),
        "ab ⏎ c"
    );
}

#[test]
fn truncate_to_display_width_cjk_safe() {
    assert_eq!(
        crate::scrollback::truncate_to_display_width("abc", 10),
        "abc"
    );
    assert_eq!(
        crate::scrollback::truncate_to_display_width("你好世界", 5),
        "你好…"
    );
    assert_eq!(
        crate::scrollback::truncate_to_display_width("abcdef", 4),
        "abc…"
    );
}

// ── I145 render tests using Buffer + InlineFrame ────────────────────────────

use crate::inline_terminal::InlineFrame;
use crate::scrollback::QueuePreviewComponent;

fn render_queue(
    snapshot: Option<&SteeringQueueSnapshot>,
    followup: usize,
    max_rows: u16,
    width: u16,
) -> (ratatui::buffer::Buffer, u16) {
    let comp = QueuePreviewComponent {
        snapshot,
        followup_count: followup,
        max_rows,
    };
    let h = comp.height_hint(width);
    let area = Rect::new(0, 0, width, h.max(1));
    let mut buf = ratatui::buffer::Buffer::empty(area);
    let mut frame = InlineFrame::new(area, &mut buf);
    comp.render(&mut frame, area);
    (buf, h)
}

fn buffer_line(buf: &ratatui::buffer::Buffer, y: u16, width: u16) -> String {
    (0..width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect::<String>()
        .trim_end()
        .to_string()
}

#[test]
fn render_8_entries_produces_header_4_entries_summary_total_6_rows() {
    let entries: Vec<String> = (0..8).map(|i| format!("msg{i}")).collect();
    let refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let s = snap(&refs, 8);
    let (buf, h) = render_queue(Some(&s), 0, 6, 80);

    assert_eq!(h, 6);
    assert!(buffer_line(&buf, 0, 80).contains("8 queued inputs"));
    for i in 0..4 {
        let line = buffer_line(&buf, (i + 1) as u16, 80);
        assert!(
            line.contains(&format!("msg{i}")),
            "row {} should contain msg{}",
            i + 1,
            i
        );
    }
    assert!(
        buffer_line(&buf, 5, 80).contains("+4 more"),
        "row 5 should show +4 more"
    );
}

#[test]
fn render_truncated_entry_with_narrow_width_no_overflow() {
    let s = SteeringQueueSnapshot {
        entries: vec![SteeringQueueEntry {
            text: "x".repeat(100),
            truncated: true,
        }],
        total_count: 1,
        omitted_count: 0,
    };
    let width = 20u16;
    let (buf, h) = render_queue(Some(&s), 0, 6, width);
    assert_eq!(h, 2); // header + 1 entry

    let entry_line = buffer_line(&buf, 1, width);
    let display_width = unicode_width::UnicodeWidthStr::width(entry_line.as_str());
    assert!(
        display_width <= width as usize,
        "entry line display width {display_width} must be <= area width {width}"
    );
    assert!(
        entry_line.contains("⚠"),
        "truncated entry must show warning marker"
    );
}

#[test]
fn render_cjk_entry_uses_display_width_not_byte_count() {
    let s = snap(&["你好世界你好世界"], 1);
    let width = 15u16;
    let (buf, h) = render_queue(Some(&s), 0, 6, width);
    assert_eq!(h, 2); // header + 1 entry

    let entry_line = buffer_line(&buf, 1, width);
    assert!(
        entry_line.contains('…'),
        "CJK text exceeding budget must be truncated with ellipsis, got: {entry_line}"
    );
}

#[test]
fn render_multiline_entry_normalized_to_single_line() {
    let s = snap(&["line one\nline two"], 1);
    let (buf, h) = render_queue(Some(&s), 0, 6, 80);
    assert_eq!(h, 2); // header + 1 entry (not 3)

    let entry_line = buffer_line(&buf, 1, 80);
    assert!(
        entry_line.contains("⏎"),
        "newline should be normalized to visible indicator"
    );
    assert!(
        !entry_line.contains('\n'),
        "no raw newline in rendered line"
    );
}

#[test]
fn render_empty_snapshot_when_total_zero() {
    let s = snap(&[], 0);
    let (_, h) = render_queue(Some(&s), 0, 6, 80);
    assert_eq!(h, 0, "empty snapshot must produce 0 rows");
}

#[test]
fn render_narrow_terminal_compresses_queue_max_rows() {
    let entries: Vec<String> = (0..8).map(|i| format!("msg{i}")).collect();
    let refs: Vec<&str> = entries.iter().map(|s| s.as_str()).collect();
    let s = snap(&refs, 8);
    let (_, h) = render_queue(Some(&s), 0, 2, 80);
    assert!(h <= 2, "max_rows=2 must cap at 2, got {h}");
}

#[test]
fn session_boundary_empty_snapshot_produces_zero_height() {
    let empty = SteeringQueueSnapshot {
        entries: vec![],
        total_count: 0,
        omitted_count: 0,
    };
    assert_eq!(empty.total_count, 0);
    assert_eq!(empty.omitted_count, 0);
    assert!(empty.entries.is_empty());

    let comp = QueuePreviewComponent {
        snapshot: Some(&empty),
        followup_count: 0,
        max_rows: 6,
    };
    assert_eq!(
        comp.height_hint(80),
        0,
        "empty snapshot must produce 0 height"
    );
}

// ── I145 app-level layout tests using compress_layout (production helper) ──

#[test]
fn compress_layout_no_compression_when_fits() {
    let r = crate::scrollback::compress_layout(24, 5, 3, 4);
    assert_eq!(r.panel_max_height, 5);
    assert_eq!(r.queue_max_rows, 4);
    assert_eq!(r.input_max_height, 3);
}

#[test]
fn compress_layout_preserves_panel_then_composer_then_queue() {
    // Content budget 10: panel=5 and composer=3 are preserved; queue gets 2.
    let r = crate::scrollback::compress_layout(10, 5, 3, 6);
    assert_eq!(r.panel_max_height, 5);
    assert_eq!(r.input_max_height, 3);
    assert_eq!(r.queue_max_rows, 2);
}

#[test]
fn compress_layout_queue_zeroed_before_composer_is_reduced() {
    // Content budget 8: panel=5, composer=3, queue=0.
    let r = crate::scrollback::compress_layout(8, 5, 5, 6);
    assert_eq!(r.panel_max_height, 5);
    assert_eq!(r.queue_max_rows, 0);
    assert_eq!(r.input_max_height, 3);
}

#[test]
fn compress_layout_composer_never_below_one() {
    // Content budget 7: panel=5, queue=0, composer retains 2 rows.
    let r = crate::scrollback::compress_layout(7, 5, 10, 6);
    assert_eq!(r.panel_max_height, 5);
    assert_eq!(r.queue_max_rows, 0);
    assert_eq!(r.input_max_height, 2, "composer must not disappear");
}

#[test]
fn compress_layout_compresses_panel_only_after_reserving_composer() {
    // Content budget 1: no queue and no panel rows, but composer retains one row.
    let r = crate::scrollback::compress_layout(6, 5, 10, 6);
    assert_eq!(r.panel_max_height, 5);
    assert_eq!(r.queue_max_rows, 0);
    assert_eq!(r.input_max_height, 1);

    let r = crate::scrollback::compress_layout(1, 5, 10, 6);
    assert_eq!(r.panel_max_height, 0);
    assert_eq!(r.queue_max_rows, 0);
    assert_eq!(r.input_max_height, 1);
}

#[test]
fn compress_layout_zero_queue_natural() {
    let r = crate::scrollback::compress_layout(10, 5, 3, 0);
    assert_eq!(r.panel_max_height, 5);
    assert_eq!(r.queue_max_rows, 0);
    assert_eq!(r.input_max_height, 3);
}

// ── I145 ComponentStack layout verification ──────────────────────────────
//
// Construct real component stacks (same types as draw_frame) and verify
// ComponentStack::total_height respects compressed heights.

struct StubComponent {
    h: u16,
}

impl crate::inline_terminal::ViewportComponent for StubComponent {
    fn height_hint(&self, _w: u16) -> u16 {
        self.h
    }
    fn render(&self, _: &mut crate::inline_terminal::InlineFrame, _: ratatui::layout::Rect) {}
}

#[test]
fn stack_total_height_matches_sum_of_components() {
    let c1 = StubComponent { h: 3 };
    let c2 = StubComponent { h: 5 };
    let c3 = StubComponent { h: 2 };
    let stack = crate::inline_terminal::ComponentStack::new(vec![&c1, &c2, &c3]);
    assert_eq!(stack.total_height(80), 10);
}

#[test]
fn stack_total_height_with_zero_height_component() {
    let c1 = StubComponent { h: 3 };
    let c2 = StubComponent { h: 0 };
    let c3 = StubComponent { h: 2 };
    let stack = crate::inline_terminal::ComponentStack::new(vec![&c1, &c2, &c3]);
    // total_height sums ALL components including 0
    assert_eq!(stack.total_height(80), 5);
    // But layout() skips 0-height components
    let area = ratatui::layout::Rect::new(0, 0, 80, 24);
    let layout = stack.layout(area, 80);
    assert_eq!(
        layout.len(),
        2,
        "0-height component should be skipped in layout"
    );
}

#[test]
fn stack_layout_compressed_queue_and_composer_fit_screen() {
    // Screen=9, fixed=5, so the production content budget is 4.
    // Panel is closed, composer gets 4 rows, and queue is compressed to zero.
    let compressed = crate::scrollback::compress_layout(4, 0, 5, 4);
    assert_eq!(compressed.queue_max_rows, 0);
    assert_eq!(compressed.input_max_height, 4);

    // Build stack with compressed heights
    let modal = StubComponent { h: 0 };
    let fixed = StubComponent { h: 5 };
    let composer = StubComponent { h: 4 };
    let queue = StubComponent { h: 0 };
    let stack =
        crate::inline_terminal::ComponentStack::new(vec![&modal, &fixed, &composer, &queue]);
    let total = stack.total_height(80);
    assert!(
        total <= 9,
        "compressed stack total {} must fit screen height 9",
        total
    );
}

#[test]
fn stack_layout_composer_always_at_least_one() {
    // Screen=7, fixed=5, so the production content budget is 2.
    let compressed = crate::scrollback::compress_layout(2, 0, 10, 6);
    let composer_h = compressed.input_max_height;
    assert!(
        composer_h >= 1,
        "composer must be at least 1 row, got {}",
        composer_h
    );

    let fixed = StubComponent { h: 5 };
    let composer = StubComponent { h: composer_h };
    let queue = StubComponent {
        h: compressed.queue_max_rows,
    };
    let stack = crate::inline_terminal::ComponentStack::new(vec![&fixed, &composer, &queue]);
    let total = stack.total_height(80);
    assert!(total <= 7, "stack total {} must fit screen height 7", total);
}
