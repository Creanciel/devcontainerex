#!/bin/bash

this_directory="$(cd $(dirname ${BASH_SOURCE:-$0}); pwd)"
project_name='devcontainerex'
command_path="${this_directory}/cli/commands"

code_workspace="devcontainerex.code-workspace"

_help() {
  echo "Command List"
  echo "    edit|code : Open in VSCode"
  echo "    example   : Run example command"
  echo ""
}

_devcontainerex() {
  script_name=""
  command="$1"
  [ $# -gt 0 ] && shift
  case "$command" in
    edit | code ) script_name="$command_path/edit.sh" ;;
    example ) script_name="$command_path/example.sh" ;;
    *) _help; exit 0 ;;
  esac

  [ -n "$script_name" ] || exit 1

  PROJECT_NAME="$project_name" \
  PROJECT_ROOT="$this_directory" \
  CODE_WORKSPACE="$code_workspace" \
    "$script_name" "$@"
}

_devcontainerex "$@"
