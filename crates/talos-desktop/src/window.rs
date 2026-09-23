use crate::input::TextInput;
use crate::presentation::{
    Capability, Locale, Page, Presentation, Preset, TASK_FIXTURES, TaskTab, Text, model_label,
};
use gpui::{
    App, Bounds, Context, FocusHandle, Focusable, KeyBinding, Window, WindowBounds, WindowOptions,
    actions, div, prelude::*, px, rgb, size,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

actions!(desktop, [NextFocus, PreviousFocus]);

#[derive(Default, Debug, PartialEq, Eq)]
enum LiveStatus {
    #[default]
    Ready,
    Connecting,
    Streaming,
    Cancelling,
    Closing,
    Finished,
    Cancelled,
    Stopped,
    MissingInput,
    WorkspaceChanged,
    HostBusy,
    CancelFailed,
    CloseFailed,
    Error(String),
}

impl LiveStatus {
    fn label(&self, locale: Locale) -> &str {
        let (en, zh) = match self {
            Self::Ready => ("Ready", "就绪"),
            Self::Connecting => ("Connecting", "正在连接"),
            Self::Streaming => ("Streaming", "正在输出"),
            Self::Cancelling => ("Cancelling", "正在取消"),
            Self::Closing => ("Stopping before closing", "正在停止，完成后关闭"),
            Self::Finished => ("Finished", "已完成"),
            Self::Cancelled => ("Cancelled", "已取消"),
            Self::Stopped => ("Stopped", "已停止"),
            Self::MissingInput => ("Enter a task and workspace", "请填写任务和工作区"),
            Self::WorkspaceChanged => (
                "Restart live mode to change workspace",
                "更换工作区请重新启动",
            ),
            Self::HostBusy => ("Host unavailable or busy", "执行服务未就绪或繁忙"),
            Self::CancelFailed => ("Cancellation could not be queued", "无法提交取消请求"),
            Self::CloseFailed => (
                "Could not request shutdown; retry closing",
                "关闭请求未提交，请重试",
            ),
            Self::Error(error) => return error,
        };
        match locale {
            Locale::English => en,
            Locale::Chinese => zh,
        }
    }
}

#[derive(Default)]
struct LiveTask {
    commands: Option<tokio::sync::mpsc::Sender<crate::runtime_host::RuntimeCommand>>,
    observer: Option<gpui::Task<()>>,
    output: String,
    status: LiveStatus,
    turn_id: Option<String>,
    unavailable_tools: Vec<String>,
    running: bool,
    closing: bool,
    workspace: Option<String>,
    session_external_id: Option<String>,
    resume_existing: bool,
    pending_approval: Option<(u64, String, String, String)>,
}

impl LiveTask {
    // The window stays alive until a successful Runtime shutdown result arrives.
    fn request_close(&mut self) -> bool {
        let Some(commands) = &self.commands else {
            return true;
        };
        if self.closing {
            return false;
        }
        match commands.try_send(crate::runtime_host::RuntimeCommand::Shutdown) {
            Ok(()) => {
                self.closing = true;
                self.status = LiveStatus::Closing;
            }
            Err(_) => {
                self.status = LiveStatus::CloseFailed;
            }
        }
        false
    }
}

const MODEL_CHOICES: [&str; 3] = ["fixture/default", "fixture/quick", "fixture/deep"];

struct DesktopAssets;

const ICONS: [(&str, &[u8]); 18] = [
    ("check", include_bytes!("../assets/icons/check.svg")),
    (
        "chevron-right",
        include_bytes!("../assets/icons/chevron-right.svg"),
    ),
    ("copy", include_bytes!("../assets/icons/copy.svg")),
    ("star", include_bytes!("../assets/icons/star.svg")),
    ("trash", include_bytes!("../assets/icons/trash.svg")),
    (
        "user-round",
        include_bytes!("../assets/icons/user-round.svg"),
    ),
    ("wrench", include_bytes!("../assets/icons/wrench.svg")),
    ("globe", include_bytes!("../assets/icons/globe.svg")),
    ("database", include_bytes!("../assets/icons/database.svg")),
    ("plus", include_bytes!("../assets/icons/plus.svg")),
    ("bookmark", include_bytes!("../assets/icons/bookmark.svg")),
    ("settings", include_bytes!("../assets/icons/settings.svg")),
    ("folder", include_bytes!("../assets/icons/folder.svg")),
    (
        "arrow-left",
        include_bytes!("../assets/icons/arrow-left.svg"),
    ),
    (
        "grip-vertical",
        include_bytes!("../assets/icons/grip-vertical.svg"),
    ),
    ("code", include_bytes!("../assets/icons/code.svg")),
    ("search", include_bytes!("../assets/icons/search.svg")),
    (
        "message-square",
        include_bytes!("../assets/icons/message-square.svg"),
    ),
];

impl gpui::AssetSource for DesktopAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        Ok(ICONS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| std::borrow::Cow::Borrowed(*bytes)))
    }

    fn list(&self, _: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(ICONS.iter().map(|(name, _)| (*name).into()).collect())
    }
}

fn icon(name: &'static str) -> gpui::Svg {
    gpui::svg()
        .path(name)
        .size(px(20.))
        .flex_shrink_0()
        .text_color(rgb(0x5e81ac))
}

fn selected_workspace_path(paths: Option<Vec<std::path::PathBuf>>) -> Result<Option<String>, ()> {
    let Some(paths) = paths else {
        return Ok(None);
    };
    if paths.len() != 1 {
        return Err(());
    }
    let path = paths[0]
        .to_str()
        .filter(|path| !path.is_empty())
        .ok_or(())?;
    Ok(Some(path.to_owned()))
}

#[derive(Clone)]
struct PresetDrag {
    preset: Preset,
    name: String,
}

impl Render for PresetDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(rgb(0xe8edf5))
            .text_color(rgb(0x2e4f82))
            .shadow_md()
            .child(self.name.clone())
    }
}

#[derive(Clone, Copy)]
enum Command {
    OpenLive,
    LiveLocale(Locale),
    SubmitLive,
    CancelLive,
    Tasks,
    OpenFixture(usize),
    NewTask,
    CreatePreview,
    SelectPreset(Preset),
    OpenPreview,
    Presets,
    BackToPresets,
    EditPreset(Preset),
    SavePreset,
    DefaultPreset(Preset),
    ToggleCapability(Capability),
    CopyPreset,
    DeletePreset,
    ConfirmDelete,
    CancelDelete,
    ChooseModel(usize, usize),
    ToggleEvidence,
    ViewChanges,
    NewPreset,
    Settings,
    ToggleTaskOption(usize),
    ResumeTask(usize),
    ChooseWorkspace,
    ApproveOnce(u64),
    ApproveSession(u64),
    DenyApproval(u64),
}

struct DesktopWindow {
    live_form_scroll: gpui::ScrollHandle,
    host_exits: std::rc::Rc<std::cell::RefCell<Vec<crate::runtime_host::HostExit>>>,
    live: Option<LiveTask>,
    state: Presentation,
    input: gpui::Entity<TextInput>,
    goal_input: gpui::Entity<TextInput>,
    workspace_input: gpui::Entity<TextInput>,
    preset_fields: [gpui::Entity<TextInput>; 6],
    root_focus: FocusHandle,
    english_focus: FocusHandle,
    chinese_focus: FocusHandle,
    tab_focus: [FocusHandle; 5],
    model_focus: [FocusHandle; 3],
    model_choice_focus: [FocusHandle; 9],
    open_model: Option<usize>,
    preset_open: bool,
    restore_preset_focus: bool,
    preset_focus: FocusHandle,
    preset_trigger_bounds: std::rc::Rc<std::cell::RefCell<Bounds<gpui::Pixels>>>,
    evidence_open: bool,
    settings_focus: FocusHandle,
    settings_scroll: gpui::ScrollHandle,
    preset_scroll: gpui::ScrollHandle,
    preset_menu_scroll: gpui::ScrollHandle,
    #[cfg(feature = "visual-test")]
    preset_menu_bounds: std::rc::Rc<std::cell::RefCell<Vec<Bounds<gpui::Pixels>>>>,
    #[cfg(feature = "visual-test")]
    delete_bounds: std::rc::Rc<std::cell::RefCell<[Bounds<gpui::Pixels>; 3]>>,
    #[cfg(feature = "visual-test")]
    preset_edit_bounds: std::rc::Rc<std::cell::RefCell<Bounds<gpui::Pixels>>>,
    save_focus: FocusHandle,
    task_option_focus: [FocusHandle; 2],
    task_scroll: gpui::ScrollHandle,
    workspace_prompt: Option<gpui::Task<()>>,
    workspace_prompt_failed: bool,
    workspace_prompt_stale: bool,
    durable_task_ids: Vec<String>,
    durable_task_error: Option<String>,
    durable_task_workspace: Option<String>,
    #[cfg(feature = "visual-test")]
    drag_bounds:
        std::rc::Rc<std::cell::RefCell<std::collections::HashMap<usize, Bounds<gpui::Pixels>>>>,
}

impl DesktopWindow {
    fn new(locale: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root_focus = cx.focus_handle();
        window.focus(&root_focus, cx);
        Self {
            live_form_scroll: gpui::ScrollHandle::new(),
            host_exits: Default::default(),
            live: None,
            model_focus: std::array::from_fn(|_| cx.focus_handle().tab_stop(true)),
            model_choice_focus: std::array::from_fn(|_| cx.focus_handle().tab_stop(true)),
            open_model: None,
            preset_open: false,
            restore_preset_focus: false,
            preset_focus: cx.focus_handle().tab_stop(true),
            preset_trigger_bounds: Default::default(),
            evidence_open: false,
            settings_focus: cx.focus_handle().tab_stop(true),
            settings_scroll: gpui::ScrollHandle::new(),
            preset_scroll: gpui::ScrollHandle::new(),
            preset_menu_scroll: gpui::ScrollHandle::new(),
            #[cfg(feature = "visual-test")]
            preset_menu_bounds: Default::default(),
            #[cfg(feature = "visual-test")]
            delete_bounds: Default::default(),
            #[cfg(feature = "visual-test")]
            preset_edit_bounds: Default::default(),
            save_focus: cx.focus_handle().tab_stop(true),
            task_option_focus: std::array::from_fn(|_| cx.focus_handle().tab_stop(true)),
            task_scroll: gpui::ScrollHandle::new(),
            workspace_prompt: None,
            workspace_prompt_failed: false,
            workspace_prompt_stale: false,
            durable_task_ids: Vec::new(),
            durable_task_error: None,
            durable_task_workspace: None,
            #[cfg(feature = "visual-test")]
            drag_bounds: Default::default(),
            state: Presentation::new(locale),
            input: cx.new(|cx| TextInput::new(window, cx)),
            goal_input: cx.new(|cx| {
                let mut input = TextInput::new(window, cx).multiline();
                input.label = Text::Goal;
                input
            }),
            workspace_input: cx.new(|cx| {
                let mut input = TextInput::new(window, cx);
                input.label = Text::Workspace;
                input
            }),
            root_focus,
            preset_fields: [
                Text::Name,
                Text::Description,
                Text::Instructions,
                Text::DefaultModel,
                Text::QuickModel,
                Text::DeepModel,
            ]
            .map(|label| {
                cx.new(|cx| {
                    let mut input = TextInput::new(window, cx);
                    if matches!(label, Text::Instructions) {
                        input = input.multiline();
                    } else {
                        input.compact = true;
                    }
                    input.label = label;
                    input
                })
            }),
            english_focus: cx.focus_handle().tab_index(1).tab_stop(true),
            chinese_focus: cx.focus_handle().tab_index(2).tab_stop(true),
            tab_focus: std::array::from_fn(|index| {
                cx.focus_handle()
                    .tab_index(index as isize + 4)
                    .tab_stop(true)
            }),
        }
    }

    fn execute(&mut self, command: Command, window: &mut Window, cx: &mut Context<Self>) {
        let previous_page = self.state.page;
        match command {
            Command::OpenLive => self.state.page = Page::NewTask,
            Command::LiveLocale(locale) => self.select_language(locale, window, cx),
            Command::SubmitLive => self.submit_live(cx),
            Command::CancelLive => {
                if let Some(live) = &mut self.live
                    && let Some(commands) = &live.commands
                {
                    if commands
                        .try_send(crate::runtime_host::RuntimeCommand::Interrupt)
                        .is_ok()
                    {
                        live.status = LiveStatus::Cancelling;
                        live.pending_approval = None;
                    } else {
                        live.status = LiveStatus::CancelFailed;
                    }
                }
            }
            Command::ApproveOnce(request_id)
            | Command::ApproveSession(request_id)
            | Command::DenyApproval(request_id) => {
                if let Some(live) = &mut self.live
                    && live.running
                    && !live.closing
                    && live
                        .pending_approval
                        .as_ref()
                        .is_some_and(|approval| approval.0 == request_id)
                    && let Some(commands) = &live.commands
                {
                    let choice = match command {
                        Command::ApproveOnce(_) => talos_runtime::ApprovalChoice::ApproveOnce,
                        Command::ApproveSession(_) => talos_runtime::ApprovalChoice::AlwaysApprove,
                        Command::DenyApproval(_) => talos_runtime::ApprovalChoice::Deny,
                        _ => unreachable!(),
                    };
                    if commands
                        .try_send(crate::runtime_host::RuntimeCommand::ApprovalResponse {
                            request_id,
                            choice,
                        })
                        .is_ok()
                    {
                        live.pending_approval = None;
                    }
                }
            }
            Command::Tasks => {
                self.state.page = Page::Tasks;
                self.durable_task_error = None;
                self.durable_task_ids.clear();
                let workspace = self.workspace_input.read(cx).text().trim().to_owned();
                let workspace = if workspace.is_empty() {
                    std::env::current_dir()
                        .ok()
                        .and_then(|path| path.to_str().map(str::to_owned))
                        .unwrap_or_default()
                } else {
                    workspace
                };
                self.durable_task_workspace = Some(workspace.clone());
                cx.spawn(async move |this, cx| {
                    let result = cx
                        .background_spawn(async move {
                            crate::runtime_host::list_task_external_ids(std::path::Path::new(
                                &workspace,
                            ))
                        })
                        .await;
                    let _ = this.update(cx, |this, cx| {
                        match result {
                            Ok(ids) => this.durable_task_ids = ids,
                            Err(error) => this.durable_task_error = Some(error),
                        }
                        cx.notify();
                    });
                })
                .detach();
            }
            Command::OpenFixture(index) => {
                if self.state.open_fixture(index) {
                    self.tab_focus[0].focus(window, cx);
                }
            }
            Command::ChooseWorkspace => {
                if self.workspace_prompt.is_some() {
                    return;
                }
                self.workspace_prompt_failed = false;
                self.workspace_prompt_stale = false;
                let original_path = self.workspace_input.read(cx).text().to_owned();
                let prompt = catch_unwind(AssertUnwindSafe(|| {
                    cx.prompt_for_paths(gpui::PathPromptOptions {
                        files: false,
                        directories: true,
                        multiple: false,
                        prompt: Some(self.state.locale.text(Text::ChooseWorkspace).into()),
                    })
                }));
                match prompt {
                    Ok(prompt) => {
                        self.workspace_prompt = Some(cx.spawn(async move |this, cx| {
                            let result = prompt.await;
                            let _ = this.update(cx, |this, cx| {
                                this.workspace_prompt = None;
                                if this.workspace_prompt_stale
                                    || this.state.page != Page::NewTask
                                    || this.workspace_input.read(cx).text() != original_path
                                {
                                    return;
                                }
                                let selection = match result {
                                    Ok(Ok(paths)) => selected_workspace_path(paths),
                                    _ => Err(()),
                                };
                                match selection {
                                    Ok(None) => {}
                                    Ok(Some(path)) => this
                                        .workspace_input
                                        .update(cx, |input, cx| input.set_text(&path, cx)),
                                    _ => this.workspace_prompt_failed = true,
                                }
                                cx.notify();
                            });
                        }));
                    }
                    Err(_) => self.workspace_prompt_failed = true,
                }
            }
            Command::ToggleTaskOption(index) => {
                if let Some(value) = self.state.task_options.get_mut(index) {
                    *value = !*value;
                }
            }
            Command::ResumeTask(index) => {
                let Some(external_id) = self.durable_task_ids.get(index).cloned() else {
                    return;
                };
                let Some(workspace) = self.durable_task_workspace.clone() else {
                    return;
                };
                self.workspace_input
                    .update(cx, |input, cx| input.set_text(&workspace, cx));
                self.goal_input
                    .update(cx, |input, cx| input.set_text("", cx));
                self.live = Some(LiveTask {
                    session_external_id: Some(external_id),
                    resume_existing: true,
                    workspace: Some(workspace.clone()),
                    ..LiveTask::default()
                });
                self.state.page = Page::NewTask;
                self.start_live_host(workspace, cx);
            }
            Command::Settings => {
                self.state.page = Page::Presets;
                window.focus(&self.root_focus, cx);
            }
            Command::NewPreset => {
                self.state.begin_preset();
                for (index, field) in self.preset_fields.iter().enumerate() {
                    let value = index
                        .checked_sub(3)
                        .and_then(|role| MODEL_CHOICES.get(role))
                        .copied()
                        .unwrap_or("");
                    field.update(cx, |input, cx| input.set_text(value, cx));
                }
            }
            Command::ToggleEvidence => self.evidence_open = !self.evidence_open,
            Command::ViewChanges => {
                self.state.tab = TaskTab::Changes;
                self.tab_focus[2].focus(window, cx);
            }
            Command::ChooseModel(role, choice) => {
                if let (Some(field), Some(value), Some(focus)) = (
                    self.preset_fields.get(role + 3),
                    MODEL_CHOICES.get(choice),
                    self.model_focus.get(role),
                ) {
                    field.update(cx, |input, cx| input.set_text(value, cx));
                    self.open_model = None;
                    window.focus(focus, cx);
                }
            }
            Command::CopyPreset => {
                let preset = self.state.duplicate_preset();
                self.execute(Command::EditPreset(preset), window, cx);
            }
            Command::DeletePreset => self.state.request_delete(),
            Command::ConfirmDelete => {
                self.state.confirm_delete();
            }
            Command::CancelDelete => self.state.pending_delete = None,
            Command::ToggleCapability(capability) => {
                let value = &mut self.state.draft_capabilities[capability as usize];
                *value = !*value;
            }
            Command::DefaultPreset(preset) => {
                self.state.select_preset(preset, true);
            }
            Command::Presets | Command::BackToPresets => {
                self.state.cancel_preset();
            }
            Command::EditPreset(preset) => {
                self.open_model = None;
                if !self.state.edit_preset(preset) {
                    return;
                }
                let template = &self.state.templates[preset.index()];
                let values = [
                    &template.name,
                    &template.description,
                    &template.instructions,
                    &template.models[0],
                    &template.models[1],
                    &template.models[2],
                ];
                for (field, value) in self.preset_fields.iter().zip(values) {
                    field.update(cx, |input, cx| input.set_text(value, cx));
                }
            }
            Command::SavePreset => {
                let values = self
                    .preset_fields
                    .each_ref()
                    .map(|field| field.read(cx).text().to_owned());
                let [name, description, instructions, default, quick, deep] = values;
                self.state.save_preset(crate::presentation::PresetTemplate {
                    name,
                    description,
                    instructions,
                    models: [default, quick, deep],
                    capabilities: self.state.draft_capabilities,
                });
            }
            Command::NewTask => self.state.begin_task(),
            Command::SelectPreset(preset) => {
                self.state.select_preset(preset, false);
                self.preset_open = false;
                window.focus(&self.preset_focus, cx);
            }
            Command::OpenPreview => self.state.page = Page::Preview,
            Command::CreatePreview => {
                let goal = self.goal_input.read(cx).text().to_owned();
                let workspace = self.workspace_input.read(cx).text().to_owned();
                self.state.create_preview(&goal, &workspace);
            }
        }
        if self.state.page != previous_page
            || matches!(
                command,
                Command::CopyPreset
                    | Command::DeletePreset
                    | Command::CancelDelete
                    | Command::NewTask
                    | Command::Tasks
                    | Command::OpenFixture(_)
                    | Command::Presets
                    | Command::BackToPresets
                    | Command::OpenPreview
            )
        {
            self.workspace_prompt_stale = true;
            self.open_model = None;
            self.preset_open = false;
            let focus = match self.state.page {
                Page::Fixture => self.tab_focus[0].clone(),
                Page::NewTask => self.goal_input.read(cx).focus_handle(cx),
                Page::PresetDetail => self.preset_fields[0].read(cx).focus_handle(cx),
                _ => self.root_focus.clone(),
            };
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    fn select_language(&mut self, locale: Locale, window: &mut Window, cx: &mut Context<Self>) {
        let (locale_id, focus) = match locale {
            Locale::English => ("en-US", &self.english_focus),
            Locale::Chinese => ("zh-CN", &self.chinese_focus),
        };
        self.state.set_locale(locale_id);
        window.focus(focus, cx);
        cx.notify();
    }

    fn command(
        &self,
        id: impl Into<gpui::ElementId>,
        label: impl Into<gpui::SharedString>,
        command: Command,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let label = label.into();
        let control = div();
        #[cfg(feature = "visual-test")]
        let control = if let Some(index) = match command {
            Command::DeletePreset => Some(0),
            Command::CancelDelete => Some(1),
            Command::ConfirmDelete => Some(2),
            _ => None,
        } {
            let bounds = self.delete_bounds.clone();
            control.on_children_prepainted(move |children, _, _| {
                if let Some(child) = children.first() {
                    bounds.borrow_mut()[index] = *child;
                }
            })
        } else {
            control
        };
        control
            .id(id)
            .role(if matches!(command, Command::ToggleTaskOption(_)) {
                gpui::Role::CheckBox
            } else if matches!(command, Command::SelectPreset(_) | Command::ChooseModel(_, _) | Command::LiveLocale(_)) {
                gpui::Role::RadioButton
            } else {
                gpui::Role::Button
            })
            .when(matches!(command, Command::SelectPreset(_)), |control| {
                let selected =
                    matches!(command, Command::SelectPreset(preset) if preset == self.state.preset);
                control.aria_toggled(if selected {
                    gpui::accesskit::Toggled::True
                } else {
                    gpui::accesskit::Toggled::False
                })
            })
            .when(matches!(command, Command::ChooseModel(_, _)), |control| {
                let selected = match command {
                    Command::ChooseModel(role, choice) => self.preset_fields[role + 3].read(cx).text() == MODEL_CHOICES[choice],
                    _ => false,
                };
                control.aria_toggled(if selected { gpui::Toggled::True } else { gpui::Toggled::False })
            })
            .aria_label(label.clone())
            .when(matches!(command, Command::ToggleTaskOption(_)), |control| {
                let selected = matches!(command, Command::ToggleTaskOption(index) if self.state.task_options[index]);
                control.aria_toggled(if selected { gpui::Toggled::True } else { gpui::Toggled::False })
                    .child(div().size(px(16.)).flex_shrink_0().border_1().rounded_sm()
                        .border_color(rgb(0x5e81ac)).flex().items_center().justify_center()
                        .when(selected, |checkbox| checkbox.bg(rgb(0x5e81ac))
                            .child(icon("check").size(px(14.)).text_color(rgb(0xffffff)))))
            })
            .when(matches!(command, Command::DefaultPreset(_)), |control| {
                control.aria_toggled(if matches!(command, Command::DefaultPreset(preset) if preset == self.state.default_preset) {
                    gpui::Toggled::True
                } else { gpui::Toggled::False })
            })
            .when(matches!(command, Command::ToggleEvidence), |control| {
                control.aria_expanded(self.evidence_open)
            })
            .focusable()
            .tab_stop(true)
            .when(matches!(command, Command::LiveLocale(_)), |control| {
                let choice = match command { Command::LiveLocale(choice) => choice, _ => self.state.locale };
                control.track_focus(match choice { Locale::English => &self.english_focus, Locale::Chinese => &self.chinese_focus })
                    .aria_toggled(if choice == self.state.locale { gpui::Toggled::True } else { gpui::Toggled::False })
                    .when(choice == self.state.locale, |item| item.bg(rgb(0xddeef8)))
            })
            .flex().items_center().gap_2()
            .when(matches!(command, Command::OpenFixture(0)), |button| button.child(div().size(px(12.)).flex_shrink_0().rounded_full().bg(rgb(0x487bc3))))
            .when_some(match command {
                Command::NewTask | Command::NewPreset => Some("plus"),
                Command::Presets => Some("bookmark"),
                Command::BackToPresets => Some("arrow-left"),
                Command::Settings => Some("settings"),
                Command::ChooseWorkspace => Some("folder"),
                Command::OpenFixture(1) => Some("wrench"),
                Command::OpenFixture(2) => Some("globe"),
                Command::OpenFixture(3) => Some("database"),
                Command::CopyPreset => Some("copy"),
                Command::DefaultPreset(_) => Some("star"),
                Command::SelectPreset(Preset::Coding) => Some("code"),
                Command::SelectPreset(Preset::Research) => Some("search"),
                Command::SelectPreset(_) => Some("message-square"),
                Command::DeletePreset | Command::ConfirmDelete => Some("trash"),
                _ => None,
            }, |button, name| button.child(icon(name).text_color(rgb(if matches!(command, Command::NewTask | Command::NewPreset) { 0xffffff } else if matches!(command, Command::DeletePreset | Command::ConfirmDelete) { 0xbf616a } else { 0x5e81ac }))))
            .when(matches!(command, Command::ToggleTaskOption(_)), |control| {
                if let Command::ToggleTaskOption(index) = command {
                    control.track_focus(&self.task_option_focus[index])
                } else { control }
            })
            .when(matches!(command, Command::SavePreset), |control| control.track_focus(&self.save_focus))
            .when(matches!(command, Command::ChooseModel(_, _)), |control| {
                if let Command::ChooseModel(role, choice) = command {
                    control.track_focus(&self.model_choice_focus[role * 3 + choice])
                } else { control }
            })
            .when(matches!(command, Command::Settings), |control| {
                control.track_focus(&self.settings_focus)
            })
            .px_3()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .text_color(rgb(0x5e81ac))
            .when(matches!(command, Command::OpenFixture(_)), |button| button.text_color(rgb(0x4c566a)))
            .when(matches!(command, Command::OpenFixture(index) if index == self.state.active_fixture && self.state.page == Page::Fixture), |button| button.bg(rgb(0xe5e9f0)).text_color(rgb(0x2e3440)))
            .hover(move |style| {
                style.bg(rgb(
                    if matches!(
                        command,
                        Command::NewTask | Command::NewPreset | Command::CreatePreview | Command::SavePreset
                    ) {
                        0x507299
                    } else {
                        0xeceff4
                    },
                ))
            })
            .when(
                matches!(
                    command,
                    Command::NewTask | Command::NewPreset | Command::CreatePreview | Command::SavePreset
                ),
                |button| button.bg(rgb(0x5e81ac)).text_color(rgb(0xffffff)),
            )
            .when(
                matches!(command, Command::DeletePreset | Command::ConfirmDelete),
                |button| button.text_color(rgb(0xbf616a)),
            )
            .when(
                matches!(command, Command::Settings)
                    && matches!(self.state.page, Page::Presets | Page::PresetDetail),
                |button| button.bg(rgb(0xe8edf5)).text_color(rgb(0x2e4f82)),
            )
            .focus(|style| style.bg(rgb(0xd8dee9)))
            .child(label)
            .on_click(cx.listener(move |this, _, window, cx| this.execute(command, window, cx)))
            .on_key_down(
                cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        this.execute(command, window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
    }

    fn preset_picker(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let template = &self.state.templates[self.state.preset.index()];
        let trigger_bounds = self.preset_trigger_bounds.clone();
        div()
            .on_children_prepainted(move |children, _, _| {
                if let Some(trigger) = children.first() {
                    *trigger_bounds.borrow_mut() = *trigger;
                }
            })
            .relative()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("preset-trigger")
                    .role(gpui::Role::Button)
                    .aria_label(format!(
                        "{}: {}",
                        self.state.locale.text(Text::Preset),
                        template.name
                    ))
                    .aria_expanded(self.preset_open)
                    .track_focus(&self.preset_focus)
                    .p_4()
                    .border_1()
                    .border_color(rgb(0xd8dee9))
                    .rounded_md()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .cursor_pointer()
                    .focus(|style| style.border_color(rgb(0x5e81ac)))
                    .child(icon(match self.state.preset {
                        Preset::Coding => "code",
                        Preset::Research => "search",
                        _ => "message-square",
                    }))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(template.name.clone())
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x68758c))
                                    .child(template.description.clone()),
                            ),
                    )
                    .child("⌄")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.preset_open = !this.preset_open;
                        window.focus(&this.preset_focus, cx);
                        cx.notify();
                    }))
                    .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            this.preset_open = !this.preset_open;
                            cx.stop_propagation();
                            cx.notify();
                        }
                    })),
            )
            .when(self.preset_open, |picker| {
                picker.child(
                    gpui::deferred(
                        gpui::anchored().snap_to_window_with_margin(px(12.)).child(
                            div()
                                .id("preset-options")
                                .track_scroll(&self.preset_menu_scroll)
                                .occlude()
                                .w(px(300.))
                                .max_h((window.viewport_size().height - px(24.)).min(px(480.)))
                                .overflow_y_scroll()
                                .p_2()
                                .bg(rgb(0xffffff))
                                .border_1()
                                .border_color(rgb(0xd8dee9))
                                .rounded_md()
                                .shadow_lg()
                                .flex()
                                .flex_col()
                                .on_mouse_down_out(cx.listener(
                                    |this, event: &gpui::MouseDownEvent, window, cx| {
                                        if this
                                            .preset_trigger_bounds
                                            .borrow()
                                            .contains(&event.position)
                                        {
                                            return;
                                        }
                                        this.preset_open = false;
                                        this.restore_preset_focus = true;
                                        window.focus(&this.preset_focus, cx);
                                        cx.notify();
                                    },
                                ))
                                .children(self.state.presets.iter().copied().map(|preset| {
                                    let template = &self.state.templates[preset.index()];
                                    let row = div();
                                    #[cfg(feature = "visual-test")]
                                    let row = {
                                        let bounds = self.preset_menu_bounds.clone();
                                        row.on_children_prepainted(move |children, _, _| {
                                            if let Some(button) = children.first() {
                                                let mut bounds = bounds.borrow_mut();
                                                if bounds.len() <= preset.index() {
                                                    bounds.resize(
                                                        preset.index() + 1,
                                                        Bounds::default(),
                                                    );
                                                }
                                                bounds[preset.index()] = *button;
                                            }
                                        })
                                    };
                                    row.flex_shrink_0()
                                        .flex()
                                        .flex_col()
                                        .px_1()
                                        .py_1()
                                        .child(self.command(
                                            ("select-preset", preset.index()),
                                            format!(
                                                "{}{}",
                                                if self.state.preset == preset {
                                                    "✓  "
                                                } else {
                                                    ""
                                                },
                                                template.name
                                            ),
                                            Command::SelectPreset(preset),
                                            cx,
                                        ))
                                        .child(
                                            div()
                                                .pl(px(40.))
                                                .pr_2()
                                                .pb_1()
                                                .text_sm()
                                                .text_color(rgb(0x68758c))
                                                .child(template.description.clone()),
                                        )
                                }))
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .border_t_1()
                                        .border_color(rgb(0xd8dee9))
                                        .mt_2()
                                        .child(self.command(
                                            "manage-presets",
                                            self.state.locale.text(Text::Presets),
                                            Command::Presets,
                                            cx,
                                        )),
                                ),
                        ),
                    )
                    .priority(1),
                )
            })
    }

    fn model_picker(&self, role: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let field = self.preset_fields[role + 3].read(cx);
        let label = self.state.locale.text(field.label);
        let current = field.text().to_owned();
        let expanded = self.open_model == Some(role);
        div()
            .id(("model-picker", role))
            .flex()
            .flex_col()
            .child(
                div()
                    .id(("model-trigger", role))
                    .role(gpui::Role::Button)
                    .aria_label(format!(
                        "{label}: {}",
                        model_label(&current, self.state.locale)
                    ))
                    .aria_expanded(expanded)
                    .track_focus(&self.model_focus[role])
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_4()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(0xe1e6ef))
                    .cursor_pointer()
                    .focus(|style| style.bg(rgb(0xe8edf5)))
                    .child(label)
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .text_color(rgb(0x5e6f8d))
                            .child(model_label(&current, self.state.locale).to_owned())
                            .child(if expanded { "⌄" } else { "›" }),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_model = if expanded { None } else { Some(role) };
                        window.focus(&this.model_focus[role], cx);
                        cx.notify();
                    }))
                    .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            this.open_model = if expanded { None } else { Some(role) };
                            cx.stop_propagation();
                            cx.notify();
                        }
                    })),
            )
            .when(expanded, |picker| {
                picker.child(
                    div()
                        .id(("model-options", role))
                        .flex()
                        .flex_col()
                        .bg(rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(0xd8dee9))
                        .children(
                            MODEL_CHOICES
                                .into_iter()
                                .enumerate()
                                .map(|(choice, value)| {
                                    self.command(
                                        ("model-choice", role * 3 + choice),
                                        format!(
                                            "{} {}",
                                            if current == value { "✓" } else { " " },
                                            model_label(value, self.state.locale)
                                        ),
                                        Command::ChooseModel(role, choice),
                                        cx,
                                    )
                                    .into_any_element()
                                }),
                        ),
                )
            })
    }
}

impl Render for DesktopWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let locale = self.state.locale;
        let compact = window.viewport_size().width < px(900.);
        self.input.update(cx, |input, cx| {
            if input.locale != locale {
                input.locale = locale;
                cx.notify();
            }
        });
        let fixture = self.state.fixture();
        let durable_task_error = self.durable_task_error.clone();
        for field in [&self.goal_input, &self.workspace_input]
            .into_iter()
            .chain(self.preset_fields.iter())
        {
            field.update(cx, |input, cx| {
                if input.locale != locale {
                    input.locale = locale;
                    cx.notify();
                }
            });
        }
        if self.live.is_some() {
            return self.render_live(window, cx).into_any_element();
        }
        div()
            .id("desktop-root")
            .role(gpui::Role::Application)
            .aria_label(locale.text(Text::Title))
            .track_focus(&self.root_focus)
            .on_any_mouse_down(cx.listener(|this, _, window, cx| {
                if std::mem::take(&mut this.restore_preset_focus) && !window.default_prevented() {
                    // Child controls retain their own click focus; blank space returns to the picker.
                    window.focus(&this.preset_focus, cx);
                    window.prevent_default();
                }
            }))
            .on_action(cx.listener(|_, _: &NextFocus, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &PreviousFocus, window, cx| window.focus_prev(cx)))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    if this.preset_open {
                        this.preset_open = false;
                        window.focus(&this.preset_focus, cx);
                        cx.stop_propagation();
                        cx.notify();
                    } else if let Some(role) = this.open_model.take() {
                        window.focus(&this.model_focus[role], cx);
                        cx.stop_propagation();
                        cx.notify();
                    }
                }
            }))
            .size_full()
            .flex()
            .when(compact, |root| root.flex_col())
            .bg(rgb(0xf8f9fc))
            .text_color(rgb(0x2e3440))
            .text_size(px(15.))
            .child(
                div()
                    .id("global-navigation")
                    .role(gpui::Role::Navigation)
                    .flex_shrink_0()
                    .flex()
                    .when(compact, |nav| nav.w_full().flex_wrap().items_center().gap_2().p_2().border_b_1())
                    .when(!compact, |nav| nav.w(px(274.)).h_full().flex_col().gap_6().p_6().border_r_1())
                    .bg(rgb(0xf3f5fa))
                    .border_color(rgb(0xd8dee9))
                    .when(!compact, |nav| nav.child(div().flex().items_center().gap_3()
                        .child(div().size(px(40.)).rounded_md().bg(rgb(0x2e4f82)).text_color(rgb(0xffffff)).flex().items_center().justify_center().text_xl().child("T"))
                        .child(div().text_size(px(17.)).child(gpui::text!("Talos Desktop")))))
                    .child(self.command(
                        "new-task",
                        locale.text(Text::NewTask),
                        Command::NewTask,
                        cx,
                    ))
                    .when(!compact, |nav| nav.child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x4c566a))
                            .child(gpui::text!(locale.text(Text::RecentTasks))),
                    ))
                    .when(compact, |nav| nav.child(self.command("recent-fixture-task", locale.text(Text::RecentTasks), Command::Tasks, cx)))
                    .when(!compact, |nav| nav.child(div().flex().flex_col().gap_2().children(
                        TASK_FIXTURES.iter().enumerate().map(|(index, task)| self.command(("recent-task", index), task.title.text(locale), Command::OpenFixture(index), cx).into_any_element())
                    )))
                    .when(!compact, |nav| nav.child(self.command("all-tasks", locale.text(Text::AllTasks), Command::Tasks, cx)))
                    .when(self.state.preview.is_some(), |nav| {
                        nav.child(self.command(
                            "open-preview",
                            locale.text(Text::PreviewOnly),
                            Command::OpenPreview,
                            cx,
                        ))
                    })
                    .when(!compact, |nav| nav.child(div().flex_1()))
                    .child(div().when(!compact, |footer| footer.border_t_1().border_color(rgb(0xd8dee9)).pt_4())
                        .child(self.command("settings", locale.text(Text::Settings), Command::Settings, cx)))
                    .when(!compact, |nav| nav.child(
                        div()
                            .flex().items_center().gap_3()
                            .text_sm()
                            .text_color(rgb(0x4c566a))
                            .child(div().size(px(36.)).rounded_full().bg(rgb(0xe5e9f0)).flex().items_center().justify_center().child(icon("user-round")))
                            .child(locale.text(Text::LocalUser)),
                    )),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .when(!compact, |content| content.h_full())
                    .relative()
                    .flex()
                    .flex_col()
                    .when(self.state.page == Page::Fixture, |root| root.child(
                        div().px_6().py_4().text_xl().child(fixture.title.text(locale))
                    ))
                    .when(self.state.page == Page::Tasks, |root| root.child(
                        div().id("task-list").flex_1().min_h_0().overflow_y_scroll().p_6().flex().flex_col().gap_4()
                            .child(div().text_xl().child(locale.text(Text::RecentTasks)))
                            .when(durable_task_error.is_some(), |list| list.child(
                                div().text_sm().text_color(rgb(0xbf616a)).child(
                                    durable_task_error
                                        .unwrap_or_else(|| "Unable to load durable tasks".into()),
                                ),
                            ))
                            .when(!self.durable_task_ids.is_empty(), |list| list.child(
                                div().text_sm().text_color(rgb(0x5e6f8d)).child("Saved durable sessions"),
                            ).children(self.durable_task_ids.iter().enumerate().map(|(index, external_id)|
                                {
                                    let label = if external_id.starts_with("desktop-task-v1-") {
                                        external_id.clone()
                                    } else if locale == Locale::Chinese {
                                        "旧版已保存会话".to_owned()
                                    } else {
                                        "Legacy saved session".to_owned()
                                    };
                                    div().flex().items_center().justify_between().gap_3().py_2().border_b_1().border_color(rgb(0xd8dee9))
                                        .child(div().flex_1().min_w_0().text_sm().child(label))
                                        .child(self.command(("resume-task", index), "Resume", Command::ResumeTask(index), cx))
                                }
                            )))
                            .children(TASK_FIXTURES.iter().enumerate().map(|(index, task)|
                                div().flex().flex_col().gap_2().py_3().border_b_1().border_color(rgb(0xd8dee9))
                                    .child(self.command(("open-task", index), task.title.text(locale), Command::OpenFixture(index), cx))
                                    .child(div().px_3().text_color(rgb(0x68758c)).child(task.goal.text(locale)))
                            ))
                    ))
                    .when(self.state.page == Page::NewTask, |root| {
                        root.child(
                            div()
                                .id("new-task-form")
                                .track_scroll(&self.task_scroll)
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .p(px(if window.viewport_size().width < px(900.) { 24. } else { 48. }))
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xl()
                                        .child(gpui::text!(locale.text(Text::NewTask))),
                                )
                                .child(div().w_full().max_w(px(768.)).flex().flex_col().gap_3().mt_12()
                                    .child(div().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::Goal)))
                                    .child(self.goal_input.clone())
                                    .child(div().mt_8().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::Preset)))
                                    .child(self.preset_picker(window, cx))
                                    .child(div().mt_8().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::Workspace)))
                                    .child(div().flex().items_center().gap_2()
                                        .child(div().flex_1().min_w_0().child(self.workspace_input.clone()))
                                        .child(self.command("choose-workspace", locale.text(Text::ChooseWorkspace), Command::ChooseWorkspace, cx)))
                                    .when(self.workspace_prompt_failed, |form| form.child(div().text_sm().text_color(rgb(0xbf616a)).child(locale.text(Text::WorkspaceUnavailable))))
                                    .child(div().flex().flex_wrap().gap_4().mt_3().children(
                                        [Text::CreateBranch, Text::AutoVerify].into_iter().enumerate().map(|(index, text)|
                                            self.command(("task-option", index), locale.text(text), Command::ToggleTaskOption(index), cx).into_any_element()
                                        )
                                    ))
                                    .when(self.state.invalid_form, |form| {
                                        form.child(div().text_color(rgb(0xbf616a)).child(locale.text(Text::RequiredFields)))
                                    })
                                    .child(div().flex().justify_end().mt_12().child(self.command(
                                        "create-preview", locale.text(Text::CreatePreview), Command::CreatePreview, cx,
                                    )))),
                        )
                    })
                    .when(self.state.page == Page::Presets, |root| {
                        root.child(
                            div()
                                .id("settings-page")
                                .track_scroll(&self.settings_scroll)
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .p(px(if window.viewport_size().width < px(900.) { 24. } else { 40. }))
                                .flex()
                                .flex_col()
                                .gap(px(if compact { 12. } else { 24. }))
                                .child(div().flex().justify_between().items_center().gap_3().child(
                                    div()
                                        .id("settings-heading")
                                        .role(gpui::Role::Heading)
                                        .aria_level(1)
                                        .text_xl()
                                        .child(gpui::text!(locale.text(Text::Settings))),
                                ).child(self.command("new-preset", locale.text(Text::NewPreset), Command::NewPreset, cx)))
                                .child(div().mt_2().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::Language)))
                                .child(
                                    div().id("settings-language").role(gpui::Role::RadioGroup).aria_label(locale.text(Text::Language)).flex().gap_2().children([
                                        ("settings-language-en", "English", Locale::English),
                                        ("settings-language-zh", "简体中文", Locale::Chinese),
                                    ].into_iter().map(|(id, label, choice)| {
                                        div()
                                            .id(id)
                                            .role(gpui::Role::RadioButton)
                                            .aria_label(label)
                                            .aria_selected(locale == choice)
                                            .aria_toggled(if locale == choice { gpui::Toggled::True } else { gpui::Toggled::False })
                                            .track_focus(match choice { Locale::English => &self.english_focus, Locale::Chinese => &self.chinese_focus })
                                            .px_3().py_2().rounded_sm()
                                            .border_1().border_color(rgb(0xd8dee9))
                                            .when(locale == choice, |item| item.bg(rgb(0xddeef8)))
                                            .focus(|item| item.border_color(rgb(0x5e81ac)))
                                            .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
                                                let next = match event.keystroke.key.as_str() {
                                                    "enter" | "space" => choice,
                                                    "left" | "right" | "up" | "down" => match choice {
                                                        Locale::English => Locale::Chinese,
                                                        Locale::Chinese => Locale::English,
                                                    },
                                                    _ => return,
                                                };
                                                this.select_language(next, window, cx);
                                                cx.stop_propagation();
                                            }))
                                            .cursor_pointer()
                                            .child(label)
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.select_language(choice, window, cx);
                                            }))
                                            .into_any_element()
                                    }))
                                )
                                .child(div().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::DefaultPreset)))
                                .child(div().p_3().border_1().rounded_lg().bg(rgb(0xebf0f8)).border_color(rgb(0xd8dee9)).flex().items_center().gap_3()
                                    .child(div().size(px(40.)).flex_shrink_0().rounded_md().bg(rgb(0xe0e8f4)).flex().items_center().justify_center().child(icon(match self.state.default_preset { Preset::Coding => "code", Preset::Research => "search", _ => "message-square" })))
                                    .child(div().flex_1().min_w_0().flex().flex_col()
                                        .child(self.command("edit-default-preset",
                                        self.state.templates[self.state.default_preset.index()].name.clone(),
                                        Command::EditPreset(self.state.default_preset), cx))
                                    .child(div().px_3().text_color(rgb(0x5e6f8d))
                                        .child(self.state.templates[self.state.default_preset.index()].description.clone())))
                                    .child(icon("star")))
                                .child(div().mt_4().text_sm().text_color(rgb(0x5e6f8d)).child(locale.text(Text::AllPresets)))
                                .children(self.state.presets.iter().copied().map(|preset| {
                                    let template = &self.state.templates[preset.index()];
                                    let row = div();
                                    #[cfg(feature = "visual-test")]
                                    let row = {
                                        let bounds = self.drag_bounds.clone();
                                        let edit_bounds = self.preset_edit_bounds.clone();
                                        row.on_children_prepainted(move |children, _, _| {
                                            if let Some(handle) = children.first() {
                                                bounds.borrow_mut().insert(preset.index(), *handle);
                                            }
                                            if preset == Preset::Coding && let Some(edit) = children.get(2) {
                                                *edit_bounds.borrow_mut() = *edit;
                                            }
                                        })
                                    };
                                    row
                                        .id(("preset-row", preset.index()))
                                        .role(gpui::Role::ListItem)
                                        .aria_label(template.name.clone())
                                        .focusable().tab_stop(true)
                                        .focus(|style| style.bg(rgb(0xe8edf5)))
                                        .on_drop(cx.listener(move |this, drag: &PresetDrag, _, cx| {
                                            if let Some(destination) = this.state.presets.iter().position(|id| *id == preset)
                                                && this.state.move_preset(drag.preset, destination) {
                                                cx.notify();
                                            }
                                        }))
                                        .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
                                            if !event.keystroke.modifiers.alt { return; }
                                            let Some(index) = this.state.presets.iter().position(|id| *id == preset) else { return; };
                                            let destination = match event.keystroke.key.as_str() {
                                                "up" => index.checked_sub(1),
                                                "down" => index.checked_add(1),
                                                _ => return,
                                            };
                                            if let Some(destination) = destination {
                                                this.state.move_preset(preset, destination);
                                            }
                                            cx.stop_propagation();
                                            cx.notify();
                                        }))
                                        .border_b_1()
                                        .border_color(rgb(0xd8dee9))
                                        .py_4()
                                        .flex()
                                        .when(compact, |row| row.flex_wrap())
                                        .items_center()
                                        .gap_4()
                                        .child(div().id(("preset-drag-handle", preset.index()))
                                            .aria_label(locale.text(Text::ReorderPresets))
                                            .cursor_move().px_2().py_3().text_color(rgb(0x68758c)).child(icon("grip-vertical"))
                                            .on_drag(PresetDrag { preset, name: template.name.clone() }, |drag, _, _, cx| cx.new(|_| drag.clone())))
                                        .child(div().p_3().rounded_md().bg(rgb(0xe8edf5)).text_color(rgb(0x5e81ac))
                                            .child(icon(match preset { Preset::Coding => "code", Preset::Research => "search", _ => "message-square" })))
                                        .child(div().id(("edit-preset", preset.index()))
                                            .role(gpui::Role::Button).aria_label(template.name.clone())
                                            .focusable().tab_stop(true).cursor_pointer()
                                            .flex_1().min_w_0().flex().items_center().gap_2().p_3().rounded_md()
                                            .hover(|style| style.bg(rgb(0xeceff4)))
                                            .focus(|style| style.bg(rgb(0xd8dee9)))
                                            .child(div().flex_1().min_w_0().flex().flex_col().gap_2()
                                                .child(div().text_color(rgb(0x5e81ac)).child(template.name.clone()))
                                                .child(div().text_color(rgb(0x5e6f8d)).child(template.description.clone())))
                                            .child(icon("chevron-right"))
                                            .on_click(cx.listener(move |this, _, window, cx| this.execute(Command::EditPreset(preset), window, cx)))
                                            .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
                                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                                    this.execute(Command::EditPreset(preset), window, cx);
                                                    cx.stop_propagation();
                                                }
                                            })))
                                        .child(div().flex().flex_col().gap_3().justify_center().when(compact, |metadata| metadata.w_full().pl(px(60.)))
                                        .child(div().text_sm().text_color(rgb(0x5e6f8d)).child(template.models.iter().map(|id| model_label(id, locale)).collect::<Vec<_>>().join(" · ")))
                                        .child(
                                            div()
                                                .id(("default-preset", preset.index()))
                                                .role(gpui::Role::RadioButton)
                                                .aria_label(format!("{}: {}", locale.text(Text::DefaultPreset), template.name))
                                                .aria_toggled(if self.state.default_preset == preset {
                                                    gpui::accesskit::Toggled::True
                                                } else {
                                                    gpui::accesskit::Toggled::False
                                                })
                                                .focusable()
                                                .tab_stop(true)
                                                .flex()
                                                .gap_2()
                                                .cursor_pointer()
                                                .focus(|style| style.bg(rgb(0xe5e9f0)))
                                                .child(if self.state.default_preset == preset { "◉" } else { "○" })
                                                .child(locale.text(Text::DefaultPreset))
                                                .on_click(cx.listener(move |this, _, window, cx| this.execute(Command::DefaultPreset(preset), window, cx)))
                                                .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
                                                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                                        this.execute(Command::DefaultPreset(preset), window, cx);
                                                        cx.stop_propagation();
                                                    }
                                                })),
                                        ))
                                })),
                        )
                    })
                    .when(self.state.page == Page::PresetDetail, |root| {
                        root.child(
                            div()
                                .id("preset-editor")
                                .track_scroll(&self.preset_scroll)
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .px(px(if window.viewport_size().width < px(900.) { 24. } else { 48. }))
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    div()
                                        .flex().items_center().gap_3()
                                        .child(div().text_sm().child(self.command("back-to-presets", locale.text(Text::Presets), Command::BackToPresets, cx)))
                                        .child(div().text_xl().child(if self.state.creating_preset { locale.text(Text::NewPreset).to_owned() } else { self.state.templates[self.state.editing_preset.index()].name.clone() })),
                                )
                                .child(div().text_color(rgb(0x5e6f8d))
                                    .child(if self.state.creating_preset { String::new() } else { self.state.templates[self.state.editing_preset.index()].description.clone() }))
                                .child(div().mt_4().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::BasicInfo)))
                                .child(div().flex().flex_col().border_1().rounded_lg().border_color(rgb(0xdbe2ed)).children(self.preset_fields.iter().take(2).enumerate().map(
                                    |(index, field)| {
                                        div()
                                            .id(("preset-field", index))
                                            .flex()
                                            .items_center()
                                            .gap_2().px_4()
                                            .when(index == 0, |row| row.border_b_1().border_color(rgb(0xe1e6ef)))
                                            .child(div()
                                                .w(px(152.)).flex_shrink_0()
                                                .child(gpui::text!(locale.text(field.read(cx).label))))
                                            .child(div().min_w_0().flex_1().child(field.clone()))
                                    },
                                )))
                                .child(div().mt_4().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::Instructions)))
                                .child(self.preset_fields[2].clone())
                                .child(div().mt_4().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::Models)))
                                .child(div().flex().flex_col().px_4().border_1().rounded_lg().border_color(rgb(0xdbe2ed)).children((0..3).map(|role| self.model_picker(role, cx).into_any_element())))
                                .when(self.state.invalid_preset, |form| form.child(
                                    div().text_color(rgb(0xbf616a)).child(locale.text(Text::PresetNameRequired))
                                ))
                                .child(div().mt_4().text_sm().text_color(rgb(0x5e81ac)).child(gpui::text!(locale.text(Text::Capabilities))))
                                .child(div().flex().flex_col().border_1().rounded_lg().border_color(rgb(0xdbe2ed)).children(Capability::ALL.into_iter().map(|capability| {
                                    let enabled = self.state.draft_capabilities[capability as usize];
                                    div()
                                        .id(("capability", capability as usize))
                                        .role(gpui::Role::CheckBox)
                                        .aria_label(capability.label(locale))
                                        .aria_toggled(if enabled { gpui::accesskit::Toggled::True } else { gpui::accesskit::Toggled::False })
                                        .focusable().tab_stop(true)
                                        .flex().items_center().gap_2().px_4().py_2().when(!matches!(capability, Capability::Web), |row| row.border_b_1().border_color(rgb(0xe1e6ef))).cursor_pointer()
                                        .focus(|style| style.bg(rgb(0xe5e9f0)))
                                        .child(if enabled { "☑" } else { "☐" })
                                        .child(capability.label(locale))
                                        .child(div().flex_1())
                                        .child(div().text_sm().text_color(rgb(0x5e6f8d)).child(locale.text(if enabled { Text::Enabled } else { Text::Disabled })))
                                        .on_click(cx.listener(move |this, _, window, cx| this.execute(Command::ToggleCapability(capability), window, cx)))
                                        .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
                                            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                                this.execute(Command::ToggleCapability(capability), window, cx);
                                                cx.stop_propagation();
                                            }
                                        }))
                                })))
                                .child(
                                    div()
                                        .flex()
                                        .gap_4()
                                        .child(self.command(
                                            "save-preset",
                                            locale.text(Text::Save),
                                            Command::SavePreset,
                                            cx,
                                        ))
                                        .child(self.command(
                                            "cancel-preset",
                                            locale.text(Text::Cancel),
                                            Command::Presets,
                                            cx,
                                        )),
                                )
                                .when(!self.state.creating_preset, |form| form
                                    .child(div().mt_4().text_sm().text_color(rgb(0x5e81ac)).child(locale.text(Text::Other)))
                                    .child(div().flex().flex_col().p_2().border_1().rounded_lg().border_color(rgb(0xdbe2ed))
                                        .child(self.command("make-default-preset", locale.text(Text::DefaultPreset), Command::DefaultPreset(self.state.editing_preset), cx))
                                        .child(self.command("copy-preset", locale.text(Text::CopyPreset), Command::CopyPreset, cx))
                                        .when(self.state.presets.len() > 1 && self.state.pending_delete.is_none(), |actions| actions
                                            .child(self.command("delete-preset", locale.text(Text::DeletePreset), Command::DeletePreset, cx)))
                                        .when(self.state.pending_delete.is_some(), |actions| actions.child(div().flex().flex_wrap().gap_4()
                                        .child(self.command("confirm-delete", locale.text(Text::ConfirmDelete), Command::ConfirmDelete, cx))
                                        .child(self.command("cancel-delete", locale.text(Text::Cancel), Command::CancelDelete, cx))))
                                    )),
                        )
                    })
                    .when(self.state.page == Page::Preview, |root| {
                        root.child(
                            div()
                                .id("preview-scroll")
                                .flex_1()
                                .min_h_0()
                                .min_w_0()
                                .overflow_y_scroll()
                                .p_6()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(gpui::text!(locale.text(Text::PreviewOnly)))
                                .when_some(self.state.preview.as_ref(), |panel, task| {
                                    panel
                                        .child(gpui::text!(task.goal.clone()))
                                        .child(gpui::text!(task.workspace.clone()))
                                        .children([Text::CreateBranch, Text::AutoVerify].into_iter().enumerate().map(|(index, text)|
                                            div().flex().gap_3().child(locale.text(text))
                                                .child(locale.text(if task.options[index] { Text::Enabled } else { Text::Disabled }))
                                        ))
                                        .child(div().id(("preview-preset", task.preset.index())).child(gpui::text!(task.template.name.clone())))
                                        .child(gpui::text!(task.template.description.clone()))
                                        .child(gpui::text!(task.template.instructions.clone()))
                                        .child(task.template.models.iter().map(|id| model_label(id, locale)).collect::<Vec<_>>().join(" · "))
                                        .child(gpui::text!(locale.text(Text::Capabilities)))
                                        .children(Capability::ALL.into_iter().map(|capability| {
                                            let enabled = task.template.capabilities[capability as usize];
                                            div()
                                                .id(("preview-capability", capability as usize))
                                                .role(gpui::Role::Label)
                                                .aria_label(format!("{}: {}", capability.label(locale), locale.text(if enabled { Text::Enabled } else { Text::Disabled })))
                                                .flex().flex_wrap().gap_2()
                                                .child(capability.label(locale))
                                                .child(locale.text(if enabled { Text::Enabled } else { Text::Disabled }))
                                        }))
                                }),
                        )
                    })
                    .when(self.state.page == Page::Fixture, |root| {
                        root.child(
                            div()
                                .id("task-tabs")
                                .role(gpui::Role::TabList)
                                .flex()
                                .flex_wrap()
                                .gap_4()
                                .px_6()
                                .border_b_1()
                                .border_color(rgb(0xd8dee9))
                                .children(TaskTab::ALL.into_iter().enumerate().map(
                                    |(index, tab)| {
                                        div()
                                    .id(("task-tab", index))
                                    .role(gpui::Role::Tab)
                                    .aria_label(tab.label(locale))
                                    .aria_selected(self.state.tab == tab)
                                    .track_focus(&self.tab_focus[index])
                                    .px_2()
                                    .py_3()
                                    .cursor_pointer()
                                    .focus(|style| style.bg(rgb(0xe5e9f0)))
                                    .when(self.state.tab == tab, |item| {
                                        item.border_b_2().border_color(rgb(0x5e81ac))
                                    })
                                    .child(tab.label(locale))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.state.tab = tab;
                                        window.focus(&this.tab_focus[index], cx);
                                        cx.notify();
                                    }))
                                    .on_key_down(cx.listener(
                                        move |this, event: &gpui::KeyDownEvent, window, cx| {
                                            let next = match event.keystroke.key.as_str() {
                                                "left" => (index + 4) % 5,
                                                "right" => (index + 1) % 5,
                                                "home" => 0,
                                                "end" => 4,
                                                "enter" | "space" => index,
                                                _ => return,
                                            };
                                            this.state.tab = TaskTab::ALL[next];
                                            window.focus(&this.tab_focus[next], cx);
                                            cx.stop_propagation();
                                            cx.notify();
                                        },
                                    ))
                                    },
                                )),
                        )
                    })
                    .when(
                        self.state.page == Page::Fixture && self.state.tab == TaskTab::Overview,
                        |root| {
                            root.child(
                                div()
                                    .id("overview-scroll")
                                    .overflow_y_scroll()
                                    .flex_1()
                                    .min_h_0()
                                    .p(px(if window.viewport_size().width < px(900.) { 24. } else { 48. }))
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .child(div().mt_4().text_sm().text_color(rgb(0x5e6f8d)).child(locale.text(Text::Goal)))
                                    .child(div().text_size(px(30.)).font_weight(gpui::FontWeight::SEMIBOLD).child(fixture.goal.text(locale)))
                                    .child(div().text_color(rgb(0x68758c)).child(fixture.goal_description.text(locale)))
                                    .child(div().mt_6().text_sm().text_color(rgb(0x5e6f8d)).child(locale.text(Text::CurrentWork)))
                                    .child(div().flex().items_center().gap_4()
                                        .child(div().size(px(14.)).rounded_full().bg(rgb(0x487bc3)))
                                        .child(div().text_size(px(22.)).font_weight(gpui::FontWeight::SEMIBOLD).child(fixture.work.text(locale))))
                                    .child(div().pl(px(30.)).text_color(rgb(0x68758c)).child(fixture.work_description.text(locale)))
                                    .child(div().mt_6().text_sm().text_color(rgb(0x5e6f8d)).child(locale.text(Text::MissionPosition)))
                                    .child(div().flex().when(window.viewport_size().width < px(1000.), |path| path.flex_col()).children(
                                        fixture.stages.iter().enumerate().map(|(index, stage)| {
                                            div()
                                                .id(("mission-stage", index))
                                                .role(gpui::Role::Label)
                                                .aria_label(format!("{}: {}", index + 1, stage.text(locale)))
                                                .flex_1()
                                                .min_w_0()
                                                .flex().flex_col().items_center().gap_3()
                                                .py_2()
                                                .text_color(rgb(
                                                    if index == fixture.current_stage {
                                                        0x5e81ac
                                                    } else {
                                                        0x4c566a
                                                    },
                                                ))
                                                .child(div().w_full().flex().items_center().gap_2()
                                                    .child(div().flex_1().h(px(2.)).when(index > 0, |line| line.bg(rgb(if index <= fixture.current_stage { 0x487bc3 } else { 0xd8dee9 }))))
                                                    .child(div().size(px(24.)).flex_shrink_0().flex().items_center().justify_center().rounded_full().border_2().border_color(rgb(if index <= fixture.current_stage { 0x487bc3 } else { 0xc9cfda })).child(
                                                    if index < fixture.current_stage {
                                                        "✓"
                                                    } else if index == fixture.current_stage {
                                                        "●"
                                                    } else {
                                                        ""
                                                    }))
                                                    .child(div().flex_1().h(px(2.)).when(index + 1 < fixture.stages.len(), |line| line.bg(rgb(if index < fixture.current_stage { 0x487bc3 } else { 0xd8dee9 })))))
                                                .child(div().text_sm().child(stage.text(locale)))
                                        }),
                                    ))
                                    .child(div().flex().justify_center().child(format!(
                                        "{} / {}",
                                        fixture.current_stage + 1,
                                        fixture.stages.len()
                                    )))
                                    .child(div().mt_6().text_sm().text_color(rgb(0x5e6f8d)).child(locale.text(Text::RecentActivity)))
                                    .child(div().flex().flex_col().children(fixture.activity.iter().enumerate().map(
                                        |(index, (time, event, source))| {
                                            div()
                                                .id(("activity", index))
                                                .flex()
                                                .gap_4()
                                                .items_stretch()
                                                .child(div().w(px(14.)).flex_shrink_0().flex().flex_col().items_center()
                                                    .child(div().w(px(2.)).h(px(12.)).when(index > 0, |line| line.bg(rgb(0xd1d7e2))))
                                                    .child(div().size(px(12.)).flex_shrink_0().rounded_full().bg(rgb(if index + 1 == fixture.activity.len() { 0x487bc3 } else { 0xd1d7e2 })))
                                                    .child(div().w(px(2.)).flex_1().when(index + 1 < fixture.activity.len(), |line| line.bg(rgb(0xd1d7e2)))))
                                                .child(div().w(px(62.)).flex_shrink_0().py_2().text_sm().text_color(rgb(0x68758c)).child(*time))
                                                .child(div().flex_1().min_w_0().flex().items_start().gap_4().when(compact, |row| row.flex_col().gap_1())
                                                    .child(div().my_1().px_3().py_1().min_w_0().rounded_md().when(!compact, |event| event.w(px(260.)))
                                                    .when(index + 1 == fixture.activity.len(), |event| event.bg(rgb(0xe9eef6)).border_1().border_color(rgb(0xdbe3f0)))
                                                    .child(event.text(locale)))
                                                    .when(!source.text(locale).is_empty(), |row| row.child(div().py_2().when(compact, |source| source.px_3()).min_w_0().text_sm().text_color(rgb(0x68758c)).child(source.text(locale)))))
                                        },
                                    )))
                                    .child(div().mt_4().p_4().flex().flex_wrap().items_center().gap_4().border_1().rounded_lg().border_color(rgb(0xd8dee9)).text_sm()
                                        .child(format!("{} {}", fixture.files.len(), locale.text(Text::Changes)))
                                        .child(div().text_color(rgb(0x237347)).child(format!("+{}", fixture.files.iter().map(|(_, added, _)| added).sum::<u32>())))
                                        .child(div().text_color(rgb(0xbf616a)).child(format!("-{}", fixture.files.iter().map(|(_, _, removed)| removed).sum::<u32>())))
                                        .children(fixture.files.iter().map(|(path, _, _)| div().px_3().py_2().rounded_md().bg(rgb(0xf0f3f8)).border_1().border_color(rgb(0xe0e5ee)).text_color(rgb(0x68758c)).child(*path)))
                                        .child(div().flex_1())
                                        .child(self.command("view-changes", locale.text(Text::ViewChanges), Command::ViewChanges, cx)))
                                    .child(self.command("toggle-evidence", locale.text(Text::Evidence), Command::ToggleEvidence, cx))
                                    .when(self.evidence_open, |overview| overview.child(
                                        div()
                                            .flex().flex_col().gap_3()
                                            .text_sm()
                                            .text_color(rgb(0x4c566a))
                                            .child(gpui::text!(locale.text(Text::Evidence)))
                                            .child(gpui::text!(fixture.id))
                                            .child(gpui::text!(fixture.evidence))
                                            .child(locale.text(Text::Draft))
                                            .child(self.input.clone())
                                    )),
                            )
                        },
                    )
                    .when(
                        self.state.page == Page::Fixture && self.state.tab != TaskTab::Overview,
                        |root| {
                            let rows: Vec<String> = match self.state.tab {
                                TaskTab::Plan => fixture
                                    .stages
                                    .iter()
                                    .enumerate()
                                    .map(|(index, stage)| format!("{}. {}", index + 1, stage.text(locale)))
                                    .collect(),
                                TaskTab::Changes => fixture
                                    .files
                                    .iter()
                                    .map(|(path, added, removed)| {
                                        format!("{path}    +{added}    -{removed}")
                                    })
                                    .collect(),
                                TaskTab::Verification => fixture
                                    .activity
                                    .iter()
                                    .map(|(time, event, source)| format!("{time}    {}    {}", event.text(locale), source.text(locale)))
                                    .collect(),
                                TaskTab::Delivery => {
                                    vec![fixture.id.into(), fixture.evidence.into()]
                                }
                                TaskTab::Overview => Vec::new(),
                            };
                            root.child(
                                div()
                                    .id("task-detail")
                                    .role(gpui::Role::TabPanel)
                                    .aria_label(self.state.tab.label(locale))
                                    .flex_1()
                                    .min_h_0()
                                    .overflow_y_scroll()
                                    .p_6()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .child(gpui::text!(self.state.tab.label(locale)))
                                    .children(rows.into_iter().enumerate().map(|(index, row)| {
                                        div().id(("detail-row", index)).child(gpui::text!(row))
                                    })),
                            )
                        },
                    ),
            ).into_any_element()
    }
}

impl DesktopWindow {
    fn submit_live(&mut self, cx: &mut Context<Self>) {
        use crate::runtime_host::RuntimeCommand;
        let prompt = self.goal_input.read(cx).text().to_owned();
        let workspace = self.workspace_input.read(cx).text().to_owned();
        let Some(live) = self.live.as_ref() else {
            return;
        };
        if live.running || live.closing {
            return;
        }
        if prompt.trim().is_empty() || workspace.trim().is_empty() {
            if let Some(live) = &mut self.live {
                live.status = LiveStatus::MissingInput;
            }
            return;
        }
        if live
            .workspace
            .as_ref()
            .is_some_and(|previous| previous != &workspace)
        {
            if let Some(live) = &mut self.live {
                live.status = LiveStatus::WorkspaceChanged;
            }
            return;
        }
        if self
            .live
            .as_ref()
            .is_some_and(|live| live.commands.is_none())
        {
            self.start_live_host(workspace.clone(), cx);
        }
        let Some(live) = &mut self.live else {
            return;
        };
        if live.commands.is_none() {
            return;
        }
        if live
            .commands
            .as_ref()
            .is_some_and(|commands| commands.try_send(RuntimeCommand::Submit(prompt)).is_ok())
        {
            live.running = true;
            live.output.clear();
            live.turn_id = None;
            live.unavailable_tools.clear();
            live.status = LiveStatus::Connecting;
        } else {
            live.status = LiveStatus::HostBusy;
        }
    }

    fn start_live_host(&mut self, workspace: String, cx: &mut Context<Self>) {
        use crate::runtime_host::{RuntimeHost, RuntimeOutput, TerminalStatus};

        let Some(live) = &mut self.live else {
            return;
        };
        if live.commands.is_some() {
            return;
        }
        let session_id = live.session_external_id.clone().unwrap_or_else(|| {
            crate::runtime_host::task_external_id(
                std::path::Path::new(&workspace),
                self.goal_input.read(cx).text(),
            )
        });
        let host_result = if live.resume_existing {
            RuntimeHost::configured_for_existing_session(workspace.clone(), session_id)
        } else {
            RuntimeHost::configured_for_session(workspace.clone(), session_id)
        };
        let mut host = match host_result {
            Ok(host) => host,
            Err(error) => {
                live.status = LiveStatus::Error(error);
                return;
            }
        };
        live.commands = Some(host.command_sender());
        live.workspace = Some(workspace);
        if let Some(exit) = host.take_exit() {
            self.host_exits.borrow_mut().push(exit);
        }
        live.observer = Some(cx.spawn(async move |this, cx| {
            while let Some(event) = host.recv().await {
                if this
                    .update(cx, |this, cx| {
                        if let Some(live) = &mut this.live {
                            match event {
                                RuntimeOutput::ToolStarted { call_id, name } => {
                                    live.status = LiveStatus::Streaming;
                                    live.output.push_str(&format!("\n→ {name} [{call_id}]\n"));
                                }
                                RuntimeOutput::HistoryRestored { entries } => {
                                    live.output.push_str("[restored durable history]\n");
                                    for entry in entries {
                                        live.output.push_str(&entry);
                                        live.output.push('\n');
                                    }
                                }
                                RuntimeOutput::ToolRequestContext {
                                    call_id,
                                    provenance,
                                    requested_path,
                                } => {
                                    live.output.push_str(&format!(
                                        "[tool request {call_id}: provenance={provenance}; requested path={}; actual change attribution=unavailable]\n",
                                        requested_path.as_deref().unwrap_or("unavailable")
                                    ));
                                }
                                RuntimeOutput::ToolResult {
                                    call_id,
                                    content,
                                    is_error,
                                } => {
                                    live.output.push_str(&format!(
                                        "\n[{} {call_id}]\n{content}\n",
                                        if is_error { "✗" } else { "✓" }
                                    ));
                                }
                                RuntimeOutput::ApprovalRequested {
                                    request_id,
                                    tool_name,
                                    scope,
                                    explanation,
                                } => {
                                    live.pending_approval =
                                        Some((request_id, tool_name, scope, explanation));
                                    live.status = LiveStatus::Streaming;
                                }
                                RuntimeOutput::AutoDecision {
                                    outcome,
                                    reason,
                                    evaluator,
                                } => {
                                    live.output.push_str(&format!(
                                        "\n[Auto review: {outcome} — {reason} ({evaluator})]\n"
                                    ));
                                }
                                RuntimeOutput::ApprovalClosed { request_id } => {
                                    if live
                                        .pending_approval
                                        .as_ref()
                                        .is_some_and(|approval| approval.0 == request_id)
                                    {
                                        live.pending_approval = None;
                                    }
                                }
                                RuntimeOutput::Started { turn_id } => {
                                    live.turn_id = Some(turn_id);
                                    live.status = LiveStatus::Connecting
                                }
                                RuntimeOutput::Text(text) => {
                                    live.status = LiveStatus::Streaming;
                                    live.output.push_str(&text);
                                }
                                RuntimeOutput::Completed { status } => {
                                    live.pending_approval = None;
                                    live.running = false;
                                    live.status = match status {
                                        TerminalStatus::Success => LiveStatus::Finished,
                                        TerminalStatus::Cancelled => LiveStatus::Cancelled,
                                        TerminalStatus::Error(error) => LiveStatus::Error(error),
                                    };
                                }
                                RuntimeOutput::Error(error) => {
                                    live.pending_approval = None;
                                    live.status = LiveStatus::Error(error);
                                    live.running = false;
                                    live.closing = false;
                                }
                                RuntimeOutput::Stopped => {
                                    live.status = LiveStatus::Stopped;
                                    live.running = false;
                                    if live.closing {
                                        cx.quit();
                                    }
                                }
                            }
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
            let _ = this.update(cx, |this, cx| {
                if let Some(live) = &mut this.live {
                    live.commands = None;
                    live.running = false;
                }
                cx.notify();
            });
        }));
    }

    fn render_live(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let locale = self.state.locale;
        let compact = window.viewport_size().width < px(900.);
        let live = self
            .live
            .as_ref()
            .expect("live rendering requires live state");
        let content = div()
            .id("live-task")
            .when(compact, |content| {
                content
                    .overflow_y_scroll()
                    .track_scroll(&self.live_form_scroll)
            })
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .bg(rgb(0xf8f9fc))
            .text_color(rgb(0x2e3440))
            .child(div().text_xl().child(if locale == Locale::Chinese {
                "当前任务"
            } else {
                "Current task"
            }))
            .child(div().child(locale.text(Text::Workspace)))
            .child(self.workspace_input.clone())
            .child(self.command(
                "live-workspace",
                locale.text(Text::ChooseWorkspace),
                Command::ChooseWorkspace,
                cx,
            ))
            .child(div().child(locale.text(Text::Goal)))
            .child(self.goal_input.clone())
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.command(
                        "live-submit",
                        if locale == Locale::Chinese {
                            "发送"
                        } else {
                            "Send"
                        },
                        Command::SubmitLive,
                        cx,
                    ))
                    .child(self.command(
                        "live-cancel",
                        if locale == Locale::Chinese {
                            "取消"
                        } else {
                            "Cancel"
                        },
                        Command::CancelLive,
                        cx,
                    )),
            )
            .child(div().child(if locale == Locale::Chinese {
                "工具遵循 Runtime 权限门禁；成功回合持久化到当前任务会话。"
            } else {
                "Tools use the Runtime permission gate; successful turns persist to this task session."
            }))
            .child(div().text_sm().text_color(rgb(0x68758c)).child(if locale == Locale::Chinese {
                "评估状态：不可用（当前未连接共享评估存储）；不会据此判定可交付。"
            } else {
                "Evaluation: unavailable (shared evaluation storage is not connected); Delivery is not inferred."
            }))
            .child(div().child(live.status.label(locale).to_owned()))
            .when_some(live.pending_approval.as_ref(), |view, approval| {
                view.child(div().p_3().rounded_md().bg(rgb(0xe5e9f0)).child(format!(
                    "{}: {} [{}] {}",
                    if locale == Locale::Chinese {
                        "需要审批"
                    } else {
                        "Approval required"
                    },
                    approval.1,
                    approval.2,
                    approval.3
                )))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(self.command(
                            "approval-once",
                            if locale == Locale::Chinese {
                                "本次允许"
                            } else {
                                "Allow once"
                            },
                            Command::ApproveOnce(approval.0),
                            cx,
                        ))
                        .child(self.command(
                            "approval-session",
                            if locale == Locale::Chinese {
                                "会话允许"
                            } else {
                                "Allow session"
                            },
                            Command::ApproveSession(approval.0),
                            cx,
                        ))
                        .child(self.command(
                            "approval-deny",
                            if locale == Locale::Chinese {
                                "拒绝"
                            } else {
                                "Deny"
                            },
                            Command::DenyApproval(approval.0),
                            cx,
                        )),
                )
            })
            .children(live.unavailable_tools.iter().map(|name| {
                div().text_color(rgb(0xa34c42)).child(match locale {
                    Locale::English => format!("Tool unavailable; not executed: {name}"),
                    Locale::Chinese => format!("工具不可用，未执行：{name}"),
                })
            }))
            .when_some(live.turn_id.as_ref(), |view, turn_id| {
                view.child(div().text_sm().child(turn_id.clone()))
            })
            .child(
                div()
                    .id("live-output")
                    .flex_1()
                    .min_h_0()
                    .when(compact, |output| output.min_h(px(200.)).flex_shrink_0())
                    .overflow_y_scroll()
                    .track_scroll(&self.task_scroll)
                    .p_4()
                    .rounded_lg()
                    .bg(rgb(0xffffff))
                    .border_1()
                    .border_color(rgb(0xd8dee9))
                    .child(live.output.clone()),
            );
        let settings = div()
            .id("live-settings-page")
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_6()
            .child(div().text_xl().child(locale.text(Text::Settings)))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x5e81ac))
                    .child(locale.text(Text::Language)),
            )
            .child(
                div().flex().gap_3().children(
                    [
                        ("live-en", "English", Locale::English),
                        ("live-zh", "简体中文", Locale::Chinese),
                    ]
                    .into_iter()
                    .map(|(id, label, choice)| {
                        self.command(id, label, Command::LiveLocale(choice), cx)
                            .into_any_element()
                    }),
                ),
            );
        div()
            .id("live-root")
            .role(gpui::Role::Application)
            .track_focus(&self.root_focus)
            .on_action(cx.listener(|_, _: &NextFocus, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &PreviousFocus, window, cx| window.focus_prev(cx)))
            .size_full()
            .flex()
            .when(compact, |root| root.flex_col())
            .bg(rgb(0xf8f9fc))
            .text_color(rgb(0x2e3440))
            .text_size(px(15.))
            .child(
                div()
                    .id("live-navigation")
                    .role(gpui::Role::Navigation)
                    .flex()
                    .flex_shrink_0()
                    .when(compact, |nav| {
                        nav.w_full()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .p_2()
                            .border_b_1()
                    })
                    .when(!compact, |nav| {
                        nav.w(px(274.))
                            .h_full()
                            .flex_col()
                            .gap_6()
                            .p_6()
                            .border_r_1()
                    })
                    .bg(rgb(0xf3f5fa))
                    .border_color(rgb(0xd8dee9))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size(px(40.))
                                    .rounded_md()
                                    .bg(rgb(0x2e4f82))
                                    .text_color(rgb(0xffffff))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xl()
                                    .child("T"),
                            )
                            .child(div().text_size(px(17.)).child("Talos Desktop")),
                    )
                    .child(self.command(
                        "live-current-task",
                        if locale == Locale::Chinese {
                            "当前任务"
                        } else {
                            "Current task"
                        },
                        Command::OpenLive,
                        cx,
                    ))
                    .when(!compact, |nav| nav.child(div().flex_1()))
                    .child(self.command(
                        "live-settings",
                        locale.text(Text::Settings),
                        Command::Settings,
                        cx,
                    )),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .when(!compact, |body| body.h_full())
                    .child(if self.state.page == Page::Presets {
                        settings.into_any_element()
                    } else {
                        content.into_any_element()
                    }),
            )
    }
}

#[cfg(feature = "visual-test")]
fn print_performance_report(window: &Window) {
    let input = window.input_latency_snapshot();
    let frames = window.frame_duration_snapshot();
    eprintln!("Desktop timing: software timestamps only; not compositor/scanout latency.");
    for (name, histogram) in [
        (
            "coalesced-input-to-platform-submit-return",
            &input.latency_histogram,
        ),
        (
            "dirty-to-platform-submit-return",
            &frames.dirty_to_present_histogram,
        ),
        ("cpu-draw", &frames.draw_duration_histogram),
        (
            "active-animation-present-interval",
            &frames.present_interval_histogram,
        ),
    ] {
        if histogram.is_empty() {
            eprintln!("{name}: samples=0, unavailable");
        } else {
            eprintln!(
                "{name}: samples={} p50_ns={} p95_ns={} p99_ns={} max_ns={}",
                histogram.len(),
                histogram.value_at_quantile(0.50),
                histogram.value_at_quantile(0.95),
                histogram.value_at_quantile(0.99),
                histogram.max(),
            );
        }
    }
    eprintln!(
        "input coverage: frames_with_input={} max_events_per_frame={} mid_draw_events_excluded={}",
        input.events_per_frame_histogram.len(),
        input.events_per_frame_histogram.max(),
        input.mid_draw_events_dropped,
    );
}

pub(crate) fn run(locale: String, live: bool) -> std::process::ExitCode {
    #[cfg(target_os = "linux")]
    if !has_display(
        std::env::var_os("DISPLAY").as_deref(),
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
    ) {
        eprintln!("Desktop unavailable: neither DISPLAY nor WAYLAND_DISPLAY is configured");
        return std::process::ExitCode::FAILURE;
    }
    let failed = Arc::new(AtomicBool::new(false));
    let launch_failed = failed.clone();
    let host_exits = std::rc::Rc::new(std::cell::RefCell::new(
        Vec::<crate::runtime_host::HostExit>::new(),
    ));
    let window_exits = host_exits.clone();
    // Contains ordinary Rust startup panics, not native faults or ABI aborts.
    let result = catch_unwind(AssertUnwindSafe(|| {
        gpui_platform::application()
            .with_assets(DesktopAssets)
            .run(move |cx: &mut App| {
                let launched = catch_unwind(AssertUnwindSafe(|| {
                    cx.bind_keys([
                        KeyBinding::new("tab", NextFocus, None),
                        KeyBinding::new("shift-tab", PreviousFocus, None),
                    ]);
                    cx.on_window_closed(|cx, _| {
                        if cx.windows().is_empty() {
                            cx.quit();
                        }
                    })
                    .detach();
                    let bounds = Bounds::centered(None, size(px(1000.), px(700.)), cx);
                    cx.open_window(
                        WindowOptions {
                            window_bounds: Some(WindowBounds::Windowed(bounds)),
                            window_min_size: Some(size(px(640.), px(480.))),
                            ..Default::default()
                        },
                        |window, cx| {
                            #[cfg(feature = "visual-test")]
                            window.on_window_should_close(cx, |window, _| {
                                if catch_unwind(AssertUnwindSafe(|| {
                                    print_performance_report(window)
                                }))
                                .is_err()
                                {
                                    eprintln!("Desktop timing unavailable: profiler panicked");
                                }
                                true
                            });
                            let view = cx.new(|cx| {
                                let mut view = DesktopWindow::new(&locale, window, cx);
                                view.host_exits = window_exits.clone();
                                if live {
                                    view.live = Some(LiveTask::default());
                                    view.state.page = Page::NewTask;
                                }
                                view
                            });
                            if live {
                                let weak = view.downgrade();
                                window.on_window_should_close(cx, move |_, cx| {
                                    weak.update(cx, |view, cx| {
                                        let Some(live) = &mut view.live else {
                                            return true;
                                        };
                                        let allow_close = live.request_close();
                                        cx.notify();
                                        allow_close
                                    })
                                    .unwrap_or(true)
                                });
                            }
                            view
                        },
                    )
                }));
                match launched {
                    Ok(Ok(_)) => cx.activate(true),
                    Ok(Err(error)) => {
                        launch_failed.store(true, Ordering::Relaxed);
                        eprintln!("Desktop unavailable: window initialization failed: {error:#}");
                        cx.quit();
                    }
                    Err(_) => {
                        launch_failed.store(true, Ordering::Relaxed);
                        eprintln!("Desktop unavailable: window initialization panicked");
                        cx.quit();
                    }
                }
            });
    }));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(35);
    for exit in host_exits.borrow_mut().drain(..) {
        if let Err(error) =
            exit.finish(deadline.saturating_duration_since(std::time::Instant::now()))
        {
            eprintln!("Desktop runtime cleanup failed: {error}");
            failed.store(true, Ordering::Relaxed);
        }
    }
    if result.is_err() || failed.load(Ordering::Relaxed) {
        eprintln!("Desktop unavailable: renderer initialization failed");
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

#[cfg(feature = "visual-test")]
fn save_capture(window: &mut Window, path: &std::path::Path) -> Result<(), String> {
    let image = window
        .render_to_image()
        .map_err(|error| error.to_string())?;
    if image.width() == 0
        || image.height() == 0
        || image.pixels().all(|pixel| pixel == image.get_pixel(0, 0))
    {
        return Err("Renderer produced a blank image".into());
    }
    image.save(path).map_err(|error| error.to_string())?;
    eprintln!(
        "Captured {} ({}x{})",
        path.display(),
        image.width(),
        image.height()
    );
    Ok(())
}

#[cfg(feature = "visual-test")]
pub(crate) fn capture(directory: std::path::PathBuf, live_only: bool) -> std::process::ExitCode {
    // A fresh directory avoids silently replacing prior visual evidence.
    if let Err(error) = std::fs::create_dir(&directory) {
        eprintln!("Cannot create capture directory: {error}");
        return std::process::ExitCode::FAILURE;
    }
    let failed = Arc::new(AtomicBool::new(false));
    let result_failed = failed.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        gpui_platform::application().with_assets(DesktopAssets).run(move |cx: &mut App| {
            let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
                for locale in ["en-US", "zh-CN"] {
                    for (width, height) in [(1448., 1086.), (640., 480.)] {
                        for (name, page) in [
                            ("live-task", Page::NewTask),
                            ("live-task-bottom", Page::NewTask),
                            ("live-settings", Page::Presets),
                            ("overview", Page::Fixture),
                            ("task-navigation", Page::Fixture),
                            ("tasks", Page::Tasks),
                            ("changes", Page::Fixture),
                            ("new-task", Page::NewTask),
                            ("new-task-filled", Page::NewTask),
                            ("preset-picker", Page::NewTask),
                            ("preset-picker-many", Page::NewTask),
                            ("task-options", Page::NewTask),
                            ("presets", Page::Presets),
                            ("preset-edit", Page::Presets),
                            ("preset-detail", Page::PresetDetail),
                            ("preset-delete", Page::PresetDetail),
                            ("model-picker", Page::PresetDetail),
                            ("settings", Page::Fixture),
                            ("preset-drag", Page::Presets),
                        ] {
                            if live_only && !name.starts_with("live-") { continue; }
                            if name == "preset-drag" && width < 900. { continue; }
                            let handle = cx
                                .open_window(
                                    WindowOptions {
                                        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                                            gpui::point(px(0.), px(0.)),
                                            size(px(width), px(height)),
                                        ))),
                                        show: false,
                                        focus: false,
                                        ..Default::default()
                                    },
                                    |window, cx| {
                                        cx.new(|cx| {
                                            let mut view = DesktopWindow::new(locale, window, cx);
                                            if name.starts_with("live-") {
                                                view.live = Some(LiveTask {
                                                    output: "Local visual fixture — no provider request.\n本地渲染样例，未发送模型请求。\n".repeat(12),
                                                    status: LiveStatus::Streaming,
                                                    turn_id: Some("visual-fixture-i282".into()),
                                                    ..Default::default()
                                                });
                                                view.goal_input.update(cx, |input, cx| input.set_text("Explain this workspace / 介绍此工作区", cx));
                                                view.workspace_input.update(cx, |input, cx| input.set_text("/tmp/i282-visual-fixture", cx));
                                            }
                                            match page {
                                                Page::NewTask => {
                                                    view.execute(Command::NewTask, window, cx)
                                                }
                                                Page::PresetDetail => view.execute(
                                                    Command::EditPreset(Preset::Coding),
                                                    window,
                                                    cx,
                                                ),
                                                _ => view.state.page = page,
                                            }
                                            if name == "preset-picker-many" {
                                                for _ in 0..8 { view.state.duplicate_preset(); }
                                                view.execute(Command::NewTask, window, cx);
                                            }
                                            if matches!(name, "preset-picker" | "preset-picker-many") {
                                                view.preset_open = true;
                                            }
                                            if matches!(name, "new-task-filled" | "preset-picker") {
                                                let goal = view.state.fixture().goal.text(view.state.locale);
                                                view.goal_input.update(cx, |input, cx| input.set_text(goal, cx));
                                                view.workspace_input.update(cx, |input, cx| input.set_text("~/Workspace/talos-desktop", cx));
                                            }
                                            if name == "changes" {
                                                view.execute(Command::ViewChanges, window, cx);
                                            }
                                            view
                                        })
                                    },
                                )
                                .map_err(|error| error.to_string())?;
                            let path = directory
                                .join(format!("{locale}-{name}-{width:.0}x{height:.0}.png"));
                            cx.update_window(
                                handle.into(),
                                |root, window, cx| -> Result<(), String> {
                                    let _ = window.draw(cx);
                                    if name == "preset-picker" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        view.read(cx).task_scroll.scroll_to_bottom();
                                        let _ = window.draw(cx);
                                        for expected_open in [false, true] {
                                            let bounds = *view.read(cx).preset_trigger_bounds.borrow();
                                            let target = gpui::point(bounds.right() - px(12.), bounds.center().y);
                                            for event in [
                                                gpui::PlatformInput::MouseDown(gpui::MouseDownEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1, first_mouse: false }),
                                                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1 }),
                                            ] {
                                                window.dispatch_event(event, cx);
                                                let _ = window.draw(cx);
                                            }
                                            if view.read(cx).preset_open != expected_open || !view.read(cx).preset_focus.is_focused(window) {
                                                return Err("Preset trigger pointer toggle/focus failed".into());
                                            }
                                        }
                                        window.focus_next(cx);
                                        let _ = window.draw(cx);
                                        let target = gpui::point(window.viewport_size().width - px(8.), px(100.));
                                        for event in [
                                            gpui::PlatformInput::MouseDown(gpui::MouseDownEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1, first_mouse: false }),
                                            gpui::PlatformInput::MouseUp(gpui::MouseUpEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1 }),
                                        ] { window.dispatch_event(event, cx); }
                                        let _ = window.draw(cx);
                                        if view.read(cx).preset_open || !view.read(cx).preset_focus.is_focused(window) {
                                            return Err("Outside preset dismissal did not restore trigger focus".into());
                                        }
                                        window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                            keystroke: gpui::Keystroke::parse("enter").map_err(|error| error.to_string())?,
                                            is_held: false, prefer_character_input: false,
                                        }), cx);
                                        let _ = window.draw(cx);
                                        if !view.read(cx).preset_open {
                                            return Err("Preset picker cannot reopen from restored focus".into());
                                        }
                                    }
                                    if name == "preset-edit" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        let bounds = *view.read(cx).preset_edit_bounds.borrow();
                                        if bounds.bottom() >= window.viewport_size().height {
                                            view.read(cx).settings_scroll.set_offset(gpui::point(px(0.), window.viewport_size().height - bounds.bottom() - px(24.)));
                                            let _ = window.draw(cx);
                                        }
                                        let bounds = *view.read(cx).preset_edit_bounds.borrow();
                                        // The lower half is the description, not the former name-only button.
                                        let target = bounds.origin + gpui::point(bounds.size.width / 2., bounds.size.height * 0.75);
                                        if target.y >= window.viewport_size().height {
                                            return Err("Preset edit description is outside viewport".into());
                                        }
                                        for event in [
                                            gpui::PlatformInput::MouseDown(gpui::MouseDownEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1, first_mouse: false }),
                                            gpui::PlatformInput::MouseUp(gpui::MouseUpEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1 }),
                                        ] { window.dispatch_event(event, cx); }
                                        let _ = window.draw(cx);
                                        let state = view.read(cx);
                                        if state.state.page != Page::PresetDetail || state.state.editing_preset != Preset::Coding
                                            || !state.preset_fields[0].focus_handle(cx).is_focused(window) {
                                            return Err("Preset description click did not open and focus editor".into());
                                        }
                                    }
                                    if name == "preset-delete" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        let original = view.read(cx).state.presets.clone();
                                        for (step, index) in [0, 1, 0, 2].into_iter().enumerate() {
                                            view.read(cx).preset_scroll.scroll_to_bottom();
                                            let _ = window.draw(cx);
                                            let target = view.read(cx).delete_bounds.borrow()[index].center();
                                            if target.y < px(0.) || target.y >= window.viewport_size().height {
                                                return Err("Delete action outside viewport after scrolling".into());
                                            }
                                            for event in [
                                                gpui::PlatformInput::MouseDown(gpui::MouseDownEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1, first_mouse: false }),
                                                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1 }),
                                            ] { window.dispatch_event(event, cx); }
                                            let _ = window.draw(cx);
                                            let state = &view.read(cx).state;
                                            let valid = match index {
                                                0 => state.pending_delete == Some(Preset::Coding) && state.presets == original,
                                                1 => state.pending_delete.is_none() && state.presets == original,
                                                _ => state.pending_delete.is_none() && !state.presets.contains(&Preset::Coding) && state.presets.len() + 1 == original.len() && state.page == Page::Presets,
                                            };
                                            if !valid { return Err(format!("Delete action {index} failed its state invariant")); }
                                            if step == 0 {
                                                save_capture(window, &directory.join(format!("{locale}-preset-delete-confirmation-{width:.0}x{height:.0}.png")))?;
                                            }
                                        }
                                        view.update(cx, |view, cx| view.execute(Command::EditPreset(Preset::General), window, cx));
                                        view.read(cx).preset_scroll.scroll_to_bottom();
                                        let _ = window.draw(cx);
                                    }
                                    if name == "preset-picker-many" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        let last = *view.read(cx).state.presets.last().ok_or("No preset choices")?;
                                        view.read(cx).preset_menu_scroll.scroll_to_bottom();
                                        let _ = window.draw(cx);
                                        let target = view.read(cx).preset_menu_bounds.borrow()[last.index()].center();
                                        if target.y < px(0.) || target.y >= window.viewport_size().height {
                                            return Err("Last preset remains outside viewport after scrolling".into());
                                        }
                                        save_capture(window, &directory.join(format!("{locale}-preset-picker-many-expanded-{width:.0}x{height:.0}.png")))?;
                                        for event in [
                                            gpui::PlatformInput::MouseDown(gpui::MouseDownEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1, first_mouse: false }),
                                            gpui::PlatformInput::MouseUp(gpui::MouseUpEvent { button: gpui::MouseButton::Left, position: target, modifiers: Default::default(), click_count: 1 }),
                                        ] { window.dispatch_event(event, cx); }
                                        let _ = window.draw(cx);
                                        let state = view.read(cx);
                                        if state.state.preset != last || state.preset_open || !state.preset_focus.is_focused(window) {
                                            return Err("Scrolled last preset cannot be selected by pointer".into());
                                        }
                                    }
                                    if name == "task-navigation" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        view.update(cx, |view, cx| {
                                            view.goal_input.update(cx, |input, cx| input.set_text("Navigation draft", cx));
                                            view.workspace_input.update(cx, |input, cx| input.set_text("/tmp/i277-draft", cx));
                                        });
                                        for (origin, index) in [(Some(Command::Tasks), 1), (Some(Command::NewTask), 2), (Some(Command::Presets), 3), (None, 0), (None, 1)] {
                                            if let Some(origin) = origin {
                                                view.update(cx, |view, cx| view.execute(origin, window, cx));
                                                let _ = window.draw(cx);
                                            }
                                            view.update(cx, |view, cx| view.execute(Command::OpenFixture(index), window, cx));
                                            let _ = window.draw(cx);
                                            let state = view.read(cx);
                                            if state.state.fixture() != &TASK_FIXTURES[index]
                                                || state.state.page != Page::Fixture
                                                || state.state.tab != TaskTab::Overview
                                                || state.goal_input.read(cx).text() != "Navigation draft"
                                                || state.workspace_input.read(cx).text() != "/tmp/i277-draft"
                                                || !state.tab_focus[0].is_focused(window) {
                                                return Err(format!("Fixture {index} navigation or focus failed"));
                                            }
                                        }
                                    }
                                    if name == "changes" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        let state = view.read(cx);
                                        if state.state.tab != TaskTab::Changes || !state.tab_focus[2].is_focused(window) {
                                            return Err("Change-summary command did not select and focus Changes".into());
                                        }
                                    }
                                    if name == "preset-drag" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        let bounds = view.read(cx).drag_bounds.borrow().clone();
                                        let source = bounds.get(&0).ok_or("Missing source drag bounds")?.center();
                                        let destination = bounds.get(&1).ok_or("Missing target drag bounds")?.center();
                                        for (target, expected) in [
                                            (source, [Preset::Coding, Preset::General, Preset::Research]),
                                            (destination, [Preset::General, Preset::Coding, Preset::Research]),
                                        ] {
                                        window.dispatch_event(gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                                            button: gpui::MouseButton::Left, position: source,
                                            modifiers: Default::default(), click_count: 1, first_mouse: false,
                                        }), cx);
                                        for position in [source + gpui::point(px(12.), px(0.)), target] {
                                            window.dispatch_event(gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent {
                                                position, pressed_button: Some(gpui::MouseButton::Left), modifiers: Default::default(),
                                            }), cx);
                                            let _ = window.draw(cx);
                                        }
                                        window.dispatch_event(gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                                            button: gpui::MouseButton::Left, position: target,
                                            modifiers: Default::default(), click_count: 1,
                                        }), cx);
                                        let _ = window.draw(cx);
                                        let state = &view.read(cx).state;
                                        if state.presets != expected
                                            || state.page != Page::Presets || state.default_preset != Preset::Coding {
                                            return Err("Pointer drag failed ordering/page/default invariants".into());
                                        }
                                        }
                                    }
                                    if name == "task-options" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        view.read(cx).task_scroll.scroll_to_bottom();
                                        for (index, key) in ["space", "enter"].into_iter().enumerate() {
                                            let focus = view.read(cx).task_option_focus[index].clone();
                                            window.focus(&focus, cx);
                                            let _ = window.draw(cx);
                                            window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                                keystroke: gpui::Keystroke::parse(key).map_err(|error| error.to_string())?,
                                                is_held: false, prefer_character_input: false,
                                            }), cx);
                                            let _ = window.draw(cx);
                                            let state = view.read(cx);
                                            if !state.state.task_options[index] || !focus.is_focused(window)
                                                || state.state.preview.is_some() || state.state.page != Page::NewTask {
                                                return Err(format!("Task option {index} keyboard activation failed"));
                                            }
                                        }
                                    }
                                    if name == "model-picker" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        let state = view.read(cx);
                                        state.preset_scroll.scroll_to_top_of_item(7);
                                        let focus = state.model_focus[0].clone();
                                        window.focus(&focus, cx);
                                        let _ = window.draw(cx);
                                        for key in ["enter", "escape", "space"] {
                                            let keystroke = gpui::Keystroke::parse(key)
                                                .map_err(|error| error.to_string())?;
                                            window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                                keystroke, is_held: false, prefer_character_input: false,
                                            }), cx);
                                            let _ = window.draw(cx);
                                            let state = view.read(cx);
                                            let expected = if key == "escape" { None } else { Some(0) };
                                            if state.open_model != expected || !state.model_focus[0].is_focused(window) {
                                                return Err(format!("Model picker keyboard/focus failed after {key}"));
                                            }
                                        }
                                        let focus = view.read(cx).model_choice_focus[1].clone();
                                        window.focus(&focus, cx);
                                        let _ = window.draw(cx);
                                        window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                            keystroke: gpui::Keystroke::parse("enter").map_err(|error| error.to_string())?,
                                            is_held: false, prefer_character_input: false,
                                        }), cx);
                                        let _ = window.draw(cx);
                                        let state = view.read(cx);
                                        if state.preset_fields[3].read(cx).text() != MODEL_CHOICES[1]
                                            || state.open_model.is_some()
                                            || !state.model_focus[0].is_focused(window)
                                            || state.state.templates[0].models[0] != MODEL_CHOICES[0] {
                                            return Err("Model selection did not update only the draft and restore focus".into());
                                        }
                                        view.update(cx, |state, cx| state.execute(Command::Presets, window, cx));
                                        view.update(cx, |state, cx| state.execute(Command::EditPreset(Preset::Coding), window, cx));
                                        if view.read(cx).preset_fields[3].read(cx).text() != MODEL_CHOICES[0] {
                                            return Err("Cancelled model draft leaked into saved preset".into());
                                        }
                                        view.update(cx, |state, cx| {
                                            state.execute(Command::ChooseModel(0, 1), window, cx);
                                            state.preset_scroll.scroll_to_bottom();
                                            window.focus(&state.save_focus, cx);
                                        });
                                        let _ = window.draw(cx);
                                        window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                            keystroke: gpui::Keystroke::parse("enter").map_err(|error| error.to_string())?,
                                            is_held: false, prefer_character_input: false,
                                        }), cx);
                                        let _ = window.draw(cx);
                                        if view.read(cx).state.page != Page::Presets {
                                            return Err("Save button keyboard activation did not return to list".into());
                                        }
                                        view.update(cx, |state, cx| {
                                            state.execute(Command::EditPreset(Preset::Coding), window, cx);
                                            state.preset_scroll.scroll_to_top_of_item(7);
                                            state.open_model = Some(0);
                                            window.focus(&state.model_focus[0], cx);
                                        });
                                        if view.read(cx).preset_fields[3].read(cx).text() != MODEL_CHOICES[1] {
                                            return Err("Saved model choice was not restored on reopening".into());
                                        }
                                        let _ = window.draw(cx);
                                    }
                                    if name == "settings" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        view.update(cx, |state, cx| {
                                            state.goal_input.update(cx, |input, cx| input.set_text("Preserve this task draft / 保留任务草稿", cx));
                                        });
                                        let original_locale = view.read(cx).state.locale;
                                        let original_goal = view.read(cx).goal_input.read(cx).text().to_owned();
                                        let original_default = view.read(cx).state.default_preset;
                                        for key in ["enter", "space"] {
                                            view.update(cx, |state, cx| state.execute(Command::NewTask, window, cx));
                                            let focus = view.read(cx).settings_focus.clone();
                                            window.focus(&focus, cx);
                                            let _ = window.draw(cx);
                                            window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                                keystroke: gpui::Keystroke::parse(key).map_err(|error| error.to_string())?,
                                                is_held: false, prefer_character_input: false,
                                            }), cx);
                                            let _ = window.draw(cx);
                                            if view.read(cx).state.page != Page::Presets {
                                                return Err(format!("Settings page keyboard navigation failed after {key}"));
                                            }
                                        }
                                        for (locale_id, focus, key) in [
                                            ("zh-CN", view.read(cx).chinese_focus.clone(), "enter"),
                                            ("en-US", view.read(cx).english_focus.clone(), "space"),
                                        ] {
                                            window.focus(&focus, cx);
                                            let _ = window.draw(cx);
                                            window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                                keystroke: gpui::Keystroke::parse(key).map_err(|error| error.to_string())?,
                                                is_held: false, prefer_character_input: false,
                                            }), cx);
                                            let _ = window.draw(cx);
                                            let state = view.read(cx);
                                            if state.state.page != Page::Presets || !focus.is_focused(window)
                                                || state.state.locale != Locale::resolve(locale_id)
                                                || state.goal_input.read(cx).text() != original_goal
                                                || state.state.default_preset != original_default {
                                                return Err(format!("Settings language/draft/focus preservation failed: {locale_id}"));
                                            }
                                        }
                                        for (key, expected) in [
                                            ("right", Locale::Chinese),
                                            ("left", Locale::English),
                                            ("down", Locale::Chinese),
                                            ("up", Locale::English),
                                        ] {
                                            window.dispatch_event(gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                                                keystroke: gpui::Keystroke::parse(key).map_err(|error| error.to_string())?,
                                                is_held: false, prefer_character_input: false,
                                            }), cx);
                                            let _ = window.draw(cx);
                                            let state = view.read(cx);
                                            let focus = match expected {
                                                Locale::English => &state.english_focus,
                                                Locale::Chinese => &state.chinese_focus,
                                            };
                                            if state.state.locale != expected || !focus.is_focused(window) {
                                                return Err(format!("Settings language radio navigation failed after {key}"));
                                            }
                                        }
                                        view.update(cx, |state, cx| state.execute(Command::OpenFixture(1), window, cx));
                                        if view.read(cx).state.page != Page::Fixture || !view.read(cx).tab_focus[0].is_focused(window) {
                                            return Err("Settings page did not yield focus to task navigation".into());
                                        }
                                        view.update(cx, |state, cx| {
                                            state.state.locale = original_locale;
                                            state.execute(Command::Settings, window, cx);
                                        });
                                        let _ = window.draw(cx);
                                    }
                                    if name == "preset-detail" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        let description = view.read(cx).preset_fields[1].read(cx);
                                        if description.scroll_offset() != gpui::point(px(0.), px(0.)) {
                                            return Err("Unfocused preset description scrolled away from its beginning".into());
                                        }
                                    }
                                    if name == "live-task-bottom" {
                                        let view = root.clone().downcast::<DesktopWindow>()
                                            .map_err(|_| "Unexpected capture root".to_owned())?;
                                        view.read(cx).live_form_scroll.scroll_to_bottom();
                                        let _ = window.draw(cx);
                                    }
                                    save_capture(window, &path)?;
                                    window.remove_window();
                                    Ok(())
                                },
                            )
                            .map_err(|error| error.to_string())??;
                        }
                    }
                }
                Ok(())
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    eprintln!("Visual capture failed: {error}");
                    result_failed.store(true, Ordering::Relaxed);
                    std::process::exit(1);
                }
                Err(_) => {
                    eprintln!("Visual capture failed: native renderer panicked");
                    result_failed.store(true, Ordering::Relaxed);
                    std::process::exit(1);
                }
            }
            cx.quit();
        });
    }));
    if result.is_err() || failed.load(Ordering::Relaxed) {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}

#[cfg(any(target_os = "linux", test))]
fn has_display(x11: Option<&std::ffi::OsStr>, wayland: Option<&std::ffi::OsStr>) -> bool {
    [x11, wayland]
        .into_iter()
        .flatten()
        .any(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{has_display, selected_workspace_path};
    use std::ffi::OsStr;

    #[test]
    fn live_status_localization_keeps_state_and_raw_error_intact() {
        use super::{LiveStatus, Locale};
        let status = LiveStatus::Cancelling;
        assert_eq!(status.label(Locale::English), "Cancelling");
        assert_eq!(status.label(Locale::Chinese), "正在取消");
        assert_eq!(status, LiveStatus::Cancelling);
        let error = LiveStatus::Error("fixture transport error".into());
        assert_eq!(error.label(Locale::English), error.label(Locale::Chinese));
    }

    #[test]
    fn live_close_waits_for_shutdown_and_does_not_duplicate_requests() {
        use crate::runtime_host::RuntimeCommand;
        let (commands, mut receiver) = tokio::sync::mpsc::channel(1);
        let mut live = super::LiveTask {
            commands: Some(commands),
            ..Default::default()
        };
        assert!(!live.request_close());
        assert!(live.closing);
        assert!(matches!(receiver.try_recv(), Ok(RuntimeCommand::Shutdown)));
        assert!(!live.request_close());
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn full_queue_does_not_pretend_close_was_requested() {
        use crate::runtime_host::RuntimeCommand;
        let (commands, _receiver) = tokio::sync::mpsc::channel(1);
        commands
            .try_send(RuntimeCommand::Interrupt)
            .expect("fill queue");
        let mut live = super::LiveTask {
            commands: Some(commands),
            ..Default::default()
        };
        assert!(!live.request_close());
        assert!(!live.closing);
        assert!(super::LiveTask::default().request_close());
    }

    #[test]
    fn workspace_selection_preserves_cancel_and_rejects_ambiguous_paths() {
        assert_eq!(selected_workspace_path(None), Ok(None));
        assert_eq!(
            selected_workspace_path(Some(vec!["/workspace".into()])),
            Ok(Some("/workspace".into()))
        );
        assert_eq!(selected_workspace_path(Some(vec![])), Err(()));
        assert_eq!(
            selected_workspace_path(Some(vec!["/a".into(), "/b".into()])),
            Err(())
        );
        assert_eq!(selected_workspace_path(Some(vec!["".into()])), Err(()));
    }

    #[cfg(unix)]
    #[test]
    fn workspace_selection_never_lossily_rewrites_native_path() {
        use std::os::unix::ffi::OsStringExt;
        let path = std::ffi::OsString::from_vec(vec![b'/', 0xff]);
        assert_eq!(selected_workspace_path(Some(vec![path.into()])), Err(()));
    }

    #[test]
    fn unavailable_display_is_distinct_from_configured_but_unverified_display() {
        assert!(!has_display(None, None));
        assert!(!has_display(Some(OsStr::new("")), None));
        assert!(has_display(Some(OsStr::new(":0")), None));
        assert!(has_display(None, Some(OsStr::new("wayland-0"))));
        // Configuration is a prerequisite, not proof that a compositor is reachable.
    }
}
