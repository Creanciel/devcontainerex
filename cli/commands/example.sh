#!/bin/bash
set -e

this_directory="$(cd "$(dirname "${BASH_SOURCE:-$0}")" && pwd)"
project_directory="$this_directory/../.."

_check_environment() {
  if [ -z "${!1}" ]; then
    echo "Environment variable '$1' is undefined." >&2
    exit 1
  fi
}

_check_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Command '$1' is not installed." >&2
    exit 1
  fi
}

_example() {
  echo 'unimplemented' >&2
}

_example "$@"
