#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(pwd)"
EMSDK_DIR="$ROOT_DIR/emsdk"

if [[ ! -d "$EMSDK_DIR" ]]; then
  git clone https://github.com/emscripten-core/emsdk.git "$EMSDK_DIR"
fi

pushd "$EMSDK_DIR" >/dev/null

./emsdk install latest
./emsdk activate latest

popd >/dev/null

echo "Installed emsdk to $EMSDK_DIR"
