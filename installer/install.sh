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
  --auto-update <bool>   Install each new release by itself: true or false (default: true)
  --vault-folder <path>  Path to memory vault (e.g. ~/ai-memory)
  --init-vault <bool>    Initialize missing vault folder with tightship vault init (default: false)
  --from-dir <path>      Install from a local directory instead of downloading this installer's release
  -y, --yes              Accept defaults without interactive prompts
  -h, --help             Show this help message
EOF
}

NEWLINE='
'
COLOR_OUT=0
COLOR_ERR=0

paint() {
  if [ "$1" = 1 ]; then
    _enabled="$COLOR_OUT"
  else
    _enabled="$COLOR_ERR"
  fi
  if [ "$_enabled" = 1 ]; then
    printf '\033[%sm%s\033[0m' "$2" "$3"
  else
    printf '%s' "$3"
  fi
}

color_enabled() {
  [ -z "${NO_COLOR:-}" ] && [ "${TERM:-}" != "dumb" ] && [ -t "$1" ]
}

fail() {
  printf '%s\n' "$(paint 2 31 "✖ error: $1")" >&2
  if [ -n "${2:-}" ]; then
    printf '%s\n' "$(paint 2 2 "  hint: $2")" >&2
  fi
  exit 1
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
    fail "neither sha256sum nor shasum is available to check $1"
  fi
}

download_file() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1" -o "$2"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$2" "$1"
  else
    echo "error: neither curl nor wget is available to download $1" >&2
    return 1
  fi
}

quietly() {
  "$@" >/dev/null 2>&1 || true
}

to_bool() {
  case "$(printf '%s' "$2" | tr '[:upper:]' '[:lower:]')" in
    true|yes|y|on|1) echo "true" ;;
    false|no|n|off|0) echo "false" ;;
    *) fail "$1 expects true or false; got \"$2\"" ;;
  esac
}

run_step() {
  _description="$1"
  shift
  _log="$WORK_DIR/step.log"
  if [ -t 1 ]; then
    printf '%s  %s' "$(paint 1 36 "◒")" "$_description"
  fi
  if "$@" >"$_log" 2>&1; then
    _status=0
    _mark=$(paint 1 32 "✔")
  else
    _status=1
    _mark=$(paint 1 31 "✖")
  fi
  if [ -t 1 ]; then
    printf '\r\033[K'
  fi
  printf '%s  %s\n' "$_mark" "$_description"
  if [ "$_status" -ne 0 ]; then
    sed 's/^/   /' "$_log" >&2
  fi
  return "$_status"
}

step() {
  run_step "$@" || fail "$1 failed"
}

INSTALLED_IN=""

installed_in() {
  INSTALLED_IN="$INSTALLED_IN$1|$2$NEWLINE"
}

require_version() {
  _command=$(harness_field "$1" command)
  _minimum=$(harness_field "$1" min-version)
  if ! command -v "$_command" >/dev/null 2>&1; then
    fail "$_command $_minimum or later is required; found not installed"
  fi
  _raw=$("$_command" --version 2>&1 || true)
  _ver=$(extract_version "$_raw")
  if [ -z "$_ver" ]; then
    _ver="$_raw"
  fi
  if ! version_ge "$_ver" "$_minimum"; then
    fail "$_command $_minimum or later is required; found $_ver"
  fi
}

call_hook() {
  _hook="$1_$(harness_field "$2" id)"
  if command -v "$_hook" >/dev/null 2>&1; then
    "$_hook"
  fi
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

contains() {
  case " $1 " in
    *" $2 "*) return 0 ;;
    *) return 1 ;;
  esac
}

{{harnesses}}

{{fragments}}
HARNESS=""
CONTRACT_ARG=""
MEMORY_ARG=""
STATUS_LINE_ARG=""
AUTO_UPDATE_ARG=""
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
    --auto-update)
      AUTO_UPDATE_ARG="$2"
      shift 2
      ;;
    --auto-update=*)
      AUTO_UPDATE_ARG="${1#*=}"
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
      fail "unrecognized argument $1"
      ;;
  esac
done

if color_enabled 1; then
  COLOR_OUT=1
fi
if color_enabled 2; then
  COLOR_ERR=1
fi

TTY_FD=""
STTY_SAVED=""
if [ "$YES" -eq 0 ]; then
  if [ -t 0 ]; then
    TTY_FD="0"
  elif (exec 3</dev/tty) 2>/dev/null; then
    exec 3</dev/tty
    TTY_FD="3"
  else
    fail "standard input is not a terminal; pass --yes or provide required flags"
  fi
fi

tty_stty() {
  if [ "$TTY_FD" = "3" ]; then
    stty "$@" <&3
  else
    stty "$@"
  fi
}

read_byte() {
  if [ "$TTY_FD" = "3" ]; then
    dd bs=1 count=1 2>/dev/null <&3
  else
    dd bs=1 count=1 2>/dev/null
  fi
}

read_key() {
  _byte=$(read_byte; echo x)
  _byte="${_byte%x}"
  case "$_byte" in
    "$(printf '\033')")
      _sequence=$(read_byte; read_byte; echo x)
      case "${_sequence%x}" in
        "[A"|OA) KEY="up" ;;
        "[B"|OB) KEY="down" ;;
        "[C"|OC) KEY="right" ;;
        "[D"|OD) KEY="left" ;;
        *) KEY="other" ;;
      esac
      ;;
    " ") KEY="space" ;;
    "$NEWLINE"|"$(printf '\r')") KEY="enter" ;;
    *) KEY=$(printf '%s' "$_byte" | tr '[:upper:]' '[:lower:]') ;;
  esac
}

raw_mode() {
  tty_stty -icanon -echo min 1 time 0
  printf '\033[?25l' >&2
}

cooked_mode() {
  tty_stty "$STTY_SAVED"
  printf '\033[?25h' >&2
}

erase() {
  if [ "$1" -gt 0 ]; then
    printf '\r\033[%sA\033[J' "$1" >&2
  fi
}

answered() {
  printf '%s  %s\n%s  %s\n%s\n' "$(paint 2 32 "◇")" "$1" "│" "$(paint 2 2 "$2")" "│" >&2
}

section() {
  printf '%s\n%s  %s\n' "│" "│" "$(paint 2 '1;2' "$1")" >&2
}

ask_harnesses() {
  _title="Where should tightship be installed?"
  _checked="$detected"
  _cursor=1
  _count=0
  for _harness in $HARNESSES; do
    _count=$((_count + 1))
  done
  _notice=""
  _drawn=0
  raw_mode
  while :; do
    erase "$_drawn"
    _screen="$(paint 2 36 "◆")  $_title$NEWLINE"
    _index=0
    for _harness in $HARNESSES; do
      _index=$((_index + 1))
      _label=$(harness_field "$_harness" label)
      if ! contains "$detected" "$_harness"; then
        _row=$(paint 2 2 "◻ $_label  not found")
      else
        if contains "$_checked" "$_harness"; then
          _box=$(paint 2 32 "◼")
        else
          _box=$(paint 2 2 "◻")
        fi
        if [ "$_index" -eq "$_cursor" ]; then
          _row="$_box $(paint 2 '1;36' "$_label")"
        else
          _row="$_box $_label"
        fi
      fi
      if [ "$_index" -eq "$_cursor" ]; then
        _row="$(paint 2 36 "›") $_row"
      else
        _row="  $_row"
      fi
      _screen="${_screen}│ $_row$NEWLINE"
    done
    _screen="${_screen}│  $(paint 2 2 "↑/↓ move · space select · a all · enter confirm")$NEWLINE"
    _screen="${_screen}└  $(paint 2 33 "$_notice")$NEWLINE"
    printf '%s' "$_screen" >&2
    _drawn=$((_count + 3))
    read_key
    _notice=""
    _current=$(set -- $HARNESSES; eval "echo \${$_cursor}")
    case "$KEY" in
      up|k)
        _cursor=$((_cursor - 1))
        [ "$_cursor" -ge 1 ] || _cursor="$_count"
        ;;
      down|j)
        _cursor=$((_cursor + 1))
        [ "$_cursor" -le "$_count" ] || _cursor=1
        ;;
      space)
        if ! contains "$detected" "$_current"; then
          _notice="$(harness_field "$_current" label) is not installed."
        elif contains "$_checked" "$_current"; then
          _kept=""
          for _harness in $_checked; do
            [ "$_harness" = "$_current" ] || _kept="$_kept $_harness"
          done
          _checked="$_kept"
        else
          _checked="$_checked $_current"
        fi
        ;;
      a)
        _all=1
        for _harness in $detected; do
          contains "$_checked" "$_harness" || _all=0
        done
        if [ "$_all" -eq 1 ]; then
          _checked=""
        else
          _checked="$detected"
        fi
        ;;
      enter)
        if [ -n "$(printf '%s' "$_checked" | tr -d ' ')" ]; then
          break
        fi
        _notice="Select at least one harness."
        ;;
    esac
  done
  cooked_mode
  erase "$_drawn"
  SELECTED=""
  for _harness in $HARNESSES; do
    if contains "$_checked" "$_harness"; then
      SELECTED="$SELECTED $_harness"
    fi
  done
  SELECTED="${SELECTED# }"
  answered "$_title" "$(labels_of $SELECTED)"
}

ask_confirm() {
  _title="$1"
  _hint="$2"
  ANSWER="$3"
  _yes="${4:-Yes}"
  _no="${5:-No}"
  _drawn=0
  raw_mode
  while :; do
    erase "$_drawn"
    if [ "$ANSWER" = "true" ]; then
      _choice="$(paint 2 32 "●") $_yes $(paint 2 2 "/ ○ $_no")"
    else
      _choice="$(paint 2 2 "○ $_yes /") $(paint 2 32 "●") $_no"
    fi
    _screen="$(paint 2 36 "◆")  $_title$NEWLINE"
    _drawn=3
    if [ -n "$_hint" ]; then
      _screen="${_screen}│  $(paint 2 2 "$_hint")$NEWLINE"
      _drawn=4
    fi
    printf '%s│  %s\n└\n' "$_screen" "$_choice" >&2
    read_key
    case "$KEY" in
      up|down|left|right|space|h|j|k|l)
        if [ "$ANSWER" = "true" ]; then
          ANSWER="false"
        else
          ANSWER="true"
        fi
        ;;
      y) ANSWER="true" ;;
      n) ANSWER="false" ;;
      enter) break ;;
    esac
  done
  cooked_mode
  erase "$_drawn"
  if [ "$ANSWER" = "true" ]; then
    answered "$_title" "$_yes"
  else
    answered "$_title" "$_no"
  fi
}

ask_text() {
  _title="$1"
  _hint="$2"
  printf '%s  %s\n%s  %s\n%s  ' "$(paint 2 36 "◆")" "$_title" "│" "$(paint 2 2 "$_hint")" "│" >&2
  _line=""
  if [ "$TTY_FD" = "3" ]; then
    read -r _line <&3 || true
  else
    read -r _line || true
  fi
  ANSWER="${_line:-$3}"
  erase 3
  answered "$_title" "${ANSWER:-not set}"
}

WORK_DIR=""
cleanup() {
  if [ -n "$STTY_SAVED" ]; then
    tty_stty "$STTY_SAVED" 2>/dev/null || true
    printf '\033[?25h' >&2
  fi
  if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
    rm -rf "$WORK_DIR"
  fi
}
trap cleanup EXIT
trap 'printf "\n%s\n" "└  Cancelled." >&2; exit 130' INT TERM

if [ -n "$TTY_FD" ]; then
  STTY_SAVED=$(tty_stty -g)
fi

printf '%s  %s\n' "┌" "$(paint 1 '1;36' "tightship ${RELEASE_TAG:-dev}")"

detected=""
for harness in $HARNESSES; do
  if command -v "$(harness_field "$harness" command)" >/dev/null 2>&1; then
    detected="$detected $harness"
  fi
done
detected="${detected# }"

SKIPPED=""
HARNESS=$(printf '%s' "$HARNESS" | tr '[:upper:]' '[:lower:]')
case "$HARNESS" in
  "")
    if [ -z "$detected" ]; then
      fail "none of $(labels_of $HARNESSES) is installed"
    fi
    if [ "$YES" -eq 1 ]; then
      SELECTED="$detected"
    else
      section "Harnesses"
      ask_harnesses
    fi
    ;;
  all|both)
    if [ -z "$detected" ]; then
      fail "none of $(labels_of $HARNESSES) is installed"
    fi
    SELECTED="$detected"
    for harness in $HARNESSES; do
      contains "$detected" "$harness" || SKIPPED="$SKIPPED $harness"
    done
    SKIPPED="${SKIPPED# }"
    ;;
  *)
    if ! SELECTED=$(harness_named "$HARNESS"); then
      fail "unknown harness $HARNESS; expected {{harness-choices}}"
    fi
    ;;
esac

for harness in $SELECTED; do
  require_version "$harness"
  call_hook check "$harness"
done

existing_contract=""
existing_memory=""
existing_status_line=""
existing_auto_update=""
existing_vault_folder=""

for harness in $SELECTED; do
  call_hook read_options "$harness"
done

kept_default() {
  if [ -n "$1" ]; then
    printf 'Default: %s, from your last install.' "$2"
  else
    printf 'Default: %s.' "$2"
  fi
}

yes_no() {
  if [ "$1" = "false" ]; then
    echo "No"
  else
    echo "Yes"
  fi
}

if [ "$YES" -eq 0 ] && { [ -z "$CONTRACT_ARG" ] || [ -z "$MEMORY_ARG" ] || [ -z "$STATUS_LINE_ARG" ] || [ -z "$AUTO_UPDATE_ARG" ]; }; then
  section "Options"
fi

if [ -n "$CONTRACT_ARG" ]; then
  CONTRACT=$(to_bool --contract "$CONTRACT_ARG")
elif [ "$YES" -eq 1 ]; then
  CONTRACT="${existing_contract:-true}"
else
  ask_confirm "Load the working contract in every session?" \
    "The rules every session follows. $(kept_default "$existing_contract" "$(yes_no "$existing_contract")")" \
    "${existing_contract:-true}"
  CONTRACT="$ANSWER"
fi

if [ -n "$MEMORY_ARG" ]; then
  MEMORY=$(to_bool --memory "$MEMORY_ARG")
elif [ "$YES" -eq 1 ]; then
  MEMORY="${existing_memory:-true}"
else
  ask_confirm "Turn on the memory layer?" \
    "Curated vault notes and handoffs. $(kept_default "$existing_memory" "$(yes_no "$existing_memory")")" \
    "${existing_memory:-true}"
  MEMORY="$ANSWER"
fi

if [ -n "$STATUS_LINE_ARG" ]; then
  STATUS_LINE=$(to_bool --status-line "$STATUS_LINE_ARG")
elif [ "$YES" -eq 1 ]; then
  STATUS_LINE="${existing_status_line:-true}"
else
  ask_confirm "Show tightship's status line?" \
    "Replaces your statusLine and restores it when off. $(kept_default "$existing_status_line" "$(yes_no "$existing_status_line")")" \
    "${existing_status_line:-true}"
  STATUS_LINE="$ANSWER"
fi

if [ -n "$AUTO_UPDATE_ARG" ]; then
  AUTO_UPDATE=$(to_bool --auto-update "$AUTO_UPDATE_ARG")
elif [ "$YES" -eq 1 ]; then
  AUTO_UPDATE="${existing_auto_update:-true}"
else
  ask_confirm "Keep tightship updated automatically?" \
    "Installs each new release by itself and says so at the next session. $(kept_default "$existing_auto_update" "$(yes_no "$existing_auto_update")")" \
    "${existing_auto_update:-true}"
  AUTO_UPDATE="$ANSWER"
fi

if [ -n "$VAULT_FOLDER_ARG" ]; then
  VAULT_FOLDER="$VAULT_FOLDER_ARG"
elif [ "$YES" -eq 1 ] || [ "$MEMORY" != "true" ]; then
  VAULT_FOLDER="${existing_vault_folder:-}"
else
  section "Vault"
  if [ -n "$existing_vault_folder" ]; then
    ask_text "Where is your vault?" "Enter keeps $existing_vault_folder." "$existing_vault_folder"
  else
    ask_text "Where is your vault?" "A folder such as ~/ai-memory. Enter leaves it for later." ""
  fi
  VAULT_FOLDER="$ANSWER"
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
  elif [ "$YES" -eq 0 ]; then
    ask_confirm "$VAULT_FOLDER does not exist. Create it now?" "Runs tightship vault init. Default: Yes." "true"
    INIT_VAULT="$ANSWER"
  fi
fi

summary_row() {
  printf '%s  %s %s\n' "│" "$(paint 1 2 "$(printf '%-15s' "$1")")" "$2"
}

on_off() {
  if [ "$1" = "true" ]; then
    echo "on"
  else
    echo "off"
  fi
}

vault_summary="${VAULT_FOLDER:-not set}"
if [ "$INIT_VAULT" = "true" ]; then
  vault_summary="$vault_summary, created now"
fi

printf '%s\n%s  %s\n' "│" "$(paint 1 32 "◇")" "Summary"
summary_row "Harnesses" "$(labels_of $SELECTED)"
if [ -n "$SKIPPED" ]; then
  summary_row "Skipped" "$(labels_of $SKIPPED), not installed"
fi
summary_row "Contract" "$(on_off "$CONTRACT")"
summary_row "Memory" "$(on_off "$MEMORY")"
summary_row "Vault" "$vault_summary"
summary_row "Status line" "$(on_off "$STATUS_LINE")"
summary_row "Auto-update" "$(on_off "$AUTO_UPDATE")"
printf '%s\n' "│"

if [ "$YES" -eq 0 ]; then
  ask_confirm "Install with these choices?" "" "true" "Install" "Cancel"
  if [ "$ANSWER" != "true" ]; then
    printf '%s  %s\n' "└" "Cancelled; nothing was written."
    exit 0
  fi
fi

WORK_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'tightship')

for harness in $SELECTED; do
  "install_$(harness_field "$harness" id)"
done

printf '%s\n%s  %s\n' "│" "$(paint 1 32 "◇")" "Installed in"
printf '%s' "$INSTALLED_IN" | while IFS='|' read -r label location; do
  summary_row "$label" "$location"
done
printf '%s\n%s  %s\n' "│" "└" "$(paint 1 32 "Installed tightship for $(labels_of $SELECTED).")"
