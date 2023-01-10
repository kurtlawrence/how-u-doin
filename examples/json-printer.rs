use std::{thread::*, time::Duration};

fn main() {
    // initialise the JsonPrinter consumer
    howudoin::init(howudoin::consumers::JsonPrinter::default());

    // build a new parent reporter
    let parent_rpt = howudoin::new().label("Parent").set_len(5);

    for i in 1u8..=5 {
        parent_rpt.set_pos(i);
        parent_rpt.desc(format!("Processing child {i}"));

        // add a child reporter
        let child = howudoin::new_with_parent(parent_rpt.id())
            .label("Child")
            .set_len(100);

        for _ in 0..100 {
            sleep(Duration::from_millis(50));
            child.inc();
        }

        child.close(); // close the child down

        parent_rpt.add_info(format!("Finished child {i}"));
    }

    parent_rpt.finish();

    sleep(Duration::from_millis(100));
}
