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

# list windows to find the right --window-substring:
cargo run -- --list-windows

# screenshot: choose the right --window-substring (see docs/window-discovery.md)
# one-shot screenshot (saves flat to screenshots/<label>.png):
cargo run -- --screenshot my_label
# with custom window substring:
cargo run -- --script foo.rhai --window-substring "nhllegacy"

# watch mode: continuously update screenshots/latest.png:
cargo run -- --script smoke.rhai --watch screenshots/latest.png
# (view with: feh --reload 2 screenshots/latest.png)

# kill all NHL game processes (Proton launcher):
./scripts/kill-nhl.sh

# NOTE: the Proton-launched game spawns a process tree:
#   python3 .../proton run ./nhllegacy.exe
#   └── steam.exe ./nhllegacy.exe
#       └── nhllegacy.exe
# Kill all three to fully stop the game.

# system deps for screenshot support (xcap):
# Debian/Ubuntu: sudo apt install libxcb1-dev libxrandr-dev libdbus-1-dev libwayland-dev libegl-dev libpipewire-0.3-dev libclang-dev libgbm-dev
# Fedora: sudo dnf install libxcb-devel libXrandr-devel dbus-devel wayland-devel mesa-libEGL-devel pipewire-devel
