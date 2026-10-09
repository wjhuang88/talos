//! Typed normalized browser commands. Wire admission remains in request.rs.
use super::{
    BrowserElementRef, BrowserFrameRef, BrowserOperation, BrowserSnapshotRef, BrowserTabRef,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Closed structural command payload; construction alone does not validate semantic bounds.
/// Use `BrowserRequest::parse_raw` for admission. Sensitive input has no Debug projection.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "operation",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum BrowserCommand {
    /// Open operation with explicit target scope.
    Open {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated url.
        url: String,
        /// Validated visibility.
        visibility: Visibility,
    },
    /// Read operation with explicit target scope.
    Read {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
    },
    /// Snapshot operation with explicit target scope.
    Snapshot {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
    },
    /// CurrentUrl operation with explicit target scope.
    CurrentUrl {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
    },
    /// Screenshot operation with explicit target scope.
    Screenshot {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
    },
    /// Click operation with explicit target scope.
    Click {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated snapshot ref.
        snapshot_ref: BrowserSnapshotRef,
        /// Validated element ref.
        element_ref: BrowserElementRef,
    },
    /// Hover operation with explicit target scope.
    Hover {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated snapshot ref.
        snapshot_ref: BrowserSnapshotRef,
        /// Validated element ref.
        element_ref: BrowserElementRef,
    },
    /// Check operation with explicit target scope.
    Check {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated snapshot ref.
        snapshot_ref: BrowserSnapshotRef,
        /// Validated element ref.
        element_ref: BrowserElementRef,
    },
    /// Uncheck operation with explicit target scope.
    Uncheck {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated snapshot ref.
        snapshot_ref: BrowserSnapshotRef,
        /// Validated element ref.
        element_ref: BrowserElementRef,
    },
    /// WaitForElement operation with explicit target scope.
    WaitForElement {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated snapshot ref.
        snapshot_ref: BrowserSnapshotRef,
        /// Validated element ref.
        element_ref: BrowserElementRef,
    },
    /// Fill operation with explicit target scope.
    Fill {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated snapshot ref.
        snapshot_ref: BrowserSnapshotRef,
        /// Validated element ref.
        element_ref: BrowserElementRef,
        /// Validated text.
        text: String,
    },
    /// Select operation with explicit target scope.
    Select {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated snapshot ref.
        snapshot_ref: BrowserSnapshotRef,
        /// Validated element ref.
        element_ref: BrowserElementRef,
        /// Validated option.
        option: String,
    },
    /// Press operation with explicit target scope.
    Press {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated key.
        key: PressKey,
    },
    /// Scroll operation with explicit target scope.
    Scroll {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
        /// Validated frame ref.
        frame_ref: BrowserFrameRef,
        /// Validated direction.
        direction: Direction,
        /// Validated amount.
        amount: u16,
    },
    /// WaitMilliseconds operation with explicit target scope.
    WaitMilliseconds {
        /// Validated milliseconds.
        milliseconds: u16,
    },
    /// TabClose operation with explicit target scope.
    TabClose {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
    },
    /// TabSwitch operation with explicit target scope.
    TabSwitch {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
    },
    /// FrameTree operation with explicit target scope.
    FrameTree {
        /// Validated tab ref.
        tab_ref: BrowserTabRef,
    },
    /// TabNew operation with explicit target scope.
    TabNew {},
    /// TabList operation with explicit target scope.
    TabList {},
    /// WindowClose operation with explicit target scope.
    WindowClose {},
}
impl BrowserCommand {
    /// Returns the command discriminator.
    pub const fn operation(&self) -> BrowserOperation {
        match self {
            Self::Open { .. } => BrowserOperation::Open,
            Self::Read { .. } => BrowserOperation::Read,
            Self::Snapshot { .. } => BrowserOperation::Snapshot,
            Self::CurrentUrl { .. } => BrowserOperation::CurrentUrl,
            Self::Screenshot { .. } => BrowserOperation::Screenshot,
            Self::Click { .. } => BrowserOperation::Click,
            Self::Hover { .. } => BrowserOperation::Hover,
            Self::Check { .. } => BrowserOperation::Check,
            Self::Uncheck { .. } => BrowserOperation::Uncheck,
            Self::WaitForElement { .. } => BrowserOperation::WaitForElement,
            Self::Fill { .. } => BrowserOperation::Fill,
            Self::Select { .. } => BrowserOperation::Select,
            Self::Press { .. } => BrowserOperation::Press,
            Self::Scroll { .. } => BrowserOperation::Scroll,
            Self::WaitMilliseconds { .. } => BrowserOperation::WaitMilliseconds,
            Self::TabClose { .. } => BrowserOperation::TabClose,
            Self::TabSwitch { .. } => BrowserOperation::TabSwitch,
            Self::FrameTree { .. } => BrowserOperation::FrameTree,
            Self::TabNew { .. } => BrowserOperation::TabNew,
            Self::TabList { .. } => BrowserOperation::TabList,
            Self::WindowClose { .. } => BrowserOperation::WindowClose,
        }
    }
}
/// Closed Visibility vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Background.
    Background,
    /// Foreground.
    Foreground,
}
/// Closed Direction vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Up.
    Up,
    /// Down.
    Down,
    /// Left.
    Left,
    /// Right.
    Right,
}
/// Closed PressKey vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PressKey {
    /// Enter.
    Enter,
    /// Tab.
    Tab,
    /// Escape.
    Escape,
    /// Space.
    Space,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// ArrowUp.
    ArrowUp,
    /// ArrowDown.
    ArrowDown,
    /// ArrowLeft.
    ArrowLeft,
    /// ArrowRight.
    ArrowRight,
    /// Home.
    Home,
    /// End.
    End,
    /// PageUp.
    PageUp,
    /// PageDown.
    PageDown,
}
