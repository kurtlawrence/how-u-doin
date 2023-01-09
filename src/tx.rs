use super::*;
use flume::{bounded, unbounded, Receiver, Sender};
use parking_lot::RwLock;
use Payload::*;

static TX: StaticTx = StaticTx::none();

pub fn init<C: Consume + Send + 'static>(consumer: C) {
    let (tx, rx) = unbounded();
    TX.set_tx(tx);

    std::thread::spawn(|| super::rx::spawn(rx, consumer));
}

pub fn disable() {
    TX.disable()
}

pub fn new() -> Tx {
    new_(|x| AddReport(None, x))
}

pub fn new_with_parent(parent: Id) -> Tx {
    new_(|x| AddReport(Some(parent), x))
}

pub fn new_root() -> Tx {
    new_(|x| AddRootReport(x))
}

fn new_<F: FnOnce(Sender<Id>) -> Payload>(f: F) -> Tx {
    let (tx, rx) = bounded(1);
    TX.send(|| f(tx));
    let id = rx.recv_timeout(Duration::from_millis(500)).unwrap_or(0);

    Tx { id }
}

pub fn fetch() -> Option<Vec<report::Progress>> {
    let (tx, rx) = bounded(1);
    TX.send(|| Fetch(tx));
    rx.recv_timeout(Duration::from_millis(500)).ok()
}

pub fn cancel() {
    TX.send(|| Cancel);
}

pub fn cancelled() -> Option<bool> {
    let (tx, rx) = bounded(1);
    TX.send(|| Cancelled(tx));
    rx.recv_timeout(Duration::from_millis(500)).ok()
}

pub fn reset() {
    TX.send(|| Reset)
}

pub struct StaticTx(RwLock<Option<Sender<Payload>>>);

impl StaticTx {
    const fn none() -> Self {
        StaticTx(RwLock::new(None))
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Tx {
    id: Id,
}

impl Tx {
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn label<L: Into<String>>(self, label: L) -> Self {
        TX.send(|| SetLabel(self.id, label.into()));
        self
    }

    pub fn set_len<L: Into<Option<u64>>>(self, len: L) -> Self {
        TX.send(|| SetLen(self.id, len.into()));
        self
    }

    pub fn fmt_as_bytes(self, fmt_as_bytes: bool) -> Self {
        TX.send(|| SetFmtBytes(self.id, fmt_as_bytes));
        self
    }

    pub fn desc<D: Into<String>>(&self, desc: D) -> &Self {
        TX.send(|| SetDesc(self.id, desc.into()));
        self
    }

    pub fn inc(&self) -> &Self {
        TX.send(|| Inc(self.id, 1));
        self
    }

    pub fn inc_by<P: Into<u64>>(&self, delta: P) -> &Self {
        TX.send(|| Inc(self.id, delta.into()));
        self
    }

    pub fn set_pos<P: Into<u64>>(&self, pos: P) -> &Self {
        TX.send(|| SetPos(self.id, pos.into()));
        self
    }

    pub fn add_error<M: Into<String>>(&self, msg: M) -> &Self {
        self.add_accum(report::Severity::Error, msg)
    }

    pub fn add_warn<M: Into<String>>(&self, msg: M) -> &Self {
        self.add_accum(report::Severity::Warn, msg)
    }

    pub fn add_info<M: Into<String>>(&self, msg: M) -> &Self {
        self.add_accum(report::Severity::Info, msg)
    }

    pub fn add_accum<M: Into<String>>(&self, severity: report::Severity, msg: M) -> &Self {
        TX.send(|| Accum(self.id, severity, msg.into()));
        self
    }

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
