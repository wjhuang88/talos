use talos_conversation::ToolActivity;

#[derive(Default)]
pub(super) struct ToolActivities {
    calls: Vec<Call>,
}

struct Call {
    id: String,
    name: String,
    body: String,
    result: Option<bool>,
    layout: crate::scrollback::ToolActivityLayoutCache,
}

impl ToolActivities {
    pub(super) fn update(&mut self, event: ToolActivity) {
        match event {
            ToolActivity::ResponseStarted => self.clear(),
            ToolActivity::Requested {
                call_id,
                name,
                body,
            } => {
                if !self.calls.iter().any(|call| call.id == call_id) {
                    self.calls.push(Call {
                        id: call_id,
                        name,
                        body,
                        result: None,
                        layout: Default::default(),
                    });
                }
            }
            ToolActivity::Finished {
                call_id,
                is_error,
                body,
            } => {
                if let Some(call) = self.calls.iter_mut().find(|call| call.id == call_id)
                    && call.result.is_none()
                {
                    call.result = Some(is_error);
                    call.body = crate::scrollback::strip_llm_hints(&body);
                    call.layout = Default::default();
                }
            }
        }
    }

    pub(super) fn clear(&mut self) {
        self.calls.clear();
    }

    pub(super) fn has_activity(&self) -> bool {
        !self.calls.is_empty()
    }

    #[cfg(test)]
    fn preview(&self) -> Option<String> {
        if self.calls.is_empty() {
            return None;
        }
        Some(
            self.calls
                .iter()
                .enumerate()
                .map(|(index, call)| {
                    let state = match call.result {
                        None => "requested",
                        Some(false) => "succeeded",
                        Some(true) => "failed",
                    };
                    format!("{} #{} · {state}", call.name, index + 1)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }

    #[cfg(test)]
    fn component(&self) -> crate::scrollback::ToolActivityComponent<'_> {
        self.component_for_approval(false)
    }

    pub(super) fn component_for_approval(
        &self,
        approval_visible: bool,
    ) -> crate::scrollback::ToolActivityComponent<'_> {
        crate::scrollback::ToolActivityComponent {
            entries: self
                .calls
                .iter()
                .enumerate()
                .map(|(index, call)| {
                    let state = match call.result {
                        None => "requested",
                        Some(false) => "succeeded",
                        Some(true) => "failed",
                    };
                    (
                        format!("{} #{} · {state}", call.name, index + 1),
                        call.body.as_str(),
                        &call.layout,
                        call.result.is_some() || approval_visible,
                    )
                })
                .collect(),
            max_height: u16::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_collapses_requested_arguments_without_losing_call_state() {
        let mut calls = ToolActivities::default();
        for id in ["a", "b"] {
            calls.update(ToolActivity::Requested {
                call_id: id.into(),
                name: "bash".into(),
                body: "{\n  \"command\": \"pwd\"\n}".into(),
            });
        }
        let before = calls.component().plan(80);
        assert!(
            before
                .rows
                .iter()
                .any(|row| row.content.contains("command"))
        );
        let approval = calls.component_for_approval(true).plan(80);
        assert_eq!(approval.rows.len(), 2);
        assert!(approval.rows[0].content.contains("#1 · requested"));
        assert!(approval.rows[1].content.contains("#2 · requested"));
        assert!(!approval.rows.iter().any(|row| row.content.contains('{')));
        let restored = calls.component_for_approval(false).plan(80);
        assert_eq!(restored.rows.len(), before.rows.len());
        assert!(
            restored
                .rows
                .iter()
                .any(|row| row.content.contains("command"))
        );

        calls.update(ToolActivity::Finished {
            call_id: "b".into(),
            is_error: false,
            body: "real execution output".into(),
        });
        let approval = calls.component_for_approval(true).plan(80);
        assert!(approval.rows[0].content.contains("#1 · requested"));
        assert!(approval.rows[1].content.contains("#2 · succeeded"));
        assert_eq!(calls.calls[1].body, "real execution output");
        assert_eq!(calls.calls[0].result, None);
    }

    #[test]
    fn unchanged_body_wraps_once_across_frames_and_compression() {
        let mut calls = ToolActivities::default();
        calls.update(ToolActivity::Requested {
            call_id: "a".into(),
            name: "bash".into(),
            body: "中文abcdef\n".repeat(10_000),
        });
        for _ in 0..3 {
            let mut component = calls.component();
            let full = component.plan(60);
            assert_eq!(full.rows.len(), 11);
            assert!(full.rows[0].content.contains("10001 lines"));
            component.max_height = 3;
            assert_eq!(component.plan(60).rows.len(), 3);
        }
        assert_eq!(calls.calls[0].layout.builds(), 1);
        calls.component().plan(5);
        assert_eq!(calls.calls[0].layout.builds(), 2);
        calls.update(ToolActivity::Finished {
            call_id: "a".into(),
            is_error: false,
            body: "new result".into(),
        });
        let result = calls.component().plan(60);
        assert!(result.rows[0].content.contains("succeeded · 1 lines"));
        assert_eq!(result.rows.len(), 1);
        assert_eq!(calls.calls[0].layout.builds(), 1);
    }

    #[test]
    fn many_call_titles_stay_within_compressed_height() {
        let mut calls = ToolActivities::default();
        for n in 0..30 {
            calls.update(ToolActivity::Requested {
                call_id: n.to_string(),
                name: "bash".into(),
                body: "body".into(),
            });
        }
        for height in 0..12 {
            let mut component = calls.component();
            component.max_height = height;
            let plan = component.plan(60);
            assert_eq!(plan.rows.len(), usize::from(height));
            if height > 0 {
                assert!(
                    plan.rows
                        .last()
                        .expect("last title")
                        .content
                        .contains("#30")
                );
            }
        }
    }

    #[test]
    fn response_boundary_allows_reused_provider_call_ids() {
        let mut calls = ToolActivities::default();
        for body in ["first response", "second response"] {
            calls.update(ToolActivity::ResponseStarted);
            calls.update(ToolActivity::Requested {
                call_id: "call_0".into(),
                name: "bash".into(),
                body: body.into(),
            });
            assert_eq!(calls.calls.len(), 1);
            assert_eq!(calls.calls[0].body, body);
            assert_eq!(calls.calls[0].result, None);
            calls.update(ToolActivity::Finished {
                call_id: "call_0".into(),
                is_error: false,
                body: format!("result {body}"),
            });
            assert_eq!(calls.calls[0].body, format!("result {body}"));
        }
    }

    #[test]
    fn tool_bodies_reflow_and_roll_beneath_independent_titles() {
        let mut calls = ToolActivities::default();
        for id in ["a", "b"] {
            calls.update(ToolActivity::Requested {
                call_id: id.into(),
                name: "bash".into(),
                body: "中文中文".into(),
            });
        }
        let wide = calls.component().plan(60);
        assert_eq!(wide.rows.len(), 4);
        assert!(wide.rows[0].content.contains("#1 · requested · 1 lines"));
        assert!(wide.rows[2].content.contains("#2 · requested · 1 lines"));
        let narrow = calls.component().plan(5);
        assert_eq!(narrow.natural_height, 10);
        calls.update(ToolActivity::Finished {
            call_id: "b".into(),
            is_error: false,
            body: (0..20)
                .map(|n| format!("row{n}"))
                .collect::<Vec<_>>()
                .join("\n"),
        });
        let rolling = calls.component().plan(60);
        assert_eq!(rolling.rows.len(), 3);
        assert!(rolling.rows[0].content.contains("#1 · requested · 1 lines"));
        assert!(
            rolling.rows[2]
                .content
                .contains("#2 · succeeded · 20 lines")
        );
        assert!(!rolling.rows.iter().any(|row| row.content.contains("row19")));
        let mut tiny = calls.component();
        tiny.max_height = 1;
        assert!(tiny.plan(60).rows[0].content.contains("#2 · succeeded"));
        assert!(tiny.plan(0).rows.is_empty());
    }

    #[test]
    fn same_name_results_are_correlated_only_by_identity() {
        let mut calls = ToolActivities::default();
        for id in ["a", "b"] {
            calls.update(ToolActivity::Requested {
                call_id: id.into(),
                name: "bash".into(),
                body: "{}".into(),
            });
        }
        calls.update(ToolActivity::Finished {
            call_id: "unknown".into(),
            body: "ignored".into(),
            is_error: true,
        });
        calls.update(ToolActivity::Finished {
            call_id: "b".into(),
            body: "failed".into(),
            is_error: true,
        });
        assert_eq!(
            calls.preview().as_deref(),
            Some("bash #1 · requested\nbash #2 · failed")
        );
        calls.update(ToolActivity::Requested {
            call_id: "b".into(),
            name: "bash".into(),
            body: "{}".into(),
        });
        calls.update(ToolActivity::Finished {
            call_id: "a".into(),
            body: "ok".into(),
            is_error: false,
        });
        assert_eq!(
            calls.preview().as_deref(),
            Some("bash #1 · succeeded\nbash #2 · failed")
        );
        calls.clear();
        assert!(calls.preview().is_none());
    }

    #[test]
    fn result_body_filters_model_hints_and_ignores_duplicate_finish() {
        let mut calls = ToolActivities::default();
        calls.update(ToolActivity::Requested {
            call_id: "a".into(),
            name: "bash".into(),
            body: "{}".into(),
        });
        calls.update(ToolActivity::Finished {
            call_id: "a".into(),
            is_error: true,
            body: "error\n\n[Analyze the error above and try a different approach.]".into(),
        });
        calls.update(ToolActivity::Finished {
            call_id: "a".into(),
            is_error: false,
            body: "late duplicate".into(),
        });
        let plan = calls.component().plan(80);
        assert!(plan.rows[0].content.contains("failed · 1 lines"));
        assert_eq!(plan.rows.len(), 1);
    }
}
