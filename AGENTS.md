# Agent Instructions (Speedier)

## Overview
- Speedier is a native macOS calculator built with Rust + GPUI.
- Entry point: `src/main.rs`.
- UI: `src/app.rs` (window construction, layouts, theme, syntax highlighting).
- Evaluation/history: `src/calc/`.
- Packaging assets: `assets/icon.png`.

## Common commands
- Run locally: `cargo run`
- Build: `make build`
- Package/install `.app`: `make install` (wraps `scripts/install-macos.sh`)

## Verification
- Run `make build` after code changes to catch compile errors (including type mismatches).
- If you touched packaging or app metadata, run `make install` to verify the bundle still builds.

## Packaging notes
- The installer script uses `cargo-bundle` (`cargo bundle --release --format osx`).
- Default app id is `com.speedier.app`; override via `APP_ID=...`.
- Install location defaults to `/Applications`; override via `INSTALL_DIR=...`.
