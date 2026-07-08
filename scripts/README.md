# run-nhl-legacy.sh

Runs `nhllegacy.exe` via Proton 11.0 from the command line.

## Usage

```bash
INSTALL_DIR=/path/to/NHL\ Legacy\ Recomp ./scripts/run-nhl-legacy.sh
```

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `INSTALL_DIR` | **Yes** | — | Directory containing `nhllegacy.exe` |
| `STEAM_DIR` | No | `$HOME/.steam/steam` | Steam installation root |
| `PROTON_DIR` | No | `$STEAM_DIR/steamapps/common/Proton 11.0` | Proton tool directory |
| `STEAM_COMPAT_DATA_PATH` | No | `$STEAM_DIR/steamapps/compatdata/3623314720` | Wine prefix location |
