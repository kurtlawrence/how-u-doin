set -e

cargo c --no-default-features
cargo c
cargo c --features=serde
cargo c --features=term-line
cargo c --features=json-printer
cargo c --all-features
cargo c --all-features --all-targets
