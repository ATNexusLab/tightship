#!/bin/sh
set -e

RELEASE_TAG=""
REPOSITORY="{{repository}}"

show_help() {
  cat <<'EOF'
Usage: install.sh [OPTIONS]

Install the tightship plugin for one harness or all of them.

Options:
  --harness <name>       Target harness: {{harness-choices}}
  --contract <bool>      Enable the working contract: true or false (default: true)
  --memory <bool>        Enable memory layer: true or false (default: true)
  --status-line <bool>   Enable status line: true or false (default: true)
  --vault-folder <path>  Path to memory vault (e.g. ~/ai-memory)
  --init-vault <bool>    Initialize missing vault folder with tightship vault init (default: false)
  --from-dir <path>      Install from a local directory instead of downloading this installer's release
  -y, --yes              Accept defaults without interactive prompts
  -h, --help             Show this help message
EOF
}

version_ge() {
  if [ "$1" = "$2" ]; then
    return 0
  fi
  _v1_maj=$(printf '%s\n' "$1" | cut -d. -f1)
  _v1_min=$(printf '%s\n' "$1" | cut -d. -f2)
  _v1_pat=$(printf '%s\n' "$1" | cut -d. -f3)

  _v2_maj=$(printf '%s\n' "$2" | cut -d. -f1)
  _v2_min=$(printf '%s\n' "$2" | cut -d. -f2)
  _v2_pat=$(printf '%s\n' "$2" | cut -d. -f3)

  _v1_maj="${_v1_maj:-0}"
  _v1_min="${_v1_min:-0}"
  _v1_pat="${_v1_pat:-0}"

  _v2_maj="${_v2_maj:-0}"
  _v2_min="${_v2_min:-0}"
  _v2_pat="${_v2_pat:-0}"

  if [ "$_v1_maj" -gt "$_v2_maj" ]; then
    return 0
  elif [ "$_v1_maj" -lt "$_v2_maj" ]; then
    return 1
  fi

  if [ "$_v1_min" -gt "$_v2_min" ]; then
    return 0
  elif [ "$_v1_min" -lt "$_v2_min" ]; then
    return 1
  fi

  if [ "$_v1_pat" -ge "$_v2_pat" ]; then
    return 0
  else
    return 1
  fi
}

extract_version() {
  printf '%s\n' "$1" | grep -oE '[0-9]+(\.[0-9]+)+' | head -n1
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d' ' -f1
  else
    echo "error: neither sha256sum nor shasum is available to check $1" >&2
    return 1
  fi
}

download_file() {
  _url="$1"
  _dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$_url" -o "$_dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$_dest" "$_url"
  else
    echo "error: neither curl nor wget is available to download $_url" >&2
    exit 1
  fi
}

to_bool() {
  case "$(printf '%s' "$2" | tr '[:upper:]' '[:lower:]')" in
    true|yes|y|on|1) echo "true" ;;
    false|no|n|off|0) echo "false" ;;
    *)
      echo "error: $1 expects true or false; got \"$2\"" >&2
      return 1
      ;;
  esac
}

step() {
  _description="$1"
  shift
  echo "> $_description"
  if ! "$@"; then
    echo "error: $_description failed" >&2
    exit 1
  fi
}

require_version() {
  _command=$(harness_field "$1" command)
  _minimum=$(harness_field "$1" min-version)
  if ! command -v "$_command" >/dev/null 2>&1; then
    echo "error: $_command $_minimum or later is required; found not installed" >&2
    exit 1
  fi
  _raw=$("$_command" --version 2>&1 || true)
  _ver=$(extract_version "$_raw")
  if [ -z "$_ver" ]; then
    _ver="$_raw"
  fi
  if ! version_ge "$_ver" "$_minimum"; then
    echo "error: $_command $_minimum or later is required; found $_ver" >&2
    exit 1
  fi
}

call_hook() {
  _hook="$1_$(harness_field "$2" id)"
  if command -v "$_hook" >/dev/null 2>&1; then
    "$_hook"
  fi
}

harness_numbered() {
  _index=0
  for _harness in $HARNESSES; do
    _index=$((_index + 1))
    if [ "$_index" = "$1" ]; then
      echo "$_harness"
      return 0
    fi
  done
  return 1
}

labels_of() {
  _labels=""
  _left=$#
  for _harness in "$@"; do
    _label=$(harness_field "$_harness" label)
    _left=$((_left - 1))
    if [ -z "$_labels" ]; then
      _labels="$_label"
    elif [ "$_left" -eq 0 ]; then
      _labels="$_labels and $_label"
    else
      _labels="$_labels, $_label"
    fi
  done
  printf '%s\n' "$_labels"
}

{{harnesses}}

{{fragments}}
HARNESS=""
CONTRACT_ARG=""
MEMORY_ARG=""
STATUS_LINE_ARG=""
VAULT_FOLDER_ARG=""
INIT_VAULT_ARG=""
FROM_DIR=""
YES=0

while [ $# -gt 0 ]; do
  case "$1" in
    --harness)
      HARNESS="$2"
      shift 2
      ;;
    --harness=*)
      HARNESS="${1#*=}"
      shift
      ;;
    --contract)
      CONTRACT_ARG="$2"
      shift 2
      ;;
    --contract=*)
      CONTRACT_ARG="${1#*=}"
      shift
      ;;
    --memory)
      MEMORY_ARG="$2"
      shift 2
      ;;
    --memory=*)
      MEMORY_ARG="${1#*=}"
      shift
      ;;
    --status-line)
      STATUS_LINE_ARG="$2"
      shift 2
      ;;
    --status-line=*)
      STATUS_LINE_ARG="${1#*=}"
      shift
      ;;
    --vault-folder)
      VAULT_FOLDER_ARG="$2"
      shift 2
      ;;
    --vault-folder=*)
      VAULT_FOLDER_ARG="${1#*=}"
      shift
      ;;
    --init-vault)
      INIT_VAULT_ARG="$2"
      shift 2
      ;;
    --init-vault=*)
      INIT_VAULT_ARG="${1#*=}"
      shift
      ;;
    --from-dir)
      FROM_DIR="$2"
      shift 2
      ;;
    --from-dir=*)
      FROM_DIR="${1#*=}"
      shift
      ;;
    -y|--yes)
      YES=1
      shift
      ;;
    -h|--help)
      show_help
      exit 0
      ;;
    *)
      echo "error: unrecognized argument $1" >&2
      exit 1
      ;;
  esac
done

TTY_FD=""
if [ -t 0 ]; then
  TTY_FD="0"
elif (exec 3</dev/tty) 2>/dev/null; then
  exec 3</dev/tty
  TTY_FD="3"
fi

if [ "$YES" -eq 0 ] && [ -z "$TTY_FD" ]; then
  echo "error: standard input is not a terminal; pass --yes or provide required flags" >&2
  exit 1
fi

prompt_read() {
  _prompt="$1"
  _default="$2"
  _ans=""
  if [ -n "$TTY_FD" ]; then
    printf "%s" "$_prompt" >&2
    if [ "$TTY_FD" = "3" ]; then
      read -r _ans <&3 || true
    else
      read -r _ans || true
    fi
  fi
  if [ -z "$_ans" ]; then
    printf '%s\n' "$_default"
  else
    printf '%s\n' "$_ans"
  fi
}

if [ -n "$HARNESS" ]; then
  case "$HARNESS" in
    all|both)
      SELECTED="$HARNESSES"
      ;;
    *)
      if ! SELECTED=$(harness_named "$HARNESS"); then
        echo "error: unknown harness $HARNESS; expected {{harness-choices}}" >&2
        exit 1
      fi
      ;;
  esac
else
  detected=""
  menu=""
  count=0
  for harness in $HARNESSES; do
    count=$((count + 1))
    menu="$menu[$count] $(harness_field "$harness" label) "
    if command -v "$(harness_field "$harness" command)" >/dev/null 2>&1; then
      detected="$detected $harness"
    fi
  done
  default_selection="${detected# }"
  if [ -z "$default_selection" ]; then
    default_selection="$HARNESSES"
  fi

  if [ "$YES" -eq 1 ]; then
    SELECTED="$default_selection"
  else
    all_number=$((count + 1))
    choice=$(prompt_read "Install tightship for: $menu[$all_number] All (default: $(labels_of $default_selection)): " "")
    case "$choice" in
      "")
        SELECTED="$default_selection"
        ;;
      all|both|"$all_number")
        SELECTED="$HARNESSES"
        ;;
      *)
        SELECTED=$(harness_numbered "$choice") || SELECTED=$(harness_named "$choice") || SELECTED="$default_selection"
        ;;
    esac
  fi
fi

for harness in $SELECTED; do
  require_version "$harness"
  call_hook check "$harness"
done

existing_contract=""
existing_memory=""
existing_status_line=""
existing_vault_folder=""

for harness in $SELECTED; do
  OPTIONS_FILE=$(call_hook options_file "$harness")
  if [ -n "$OPTIONS_FILE" ] && [ -f "$OPTIONS_FILE" ]; then
    if grep -q '"contract"' "$OPTIONS_FILE" 2>/dev/null; then
      existing_contract=$(sed -n 's/.*"contract"[[:space:]]*:[[:space:]]*\(true\|false\).*/\1/p' "$OPTIONS_FILE" | head -n1)
      existing_memory=$(sed -n 's/.*"memory"[[:space:]]*:[[:space:]]*\(true\|false\).*/\1/p' "$OPTIONS_FILE" | head -n1)
      existing_status_line=$(sed -n 's/.*"status_line"[[:space:]]*:[[:space:]]*\(true\|false\).*/\1/p' "$OPTIONS_FILE" | head -n1)
      existing_vault_folder=$(sed -n 's/.*"vault_folder"[[:space:]]*:[[:space:]]*"\((\\.|[^"\\])*\)".*/\1/p' "$OPTIONS_FILE" | head -n1 | sed 's/\\(.)/\1/g')
    else
      existing_contract=$(grep -E '^[[:space:]]*contract[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*contract[[:space:]]*=[[:space:]]*//' | tr -d '"'\'' ' || true)
      existing_memory=$(grep -E '^[[:space:]]*memory[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*memory[[:space:]]*=[[:space:]]*//' | tr -d '"'\'' ' || true)
      existing_status_line=$(grep -E '^[[:space:]]*status_line[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*status_line[[:space:]]*=[[:space:]]*//' | tr -d '"'\'' ' || true)
      existing_vault_folder=$(grep -E '^[[:space:]]*vault_folder[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*vault_folder[[:space:]]*=[[:space:]]*//' | sed -E -e 's/^[[:space:]]*"((\\.|[^"\\])*)"[[:space:]]*$/\1/' -e 's/\\(.)/\1/g' || true)
    fi
  fi
done

if [ -n "$CONTRACT_ARG" ]; then
  CONTRACT=$(to_bool --contract "$CONTRACT_ARG")
elif [ "$YES" -eq 1 ]; then
  CONTRACT="${existing_contract:-true}"
else
  def="Y"
  [ "$existing_contract" = "false" ] && def="n"
  ans=$(prompt_read "Enable the working contract in every session? [Y/n]: " "$def")
  CONTRACT=$(to_bool "the answer" "$ans")
fi

if [ -n "$MEMORY_ARG" ]; then
  MEMORY=$(to_bool --memory "$MEMORY_ARG")
elif [ "$YES" -eq 1 ]; then
  MEMORY="${existing_memory:-true}"
else
  def="Y"
  [ "$existing_memory" = "false" ] && def="n"
  ans=$(prompt_read "Enable the memory layer (curated vault notes and handoffs)? [Y/n]: " "$def")
  MEMORY=$(to_bool "the answer" "$ans")
fi

if [ -n "$VAULT_FOLDER_ARG" ]; then
  VAULT_FOLDER="$VAULT_FOLDER_ARG"
elif [ "$YES" -eq 1 ]; then
  VAULT_FOLDER="${existing_vault_folder:-}"
else
  if [ "$MEMORY" = "true" ]; then
    if [ -n "$existing_vault_folder" ]; then
      VAULT_FOLDER=$(prompt_read "Vault folder path [$existing_vault_folder]: " "$existing_vault_folder")
    else
      VAULT_FOLDER=$(prompt_read "Vault folder path (for example ~/ai-memory, or leave empty to set later): " "")
    fi
  else
    VAULT_FOLDER="${existing_vault_folder:-}"
  fi
fi

INIT_VAULT="false"
expanded_vault=""
if [ -n "$VAULT_FOLDER" ]; then
  case "$VAULT_FOLDER" in
    "~"*) expanded_vault="$HOME${VAULT_FOLDER#\~}" ;;
    *) expanded_vault="$VAULT_FOLDER" ;;
  esac
fi

if [ -n "$expanded_vault" ] && [ ! -d "$expanded_vault" ]; then
  if [ -n "$INIT_VAULT_ARG" ]; then
    INIT_VAULT=$(to_bool --init-vault "$INIT_VAULT_ARG")
  elif [ "$YES" -eq 1 ]; then
    INIT_VAULT="false"
  else
    ans=$(prompt_read "Vault folder $VAULT_FOLDER does not exist. Create it now with tightship vault init? [Y/n]: " "Y")
    INIT_VAULT=$(to_bool "the answer" "$ans")
  fi
fi

if [ -n "$STATUS_LINE_ARG" ]; then
  STATUS_LINE=$(to_bool --status-line "$STATUS_LINE_ARG")
elif [ "$YES" -eq 1 ]; then
  STATUS_LINE="${existing_status_line:-true}"
else
  def="Y"
  [ "$existing_status_line" = "false" ] && def="n"
  ans=$(prompt_read "Enable tightship's status line (replaces your current statusLine and restores it when off)? [Y/n]: " "$def")
  STATUS_LINE=$(to_bool "the answer" "$ans")
fi

TMP_DIR=""
cleanup() {
  if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT INT TERM

for harness in $SELECTED; do
  "install_$(harness_field "$harness" id)"
done

echo "Installed tightship for $(labels_of $SELECTED)."
