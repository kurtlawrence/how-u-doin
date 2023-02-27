use super::*;
use flume::{bounded, unbounded, Sender};
use parking_lot::{lock_api::RawRwLock, RwLock};
use Payload::*;

static TX: StaticTx = StaticTx::none();

/// Initialise the progress consumer loop.
///
/// Progress reports are only sent through if an `init` call has previously been invoked.
///
/// # Example
/// ```rust
/// howudoin::init(howudoin::consumers::Noop::default());
/// ```
pub fn init<C: Consume + Send + 'static>(consumer: C) {
    let (tx, rx) = unbounded();
    TX.set_tx(tx);

    std::thread::spawn(|| super::rx::spawn(rx, consumer));
}

/// Disable the progress reporting consumer loop, terminating the sender side.
///
/// This is effectively the opposite of [`init`].
pub fn disable() {
    TX.disable()
}

/// Generate a new progress reporter.
///
/// The reporter's parent will be the _last_ report generated (if one exists).
///
/// # Example
/// ```rust
/// let rpt = howudoin::new().label("Progress");
/// ```
pub fn new() -> Tx {
    new_(|x| AddReport(None, x))
}

/// Generate a new progress reporter under a parent.
///
/// # Example
/// ```rust
/// let parent = howudoin::new().label("Parent");
/// let child = howudoin::new_with_parent(parent.id());
/// ```
pub fn new_with_parent(parent: Id) -> Tx {
    new_(|x| AddReport(Some(parent), x))
}

/// Generate a new progress reporter at the root level.
///
/// # Example
/// ```rust
/// let rpt = howudoin::new_root().label("Progress");
/// ```
pub fn new_root() -> Tx {
    new_(AddRootReport)
}

fn new_<F: FnOnce(Sender<Id>) -> Payload>(f: F) -> Tx {
    let (tx, rx) = bounded(1);
    TX.send(|| f(tx));
    let id = rx.recv_timeout(Duration::from_millis(500)).unwrap_or(0);

    Tx { id }
}

/// Fetch the progress report tree.
///
/// The returned structure is a tree of progress reports currently tracked.
/// If no progress has been [`init`]ialised, this will return `None`.
///
/// Note that [`report::Progress`] is serialisable with the `serde` feature.
///
/// # Example
/// ```rust
/// let a = howudoin::new();
/// let b = howudoin::new();
///
/// let progress = howudoin::fetch();
/// ```
pub fn fetch() -> Option<Vec<report::Progress>> {
    let (tx, rx) = bounded(1);
    TX.send(|| Fetch(tx));
    rx.recv_timeout(Duration::from_millis(500)).ok()
}

/// Flag for cancellation.
pub fn cancel() {
    TX.send(|| Cancel);
}

/// Check the cancellation flag.
///
/// If the progress reporter has not been [`init`]ialised, `None` is returned.
pub fn cancelled() -> Option<bool> {
    let (tx, rx) = bounded(1);
    TX.send(|| Cancelled(tx));
    rx.recv_timeout(Duration::from_millis(500)).ok()
}

/// Reset the progress consumer loop.
///
/// This resets all the tracked progress, but keeps the consumer loop alive (as opposed to stopping
/// it with [`disable`]).
/// Note that it is usually preferable to initialise a new loop with a fresh consumer.
pub fn reset() {
    TX.send(|| Reset)
}

pub struct StaticTx(RwLock<Option<Sender<Payload>>>);

impl StaticTx {
    const fn none() -> Self {
        StaticTx(RwLock::const_new(parking_lot::RawRwLock::INIT, None))
    }

    fn set_tx(&self, tx: Sender<Payload>) {
        *self.0.write() = Some(tx);
    }

    fn disable(&self) {
        *self.0.write() = None;
    }

    fn send<F: FnOnce() -> Payload>(&self, payload: F) {
        match &*self.0.read() {
            Some(tx) if !tx.is_disconnected() => tx.send(payload()).ok(),
            _ => Some(()),
        };
    }
}

/// The progress reporter transmitter.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Tx {
    id: Id,
}

impl Tx {
    /// The distinct ID.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Set the label/name of the report.
    ///
    /// ```rust
    /// howudoin::new().label("Report A");
    /// ```
    pub fn label<L: Into<String>>(self, label: L) -> Self {
        TX.send(|| SetLabel(self.id, label.into()));
        self
    }

    /// Set an upper bound on the progress.
    ///
    /// If the progress is indeterminate, `None` can be specified.
    ///
    /// ```rust
    /// howudoin::new().set_len(100);
    /// ```
    pub fn set_len<L: Into<Option<u64>>>(self, len: L) -> Self {
        TX.send(|| SetLen(self.id, len.into()));
        self
    }

    /// Flag to format the position as bytes.
    pub fn fmt_as_bytes(self, fmt_as_bytes: bool) -> Self {
        TX.send(|| SetFmtBytes(self.id, fmt_as_bytes));
        self
    }

    /// Set the report message.
    ///
    /// ```rust
    /// let a = howudoin::new();
    /// a.desc("processing");
    /// ```
    pub fn desc<D: Into<String>>(&self, desc: D) -> &Self {
        TX.send(|| SetDesc(self.id, desc.into()));
        self
    }

    /// Increment the report 1 position.
    ///
    /// ```rust
    /// let a = howudoin::new();
    /// a.inc();
    /// ```
    pub fn inc(&self) -> &Self {
        TX.send(|| Inc(self.id, 1));
        self
    }

    /// Increment the report position by `delta`.
    ///
    /// ```rust
    /// let a = howudoin::new();
    /// a.inc_by(5_u8);
    /// ```
    pub fn inc_by<P: Into<u64>>(&self, delta: P) -> &Self {
        TX.send(|| Inc(self.id, delta.into()));
        self
    }

    /// Set the report position.
    ///
    /// ```rust
    /// let a = howudoin::new();
    /// a.set_pos(5_u8);
    /// ```
    pub fn set_pos<P: Into<u64>>(&self, pos: P) -> &Self {
        TX.send(|| SetPos(self.id, pos.into()));
        self
    }

    /// Add an error message.
    ///
    /// ```rust
    /// let a = howudoin::new();
    /// a.add_err("fail!");
    /// ```
    pub fn add_err<M: Into<String>>(&self, msg: M) -> &Self {
        self.add_accum(report::Severity::Error, msg)
    }

    /// Add an warning message.
    ///
    /// ```rust
    /// let a = howudoin::new();
    /// a.add_warn("careful...");
    /// ```
    pub fn add_warn<M: Into<String>>(&self, msg: M) -> &Self {
        self.add_accum(report::Severity::Warn, msg)
    }

    /// Add an information message.
    ///
    /// ```rust
    /// let a = howudoin::new();
    /// a.add_info("hello");
    /// ```
    pub fn add_info<M: Into<String>>(&self, msg: M) -> &Self {
        self.add_accum(report::Severity::Info, msg)
    }

    /// Add an accumulation message.
    ///
    /// These messages are accumulated against a progress report, and consumers can display them
    /// for additional information.
    pub fn add_accum<M: Into<String>>(&self, severity: report::Severity, msg: M) -> &Self {
        TX.send(|| Accum(self.id, severity, msg.into()));
        self
    }

    /// Check if the consumer loop has been flagged for cancellation.
    ///
    /// It is up to the producer to decide what to do if cancellation is detected.
    pub fn cancelled(&self) -> bool {
        cancelled().unwrap_or(false)
    }

    /// Mark this report as finished but should be kept displayed.
    pub fn finish(self) {
        TX.send(|| Finish(self.id))
    }

    /// Mark this report as finished and should be removed from display.
    pub fn close(self) {
        TX.send(|| Close(self.id))
    }
}
