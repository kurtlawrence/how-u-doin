use std::{io::*, thread::*, time::Duration};

fn main() {
    // spawn a stdin reader to check for cancellation
    spawn_cancellation_detector();

    // initialise the TermLine consumer
    howudoin::init(howudoin::consumers::TermLine::new());

    // build a new parent reporter
    let parent_rpt = howudoin::new().label("Parent").set_len(5);

    for i in 1u8..=5 {
        if matches!(howudoin::cancelled(), Some(true)) {
            break;
        }

        parent_rpt.set_pos(i);
        parent_rpt.desc(format!("Processing child {i}"));

        // add a child reporter
        let child = howudoin::new_with_parent(parent_rpt.id())
            .label("Child")
            .set_len(100);
        child.desc("press Enter to cancel");

        for _ in 0..100 {
            sleep(Duration::from_millis(50));
            child.inc();

            if child.cancelled() {
                child.desc("cancelling...");
            }
        }

        child.close(); // close the child down

        parent_rpt.add_info(format!("Finished child {i}"));
    }

    parent_rpt.finish();

    sleep(Duration::from_millis(100));
}

fn spawn_cancellation_detector() {
    spawn(move || loop {
        if stdin().read(&mut []).is_ok() {
            // flag for cancellation on enter press
            howudoin::cancel();
        }
    });
}
