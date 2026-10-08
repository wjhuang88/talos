//! Versioned, host-bound browser executor protocol types.
//!
//! This module deliberately contains no browser transport or process integration.  It provides
//! the closed vocabulary used by the later admission and permission layers.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod request;
pub use request::{BrowserRequest, BrowserRequestError};

/// The separately negotiated frame-aware browser protocol version.
pub const BROWSER_EXECUTOR_V2: &str = "talos.browser.executor/v2";

/// Opaque identity for a browser resource. Values are identifiers, not authority.
macro_rules! browser_ref {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::try_new(value).map_err(serde::de::Error::custom)
            }
        }

        impl $name {
            /// Creates an opaque reference after enforcing the wire-format bounds.
            pub fn try_new(value: impl Into<String>) -> Result<Self, String> {
                let value = value.into();
                if value.is_empty() || value.len() > 128 {
                    return Err("browser reference must contain 1..=128 bytes".to_owned());
                }
                if !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
                {
                    return Err("browser reference contains an invalid character".to_owned());
                }
                Ok(Self(value))
            }
        }
    };
}

browser_ref!(
    BrowserSessionRef,
    "Opaque identity for a browser session generation."
);
browser_ref!(
    BrowserTabRef,
    "Opaque identity for a tab bound to a session generation."
);
browser_ref!(
    BrowserFrameRef,
    "Opaque identity for a frame bound to a document generation."
);
browser_ref!(
    BrowserSnapshotRef,
    "Opaque identity for a snapshot bound to one frame generation."
);
browser_ref!(
    BrowserElementRef,
    "Opaque identity for an element bound to one snapshot generation."
);

/// Closed v2 operation vocabulary. Unknown values fail deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserOperation {
    Open,
    Read,
    Snapshot,
    CurrentUrl,
    Screenshot,
    Click,
    Fill,
    Select,
    Hover,
    Check,
    Uncheck,
    Press,
    Scroll,
    WaitForElement,
    WaitMilliseconds,
    TabNew,
    TabList,
    TabClose,
    TabSwitch,
    WindowClose,
    FrameTree,
}

/// Permission resource class for an admitted browser operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserResourceClass {
    Navigate,
    Observe,
    Interact,
    SessionWait,
    SessionCreate,
    SessionInventory,
    TabLifecycle,
    SessionClose,
    FrameInventory,
}

impl BrowserOperation {
    /// Returns the dedicated browser permission resource class.
    pub const fn resource_class(self) -> BrowserResourceClass {
        match self {
            Self::Open => BrowserResourceClass::Navigate,
            Self::Read
            | Self::Snapshot
            | Self::CurrentUrl
            | Self::Screenshot
            | Self::WaitForElement => BrowserResourceClass::Observe,
            Self::Click
            | Self::Fill
            | Self::Select
            | Self::Hover
            | Self::Check
            | Self::Uncheck
            | Self::Press
            | Self::Scroll => BrowserResourceClass::Interact,
            Self::WaitMilliseconds => BrowserResourceClass::SessionWait,
            Self::TabNew => BrowserResourceClass::SessionCreate,
            Self::TabList => BrowserResourceClass::SessionInventory,
            Self::TabClose | Self::TabSwitch => BrowserResourceClass::TabLifecycle,
            Self::WindowClose => BrowserResourceClass::SessionClose,
            Self::FrameTree => BrowserResourceClass::FrameInventory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_operation_vocabulary_round_trips() {
        let json = serde_json::to_string(&BrowserOperation::WaitForElement).unwrap();
        assert_eq!(json, "\"wait-for-element\"");
        assert_eq!(
            serde_json::from_str::<BrowserOperation>(&json).unwrap(),
            BrowserOperation::WaitForElement
        );
    }

    #[test]
    fn unknown_operation_is_rejected() {
        assert!(serde_json::from_str::<BrowserOperation>("\"evaluate-script\"").is_err());
    }

    #[test]
    fn operation_resource_classes_are_deterministic() {
        assert_eq!(
            BrowserOperation::Open.resource_class(),
            BrowserResourceClass::Navigate
        );
        assert_eq!(
            BrowserOperation::Click.resource_class(),
            BrowserResourceClass::Interact
        );
        assert_eq!(
            BrowserOperation::TabNew.resource_class(),
            BrowserResourceClass::SessionCreate
        );
        assert_eq!(
            BrowserOperation::FrameTree.resource_class(),
            BrowserResourceClass::FrameInventory
        );
    }

    #[test]
    fn opaque_refs_serialize_as_values_without_exposing_structure() {
        let session = BrowserSessionRef::try_new("session-generation-1").unwrap();
        assert_eq!(
            serde_json::to_string(&session).unwrap(),
            "\"session-generation-1\""
        );
    }

    #[test]
    fn opaque_refs_reject_invalid_wire_values() {
        assert!(BrowserSessionRef::try_new("").is_err());
        assert!(BrowserSessionRef::try_new("has space").is_err());
        assert!(BrowserSessionRef::try_new("é").is_err());
        assert!(serde_json::from_str::<BrowserSessionRef>("\"bad/ref\"").is_err());
    }
}
