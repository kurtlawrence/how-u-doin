# how-u-doin

Progress reporting abstraction for Rust

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

Take a look at the [examples](https://github.com/kdr-aus/how-u-doin/tree/main/examples) for example consumers.

- [API Docs](https://docs.rs/howudoin)

## Example terminal consumer

![Screenshot from 2023-01-10 15-36-43](https://user-images.githubusercontent.com/13831379/211470141-2879f70a-42b5-49ad-894a-3a0bb9c57bac.png)

## Support

Please help support this project by [sponsoring 💗](https://github.com/sponsors/kdr-aus)
