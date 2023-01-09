use super::*;

#[cfg(feature = "termline")]
mod term_line;

#[cfg(feature = "termline")]
pub use term_line::TermLine;

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
}
