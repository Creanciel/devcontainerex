#!/bin/bash
this_directory="$(cd "$(dirname "${BASH_SOURCE:-$0}")" && pwd)"
project_directory="$this_directory/../.."

_edit() {
  # select editor
  editor="${CROSS_EDITOR:-code}"
  if ! command -v "$editor" &> /dev/null; then
    echo "Error: '$editor' is not installed or not in PATH" >&2
    return 1
  fi

  # check to open by code-workspace
  if [ "$editor" = 'code' ] && [ -n "$CODE_WORKSPACE" ] ; then
    code_workspace="$project_directory/$CODE_WORKSPACE"
    if [ -f "$code_workspace" ]; then
      "$editor" "$code_workspace"
      return $?
    fi
  fi

  "$editor" "$project_directory"
}

_edit "$@"
