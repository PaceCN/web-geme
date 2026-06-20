#!/usr/bin/env bash
set -eu

export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup not found; installing Rust stable toolchain..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
fi

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

rustup target add wasm32-unknown-unknown

if ! command -v trunk >/dev/null 2>&1; then
  echo "trunk not found; installing Trunk..."
  cargo install trunk
fi

trunk build --release
