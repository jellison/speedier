#!/usr/bin/env bash
set -euo pipefail

APP_NAME="Speedier"
APP_ID="${APP_ID:-com.speedier.app}"
INSTALL_DIR="${INSTALL_DIR:-/Applications}"
WORK_DIR="$(pwd)"
SUDO="${SUDO:-auto}"

TMP_DIR="$(mktemp -d)"
ORIGINAL_CARGO_TOML=""
cleanup() {
  if [[ -n "$ORIGINAL_CARGO_TOML" ]]; then
    cp "$ORIGINAL_CARGO_TOML" "$WORK_DIR/Cargo.toml"
  fi
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

if ! cargo bundle --version >/dev/null 2>&1; then
  echo "cargo-bundle is required. Install it with: cargo install cargo-bundle"
  exit 1
fi

if [[ -f "$WORK_DIR/Cargo.toml" ]]; then
  ORIGINAL_CARGO_TOML="$TMP_DIR/Cargo.toml.bak"
  cp "$WORK_DIR/Cargo.toml" "$ORIGINAL_CARGO_TOML"
  python3 - <<PY
import pathlib
import re

path = pathlib.Path("$WORK_DIR/Cargo.toml")
text = path.read_text()
pattern = r'(\n\[package\.metadata\.bundle\][\s\S]*?\nidentifier\s*=\s*")([^"]+)(")'
match = re.search(pattern, text)
if match:
    updated = re.sub(pattern, r'\1' + "$APP_ID" + r'\3', text)
    path.write_text(updated)
PY
fi

echo "Packaging ${APP_NAME}.app..."
cd "$WORK_DIR"
cargo bundle --release --format osx

APP_BUNDLE="$WORK_DIR/target/release/bundle/osx/${APP_NAME}.app"
if [[ ! -d "$APP_BUNDLE" ]]; then
  echo "Expected app bundle not found at $APP_BUNDLE"
  exit 1
fi

mv "$APP_BUNDLE" "$TMP_DIR/"
APP_BUNDLE="$TMP_DIR/${APP_NAME}.app"

TARGET="${INSTALL_DIR}/${APP_NAME}.app"
if [[ "$SUDO" == "auto" ]]; then
  if [[ -w "$INSTALL_DIR" ]]; then
    SUDO=""
  else
    SUDO="sudo"
  fi
elif [[ "$SUDO" == "1" ]]; then
  SUDO="sudo"
else
  SUDO=""
fi

echo "Installing to $TARGET (replacing existing copy)..."
$SUDO rm -rf "$TARGET"
$SUDO cp -R "$APP_BUNDLE" "$TARGET"

cd "$WORK_DIR"
echo "Done. ${APP_NAME} is installed."
