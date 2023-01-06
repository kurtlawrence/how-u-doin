
// pub mod consumers;
// mod report;
// mod rx;
// mod tx;
mod progress;

pub type Id = u64;


// #[derive(Clone)]
// pub struct ProgressTx(UnboundedSender<Msg>);
// 
// pub struct ProgressRx {
//     rx: UnboundedReceiver<Msg>,
//     reports: Vec<Option<Report>>,
//     ids: HashMap<Id, usize>,
// }
// 
// #[derive(Clone)]
// pub struct Reporter {
//     tx: ProgressTx,
//     id: Id,
// }

#[derive(Default, Clone, Debug)]
pub struct Report {
    /// The report's label.
    pub label: String,

    /// The report's description. Leave empty if not used.
    pub desc: String,

    /// Optional length, if empty report is indeterminate.
    pub len: Option<u64>,

    /// Current report position.
    pub pos: u64,

    /// The len/pos should be formatted in bytes.
    pub bytes: bool,

    /// History of position updates, with timestamps snapshots.
    pub history: Vec<(std::time::Instant, u64)>,

    /// Report has finished.
    pub finished: bool,
}

// pub struct ReportChgs {
//     pub label: bool,
//     pub desc: bool,
//     pub len: bool,
//     pub pos: bool,
//     pub bytes: bool,
//     pub history: bool,
// }
// 
// struct Msg {
//     id: Id,
//     payload: Payload,
// }

enum Payload {
    /// Add a new reporter.
    AddReport,
    /// Set the label.
    SetLabel(String),
    /// Set the description.
    SetDesc(String),
    /// Set the progress length. If `None`, this progress is indeterminate.
    SetLen(Option<u64>),
    /// Set whether to format the length and position as bytes.
    SetFmtBytes(bool),
    /// Increment the progress position by a number of ticks.
    Inc(u64),
    /// Set the progress position.
    SetPos(u64),
    /// Reporter has finished, but should be kept displayed.
    Finish,
    /// Reporter has finished and should be removed from display.
    Close,
}

pub trait Consume {
    // fn recv(&mut self, reports: Reports);
}

// impl<F> Consume for F
// where
//     F: FnMut(Reports),
// {
//     fn recv(&mut self, reports: Reports) {
//         self(reports)
//     }
// }
