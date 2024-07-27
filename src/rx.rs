use super::*;
use crate::{
    flat_tree::FlatTree,
    report::{Message, Report, State},
};
use flume::Receiver;
use std::collections::BTreeSet;
use Payload::*;

pub(crate) fn spawn<C: Consume>(rx: Receiver<Payload>, mut consumer: C) {
    let debounce = consumer.debounce();

    let mut controller = Controller::default();
    let mut chgd_buf = BTreeSet::new();
    let mut last = Instant::now();

    loop {
        if rx.is_disconnected() {
            break; // static tx dropped, exit receiver loop
        }

        // use a timeout to avoid thrashing the loop
        let x = if debounce.is_zero() {
            rx.recv().ok()
        } else {
            rx.recv_timeout(debounce).ok()
        };

        if let Some(x) = x.and_then(|x| controller.process(x)) {
            chgd_buf.insert(x);
        }

        if last.elapsed() >= debounce {
            // debounce duration has occurred; can update the consumer with any changes

            while let Some(id) = chgd_buf.pop_first() {
                if let Some(Progress_ {
                    rpt,
                    children: _,
                    parent,
                    started: _,
                }) = controller.ps.get(&id)
                {
                    consumer.rpt(rpt, id, *parent, &controller);
                } else {
                    consumer.closed(id);
                }
            }

            last = Instant::now();
        }
    }
}

/// The progress consumer loop controller.
#[derive(Default)]
pub struct Controller {
    ps: FlatTree<Id, Progress_>,
    last: Option<Id>,
    cancelled: bool,
    nextid: Id,
}

impl Controller {
    fn next_id(&mut self) -> Id {
        let id = self.nextid;
        self.nextid = self.nextid.wrapping_add(1);
        id
    }

    fn process(&mut self, payload: Payload) -> Option<Id> {
        match payload {
            AddReport(None, tx) => {
                let id = match self.last {
                    Some(parent) => self.add_child(parent),
                    None => self.add_root(),
                };

                tx.send(id).ok();
                Some(id)
            }

            AddReport(Some(parent), tx) => {
                let id = self.add_child(parent);
                tx.send(id).ok();
                Some(id)
            }

            AddRootReport(tx) => {
                let id = self.add_root();
                tx.send(id).ok();
                Some(id)
            }

            Fetch(tx) => {
                tx.send(self.build_progress_tree()).ok();
                None
            }

            SetLabel(id, label) => {
                self.set(id, |x, _| x.label = label);
                Some(id)
            }

            SetDesc(id, d) => {
                self.set(id, |x, _| x.desc = d);
                Some(id)
            }

            SetLen(id, len) => {
                self.set(id, |x, _| x.set_len(len));
                Some(id)
            }

            Inc(id, by) => {
                self.set(id, |x, e| x.inc_pos(by, e));
                Some(id)
            }

            SetPos(id, pos) => {
                self.set(id, |x, e| x.update_pos(pos, e));
                Some(id)
            }

            SetFmtBytes(id, y) => {
                self.set(id, |x, _| x.set_fmt_as_bytes(y));
                Some(id)
            }

            Accum(id, severity, msg) => {
                self.set(id, |x, _| x.accums.push(Message { severity, msg }));
                Some(id)
            }

            Finish(id) => {
                self.set(id, |x, e| {
                    x.state = State::Completed {
                        duration: e.as_secs_f32(),
                    }
                });

                // if finished, do not keep around as a parent
                if self.last == Some(id) {
                    self.last = None;
                }

                Some(id)
            }

            Close(id) => {
                self.ps.remove(&id);

                if self.last == Some(id) {
                    self.last = None;
                }

                Some(id)
            }

            Cancel => {
                self.cancelled = true;
                None
            }

            Cancelled(tx) => {
                tx.send(self.cancelled).ok();
                None
            }

            Reset => {
                *self = Self::default();
                None
            }
        }
    }

    fn add_root(&mut self) -> Id {
        let id = self.next_id();
        self.ps.insert_root(
            id,
            Progress_ {
                parent: None,
                ..Progress_::root()
            },
        );
        self.last = Some(id);
        id
    }

    fn add_child(&mut self, parent: Id) -> Id {
        let id = self.next_id();
        match self.ps.get_mut(&parent) {
            Some(p) => {
                p.children.push(id);
                self.ps.insert(
                    id,
                    Progress_ {
                        parent: Some(parent),
                        ..Progress_::root()
                    },
                );
            }
            None => {
                self.ps.insert_root(id, Progress_::root());
            }
        }

        self.last = Some(id);
        id
    }

    fn set<F: FnOnce(&mut Report, Duration)>(&mut self, id: Id, f: F) {
        if let Some(x) = self.ps.get_mut(&id) {
            f(&mut x.rpt, x.started.elapsed())
        }
    }

    /// Build the progress tree.
    ///
    /// This is utilised by [`fetch`].
    pub fn build_progress_tree(&self) -> Vec<Progress> {
        self.ps
            .roots()
            .filter_map(|(id, _)| self.build_public_prg_(id))
            .collect()
    }

    fn build_public_prg_(&self, id: &Id) -> Option<Progress> {
        self.ps.get(id).map(
            |Progress_ {
                 rpt,
                 children,
                 parent: _,
                 started: _,
             }| {
                let children = children
                    .iter()
                    .filter_map(|id| self.build_public_prg_(id))
                    .collect();

                Progress {
                    report: rpt.clone(),
                    children,
                }
            },
        )
    }
}

struct Progress_ {
    rpt: Report,
    children: Vec<Id>,
    parent: Option<Id>,
    started: Instant,
}

impl Progress_ {
    fn root() -> Self {
        Self {
            rpt: Default::default(),
            children: Default::default(),
            parent: None,
            started: Instant::now(),
        }
    }
}

impl Report {
    fn set_len(&mut self, len_: Option<u64>) {
        if let State::InProgress { len, .. } = &mut self.state {
            *len = len_
        }
    }

    fn set_fmt_as_bytes(&mut self, x: bool) {
        if let State::InProgress { bytes, .. } = &mut self.state {
            *bytes = x
        }
    }

    fn inc_pos(&mut self, ticks: u64, elapsed: Duration) {
        if let State::InProgress { pos, .. } = &self.state {
            self.update_pos(pos.saturating_add(ticks), elapsed)
        }
    }

    fn update_pos(&mut self, pos_: u64, elapsed: Duration) {
        if let State::InProgress {
            len,
            pos,
            remaining,
            ..
        } = &mut self.state
        {
            *pos = len.map(|len| len.min(pos_)).unwrap_or(pos_);

            if let Some(len) = *len {
                let rate = elapsed.as_secs_f32() / *pos as f32;
                *remaining = Some((len - *pos) as f32 * rate);
            }
        }
    }
}
