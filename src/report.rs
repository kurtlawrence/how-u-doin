//! The public structures of progress reports.
//!
//! The data structures are serialisable with the `serde` feature.
use std::fmt;

// ###### PROGRESS #############################################################

/// A progress node.
///
/// This structure is serialisable with the `serde` feature.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Progress {
    /// The report status.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub report: Report,

    /// Any sub nodes.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub children: Vec<Progress>,
}

// ###### REPORT ###############################################################

/// The report status.
///
/// This structure is serialisable with the `serde` feature.
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Report {
    /// The report's label.
    pub label: String,

    /// The report's description. Leave empty if not used.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "String::is_empty"))]
    pub desc: String,

    /// The progress state.
    pub state: State,

    /// Accumulation messages.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub accums: Vec<Message>,
}

// ###### STATE ################################################################

/// The state of the progress.
///
/// This structure is serialisable with the `serde` feature.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum State {
    /// The report is in progress.
    InProgress {
        /// Optional length, if empty, the report is indeterminate.
        #[cfg_attr(feature = "serde", serde(default))]
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        len: Option<u64>,

        /// Current report position.
        pos: u64,

        /// The len/pos should be formatted in bytes.
        bytes: bool,

        /// **Seconds** remaining.
        remaining: Option<f32>,
    },
    /// The progress reporter is finished.
    ///
    /// This occurs when [`Tx::finish`] is called.
    ///
    /// [`Tx::finish`]: crate::Tx::finish
    Completed {
        /// Duration, in **seconds**.
        duration: f32,
    },
    /// The progress was cancelled.
    Cancelled,
}

impl Default for State {
    fn default() -> Self {
        State::InProgress {
            len: None,
            pos: 0,
            bytes: false,
            remaining: None,
        }
    }
}

// ###### MESSAGE ##############################################################

/// An accumulation message.
///
/// This structure is serialisable with the `serde` feature.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// The severity of the message.
    pub severity: Severity,
    /// The message.
    pub msg: String,
}

/// Message severity.
///
/// This structure is serialisable with the `serde` feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
pub enum Severity {
    Error,
    Warn,
    #[default]
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warn => write!(f, "WARN"),
            Self::Info => write!(f, "INFO"),
        }
    }
}
