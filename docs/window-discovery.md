# Window Discovery for Screenshots

The screenshot tool (`ScreenCaptureObserver`) uses `xcap` to enumerate all open windows
and find the game window by case-insensitive substring matching on the window title.

The matching substring is set via the `--window-substring` CLI flag (default: `"NHL"`)
or by passing it to `ScreenCaptureObserver::new()`.

## Known Working Substrings

| Substring | Launcher | Confirmed | Notes |
|-----------|----------|-----------|-------|
| `NHL` | Proton / Steam (NHL Legacy Recomp) | Yes | Matches Proton window title. |
| `nhllegacy` | Proton (NHL Legacy Recomp) | Yes | Matches the exe name in the Proton render window. |

## How to discover a new substring

Use the built-in `--list-windows` flag to list all visible window titles:

```sh
cargo run -- --list-windows
```

Look for a title that uniquely identifies the game window and pass it as
`--window-substring`.

If the matched title is too generic (e.g., just `"Proton"`) and captures the
wrong window, choose a more specific part of the title.

The selected window title is logged at `info` level on every screenshot capture,
so you can verify the right window is being captured.
