use crate::*;
use report::*;
use indicatif::*;
use std::{thread::*, time::*};

const MP_STATE_DELAY: Duration = Duration::from_millis(50);

/// A terminal line consumer.
///
/// Backended by `indicatif`, this consumer will create a progress bars for each available report.
/// It provides a simple line interface.
pub struct TermLine {
    debounce: Duration,
    bars: BTreeMap<Id, ProgressBar>,
    mp: MultiProgress,
}

impl Consume for TermLine {
    fn debounce(&self) -> Duration {
        self.debounce
    }
}

impl TermLine {
    pub fn new() -> Self {
        Self {
            debounce: Duration::from_millis(50),
            mp: MultiProgress::new(),
        }
    }

    pub fn with_debounce(debounce: Duration) -> Self {
        Self {
            debounce,
            ..Self::new()
        }
    }

    /// Returns if the multiprogress that has been spawned matches the report structure.
    fn mp_matches(&self, reports: Reports) -> bool {
        fn same_state<T, U>((a, b): (&Option<T>, &Option<U>)) -> bool {
            // XOR so a.is_none() ^ b.is_some()
            //               true ^ true  --> false
            //              false ^ true  --> true
            //               true ^ false --> true
            //              false ^ false --> false
            a.is_some() ^ b.is_none()
        }

        self.bars.len() == reports.len() && self.bars.iter().zip(reports.iter()).all(same_state)
    }

    fn rebuild_and_spawn_mp(&mut self, reports: Reports) {
        // if < delay since last rebuild, need to delay to allow for MP state to catch up.
        let elapsed = self.last_rebuild.elapsed();
        if elapsed < MP_STATE_DELAY {
            sleep(MP_STATE_DELAY - elapsed);
        }

        self.last_rebuild = Instant::now();

        self.finish_and_clear();
        self.build(reports);
    }

    fn build(&mut self, reports: Reports) {
        self.bars.clear();
        self.cache.clear();

        for report in reports {
            self.bars.push(report.as_ref().map(|_| self.mp.add(pb())));
            self.cache.push(Default::default());
        }
    }

    fn finish_and_clear(&mut self) {
        // finish all the avaialable progress bars
        for pb in self.bars.drain(..).flatten() {
            pb.finish();
            self.mp.remove(&pb);
        }

        self.mp.clear().ok();
    }

    fn process_report(&mut self, idx: usize, report: &Report) {
        let mut r = std::mem::take(&mut self.cache[idx]); // take report

        let pb = &self.bars[idx].as_ref().expect("bar should exist");

        let chgs = report.chg_set(&r);

        // process changes
        if chgs.label {
            r.label = report.label.clone();
            pb.set_prefix(r.label.clone());
        }
        if chgs.desc {
            r.desc = report.desc.clone();
            pb.set_message(r.desc.clone());
        }
        if chgs.len {
            r.len = report.len;
            pb.set_length(r.len.unwrap_or(!0));
            set_style(&r, pb);
        }
        if chgs.pos {
            r.pos = report.pos;
            pb.set_position(r.pos);
        }
        if chgs.bytes {
            r.bytes = report.bytes;
            set_style(&r, pb);
        }

        if report.finished || r.finished {
            pb.finish();
            r.finished = true;
        }

        self.cache[idx] = r; // put back
    }
}

impl Default for TermLine {
    fn default() -> Self {
        Self::new()
    }
}

fn set_style(report: &Report, pb: &ProgressBar) {
    match report.len.is_some() {
        true => pb.set_style(bar_style(report.bytes)),
        false => pb.set_style(spinner_style(report.bytes)),
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
