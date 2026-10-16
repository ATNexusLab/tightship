plugin_dir="$(cd "$(dirname "$0")/.." && pwd)"
export TIGHTSHIP_ROOT="${TIGHTSHIP_ROOT:-${CURSOR_PLUGIN_ROOT:-$plugin_dir}}"
export TIGHTSHIP_DATA="${TIGHTSHIP_DATA:-{{path:data}}}"
export TIGHTSHIP_SETTINGS="${TIGHTSHIP_SETTINGS:-{{path:config}}/cli-config.json}"
export TIGHTSHIP_HOOK_OUTPUT="${TIGHTSHIP_HOOK_OUTPUT:-$plugin_dir/hook-output.json}"
export TIGHTSHIP_CHECKPOINT_OUTPUT="${TIGHTSHIP_CHECKPOINT_OUTPUT:-$plugin_dir/checkpoint-output.json}"
[ -n "$TIGHTSHIP_STATUS_LINE_COMMAND" ] || TIGHTSHIP_STATUS_LINE_COMMAND='"${CURSOR_PLUGIN_ROOT}/bin/tightship" status-line render'
export TIGHTSHIP_STATUS_LINE_COMMAND
