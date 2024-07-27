//! Simple println example of implement own `Consume`r.
use howudoin::{report::*, *};
use std::{thread::*, time::Duration};

struct SimplePrinter;

impl Consume for SimplePrinter {
    fn rpt(&mut self, report: &Report, _id: Id, _parent: Option<Id>, _controller: &Controller) {
        let Report {
            label,
            desc,
            state,
            accums: _,
        } = report;

        print!("{label}: {desc} ");
        match state {
            State::InProgress {
                len,
                pos,
                bytes: _,
                remaining,
            } => {
                let done = *pos as f32 / len.unwrap_or(0) as f32 * 100.;
                let remaining = remaining.unwrap_or(f32::INFINITY);
                println!("{done:.1}% - eta {remaining}s");
            }
            State::Completed { duration } => {
                println!("finished in {duration} seconds");
            }
            State::Cancelled => {
                println!("cancelled")
            }
        }
    }
}

fn main() {
    // initialise the consumer
    howudoin::init(SimplePrinter);

    let rpt = howudoin::new().label("Progress").set_len(20);

    for _ in 0..20 {
        sleep(Duration::from_millis(500));
        rpt.inc();
    }

    rpt.finish();
}
