set -e

cargo t --all-features -- --test-threads=1
cargo fmt --check
cargo clippy -- -D warnings