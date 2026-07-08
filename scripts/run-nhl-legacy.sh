#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:?Must set INSTALL_DIR to the game directory}"

cd "$INSTALL_DIR"

if [[ -x ./nhllegacy ]] && file ./nhllegacy | grep -q "ELF"; then
  exec ./nhllegacy "$@"
fi

STEAM_DIR="${STEAM_DIR:-$HOME/.steam/steam}"
PROTON_DIR="${PROTON_DIR:-$STEAM_DIR/steamapps/common/Proton 11.0}"
COMPAT_DATA="${STEAM_COMPAT_DATA_PATH:-$STEAM_DIR/steamapps/compatdata/3623314720}"

export STEAM_COMPAT_CLIENT_INSTALL_PATH="$STEAM_DIR"
export STEAM_COMPAT_DATA_PATH="$COMPAT_DATA"
mkdir -p "$COMPAT_DATA"

PROTON_USE_WINED3D=0 "$PROTON_DIR/proton" run ./nhllegacy.exe "$@"
