# Scripting

Automation scripts are written in [Rhai](https://rhai.rs), a Rust-native
embedded scripting language.

## Controller functions

| Function | Description |
|---|---|
| `press(button)` | Press and hold a button |
| `release(button)` | Release a button |
| `tap(button)` | Press + release with a 16 ms gap |
| `hold(button, secs)` | Press, wait `secs`, release |
| `wait(secs)` | Sleep for `secs` seconds |
| `set_axis(axis, value)` | Set an axis to `value` |
| `set_stick(side, x, y)` | Set a thumbstick position |

## Button names

`"a"`, `"b"`, `"x"`, `"y"`, `"start"`, `"back"` (or `"select"`),
`"left_bumper"` (or `"lb"`), `"right_bumper"` (or `"rb"`),
`"left_thumb"` (or `"l3"`), `"right_thumb"` (or `"r3"`),
`"guide"` (or `"xbox"`),
`"dpad_up"` (or `"up"`), `"dpad_down"` (or `"down"`),
`"dpad_left"` (or `"left"`), `"dpad_right"` (or `"right"`)

## Axis names

`"left_stick_x"` (or `"lsx"`), `"left_stick_y"` (or `"lsy"`),
`"right_stick_x"` (or `"rsx"`), `"right_stick_y"` (or `"rsy"`),
`"left_trigger"` (or `"lt"`), `"right_trigger"` (or `"rt"`),
`"dpad_x"` (or `"dx"`), `"dpad_y"` (or `"dy"`)

## Axis values

- **Triggers**: 0.0 (released) to 1.0 (fully pressed)
- **Sticks**: -1.0 (left/down) to 1.0 (right/up), 0.0 (centered)
- **D-pad**: -1.0 / 0.0 / 1.0

## Examples

### Basic button spam

```rhai
loop {
    tap("a");
    wait(1.0);
}
```

### Move left stick to top-right

```rhai
set_stick("left", 1.0, -1.0);
wait(0.5);
set_stick("left", 0.0, 0.0);
```

### Full trigger pull

```rhai
set_axis("rt", 1.0);
wait(0.1);
set_axis("rt", 0.0);
```
