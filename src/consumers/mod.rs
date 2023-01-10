use super::*;

#[cfg(feature = "term-line")]
pub mod term_line;
#[cfg(feature = "term-line")]
pub use term_line::TermLine;

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

impl Consume for Noop {
    fn debounce(&self) -> Duration {
        self.0
    }

    fn rpt(&mut self, _: &report::Report, _: Id, _: Option<Id>, _: &Controller) {}
}
