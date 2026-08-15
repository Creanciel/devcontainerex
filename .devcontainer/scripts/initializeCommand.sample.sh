#!/bin/sh
# Runs on the host before the container is created.
# Copied to initializeCommand.sh by devcontainer.json; edit that copy, it is gitignored.
set -e

cd "$(dirname "$0")/.."

# Bind mount sources must exist on the host beforehand, or Docker creates them as directories.
mkdir -p "$HOME/.claude"
[ -f "$HOME/.claude.json" ] || touch "$HOME/.claude.json"

[ -f docker/Dockerfile ] || cp docker/Dockerfile.sample docker/Dockerfile
[ -f .env ] || cp .env.sample .env
[ -f docker/custom.compose.yaml ] || cp docker/custom.sample.compose.yaml docker/custom.compose.yaml
[ -f claude/settings.json ] || cp claude/settings.sample.json claude/settings.json

# Lifecycle scripts
[ -f scripts/postStartCommand.sh ] || cp scripts/postStartCommand.sample.sh scripts/postStartCommand.sh
