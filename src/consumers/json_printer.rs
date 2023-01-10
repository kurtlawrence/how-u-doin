use crate::*;
use std::{io::Write, time::Duration};

/// Prints progress tree as JSON to stdout. Requires `json-printer` feature.
///
/// The inner value is the debounce duration.
///
/// To see an example using `JsonPrinter`, `cargo run --all-features --example json-printer` can be run
/// in the repository.
pub struct JsonPrinter(pub Duration);

impl Default for JsonPrinter {
    fn default() -> Self {
        JsonPrinter(Duration::from_millis(500))
    }
}

impl Consume for JsonPrinter {
    fn debounce(&self) -> Duration {
        self.0
    }

    fn rpt(&mut self, _: &report::Report, _: Id, _: Option<Id>, controller: &Controller) {
        let p = controller.build_progress_tree();
        let mut stdout = std::io::stdout().lock();
        if serde_json::to_writer(&mut stdout, &p).is_ok() {
            writeln!(stdout).ok();
        }
    }
}
