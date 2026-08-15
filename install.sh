#!/bin/sh
set -eu

cd "$(dirname "$0")"

target="$(uname -m)-unknown-linux-musl"
bin_dir="${HOME}/.local/bin"

rustup target add "$target"
cargo build --release --target "$target" --manifest-path apps/devcontainerex/Cargo.toml

mkdir -p "$bin_dir"
install -m 755 "apps/devcontainerex/target/${target}/release/devcontainerex" "$bin_dir/devcontainerex"
echo "installed: ${bin_dir}/devcontainerex"
