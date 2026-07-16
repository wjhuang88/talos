use super::*;

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
    let action = state.accept_selected_panel_item();
    assert_eq!(action, crate::state::PanelAction::None);
    assert_eq!(state.input_buffer, "/model ");
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
    }
}

fn sample_model_picker_data() -> ModelPickerData {
    ModelPickerData {
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
fn model_picker_search_matching_provider_hides_other_groups() {
    let data = sample_model_picker_data();
    let menu = BottomPanelState::open_model_picker(&data);

    // Groups: "Current" (claude-sonnet-4-5/anthropic), "anthropic"
    // (claude-opus-4-1), "openai" (gpt-4o, o3).
    let indices = menu.filtered_indices("gpt");
    let visible_labels: Vec<&str> = indices
        .iter()
        .map(|&i| menu.items[i].label.as_str())
        .collect();

    assert!(
        visible_labels.contains(&"openai"),
        "openai header must be visible: {visible_labels:?}"
    );
    assert!(
        visible_labels.iter().any(|l| l.contains("gpt-4o")),
        "matching item must be visible: {visible_labels:?}"
    );
    assert!(
        !visible_labels.iter().any(|l| l.contains("o3")),
        "non-matching sibling must be hidden: {visible_labels:?}"
    );
    assert!(
        !visible_labels.contains(&"Current"),
        "non-matching Current group must be hidden: {visible_labels:?}"
    );
    assert!(
        !visible_labels.iter().any(|l| l.contains("claude")),
        "non-matching anthropic group must be hidden entirely: {visible_labels:?}"
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
fn model_picker_navigation_skips_headers_and_filtered_out_items() {
    let data = sample_model_picker_data();
    let mut menu = BottomPanelState::open_model_picker(&data);

    // Filter to only the "openai" group (gpt-4o, o3).
    menu.selected_index = menu
        .filtered_indices("openai")
        .into_iter()
        .find(|&i| menu.items[i].action != PanelItemAction::Header)
        .unwrap();

    let first_selection = menu.selected_index;
    assert_eq!(
        menu.items[first_selection].action != PanelItemAction::Header,
        true
    );

    menu.select_next("openai");
    assert_ne!(
        menu.items[menu.selected_index].action,
        PanelItemAction::Header,
        "select_next must never land on a Header"
    );
    assert_ne!(
        menu.selected_index, first_selection,
        "select_next must move within the filtered openai group"
    );

    // Navigating past the last item in the filtered set wraps back
    // without ever landing on a hidden (anthropic) item or a header.
    menu.select_next("openai");
    let after_wrap = menu.selected_index;
    assert!(
        menu.items[after_wrap].label.contains("gpt-4o")
            || menu.items[after_wrap].label.contains("o3"),
        "wrapped selection must stay within the openai group, got {:?}",
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
fn model_picker_enter_selects_correct_original_item_after_filtering() {
    let data = sample_model_picker_data();
    let mut menu = BottomPanelState::open_model_picker(&data);

    let target_idx = menu
        .items
        .iter()
        .position(
            |i| matches!(&i.action, PanelItemAction::Select { value, .. } if value == "gpt-4o"),
        )
        .expect("gpt-4o item must exist");
    menu.selected_index = target_idx;

    // selected_index must remain the correct raw index even though the
    // filtered/visible set has shrunk to a single group.
    let indices = menu.filtered_indices("gpt");
    assert!(indices.contains(&target_idx));

    let action = menu.items[menu.selected_index].action.clone();
    match action {
        PanelItemAction::Select { command, value } => {
            assert_eq!(command, "/model");
            assert_eq!(value, "gpt-4o");
        }
        other => panic!("expected Select action, got {other:?}"),
    }
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
    let mut menu = BottomPanelState::open_model_picker(&data);

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
