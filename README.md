# how-u-doin

Progress reporting abstraction for Rust

A progress reporting and consuming abstraction.

`howudoin` intends to make producing and consuming progress reports simple and ergonomic.

```rust
// initialise a consumer loop
howudoin::init(howudoin::consumers::Noop::default());

let rpt = howudoin::new().label("Progress").set_len(10);

for _ in 0..10 {
    rpt.inc(); // increment the progress
    // check for cancellation
    if rpt.cancelled() {
        break;
    }
}

rpt.finish(); // finalise progress

// fetch the tree of progress
let progress = howudoin::fetch();
```

Features:
- Lightweight
- Unobtrusive interface
- Nestable reports
- Automatic timers
- Message accumulation
- Cancellation

## Progress Reporting

Producing a progress report can be done anywhere in code without any references.

```rust
// creates a new report
let rpt = howudoin::new();
// creates a report below `rpt`
let child = howudoin::new_with_parent(rpt.id());
// creates a report at the root
let rpt2 = howudoin::new_root();

// progress reporting
let rpt = rpt
    .label("Label") // set a label/name
    .set_len(1000); // progress is bounded

rpt.desc("processing"); // progress message
for i in 1_u32..=1000 {
    rpt.inc();      // increment progress position
    rpt.set_pos(i); // set progress position
}

rpt.finish(); // finished a report
rpt.close();  // close a report from display
```

## Progress Display

Progress display is abstracted from the producer.
A display mechanism implements the [`Consume`] trait, and is sent to the consumer loop with
[`init`].
There exist a few predefined consumers in the [`consumers`] module, which are feature gated.
Consumers are generally defined for mechanisms that are _invoked_.

```rust
// initialise a term-line consumer
howudoin::init(howudoin::consumers::TermLine::default());
```

## Progress Consumption

Progress reports can also be _requested_ from the consumer loop.
This pattern is used when a progress update is _requested_ from elsewhere (for
example, a REST API).

```rust
// initialise a no-op consumer
howudoin::init(howudoin::consumers::Noop::default());

// fetch the progress tree
let progress = howudoin::fetch();
```

## Opt-in

Progress reports are only sent to a consumer if the consumer loop has been initialised.
In situations where the loop has not been initialised, progress reporting is a very cheap void
operation.
This means producers can be neatly separated from consumers.
