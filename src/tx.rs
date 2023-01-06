use super::*;
use Payload::*;

impl ProgressTx {
    pub fn add(&self) -> Reporter {
        let id = rand::random();
        self.send(id, AddReport);

        Reporter {
            id,
            tx: self.clone(),
        }
    }

    fn send(&self, id: Id, payload: Payload) {
        self.0
            .send(Msg { id, payload })
            .log_warn("failed msg send, swallowing error");
    }
}

impl Reporter {
    fn send(&self, payload: Payload) {
        self.tx.send(self.id, payload)
    }

    pub fn label<L: Into<String>>(self, label: L) -> Self {
        self.send(SetLabel(label.into()));
        self
    }

    pub fn set_len<L: Into<Option<u64>>>(self, len: L) -> Self {
        self.send(SetLen(len.into()));
        self
    }

    pub fn fmt_as_bytes(self, fmt_as_bytes: bool) -> Self {
        self.send(SetFmtBytes(fmt_as_bytes));
        self
    }

    pub fn desc<D: Into<String>>(&self, desc: D) -> &Self {
        self.send(SetDesc(desc.into()));
        self
    }

    pub fn inc(&self) -> &Self {
        self.send(Inc(1));
        self
    }

    pub fn inc_by<P: Into<u64>>(&self, delta: P) -> &Self {
        self.send(Inc(delta.into()));
        self
    }

    pub fn set_pos<P: Into<u64>>(&self, pos: P) -> &Self {
        self.send(SetPos(pos.into()));
        self
    }

    /// Mark this report as finished but should be kept displayed.
    pub fn finish(self) {
        self.send(Finish)
    }

    /// Mark this report as finished and should be removed from display.
    pub fn close(self) {
        self.send(Close)
    }
}
