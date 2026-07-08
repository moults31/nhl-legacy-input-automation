# build
cargo build --workspace

# lint
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings

# test
cargo test --workspace

# run (needs uinput permissions, see docs/setup.md)
cargo run -- --script scripts/examples/spam-a-start.rhai
