#!/bin/sh
set -e

# Version requirements
CLAUDE_MIN_VERSION="2.1.295"
AGY_MIN_VERSION="1.4.0"

show_help() {
  cat <<'EOF'
Usage: install.sh [OPTIONS]

Install the tightship plugin for Claude Code, Antigravity CLI, or both.

Options:
  --harness <name>       Target harness: claude-code, antigravity-cli, or all
  --contract <mode>      Contract mode: strict, lenient, off, true, or false (default: strict)
  --memory <bool>        Enable memory layer: true or false (default: true)
  --status-line <bool>   Enable status line: true or false (default: true)
  --vault-folder <path>  Path to memory vault (e.g. ~/ai-memory)
  --init-vault <bool>    Initialize missing vault folder with tightship vault init (default: false)
  --version <tag>        Release version to install (default: latest)
  --from-dir <path>      Install from local directory instead of downloading release
  -y, --yes              Accept defaults without interactive prompts
  -h, --help             Show this help message
EOF
}

# Version comparison: returns 0 if $1 >= $2, 1 otherwise. Dotted semver (e.g. 2.1.295).
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

# Download helper
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

# Argument parsing
HARNESS=""
CONTRACT_ARG=""
MEMORY_ARG=""
STATUS_LINE_ARG=""
VAULT_FOLDER_ARG=""
INIT_VAULT_ARG=""
VERSION="latest"
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
    --version)
      VERSION="$2"
      shift 2
      ;;
    --version=*)
      VERSION="${1#*=}"
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

# Check terminal interaction capability
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
    echo "$_default"
  else
    echo "$_ans"
  fi
}

# Validate or detect harness
has_claude=0
has_agy=0
command -v claude >/dev/null 2>&1 && has_claude=1
command -v agy >/dev/null 2>&1 && has_agy=1

if [ -n "$HARNESS" ]; then
  case "$HARNESS" in
    claude|claude-code)
      HARNESS="claude-code"
      ;;
    antigravity-cli|agy)
      HARNESS="antigravity-cli"
      ;;
    all|both)
      HARNESS="all"
      ;;
    *)
      echo "error: unknown harness $HARNESS; expected claude-code, antigravity-cli, or all" >&2
      exit 1
      ;;
  esac
else
  # Detect default
  default_harness="all"
  default_label="Both"
  if [ "$has_claude" -eq 1 ] && [ "$has_agy" -eq 0 ]; then
    default_harness="claude-code"
    default_label="Claude Code"
  elif [ "$has_claude" -eq 0 ] && [ "$has_agy" -eq 1 ]; then
    default_harness="antigravity-cli"
    default_label="Antigravity CLI"
  fi

  if [ "$YES" -eq 1 ]; then
    HARNESS="$default_harness"
  else
    choice=$(prompt_read "Install tightship for: [1] Claude Code [2] Antigravity CLI [3] Both (default: $default_label): " "$default_harness")
    case "$choice" in
      1|claude|claude-code)
        HARNESS="claude-code"
        ;;
      2|antigravity-cli|agy)
        HARNESS="antigravity-cli"
        ;;
      3|all|both)
        HARNESS="all"
        ;;
      *)
        HARNESS="$default_harness"
        ;;
    esac
  fi
fi

# Version verification for selected harnesses
check_claude() {
  if ! command -v claude >/dev/null 2>&1; then
    echo "error: claude $CLAUDE_MIN_VERSION or later is required; found not installed" >&2
    exit 1
  fi
  _raw=$(claude --version 2>&1 || true)
  _ver=$(extract_version "$_raw")
  if [ -z "$_ver" ]; then
    _ver="$_raw"
  fi
  if ! version_ge "$_ver" "$CLAUDE_MIN_VERSION"; then
    echo "error: claude $CLAUDE_MIN_VERSION or later is required; found $_ver" >&2
    exit 1
  fi
}

check_agy() {
  if ! command -v agy >/dev/null 2>&1; then
    echo "error: agy $AGY_MIN_VERSION or later is required; found not installed" >&2
    exit 1
  fi
  _raw=$(agy --version 2>&1 || true)
  _ver=$(extract_version "$_raw")
  if [ -z "$_ver" ]; then
    _ver="$_raw"
  fi
  if ! version_ge "$_ver" "$AGY_MIN_VERSION"; then
    echo "error: agy $AGY_MIN_VERSION or later is required; found $_ver" >&2
    exit 1
  fi
}

case "$HARNESS" in
  claude-code)
    check_claude
    ;;
  antigravity-cli)
    check_agy
    ;;
  all)
    check_claude
    check_agy
    ;;
esac

# Options configuration
TIGHTSHIP_DATA="${TIGHTSHIP_DATA:-${PLUGIN_DATA:-$HOME/.gemini/config/plugin_data/tightship}}"
OPTIONS_FILE="$TIGHTSHIP_DATA/options.toml"

existing_contract=""
existing_memory=""
existing_status_line=""
existing_vault_folder=""

if [ -f "$OPTIONS_FILE" ]; then
  existing_contract=$(grep -E '^[[:space:]]*contract[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*contract[[:space:]]*=[[:space:]]*//' | tr -d '"'\'' ' || true)
  existing_memory=$(grep -E '^[[:space:]]*memory[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*memory[[:space:]]*=[[:space:]]*//' | tr -d '"'\'' ' || true)
  existing_status_line=$(grep -E '^[[:space:]]*status_line[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*status_line[[:space:]]*=[[:space:]]*//' | tr -d '"'\'' ' || true)
  existing_vault_folder=$(grep -E '^[[:space:]]*vault_folder[[:space:]]*=' "$OPTIONS_FILE" | sed -E 's/^[[:space:]]*vault_folder[[:space:]]*=[[:space:]]*//' | sed -E 's/^[[:space:]]*"([^"]*)"[[:space:]]*$/\1/' || true)
fi

# Contract
if [ -n "$CONTRACT_ARG" ]; then
  case "$CONTRACT_ARG" in
    off|false|0) CONTRACT="false" ;;
    *) CONTRACT="true" ;;
  esac
elif [ "$YES" -eq 1 ]; then
  CONTRACT="${existing_contract:-true}"
else
  def="Y"
  [ "$existing_contract" = "false" ] && def="n"
  ans=$(prompt_read "Enable the working contract in every session? [Y/n]: " "$def")
  case "$ans" in
    n|N|no|false|0) CONTRACT="false" ;;
    *) CONTRACT="true" ;;
  esac
fi

# Memory
if [ -n "$MEMORY_ARG" ]; then
  case "$MEMORY_ARG" in
    off|false|0) MEMORY="false" ;;
    *) MEMORY="true" ;;
  esac
elif [ "$YES" -eq 1 ]; then
  MEMORY="${existing_memory:-true}"
else
  def="Y"
  [ "$existing_memory" = "false" ] && def="n"
  ans=$(prompt_read "Enable the memory layer (curated vault notes and handoffs)? [Y/n]: " "$def")
  case "$ans" in
    n|N|no|false|0) MEMORY="false" ;;
    *) MEMORY="true" ;;
  esac
fi

# Vault folder
if [ -n "$VAULT_FOLDER_ARG" ]; then
  VAULT_FOLDER="$VAULT_FOLDER_ARG"
elif [ "$YES" -eq 1 ]; then
  VAULT_FOLDER="${existing_vault_folder:-}"
else
  if [ "$MEMORY" = "true" ]; then
    def="${existing_vault_folder:-~/ai-memory}"
    ans=$(prompt_read "Vault folder path (for example ~/ai-memory, or leave empty to set later): " "")
    VAULT_FOLDER="$ans"
  else
    VAULT_FOLDER="${existing_vault_folder:-}"
  fi
fi

# Init vault
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
    case "$INIT_VAULT_ARG" in
      true|1|yes|y) INIT_VAULT="true" ;;
      *) INIT_VAULT="false" ;;
    esac
  elif [ "$YES" -eq 1 ]; then
    INIT_VAULT="false"
  else
    ans=$(prompt_read "Vault folder $VAULT_FOLDER does not exist. Create it now with tightship vault init? [Y/n]: " "Y")
    case "$ans" in
      n|N|no|false|0) INIT_VAULT="false" ;;
      *) INIT_VAULT="true" ;;
    esac
  fi
fi

# Status line
if [ -n "$STATUS_LINE_ARG" ]; then
  case "$STATUS_LINE_ARG" in
    off|false|0) STATUS_LINE="false" ;;
    *) STATUS_LINE="true" ;;
  esac
elif [ "$YES" -eq 1 ]; then
  STATUS_LINE="${existing_status_line:-true}"
else
  def="Y"
  [ "$existing_status_line" = "false" ] && def="n"
  ans=$(prompt_read "Enable tightship's status line (replaces your current statusLine and restores it when off)? [Y/n]: " "$def")
  case "$ans" in
    n|N|no|false|0) STATUS_LINE="false" ;;
    *) STATUS_LINE="true" ;;
  esac
fi

# Temporary directory for release downloads if needed
TMP_DIR=""
cleanup() {
  if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT INT TERM

# Installation: Claude Code
if [ "$HARNESS" = "claude-code" ] || [ "$HARNESS" = "all" ]; then
  claude plugin marketplace add DietrichGebert/ponytail || true
  claude plugin marketplace add ayghri/i-have-adhd || true
  claude plugin marketplace add blader/humanizer || true

  if [ -n "$FROM_DIR" ]; then
    claude plugin marketplace add "$FROM_DIR" || true
  else
    claude plugin marketplace add ATNexusLab/tightship || true
  fi

  claude_args="--config contract=$CONTRACT --config memory=$MEMORY --config status_line=$STATUS_LINE"
  if [ -n "$VAULT_FOLDER" ]; then
    claude_args="$claude_args --config vault_folder=$VAULT_FOLDER"
  fi

  # shellcheck disable=SC2086
  claude plugin install tightship@tightship $claude_args
fi

# Installation: Antigravity CLI
if [ "$HARNESS" = "antigravity-cli" ] || [ "$HARNESS" = "all" ]; then
  # Companions
  agy plugin install https://github.com/DietrichGebert/ponytail || true
  agy plugin install https://github.com/ayghri/i-have-adhd || true

  humanizer_dir="$HOME/.gemini/config/skills/humanizer"
  mkdir -p "$humanizer_dir"
  download_file "https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md" "$humanizer_dir/SKILL.md"

  # Source resolution
  source_dir=""
  if [ -n "$FROM_DIR" ]; then
    source_dir="$FROM_DIR"
  else
    TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'tightship')
    zip_dest="$TMP_DIR/tightship-antigravity-cli.zip"
    if [ "$VERSION" = "latest" ]; then
      download_url="https://github.com/ATNexusLab/tightship/releases/latest/download/tightship-antigravity-cli.zip"
    else
      download_url="https://github.com/ATNexusLab/tightship/releases/download/$VERSION/tightship-antigravity-cli.zip"
    fi
    download_file "$download_url" "$zip_dest"
    unzip -q "$zip_dest" -d "$TMP_DIR/extracted"
    source_dir="$TMP_DIR/extracted"
  fi

  agy plugin install "$source_dir" || true

  plugin_target="$HOME/.gemini/config/plugins/tightship"
  mkdir -p "$plugin_target"
  if [ ! -f "$plugin_target/plugin.json" ] || [ -n "$FROM_DIR" ]; then
    cp -R "$source_dir"/* "$plugin_target/"
  fi
  chmod +x "$plugin_target/bin/tightship" 2>/dev/null || true
  find "$plugin_target/bin" -type f -exec chmod +x {} + 2>/dev/null || true

  # Write options.toml
  mkdir -p "$TIGHTSHIP_DATA"
  {
    printf "contract = %s\n" "$CONTRACT"
    printf "memory = %s\n" "$MEMORY"
    printf "status_line = %s\n" "$STATUS_LINE"
    if [ -n "$VAULT_FOLDER" ]; then
      printf "vault_folder = \"%s\"\n" "$VAULT_FOLDER"
    fi
  } > "$OPTIONS_FILE"

  # Status line hook
  export TIGHTSHIP_DATA
  mkdir -p "$HOME/.gemini/antigravity-cli"
  "$plugin_target/bin/tightship" hook status-line || true

  # Vault init if requested
  if [ "$INIT_VAULT" = "true" ] && [ -n "$VAULT_FOLDER" ]; then
    "$plugin_target/bin/tightship" vault init "$VAULT_FOLDER" || true
  fi
fi

case "$HARNESS" in
  claude-code)
    echo "Installed tightship for Claude Code."
    ;;
  antigravity-cli)
    echo "Installed tightship for Antigravity CLI."
    ;;
  all)
    echo "Installed tightship for Claude Code and Antigravity CLI."
    ;;
esac
