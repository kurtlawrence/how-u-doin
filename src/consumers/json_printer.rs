use crate::*;
use std::{io::Write, time::Duration};

pub struct JsonPrinter {
    debounce: Duration,
}

impl Default for JsonPrinter {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(500),
        }
    }
}

impl Consume for JsonPrinter {
    fn debounce(&self) -> Duration {
        self.debounce
    }

    fn rpt(&mut self, _: &report::Report, _: Id, _: Option<Id>, controller: &Controller) {
        let p = controller.build_progress_tree();
        let mut stdout = std::io::stdout().lock();
        if serde_json::to_writer(&mut stdout, &p).is_ok() {
            writeln!(stdout).ok();
        }
    }
}
