# Window Discovery for Screenshots

The screenshot tool (`ScreenCaptureObserver`) uses `xcap` to enumerate all open windows
and find the game window by case-insensitive substring matching on the window title.

The matching substring is set via the `--window-substring` CLI flag (default: `"NHL"`)
or by passing it to `ScreenCaptureObserver::new()`.

## Known Working Substrings

| Substring | Launcher | Confirmed | Notes |
|-----------|----------|-----------|-------|
| `NHL` | Proton / Steam (NHL Legacy Recomp) | Yes | Default. Matches Proton window title. |
| `nhllegacy` | Native ELF binary | Expected | Falls back to the binary name. |

## How to discover a new substring

```bash
# List all open window titles
python3 -c "
import subprocess
out = subprocess.check_output(['xdotool', 'search', '', 'getwindowname', '%1']).decode().strip().split('\n')
print('\n'.join(out))
"
```

Or with `xcap` directly, you can add a debug print in `ScreenCaptureObserver::find_window()` to see all window titles.

Update this file when you find a working substring for a new launcher configuration.
