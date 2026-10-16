plugin_dir="$(cd "$(dirname "$0")/.." && pwd)"
export TIGHTSHIP_ROOT="${TIGHTSHIP_ROOT:-${CURSOR_PLUGIN_ROOT:-$plugin_dir}}"
export TIGHTSHIP_DATA="${TIGHTSHIP_DATA:-{{path:data}}}"
