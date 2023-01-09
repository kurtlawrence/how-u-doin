use flume::{Receiver, Sender};
use report::Progress;
use std::time::{Duration, Instant};

pub mod consumers;
pub mod report;
mod rx;
#[cfg(test)]
mod tests;
mod tx;

pub type Id = usize;

pub use tx::{cancel, cancelled, disable, fetch, init, new, new_root, new_with_parent, reset};

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

pub trait Consume {
    /// Set the debounce timeout.
    ///
    /// Defaults to 50 milliseconds. This is the time waited for before processing new messages.
    /// Only the **last** message is considered.
    fn debounce(&self) -> Duration {
        Duration::from_millis(50)
    }
}
