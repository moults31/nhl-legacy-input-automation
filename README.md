# NHL Legacy Input Automation

[![CI](https://github.com/moults31/nhl-legacy-input-automation/actions/workflows/ci.yml/badge.svg)](https://github.com/moults31/nhl-legacy-input-automation/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)
[![Audit](https://github.com/moults31/nhl-legacy-input-automation/actions/workflows/audit.yml/badge.svg)](https://github.com/moults31/nhl-legacy-input-automation/actions/workflows/audit.yml)

Virtual Xbox controller input automation for NHL Legacy Edition.

Creates a virtual Xbox One controller via the Linux `uinput` kernel interface and replays user-provided scripts (written in [Rhai](https://rhai.rs)) against it.

> **Warning:** the Rust implementation in this repository was written entirely by AI. See [AI_DISCLOSURE.md](AI_DISCLOSURE.md).

## Quickstart

1. Set up uinput permissions (one-time): see [docs/setup.md](docs/setup.md)
2. Run the example spam script:

```sh
cargo run -- --script scripts/examples/spam-a-start.rhai
```

## Scripting

See [docs/scripting.md](docs/scripting.md) for the full API.

Minimal example — tap A and Start forever:

```rhai
loop {
    tap("a");
    wait(1.0);
    tap("start");
    wait(1.0);
}
```

## Architecture

```
rhai script → script engine → controller trait → uinput (Linux evdev)
                                        ↑
                                 observer trait (feedback)
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
