use super::*;
use Payload::*;

impl ProgressRx {
    pub async fn consume<C: Consume>(mut self, mut consumer: C) {
        while let Some(Msg { id, payload }) = self.rx.recv().await {
            let report = self.get_or_create_report(id);

            match payload {
                AddReport => (), // the act of get or create handles this
                SetLabel(label) => report.label = label,
                SetDesc(desc) => report.desc = desc,
                SetLen(len) => report.len = len,
                SetFmtBytes(x) => report.bytes = x,
                Inc(ticks) => report.update_pos(report.pos.saturating_add(ticks)),
                SetPos(pos) => report.update_pos(pos),
                Finish => report.finished = true,
                Close => self.close_report(id),
            }

            consumer.recv(self.reports.as_slice());
        }
    }

    /// Type helper for `self.consume(consumer)`.
    pub async fn consume_fn<F>(self, consumer: F)
    where
        F: FnMut(Reports),
    {
        self.consume(consumer).await
    }

    fn get_or_create_report(&mut self, id: Id) -> &mut Report {
        let i = *self.ids.entry(id).or_insert_with(|| self.reports.len());

        if i >= self.reports.len() {
            self.reports.resize_with(i + 1, || None);
        }

        if self.reports[i].is_none() {
            self.reports[i] = Some(Report::new(format!("Report{:02}", i + 1)));
        }

        self.reports[i].as_mut().expect("just checked and created")
    }

    /// Mark the report as None.
    fn close_report(&mut self, id: Id) {
        if let Some(i) = self.ids.get(&id).copied() {
            if let Some(x) = self.reports.get_mut(i) {
                *x = None;
            }
        }
    }
}

impl Report {
    fn new(label: String) -> Self {
        Self {
            label,
            ..Self::default()
        }
    }

    /// Handles adding a history entry.
    fn update_pos(&mut self, pos: u64) {
        let pos = self.len.map(|len| len.min(pos)).unwrap_or(pos);
        let now = std::time::Instant::now();
        self.history.push((now, pos));
        self.pos = pos;
    }
}
