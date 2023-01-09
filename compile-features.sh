set -e

cargo c --no-default-features
cargo c
cargo c --features=serde
cargo c --all-features