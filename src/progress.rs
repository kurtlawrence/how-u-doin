use crate::{Payload, Consume, report::Report};
use parking_lot::RwLock;
use flume::{Sender, Receiver, unbounded};
use std::sync::{Arc, atomic::AtomicUsize};

pub static TX: Tx = Tx::none();
static ID: AtomicUsize = AtomicUsize::new(0);

pub fn init<C: Consume + Send + 'static>(consumer: C) {
    let (tx, rx) = unbounded();
    TX.set_tx(tx);

    std::thread::spawn(|| spawn(rx, consumer));
}

fn spawn<C: Consume>(rx: Receiver<Payload>, consumer: C) {

}

pub struct Tx(RwLock<Option<Sender<Payload>>>);

impl Tx {
    const fn none() -> Self {
        Tx(RwLock::new(None))
    }

    fn set_tx(&self, tx: Sender<Payload>) {
        *self.0.write() = Some(tx);
    }

    pub fn send(&self, payload: Payload) {
        match &*self.0.read() {
            Some(tx) => tx.send(payload).ok(),
            None => Some(()),
        };
    }
}

pub enum Progress {
    InProgress {
        rpt: Report,
        children: Vec<Progress>
    },
    Completed {
    }
}

