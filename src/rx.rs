use crate::report::{Message, Report, State};

use super::*;
use flume::{unbounded, Receiver, Sender};
use std::{collections::BTreeMap, sync::Arc};

// impl ProgressRx {
//     pub async fn consume<C: Consume>(mut self, mut consumer: C) {
//         while let Some(Msg { id, payload }) = self.rx.recv().await {
//             let report = self.get_or_create_report(id);

//             match payload {
//                 AddReport => (), // the act of get or create handles this
//                 SetLabel(label) => report.label = label,
//                 SetDesc(desc) => report.desc = desc,
//                 SetLen(len) => report.len = len,
//                 SetFmtBytes(x) => report.bytes = x,
//                 Inc(ticks) => report.update_pos(report.pos.saturating_add(ticks)),
//                 SetPos(pos) => report.update_pos(pos),
//                 Finish => report.finished = true,
//                 Close => self.close_report(id),
//             }

//             consumer.recv(self.reports.as_slice());
//         }
//     }

//     /// Type helper for `self.consume(consumer)`.
//     pub async fn consume_fn<F>(self, consumer: F)
//     where
//         F: FnMut(Reports),
//     {
//         self.consume(consumer).await
//     }

//     fn get_or_create_report(&mut self, id: Id) -> &mut Report {
//         let i = *self.ids.entry(id).or_insert_with(|| self.reports.len());

//         if i >= self.reports.len() {
//             self.reports.resize_with(i + 1, || None);
//         }

//         if self.reports[i].is_none() {
//             self.reports[i] = Some(Report::new(format!("Report{:02}", i + 1)));
//         }

//         self.reports[i].as_mut().expect("just checked and created")
//     }

//     /// Mark the report as None.
//     fn close_report(&mut self, id: Id) {
//         if let Some(i) = self.ids.get(&id).copied() {
//             if let Some(x) = self.reports.get_mut(i) {
//                 *x = None;
//             }
//         }
//     }
// }

// impl Report {
//     fn new(label: String) -> Self {
//         Self {
//             label,
//             ..Self::default()
//         }
//     }

//     /// Handles adding a history entry.
// }

pub(crate) fn spawn<C: Consume>(rx: Receiver<Payload>, mut consumer: C) {
    let debounce = consumer.debounce();

    let mut controller = Controller::default();

    loop {
        if rx.is_disconnected() {
            break; // static tx dropped, exit receiver loop
        }

        if debounce.is_zero() {
            if let Ok(x) = rx.recv() {
                controller.process(x, &mut consumer);
            }
        } else {
            std::thread::sleep(debounce);

            if let Some(last) = rx.drain().last() {
                controller.process(last, &mut consumer);
            }
        }
    }
}

#[derive(Default)]
struct Controller {
    ps: BTreeMap<Id, Progress_>,
    roots: Vec<Id>,
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

    fn process<C: Consume>(&mut self, payload: Payload, consumer: &mut C) {
        use Payload::*;

        match payload {
            AddReport(None, tx) => {
                let id = self.next_id();

                match self.last {
                    Some(parent) => self.add_root_or_parent(parent, id),
                    None => self.roots.push(id),
                }

                self.add_rpt(id);

                tx.send(id).ok();
            }

            AddReport(Some(parent), tx) => {
                let id = self.next_id();

                self.add_root_or_parent(parent, id);

                self.add_rpt(id);

                tx.send(id).ok();
            }

            AddRootReport(tx) => {
                let id = self.next_id();

                self.roots.push(id);

                self.add_rpt(id);

                tx.send(id).ok();
            }

            Fetch(tx) => {
                tx.send(self.build_public_prg()).ok();
            }

            SetLabel(id, label) => {
                self.set(id, |x, _| x.label = label);
            }

            SetDesc(id, d) => {
                self.set(id, |x, _| x.desc = d);
            }

            SetLen(id, len) => {
                self.set(id, |x, _| x.set_len(len));
            }

            Inc(id, by) => {
                self.set(id, |x, e| x.inc_pos(by, e));
            }

            SetPos(id, pos) => {
                self.set(id, |x, e| x.update_pos(pos, e));
            }

            SetFmtBytes(id, y) => {
                self.set(id, |x, _| x.set_fmt_as_bytes(y));
            }

            Accum(id, severity, msg) => {
                self.set(id, |x, _| x.accums.push(Message { severity, msg }));
            }

            Finish(id) => {
                self.set(id, |x, e| {
                    x.state = State::Completed {
                        duration: e.as_secs_f32(),
                    }
                });
            }

            Close(id) => {
                self.ps.remove(&id);
                if let Some(idx) = self
                    .roots
                    .iter()
                    .enumerate()
                    .find_map(|(i, x)| id.eq(x).then_some(i))
                {
                    self.roots.remove(idx);
                }

                if self.last == Some(id) {
                    self.last = None;
                }
            }

            Cancel => {
                self.cancelled = true;
            }

            Cancelled(tx) => {
                tx.send(self.cancelled).ok();
            }

            Reset => *self = Self::default(),
        }
    }

    fn add_root_or_parent(&mut self, parent: Id, child: Id) {
        match self.ps.get_mut(&parent) {
            Some(p) => p.children.push(child),
            None => self.roots.push(child),
        }
    }

    fn add_rpt(&mut self, id: Id) {
        self.ps.insert(id, Progress_::new());
        self.last = Some(id);
    }

    fn set<F: FnOnce(&mut Report, Duration)>(&mut self, id: Id, f: F) {
        if let Some(x) = self.ps.get_mut(&id) {
            f(&mut x.rpt, x.started.elapsed())
        }
    }

    fn build_public_prg(&self) -> Vec<Progress> {
        self.roots
            .iter()
            .filter_map(|id| self.build_public_prg_(id))
            .collect()
    }

    fn build_public_prg_(&self, id: &Id) -> Option<Progress> {
        self.ps.get(id).map(
            |Progress_ {
                 rpt,
                 children,
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
    started: Instant,
}

impl Progress_ {
    fn new() -> Self {
        Self {
            rpt: Default::default(),
            children: Default::default(),
            started: Instant::now(),
        }
    }
}

impl Report {
    fn set_len(&mut self, len_: Option<u64>) {
        match &mut self.state {
            State::InProgress { len, .. } => *len = len_,
            _ => (),
        }
    }

    fn set_fmt_as_bytes(&mut self, x: bool) {
        match &mut self.state {
            State::InProgress { bytes, .. } => *bytes = x,
            _ => (),
        }
    }

    fn inc_pos(&mut self, ticks: u64, elapsed: Duration) {
        match &self.state {
            State::InProgress { pos, .. } => self.update_pos(pos.saturating_add(ticks), elapsed),
            _ => (),
        }
    }

    fn update_pos(&mut self, pos_: u64, elapsed: Duration) {
        match &mut self.state {
            State::InProgress {
                len,
                pos,
                remaining,
                ..
            } => {
                *pos = len.clone().map(|len| len.min(pos_)).unwrap_or(pos_);

                if let Some(len) = *len {
                    let rate = elapsed.as_secs_f32() / *pos as f32;
                    *remaining = (len - *pos) as f32 * rate;
                }
            }
            _ => (),
        }
    }
}
