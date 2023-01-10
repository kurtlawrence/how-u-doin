use std::fmt;

// ###### PROGRESS #############################################################

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Progress {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub report: Report,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub children: Vec<Progress>,
}

// ###### REPORT ###############################################################

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

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum State {
    InProgress {
        /// Optional length, if empty report is indeterminate.
        #[cfg_attr(feature = "serde", serde(default))]
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        len: Option<u64>,

        /// Current report position.
        pos: u64,

        /// The len/pos should be formatted in bytes.
        bytes: bool,

        /// **Seconds** remaining.
        remaining: f32,
    },
    Completed {
        /// Duration, in **seconds**.
        duration: f32,
    },
    Cancelled,
}

impl Default for State {
    fn default() -> Self {
        State::InProgress {
            len: None,
            pos: 0,
            bytes: false,
            remaining: f32::INFINITY,
        }
    }
}

// ###### MESSAGE ##############################################################

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    pub severity: Severity,
    pub msg: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
