# policy: if sudo is needed and `sudo -n` fails, stop and ask the user to run it.
# never try `sudo -S` or similar workarounds.

# pre-commit hooks (runs fmt, clippy, doc on commit; test on push)
# one-time setup: python3 -m pip install pre-commit && pre-commit install && pre-commit install --hook-type pre-push
# run all hooks on all files: pre-commit run --all-files --hook-stage pre-push

# build
cargo build --workspace

# lint
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings

# test
cargo test --workspace

# run (needs uinput permissions, see docs/setup.md)
cargo run -- --script scripts/examples/spam-a-start.rhai

# screenshot: choose the right --window-substring (see docs/window-discovery.md)
# one-shot screenshot:
cargo run -- --screenshot my_label
# with custom window substring:
cargo run -- --script foo.rhai --window-substring "NHL"

# system deps for screenshot support (xcap):
# Debian/Ubuntu: sudo apt install libxcb1-dev libxrandr-dev libdbus-1-dev libwayland-dev libegl-dev libpipewire-0.3-dev libclang-dev libgbm-dev
# Fedora: sudo dnf install libxcb-devel libXrandr-devel dbus-devel wayland-devel mesa-libEGL-devel pipewire-devel
