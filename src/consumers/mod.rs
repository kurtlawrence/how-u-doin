//! Predefined implementors of [`Consume`].
//!
//! Consumers are feature gated.
use super::*;

/// A terminal line progress display. Requires `term-line` feature.
#[cfg(feature = "term-line")]
pub mod term_line;
#[cfg(feature = "term-line")]
pub use term_line::TermLine;

/// Prints progress tree as JSON to stdout. Requires `json-printer` feature.
#[cfg(feature = "json-printer")]
pub mod json_printer;
#[cfg(feature = "json-printer")]
pub use json_printer::JsonPrinter;

/// A consumer that does not do anything.
///
/// The inner duration is the debounce duration.
/// This consumer is useful for accumulating progress messages and fetching them at a
/// later time.
pub struct Noop(pub Duration);

impl Default for Noop {
    fn default() -> Self {
        Noop(Duration::from_millis(50))
    }
}

impl Consume for Noop {
    fn debounce(&self) -> Duration {
        self.0
    }

    fn rpt(&mut self, _: &report::Report, _: Id, _: Option<Id>, _: &Controller) {}
}
