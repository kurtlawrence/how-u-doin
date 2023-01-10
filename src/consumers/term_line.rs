use crate::*;
use indicatif::*;
use report::*;

/// A terminal line consumer.
///
/// Backended by [`indicatif`], this consumer will create a progress bars for each available report.
/// It provides a simple line interface.
///
/// [`indicatif`]: https://github.com/console-rs/indicatif
pub struct TermLine {
    debounce: Duration,
    bars: flat_tree::FlatTree<Id, ProgressBar>,
    mp: MultiProgress,
}

impl Consume for TermLine {
    fn debounce(&self) -> Duration {
        self.debounce
    }

    fn rpt(&mut self, rpt: &report::Report, id: Id, parent: Option<Id>, _: &Controller) {
        match self.bars.get(&id) {
            Some(x) => update_bar(x, rpt),
            None => update_bar(&self.add_bar(id, parent), rpt),
        };
    }

    fn closed(&mut self, id: Id) {
        if let Some(bar) = self.bars.remove(&id) {
            bar.finish_and_clear();
            self.mp.remove(&bar);
        }
    }
}

impl TermLine {
    /// Create a new, default, `TermLine`.
    pub fn new() -> Self {
        Self {
            debounce: Duration::from_millis(50),
            mp: MultiProgress::new(),
            bars: Default::default(),
        }
    }

    /// Create a new `TermLine` with the debounce duration.
    pub fn with_debounce(debounce: Duration) -> Self {
        Self {
            debounce,
            ..Self::new()
        }
    }

    fn add_bar(&mut self, id: Id, parent: Option<Id>) -> ProgressBar {
        match parent.and_then(|x| self.bars.get(&x)).cloned() {
            None => {
                let bar = self.mp.add(pb());
                self.bars.insert_root(id, bar.clone());
                bar
            }
            Some(parent) => {
                let bar = self.mp.insert_after(&parent, pb());
                self.bars.insert(id, bar.clone());
                bar
            }
        }
    }
}

impl Default for TermLine {
    fn default() -> Self {
        Self::new()
    }
}

fn update_bar(pb: &ProgressBar, rpt: &Report) {
    let Report {
        label,
        desc,
        state,
        accums,
    } = rpt;

    pb.set_prefix(label.clone());
    pb.set_message(desc.clone());

    match state {
        State::InProgress {
            len,
            pos,
            bytes,
            remaining: _,
        } => {
            pb.set_length(len.unwrap_or(!0));
            pb.set_position(*pos);
            match len.is_some() {
                true => pb.set_style(bar_style(*bytes)),
                false => pb.set_style(spinner_style(*bytes)),
            }
        }

        State::Completed { duration } => {
            pb.finish_with_message(format!(
                "finished in {}",
                HumanDuration(Duration::try_from_secs_f32(*duration).unwrap_or_default())
            ));
        }

        State::Cancelled => {
            pb.abandon_with_message("cancelled");
        }
    }

    for Message { severity, msg } in accums {
        pb.println(format!("{severity}: {msg}"));
    }
}

fn pb() -> ProgressBar {
    let pb = ProgressBar::hidden().with_style(spinner_style(false));
    pb.enable_steady_tick(std::time::Duration::from_millis(250));
    pb
}

fn spinner_style(fmt_bytes: bool) -> ProgressStyle {
    let tmp = if fmt_bytes {
        format!(
            " {} {}: {} {} {}",
            SPINNER, PREFIX, BYTES, BYTES_PER_SEC, MSG
        )
    } else {
        format!(" {} {}: {} {}", SPINNER, PREFIX, POS, MSG)
    };
    ProgressStyle::default_bar()
        .template(&tmp)
        .expect("template should be fine")
        .progress_chars("=> ")
        .tick_chars(r#"|/-\|"#)
}

fn bar_style(fmt_bytes: bool) -> ProgressStyle {
    let tmp = if fmt_bytes {
        format!(
            " {} {} {} {}
 {} {} ({}/{}) {}",
            SPINNER, PREFIX, BYTES_PER_SEC, ETA, BAR, PCT, BYTES, BYTES_TOTAL, MSG
        )
    } else {
        format!(
            " {} {} {}
 {} {} ({}/{}) {}",
            SPINNER, PREFIX, ETA, BAR, PCT, POS, LEN, MSG
        )
    };

    ProgressStyle::default_bar()
        .template(&tmp)
        .expect("template should be fine")
        .progress_chars("=> ")
        .tick_chars(r#"|/-\|"#)
}

const SPINNER: &str = "{spinner:.red.bold}";
const PREFIX: &str = "{prefix:.cyan.bold}";
const BYTES: &str = "{bytes}";
const BYTES_TOTAL: &str = "{total_bytes}";
const BYTES_PER_SEC: &str = "<{binary_bytes_per_sec:.yellow.bold}>";
const POS: &str = "{pos}";
const LEN: &str = "{len}";
const ETA: &str = "({eta:.green.bold.italic})";
const BAR: &str = "[{bar:30}]";
const PCT: &str = "{percent:>03}%";
const MSG: &str = "{wide_msg:.cyan}";
