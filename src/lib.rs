
// pub mod consumers;
pub mod report;
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
