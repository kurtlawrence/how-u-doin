//! A progress reporting and consuming abstraction.
//!
//! `howudoin` intends to make producing and consuming progress reports simple and ergonomic.
//!
//! ```rust
//! // initialise a consumer loop
//! howudoin::init(howudoin::consumers::Noop::default());
//!
//! let rpt = howudoin::new().label("Progress").set_len(10);
//!
//! for _ in 0..10 {
//!     rpt.inc(); // increment the progress
//!     // check for cancellation
//!     if rpt.cancelled() {
//!         break;
//!     }
//! }
//!
//! rpt.finish(); // finalise progress
//!
//! // fetch the tree of progress
//! let progress = howudoin::fetch();
//! ```
//!
//! Features:
//! - Lightweight
//! - Unobtrusive interface
//! - Nestable reports
//! - Automatic timers
//! - Message accumulation
//! - Cancellation
//!
//! ## Progress Reporting
//!
//! Producing a progress report can be done anywhere in code without any references.
//!
//! ```rust
//! // creates a new report
//! let rpt = howudoin::new();
//! // creates a report below `rpt`
//! let child = howudoin::new_with_parent(rpt.id());
//! // creates a report at the root
//! let rpt2 = howudoin::new_root();
//!
//! // progress reporting
//! let rpt = rpt
//!     .label("Label") // set a label/name
//!     .set_len(1000); // progress is bounded
//!
//! rpt.desc("processing"); // progress message
//! for i in 1_u32..=1000 {
//!     rpt.inc();      // increment progress position
//!     rpt.set_pos(i); // set progress position
//! }
//!
//! rpt.finish(); // finished a report
//! rpt.close();  // close a report from display
//! ```
//!
//! ## Progress Display
//!
//! Progress display is abstracted from the producer.
//! A display mechanism implements the [`Consume`] trait, and is sent to the consumer loop with
//! [`init`].
//! There exist a few predefined consumers in the [`consumers`] module, which are feature gated.
//! Consumers are generally defined for mechanisms that are _invoked_.
//!
//! ```rust
//! // initialise a term-line consumer
//! howudoin::init(howudoin::consumers::TermLine::default());
//! ```
//!
//! ## Progress Consumption
//!
//! Progress reports can also be _requested_ from the consumer loop.
//! This pattern is used when a progress update is _requested_ from elsewhere (for
//! example, a REST API).
//!
//! ```rust
//! // initialise a no-op consumer
//! howudoin::init(howudoin::consumers::Noop::default());
//!
//! // fetch the progress tree
//! let progress = howudoin::fetch();
//! ```
//!
//! ## Opt-in
//!
//! Progress reports are only sent to a consumer if the consumer loop has been initialised.
//! In situations where the loop has not been initialised, progress reporting is a very cheap void
//! operation.
//! This means producers can be neatly separated from consumers.
#![warn(missing_docs)]

use flume::Sender;
use report::Progress;
use std::time::{Duration, Instant};

pub mod consumers;
pub mod flat_tree;
pub mod report;
mod rx;
#[cfg(test)]
mod tests;
mod tx;

/// A report identifier.
///
/// For a consumer instantiation, identifiers are distinct, increasing counters.
/// Note that the counter will wrap around.
pub type Id = usize;

pub use rx::Controller;
pub use tx::{cancel, cancelled, disable, fetch, init, new, new_root, new_with_parent, reset, Tx};

#[derive(Debug)]
enum Payload {
    /// Add a new reporter, optionally under the parent.
    AddReport(Option<Id>, Sender<Id>),
    /// Add a new root report.
    AddRootReport(Sender<Id>),
    /// Fetch the progress history.
    Fetch(Sender<Vec<report::Progress>>),
    /// Set the label.
    SetLabel(Id, String),
    /// Set the description.
    SetDesc(Id, String),
    /// Set the progress length. If `None`, this progress is indeterminate.
    SetLen(Id, Option<u64>),
    /// Set whether to format the length and position as bytes.
    SetFmtBytes(Id, bool),
    /// Increment the progress position by a number of ticks.
    Inc(Id, u64),
    /// Set the progress position.
    SetPos(Id, u64),
    /// Add an accumulation message.
    Accum(Id, report::Severity, String),
    /// Reporter has finished, but should be kept displayed.
    Finish(Id),
    /// Reporter has finished and should be removed from display.
    Close(Id),
    /// Set cancellation flag to true.
    Cancel,
    /// Get the cancellation status.
    Cancelled(Sender<bool>),
    /// Reset the controller's state.
    Reset,
}

/// A report consumer.
///
/// A consumer is required when initialising progress reporting.
/// They can be defined on local types, and there are a few predined consumers in the [`consumers`]
/// module.
///
/// When defining a consumer, it is recommended to take a look at the predefined ones and the
/// [examples].
///
/// Note that a consumer is _invoked_. If a _requester_ of progress is required, [`fetch`] should
/// be used.
///
/// [examples]: https://github.com/kdr-aus/how-u-doin/tree/main/examples
pub trait Consume {
    /// Set the debounce timeout.
    ///
    /// Defaults to 50 milliseconds.
    /// This is the time waited for before invoking the [`Consume::rpt`] or [`Consume::closed`].
    /// The debounce allows reports to be fully updated before being displayed, avoiding flogging
    /// the consumer.
    fn debounce(&self) -> Duration {
        Duration::from_millis(50)
    }

    /// Invoked when reports are updated in some way.
    ///
    /// `rpt` is only invoked if there have been changes, and after [`Consume::debounce`].
    /// The `report` has the meat of the progress, while there are identifiers to see where the
    /// report lands in the tree.
    fn rpt(&mut self, report: &report::Report, id: Id, parent: Option<Id>, controller: &Controller);

    /// The report with `id` was closed, indicating it should be _removed from display_.
    ///
    /// The default implementation is to do nothing.
    fn closed(&mut self, _id: Id) {}
}
