//! Presentation-local fixture state. No runtime or persistent session authority.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Locale {
    English,
    Chinese,
}

impl Locale {
    pub(crate) fn resolve(requested: &str) -> Self {
        match requested {
            "zh-CN" => Self::Chinese,
            _ => Self::English,
        }
    }

    pub(crate) fn text(self, key: Text) -> &'static str {
        match (self, key) {
            (Self::English, Text::Title) => "Talos Desktop Preview",
            (Self::Chinese, Text::Title) => "Talos 桌面预览",
            (Self::English, Text::Goal) => "Goal",
            (Self::Chinese, Text::Goal) => "目标",
            (Self::English, Text::Evidence) => "Evidence",
            (Self::Chinese, Text::Evidence) => "证据",
            (Self::English, Text::Draft) => "Draft",
            (Self::Chinese, Text::Draft) => "草稿",
            (Self::English, Text::CurrentWork) => "Current work",
            (Self::Chinese, Text::CurrentWork) => "当前工作",
            (Self::English, Text::MissionPosition) => "Mission position",
            (Self::Chinese, Text::MissionPosition) => "任务进度",
            (Self::English, Text::RecentActivity) => "Recent activity",
            (Self::Chinese, Text::RecentActivity) => "最近活动",
            (Self::English, Text::Changes) => "Changed files",
            (Self::Chinese, Text::Changes) => "变更文件",
            (Self::English, Text::ViewChanges) => "View changes",
            (Self::Chinese, Text::ViewChanges) => "查看变更",
            (Self::English, Text::RecentTasks) => "Recent tasks",
            (Self::Chinese, Text::RecentTasks) => "最近任务",
            (Self::English, Text::AllTasks) => "View all tasks",
            (Self::Chinese, Text::AllTasks) => "查看全部任务",
            (Self::English, Text::NewTask) => "New task",
            (Self::Chinese, Text::NewTask) => "新任务",
            (Self::English, Text::Workspace) => "Workspace",
            (Self::Chinese, Text::Workspace) => "工作区",
            (Self::English, Text::Preset) => "Preset",
            (Self::Chinese, Text::Preset) => "预设",
            (Self::English, Text::CreatePreview) => "Create preview",
            (Self::Chinese, Text::CreatePreview) => "创建预览",
            (Self::English, Text::PreviewOnly) => "Preview · Not executing",
            (Self::Chinese, Text::PreviewOnly) => "预览 · 未执行",
            (Self::English, Text::RequiredFields) => "Goal and workspace are required",
            (Self::Chinese, Text::RequiredFields) => "请填写目标和工作区",
            (Self::English, Text::Presets) => "Presets",
            (Self::Chinese, Text::Presets) => "预设",
            (Self::English, Text::Instructions) => "Instructions",
            (Self::Chinese, Text::Instructions) => "指令",
            (Self::English, Text::DefaultModel) => "Default model",
            (Self::Chinese, Text::DefaultModel) => "默认模型",
            (Self::English, Text::QuickModel) => "Quick model",
            (Self::Chinese, Text::QuickModel) => "快速模型",
            (Self::English, Text::DeepModel) => "Deep model",
            (Self::Chinese, Text::DeepModel) => "深度模型",
            (Self::English, Text::Save) => "Save preview",
            (Self::Chinese, Text::Save) => "保存预览",
            (Self::English, Text::Cancel) => "Cancel",
            (Self::Chinese, Text::Cancel) => "取消",
            (Self::English, Text::DefaultPreset) => "Default preset",
            (Self::Chinese, Text::DefaultPreset) => "默认预设",
            (Self::English, Text::Name) => "Name",
            (Self::Chinese, Text::Name) => "名称",
            (Self::English, Text::Description) => "Description",
            (Self::Chinese, Text::Description) => "说明",
            (Self::English, Text::PresetNameRequired) => "A preset name is required",
            (Self::Chinese, Text::PresetNameRequired) => "请填写预设名称",
            (Self::English, Text::Capabilities) => "Capabilities",
            (Self::Chinese, Text::Capabilities) => "能力",
            (Self::English, Text::Enabled) => "Enabled",
            (Self::Chinese, Text::Enabled) => "开启",
            (Self::English, Text::Disabled) => "Disabled",
            (Self::Chinese, Text::Disabled) => "关闭",
            (Self::English, Text::CopyPreset) => "Duplicate saved preset",
            (Self::Chinese, Text::CopyPreset) => "复制已保存预设",
            (Self::English, Text::DeletePreset) => "Delete preset",
            (Self::Chinese, Text::DeletePreset) => "删除预设",
            (Self::English, Text::ConfirmDelete) => "Confirm deletion",
            (Self::Chinese, Text::ConfirmDelete) => "确认删除",
            (Self::English, Text::BasicInfo) => "BASIC INFO",
            (Self::Chinese, Text::BasicInfo) => "基本信息",
            (Self::English, Text::Models) => "MODELS",
            (Self::Chinese, Text::Models) => "模型",
            (Self::English, Text::Other) => "OTHER",
            (Self::Chinese, Text::Other) => "其他",
            (Self::English, Text::AllPresets) => "ALL PRESETS",
            (Self::Chinese, Text::AllPresets) => "全部预设",
            (Self::English, Text::NewPreset) => "New preset",
            (Self::Chinese, Text::NewPreset) => "新建预设",
            (Self::English, Text::ReorderPresets) => {
                "Drag to reorder, or focus a row and press Alt+↑ / Alt+↓."
            }
            (Self::Chinese, Text::ReorderPresets) => "拖拽调整顺序，或聚焦条目后按 Alt+↑ / Alt+↓。",
            (Self::English, Text::Settings) => "Settings",
            (Self::Chinese, Text::Settings) => "设置",
            (Self::English, Text::Language) => "Language",
            (Self::Chinese, Text::Language) => "语言",
            (Self::English, Text::LocalUser) => "Local preview user",
            (Self::Chinese, Text::LocalUser) => "本地预览用户",
            (Self::English, Text::CreateBranch) => "Create Git branch",
            (Self::Chinese, Text::CreateBranch) => "创建 Git 分支",
            (Self::English, Text::AutoVerify) => "Enable automatic verification",
            (Self::Chinese, Text::AutoVerify) => "启用自动验证",
            (Self::English, Text::ChooseWorkspace) => "Choose folder",
            (Self::Chinese, Text::ChooseWorkspace) => "选择文件夹",
            (Self::English, Text::WorkspaceUnavailable) => {
                "Folder selection unavailable. Enter the path manually."
            }
            (Self::Chinese, Text::WorkspaceUnavailable) => "文件夹选择不可用，请手动输入路径。",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Text {
    Title,
    Goal,
    Evidence,
    Draft,
    CurrentWork,
    MissionPosition,
    RecentActivity,
    Changes,
    ViewChanges,
    RecentTasks,
    AllTasks,
    NewTask,
    Workspace,
    Preset,
    CreatePreview,
    PreviewOnly,
    RequiredFields,
    Presets,
    Instructions,
    DefaultModel,
    QuickModel,
    DeepModel,
    Save,
    Cancel,
    DefaultPreset,
    Name,
    Description,
    PresetNameRequired,
    Capabilities,
    Enabled,
    Disabled,
    CopyPreset,
    DeletePreset,
    ConfirmDelete,
    BasicInfo,
    Models,
    Other,
    AllPresets,
    NewPreset,
    ReorderPresets,
    Settings,
    Language,
    LocalUser,
    CreateBranch,
    AutoVerify,
    ChooseWorkspace,
    WorkspaceUnavailable,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LocalizedText {
    english: &'static str,
    chinese: &'static str,
}

impl LocalizedText {
    const fn new(english: &'static str, chinese: &'static str) -> Self {
        Self { english, chinese }
    }

    pub(crate) fn text(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::English => self.english,
            Locale::Chinese => self.chinese,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Fixture {
    pub(crate) id: &'static str,
    pub(crate) title: LocalizedText,
    pub(crate) goal: LocalizedText,
    pub(crate) goal_description: LocalizedText,
    pub(crate) evidence: &'static str,
    pub(crate) work: LocalizedText,
    pub(crate) work_description: LocalizedText,
    pub(crate) stages: &'static [LocalizedText],
    pub(crate) current_stage: usize,
    pub(crate) activity: &'static [(&'static str, LocalizedText, LocalizedText)],
    pub(crate) files: &'static [(&'static str, u32, u32)],
}

pub(crate) const EXECUTION: Fixture = Fixture {
    id: "i277-execution-001",
    title: LocalizedText::new("GitHub Models support", "GitHub Models 支持"),
    goal: LocalizedText::new(
        "Implement model retrieval and caching",
        "实现模型获取与缓存",
    ),
    goal_description: LocalizedText::new(
        "Retrieve available models from GitHub Models, with local caching and refresh.",
        "从 GitHub Models 获取可用模型，并支持本地缓存与刷新。",
    ),
    evidence: "fixture://i277/execution/001",
    work: LocalizedText::new("Implement cache refresh", "实现缓存刷新逻辑"),
    work_description: LocalizedText::new(
        "Refresh on TTL and condition changes to keep the model list up to date.",
        "按 TTL 与条件触发刷新，保证模型列表及时更新。",
    ),
    stages: &[
        LocalizedText::new("Architecture", "架构理解"),
        LocalizedText::new("Authentication", "凭证与鉴权"),
        LocalizedText::new("Models", "模型管理"),
        LocalizedText::new("Runtime", "运行时集成"),
        LocalizedText::new("Verification", "验证"),
        LocalizedText::new("Delivery", "交付"),
    ],
    current_stage: 2,
    activity: &[
        (
            "10:42",
            LocalizedText::new("Fetched 32 fixture models", "获取 32 个示例模型"),
            LocalizedText::new("GitHub Models API", "GitHub Models API"),
        ),
        (
            "10:44",
            LocalizedText::new("Cache policy prepared", "缓存策略已准备"),
            LocalizedText::new("TTL and invalidation", "TTL 与失效规则"),
        ),
        (
            "10:46",
            LocalizedText::new("12 fixture tests passed", "12 个示例测试通过"),
            LocalizedText::new("cargo test", "cargo test"),
        ),
        (
            "10:48",
            LocalizedText::new("Checking stale cache", "正在检查 stale cache 行为"),
            LocalizedText::new("", ""),
        ),
    ],
    files: &[
        ("cache.rs", 110, 12),
        ("github.rs", 50, 6),
        ("provider.rs", 24, 3),
    ],
};

pub(crate) const TASK_FIXTURES: [Fixture; 4] = [
    EXECUTION,
    Fixture {
        id: "i277-execution-002",
        title: LocalizedText::new("Tool Parser refactor", "Tool Parser 重构"),
        goal: LocalizedText::new("Unify tool-call parsing", "统一工具调用解析"),
        goal_description: LocalizedText::new(
            "Preserve native and text protocol behavior with a shared parsing boundary.",
            "在共享解析边界内保持原生与文本协议行为一致。",
        ),
        work: LocalizedText::new("Compare protocol fixtures", "比对协议示例"),
        work_description: LocalizedText::new(
            "Check structured calls and malformed input before changing the parser.",
            "在修改解析器之前检查结构化调用与异常输入。",
        ),
        evidence: "fixture://i277/execution/002",
        stages: &[
            LocalizedText::new("Inventory", "梳理"),
            LocalizedText::new("Fixtures", "示例"),
            LocalizedText::new("Refactor", "重构"),
            LocalizedText::new("Verification", "验证"),
        ],
        current_stage: 1,
        activity: &[
            (
                "09:12",
                LocalizedText::new("Mapped parser entrypoints", "已整理解析入口"),
                LocalizedText::new("Protocol inventory", "协议清单"),
            ),
            (
                "09:18",
                LocalizedText::new("Comparing malformed-call fixtures", "正在比对异常调用示例"),
                LocalizedText::new("Parser fixtures", "解析器示例"),
            ),
        ],
        files: &[("parser.rs", 18, 9), ("protocol_tests.rs", 42, 0)],
    },
    Fixture {
        id: "i277-execution-003",
        title: LocalizedText::new("Rust web retrieval", "Rust Web 抓取方案"),
        goal: LocalizedText::new("Evaluate Rust web retrieval", "评估 Rust 网页抓取方案"),
        goal_description: LocalizedText::new(
            "Compare extraction quality and dependency boundaries for a Rust-native approach.",
            "比较 Rust 原生方案的内容提取质量和依赖边界。",
        ),
        work: LocalizedText::new("Compare extraction results", "比较内容提取结果"),
        work_description: LocalizedText::new(
            "Review saved HTML fixtures without fetching live websites.",
            "使用已保存的 HTML 示例进行评估，不访问实时网站。",
        ),
        evidence: "fixture://i277/execution/003",
        stages: &[
            LocalizedText::new("Requirements", "需求"),
            LocalizedText::new("Comparison", "比较"),
            LocalizedText::new("Decision", "决策"),
        ],
        current_stage: 1,
        activity: &[
            (
                "08:35",
                LocalizedText::new("Collected extraction requirements", "已整理内容提取需求"),
                LocalizedText::new("Research notes", "调研记录"),
            ),
            (
                "08:50",
                LocalizedText::new("Reviewing saved HTML samples", "正在检查 HTML 示例"),
                LocalizedText::new("Local samples", "本地示例"),
            ),
        ],
        files: &[("web-retrieval.md", 28, 3)],
    },
    Fixture {
        id: "i277-execution-004",
        title: LocalizedText::new("SQLite corruption analysis", "SQLite 损坏分析"),
        goal: LocalizedText::new("Investigate storage recovery", "分析存储恢复流程"),
        goal_description: LocalizedText::new(
            "Document failure handling against a disposable database fixture.",
            "使用可丢弃的数据库示例记录故障处理流程。",
        ),
        work: LocalizedText::new("Review recovery evidence", "检查恢复证据"),
        work_description: LocalizedText::new(
            "Compare integrity-check output with the expected recovery contract.",
            "将完整性检查结果与预期恢复合同进行比对。",
        ),
        evidence: "fixture://i277/execution/004",
        stages: &[
            LocalizedText::new("Reproduce", "复现"),
            LocalizedText::new("Analyze", "分析"),
            LocalizedText::new("Verify", "验证"),
        ],
        current_stage: 2,
        activity: &[
            (
                "08:05",
                LocalizedText::new(
                    "Prepared disposable database fixture",
                    "已准备临时数据库示例",
                ),
                LocalizedText::new("Storage fixtures", "存储示例"),
            ),
            (
                "08:20",
                LocalizedText::new("Reviewing recovery checks", "正在检查恢复验证结果"),
                LocalizedText::new("Integrity checks", "完整性检查"),
            ),
        ],
        files: &[("storage-recovery.md", 17, 2)],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TaskTab {
    Overview,
    Plan,
    Changes,
    Verification,
    Delivery,
}

impl TaskTab {
    pub(crate) const ALL: [Self; 5] = [
        Self::Overview,
        Self::Plan,
        Self::Changes,
        Self::Verification,
        Self::Delivery,
    ];

    pub(crate) fn label(self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Overview, Locale::English) => "Overview",
            (Self::Overview, Locale::Chinese) => "概览",
            (Self::Plan, Locale::English) => "Plan",
            (Self::Plan, Locale::Chinese) => "计划",
            (Self::Changes, Locale::English) => "Changes",
            (Self::Changes, Locale::Chinese) => "变更",
            (Self::Verification, Locale::English) => "Verification",
            (Self::Verification, Locale::Chinese) => "验证",
            (Self::Delivery, Locale::English) => "Delivery",
            (Self::Delivery, Locale::Chinese) => "交付",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Page {
    Tasks,
    Fixture,
    NewTask,
    Preview,
    Presets,
    PresetDetail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Preset {
    Coding,
    General,
    Research,
    Custom(usize),
}

impl Preset {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Coding => 0,
            Self::General => 1,
            Self::Research => 2,
            Self::Custom(index) => index,
        }
    }
    pub(crate) const ALL: [Self; 3] = [Self::Coding, Self::General, Self::Research];
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Coding => "Coding",
            Self::General => "General",
            Self::Research => "Research",
            Self::Custom(_) => "Copy",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PresetTemplate {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) instructions: String,
    pub(crate) models: [String; 3],
    pub(crate) capabilities: [bool; 4],
}

pub(crate) fn model_label(id: &str, locale: Locale) -> &str {
    match (id, locale) {
        ("fixture/default", Locale::English) => "Balanced preview",
        ("fixture/default", Locale::Chinese) => "均衡预览模型",
        ("fixture/quick", Locale::English) => "Fast preview",
        ("fixture/quick", Locale::Chinese) => "快速预览模型",
        ("fixture/deep", Locale::English) => "Deep preview",
        ("fixture/deep", Locale::Chinese) => "深度预览模型",
        _ => id,
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Capability {
    Filesystem,
    Shell,
    GitHub,
    Web,
}

impl Capability {
    pub(crate) const ALL: [Self; 4] = [Self::Filesystem, Self::Shell, Self::GitHub, Self::Web];
    pub(crate) fn label(self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Filesystem, Locale::Chinese) => "文件系统",
            (Self::Filesystem, Locale::English) => "Filesystem",
            (Self::Shell, _) => "Shell",
            (Self::GitHub, _) => "GitHub",
            (Self::Web, _) => "Web",
        }
    }
}

pub(crate) struct PreviewTask {
    pub(crate) options: [bool; 2],
    pub(crate) goal: String,
    pub(crate) workspace: String,
    pub(crate) preset: Preset,
    pub(crate) template: PresetTemplate,
}

pub(crate) struct Presentation {
    pub(crate) active_fixture: usize,
    pub(crate) task_options: [bool; 2],
    pub(crate) locale: Locale,
    pub(crate) tab: TaskTab,
    pub(crate) page: Page,
    pub(crate) preset: Preset,
    pub(crate) preview: Option<PreviewTask>,
    pub(crate) default_preset: Preset,
    pub(crate) invalid_form: bool,
    pub(crate) templates: Vec<PresetTemplate>,
    pub(crate) presets: Vec<Preset>,
    pub(crate) pending_delete: Option<Preset>,
    pub(crate) editing_preset: Preset,
    pub(crate) invalid_preset: bool,
    pub(crate) draft_capabilities: [bool; 4],
    pub(crate) creating_preset: bool,
}

impl Presentation {
    pub(crate) fn new(requested_locale: &str) -> Self {
        let locale = Locale::resolve(requested_locale);
        Self {
            active_fixture: 0,
            task_options: [false; 2],
            locale,
            tab: TaskTab::Overview,
            page: Page::Fixture,
            preset: Preset::Coding,
            default_preset: Preset::Coding,
            preview: None,
            invalid_form: false,
            editing_preset: Preset::Coding,
            invalid_preset: false,
            draft_capabilities: [true; 4],
            creating_preset: false,
            presets: Preset::ALL.to_vec(),
            pending_delete: None,
            templates: Preset::ALL
                .map(|preset| PresetTemplate {
                    name: preset.name().into(),
                    description: match (preset, locale) {
                        (Preset::Coding, Locale::English) => "Software development, code changes and verification",
                        (Preset::Coding, Locale::Chinese) => "软件开发、代码修改和验证",
                        (Preset::General, Locale::English) => "Everyday tasks and general assistance",
                        (Preset::General, Locale::Chinese) => "通用任务处理与多种场景",
                        (Preset::Research, Locale::English) => "Information gathering, analysis and synthesis",
                        (Preset::Research, Locale::Chinese) => "信息收集、分析与总结",
                        (Preset::Custom(_), _) => "",
                    }
                    .into(),
                    instructions: match preset {
                        Preset::Coding => "You are an engineering agent focused on software development.\nWrite clean, correct and efficient code. Follow best practices and style guides.\nWhen modifying code, keep changes minimal and explain the rationale.\nRun tests or linters when possible and fix issues iteratively until they pass.\nAsk clarifying questions if requirements are ambiguous.",
                        Preset::General => "Clarify the goal and complete the requested work.",
                        Preset::Research => "Compare evidence and cite the sources.",
                        Preset::Custom(_) => "",
                    }
                    .into(),
                    models: [
                        "fixture/default".into(),
                        "fixture/quick".into(),
                        "fixture/deep".into(),
                    ],
                    capabilities: [true; 4],
                })
                .to_vec(),
        }
    }

    pub(crate) fn set_locale(&mut self, requested_locale: &str) {
        self.locale = Locale::resolve(requested_locale);
    }

    pub(crate) fn begin_task(&mut self) {
        self.task_options = [false; 2];
        self.preset = self.default_preset;
        self.page = Page::NewTask;
        self.invalid_form = false;
    }

    pub(crate) fn save_preset(&mut self, template: PresetTemplate) -> bool {
        if !self.creating_preset && !self.presets.contains(&self.editing_preset) {
            return false;
        }
        self.invalid_preset = template.name.trim().is_empty();
        if self.invalid_preset {
            return false;
        }
        if self.creating_preset {
            let id = Preset::Custom(self.templates.len());
            self.templates.push(template);
            self.presets.push(id);
            self.editing_preset = id;
            self.creating_preset = false;
        } else {
            self.templates[self.editing_preset.index()] = template;
        }
        self.page = Page::Presets;
        true
    }

    pub(crate) fn select_preset(&mut self, preset: Preset, default: bool) -> bool {
        if !self.presets.contains(&preset) {
            return false;
        }
        if default {
            self.default_preset = preset;
        } else {
            self.preset = preset;
        }
        true
    }

    pub(crate) fn edit_preset(&mut self, preset: Preset) -> bool {
        if !self.presets.contains(&preset) {
            return false;
        }
        self.pending_delete = None;
        self.creating_preset = false;
        self.editing_preset = preset;
        self.invalid_preset = false;
        self.draft_capabilities = self.templates[preset.index()].capabilities;
        self.page = Page::PresetDetail;
        true
    }

    pub(crate) fn begin_preset(&mut self) {
        self.creating_preset = true;
        self.pending_delete = None;
        self.invalid_preset = false;
        self.draft_capabilities = [false; 4];
        self.page = Page::PresetDetail;
    }

    pub(crate) fn move_preset(&mut self, preset: Preset, destination: usize) -> bool {
        let Some(source) = self.presets.iter().position(|id| *id == preset) else {
            return false;
        };
        if destination >= self.presets.len() || source == destination {
            return false;
        }
        self.presets.remove(source);
        self.presets.insert(destination, preset);
        true
    }

    pub(crate) fn cancel_preset(&mut self) {
        self.creating_preset = false;
        self.pending_delete = None;
        self.invalid_preset = false;
        self.page = Page::Presets;
    }

    pub(crate) fn duplicate_preset(&mut self) -> Preset {
        let mut copy = self.templates[self.editing_preset.index()].clone();
        let name = copy.name.clone();
        let mut suffix = 1;
        loop {
            copy.name = format!("{name} ({suffix})");
            if !self
                .presets
                .iter()
                .any(|id| self.templates[id.index()].name == copy.name)
            {
                break;
            }
            suffix += 1;
        }
        let id = Preset::Custom(self.templates.len());
        self.templates.push(copy);
        self.presets.push(id);
        self.edit_preset(id);
        id
    }

    pub(crate) fn request_delete(&mut self) {
        self.pending_delete =
            (!self.creating_preset && self.presets.len() > 1).then_some(self.editing_preset);
    }

    pub(crate) fn confirm_delete(&mut self) -> bool {
        let Some(id) = self.pending_delete.take() else {
            return false;
        };
        if self.presets.len() <= 1 || !self.presets.contains(&id) {
            return false;
        }
        self.presets.retain(|preset| *preset != id);
        // Retain only the slot identity; release deleted instructions and metadata.
        self.templates[id.index()] = PresetTemplate::default();
        if self.default_preset == id {
            self.default_preset = self.presets[0];
        }
        if self.preset == id {
            self.preset = self.default_preset;
        }
        self.editing_preset = self.default_preset;
        self.page = Page::Presets;
        true
    }

    pub(crate) fn create_preview(&mut self, goal: &str, workspace: &str) -> bool {
        if !self.presets.contains(&self.preset) {
            return false;
        }
        self.invalid_form = goal.trim().is_empty() || workspace.trim().is_empty();
        if self.invalid_form {
            return false;
        }
        self.preview = Some(PreviewTask {
            options: self.task_options,
            goal: goal.to_owned(),
            workspace: workspace.to_owned(),
            preset: self.preset,
            template: self.templates[self.preset.index()].clone(),
        });
        self.page = Page::Preview;
        true
    }

    pub(crate) fn fixture(&self) -> &'static Fixture {
        &TASK_FIXTURES[self.active_fixture]
    }

    pub(crate) fn open_fixture(&mut self, index: usize) -> bool {
        if index >= TASK_FIXTURES.len() {
            return false;
        }
        self.active_fixture = index;
        self.page = Page::Fixture;
        self.tab = TaskTab::Overview;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_display_names_are_localized_without_changing_stored_identifiers() {
        for locale in [Locale::English, Locale::Chinese] {
            let labels = ["fixture/default", "fixture/quick", "fixture/deep"]
                .map(|id| model_label(id, locale));
            assert_ne!(labels[0], labels[1]);
            assert_ne!(labels[1], labels[2]);
            assert!(labels.iter().all(|label| !label.starts_with("fixture/")));
            assert_eq!(model_label("custom/model", locale), "custom/model");
        }
        let state = Presentation::new("zh-CN");
        assert_eq!(
            state.templates[0].models,
            ["fixture/default", "fixture/quick", "fixture/deep"]
        );
    }

    #[test]
    fn fixture_navigation_preserves_preview_and_locale_and_rejects_unknown_identity() {
        let mut state = Presentation::new("en-US");
        state.begin_task();
        assert!(state.create_preview("Keep this draft", "/tmp/workspace"));
        for (index, fixture) in TASK_FIXTURES.iter().enumerate() {
            assert!(state.open_fixture(index));
            assert_eq!(state.fixture(), fixture);
            assert_eq!(state.page, Page::Fixture);
            assert_eq!(state.tab, TaskTab::Overview);
            state.set_locale("zh-CN");
            assert_eq!(state.fixture(), fixture);
            assert_eq!(
                state.preview.as_ref().map(|task| task.goal.as_str()),
                Some("Keep this draft")
            );
            assert!(fixture.current_stage < fixture.stages.len());
            assert!(!fixture.activity.is_empty());
            for other in &TASK_FIXTURES[..index] {
                assert_ne!(fixture.id, other.id);
                assert_ne!(fixture.evidence, other.evidence);
                assert_ne!(fixture.title, other.title);
            }
        }
        assert!(!state.open_fixture(usize::MAX));
        assert_eq!(state.fixture(), &TASK_FIXTURES[3]);
    }

    #[test]
    fn task_options_are_preview_snapshots_not_shared_mutable_settings() {
        let mut state = Presentation::new("en-US");
        state.begin_task();
        assert_eq!(state.task_options, [false; 2]);
        state.task_options = [true, false];
        state.set_locale("zh-CN");
        assert_eq!(state.task_options, [true, false]);
        assert!(state.create_preview("goal", "/workspace"));
        state.begin_task();
        assert_eq!(state.task_options, [false; 2]);
        assert_eq!(
            state.preview.as_ref().expect("preview").options,
            [true, false]
        );
    }

    #[test]
    fn preset_order_changes_without_changing_identity_or_snapshot() {
        let mut state = Presentation::new("en-US");
        assert!(state.create_preview("goal", "/workspace"));
        let snapshot = state.preview.as_ref().expect("preview").template.clone();
        let templates = state.templates.clone();
        assert!(state.move_preset(Preset::Coding, 2));
        assert_eq!(
            state.presets,
            vec![Preset::General, Preset::Research, Preset::Coding]
        );
        assert!(state.move_preset(Preset::Coding, 0));
        assert_eq!(state.presets, Preset::ALL);
        assert!(!state.move_preset(Preset::Coding, 3));
        assert!(!state.move_preset(Preset::Custom(99), 0));
        assert!(!state.move_preset(Preset::Coding, 0));
        assert_eq!(state.default_preset, Preset::Coding);
        assert_eq!(state.templates, templates);
        assert_eq!(state.preview.as_ref().expect("preview").template, snapshot);
    }

    #[test]
    fn new_preset_is_not_published_until_valid_save() {
        let mut state = Presentation::new("en-US");
        assert!(state.create_preview("goal", "/workspace"));
        let snapshot = state.preview.as_ref().expect("preview").template.clone();
        let original_count = state.templates.len();
        state.begin_preset();
        state.request_delete();
        assert!(state.pending_delete.is_none());
        assert!(!state.save_preset(PresetTemplate::default()));
        assert_eq!(state.templates.len(), original_count);
        state.cancel_preset();
        assert_eq!(state.presets.len(), original_count);
        assert!(!state.creating_preset);
        state.begin_preset();
        assert!(state.save_preset(PresetTemplate {
            name: "Local test".into(),
            ..PresetTemplate::default()
        }));
        assert_eq!(state.templates.len(), original_count + 1);
        assert_eq!(state.presets.last(), Some(&Preset::Custom(original_count)));
        assert_eq!(state.default_preset, Preset::Coding);
        assert_eq!(state.preview.as_ref().expect("preview").template, snapshot);
        assert_eq!(state.page, Page::Presets);
    }

    #[test]
    fn editing_existing_preset_exits_unsaved_creation() {
        let mut state = Presentation::new("en-US");
        state.begin_preset();
        assert!(state.edit_preset(Preset::Research));
        assert!(!state.creating_preset);
        let template = state.templates[Preset::Research.index()].clone();
        assert!(state.save_preset(template));
        assert_eq!(state.presets.len(), 3);
    }

    #[test]
    fn duplicates_have_unique_identity_and_independent_contents() {
        let mut state = Presentation::new("en-US");
        let original = state.templates[0].clone();
        let first = state.duplicate_preset();
        state.templates[first.index()].instructions = "new instructions".into();
        state.edit_preset(Preset::Coding);
        let second = state.duplicate_preset();
        assert_ne!(first, second);
        assert_ne!(
            state.templates[first.index()].name,
            state.templates[second.index()].name
        );
        assert_eq!(state.templates[0], original);
        assert_eq!(
            state.templates[second.index()].instructions,
            original.instructions
        );
    }

    #[test]
    fn delete_requires_confirmation_preserves_preview_and_keeps_a_default() {
        let mut state = Presentation::new("en-US");
        assert!(state.create_preview("goal", "/workspace"));
        let original = state.preview.as_ref().expect("preview").template.clone();
        assert!(!state.confirm_delete());
        state.request_delete();
        state.edit_preset(Preset::General);
        assert!(!state.confirm_delete());
        state.edit_preset(Preset::Coding);
        state.request_delete();
        assert!(state.confirm_delete());
        assert_eq!(state.templates[0], PresetTemplate::default());
        assert!(!state.select_preset(Preset::Coding, false));
        assert!(!state.select_preset(Preset::Coding, true));
        assert!(!state.edit_preset(Preset::Coding));
        assert_eq!(state.default_preset, Preset::General);
        assert_eq!(state.preview.as_ref().expect("preview").template, original);
        state.begin_task();
        assert_eq!(state.preset, Preset::General);
        state.edit_preset(Preset::General);
        state.request_delete();
        assert!(state.confirm_delete());
        state.edit_preset(Preset::Research);
        state.request_delete();
        assert!(!state.confirm_delete());
        assert_eq!(state.presets, vec![Preset::Research]);
    }

    #[test]
    fn capability_drafts_cancel_and_saved_templates_are_snapshotted() {
        let mut state = Presentation::new("en-US");
        state.edit_preset(Preset::Coding);
        state.draft_capabilities[Capability::Shell as usize] = false;
        assert!(state.templates[0].capabilities[Capability::Shell as usize]);
        state.edit_preset(Preset::Coding);
        assert!(state.draft_capabilities[Capability::Shell as usize]);
        state.draft_capabilities[Capability::Shell as usize] = false;
        let mut edit = state.templates[0].clone();
        edit.capabilities = state.draft_capabilities;
        assert!(state.save_preset(edit));
        assert!(state.create_preview("goal", "/workspace"));
        state.templates[0].capabilities[Capability::Shell as usize] = true;
        assert!(
            !state
                .preview
                .as_ref()
                .expect("preview")
                .template
                .capabilities[Capability::Shell as usize]
        );
    }

    #[test]
    fn preset_metadata_save_is_atomic_and_does_not_rename_existing_preview() {
        let mut state = Presentation::new("en-US");
        assert!(state.create_preview("goal", "/workspace"));
        let original = state.templates[0].clone();
        let mut edit = original.clone();
        edit.name = "  ".into();
        edit.description = "unsaved".into();
        assert!(!state.save_preset(edit.clone()));
        assert_eq!(state.templates[0], original);
        edit.name = "My Coding".into();
        assert!(state.save_preset(edit.clone()));
        assert_eq!(state.templates[0], edit);
        assert!(!state.invalid_preset);
        assert_eq!(
            state.preview.as_ref().expect("prior preview").template,
            original
        );
    }

    #[test]
    fn default_preset_applies_only_to_future_forms() {
        let mut state = Presentation::new("en-US");
        state.begin_task();
        assert_eq!(state.preset, Preset::Coding);
        assert!(state.create_preview("goal", "/workspace"));
        let original = state.preview.as_ref().expect("preview").template.clone();
        state.default_preset = Preset::Research;
        assert_eq!(state.preset, Preset::Coding);
        state.set_locale("zh-CN");
        state.begin_task();
        assert_eq!(state.preset, Preset::Research);
        let prior = state.preview.as_ref().expect("prior preview retained");
        assert_eq!(prior.preset, Preset::Coding);
        assert_eq!(prior.template, original);
        state.preset = Preset::General;
        assert_eq!(state.default_preset, Preset::Research);
        state.begin_task();
        assert_eq!(state.preset, Preset::Research);
    }

    #[test]
    fn preview_requires_goal_and_workspace_and_snapshots_preset() {
        let mut state = Presentation::new("en-US");
        assert!(!state.create_preview("  ", "/workspace"));
        assert!(!state.create_preview("goal", " "));
        assert!(state.preview.is_none());
        state.preset = Preset::Research;
        assert!(state.create_preview("研究 Rust", "/workspace"));
        let original_template = state.templates[Preset::Research.index()].clone();
        state.templates[Preset::Research.index()].instructions = "edited later".into();
        state.templates[Preset::Research.index()].models[0] = "fixture/replacement".into();
        state.preset = Preset::Coding;
        state.set_locale("zh-CN");
        let task = state.preview.as_ref().expect("created preview");
        assert_eq!(task.goal, "研究 Rust");
        assert_eq!(task.workspace, "/workspace");
        assert_eq!(task.preset, Preset::Research);
        assert_eq!(task.template, original_template);
        assert!(!state.invalid_form);
        assert_eq!(state.fixture(), &EXECUTION);
    }

    #[test]
    fn task_tabs_survive_locale_changes_without_changing_fixture() {
        let mut state = Presentation::new("en-US");
        for tab in TaskTab::ALL {
            state.tab = tab;
            state.set_locale("zh-CN");
            assert_eq!(state.tab, tab);
            assert_eq!(state.fixture(), &EXECUTION);
            assert_ne!(tab.label(Locale::English), tab.label(Locale::Chinese));
            state.set_locale("en-US");
            assert_eq!(state.tab, tab);
        }
    }

    #[test]
    fn preset_seed_language_never_overwrites_editable_content_on_locale_change() {
        for initial_locale in ["en-US", "zh-CN"] {
            let mut state = Presentation::new(initial_locale);
            assert!(
                state
                    .templates
                    .iter()
                    .all(|template| !template.description.contains(" / "))
            );
            let original = state.templates.clone();
            state.set_locale(if initial_locale == "en-US" {
                "zh-CN"
            } else {
                "en-US"
            });
            assert_eq!(state.templates, original);
            state.templates[0].description = "My edited description / 我的说明".into();
            state.templates[0].instructions = "My instructions".into();
            let edited = state.templates.clone();
            state.set_locale(initial_locale);
            assert_eq!(state.templates, edited);
        }
        assert_eq!(
            Presentation::new("zh-CN").templates[0].description,
            "软件开发、代码修改和验证"
        );
        assert_eq!(
            Presentation::new("unknown").templates,
            Presentation::new("en-US").templates
        );
    }

    #[test]
    fn execution_copy_selects_one_language_without_changing_identity() {
        assert_eq!(
            EXECUTION.goal.text(Locale::English),
            "Implement model retrieval and caching"
        );
        assert_eq!(EXECUTION.goal.text(Locale::Chinese), "实现模型获取与缓存");
        for text in [
            &EXECUTION.goal,
            &EXECUTION.goal_description,
            &EXECUTION.work,
            &EXECUTION.work_description,
        ]
        .into_iter()
        .chain(EXECUTION.stages.iter())
        .chain(EXECUTION.activity.iter().map(|(_, text, _)| text))
        {
            assert_ne!(text.text(Locale::English), text.text(Locale::Chinese));
            for locale in [Locale::English, Locale::Chinese] {
                assert!(!text.text(locale).is_empty());
                assert!(!text.text(locale).contains(" / "));
            }
        }
    }

    #[test]
    fn overview_fixture_has_valid_position_and_change_totals() {
        assert!(EXECUTION.current_stage < EXECUTION.stages.len());
        assert_eq!(
            EXECUTION
                .files
                .iter()
                .map(|(_, added, _)| added)
                .sum::<u32>(),
            184
        );
        assert_eq!(
            EXECUTION
                .files
                .iter()
                .map(|(_, _, removed)| removed)
                .sum::<u32>(),
            21
        );
        assert!(EXECUTION.evidence.starts_with("fixture://"));
    }

    #[test]
    fn locale_switch_preserves_fixture() {
        let mut state = Presentation::new("en-US");
        let original = state.fixture();
        state.set_locale("zh-CN");
        assert_eq!(state.locale.text(Text::Goal), "目标");
        assert_eq!(state.fixture(), original);
        state.set_locale("en-US");
        assert_eq!(state.locale.text(Text::Goal), "Goal");
        assert_eq!(state.fixture(), original);
    }

    #[test]
    fn unknown_locale_falls_back_without_mutating_fixture() {
        let mut state = Presentation::new("zh-CN");
        state.set_locale("unknown");
        assert_eq!(state.locale, Locale::English);
        assert_eq!(state.fixture(), &EXECUTION);
    }

    #[test]
    fn bilingual_catalog_snapshot() {
        let keys = [Text::Title, Text::Goal, Text::Evidence, Text::Draft];
        let english = keys.map(|key| Locale::English.text(key));
        let chinese = keys.map(|key| Locale::Chinese.text(key));
        assert_eq!(
            english,
            ["Talos Desktop Preview", "Goal", "Evidence", "Draft"]
        );
        assert_eq!(chinese, ["Talos 桌面预览", "目标", "证据", "草稿"]);
    }
}
