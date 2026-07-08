# NHL Legacy Input Automation

Virtual Xbox controller input automation for NHL Legacy Edition.

Creates a virtual Xbox One controller via Linux `uinput` and replays user-provided
scripts (written in [Rhai](https://rhai.rs)) against it.

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
