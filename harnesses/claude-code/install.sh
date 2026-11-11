claude_code_input() {
  printf '%s\n' "$1" | sed -nE 's/.*"'"$2"'"[[:space:]]*:[[:space:]]*"((\\.|[^"\\])*)".*/\1/p' | head -n1 | sed -E 's/\\(.)/\1/g'
}

read_options_claude_code() {
  _inputs=$(claude plugin configure tightship@tightship --json 2>/dev/null) || return 0
  for _option in contract memory status_line auto_update vault_folder; do
    _value=$(claude_code_input "$_inputs" "$_option")
    if [ -n "$_value" ]; then
      eval "existing_$_option=\$_value"
    fi
  done
}

configure_claude_code() {
  _values=$(printf '"contract": "%s", "memory": "%s", "status_line": "%s", "auto_update": "%s"' \
    "$CONTRACT" "$MEMORY" "$STATUS_LINE" "$AUTO_UPDATE")
  if [ -n "$VAULT_FOLDER" ]; then
    _values="$_values, \"vault_folder\": \"$(printf '%s' "$VAULT_FOLDER" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')\""
  fi
  printf '{%s}\n' "$_values" | claude plugin configure tightship@tightship --values-stdin
}

claude_code_root() {
  claude plugin list --json 2>/dev/null | tr -d '\n' | tr '{' '\n' | grep '"tightship@tightship"' \
    | sed -nE 's/.*"installPath"[[:space:]]*:[[:space:]]*"((\\.|[^"\\])*)".*/\1/p' | head -n1 | sed -E 's/\\(.)/\1/g'
}

install_claude_code() {
  if [ -n "$FROM_DIR" ]; then
    quietly claude plugin marketplace add "$FROM_DIR"
  else
    quietly claude plugin marketplace add "${REPOSITORY#https://github.com/}"
  fi

  if claude plugin list --json 2>/dev/null | grep -q '"tightship@tightship"'; then
    quietly claude plugin marketplace update tightship
    step "Updating tightship for {{install:label}}" claude plugin update tightship@tightship
    step "Applying the options to {{install:label}}" configure_claude_code
  else
    set -- --config "contract=$CONTRACT" --config "memory=$MEMORY" --config "status_line=$STATUS_LINE" \
      --config "auto_update=$AUTO_UPDATE"
    if [ -n "$VAULT_FOLDER" ]; then
      set -- "$@" --config "vault_folder=$VAULT_FOLDER"
    fi
    step "Installing tightship for {{install:label}}" claude plugin install tightship@tightship "$@"
  fi

  _root=$(claude_code_root)
  [ -n "$_root" ] || fail "claude plugin list does not show where tightship@tightship is installed"
  _config="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
  mkdir -p "$_config/plugins/data/tightship-tightship"
  export TIGHTSHIP_STATUS_LINE="$STATUS_LINE"
  export TIGHTSHIP_SETTINGS="$_config/settings.json"
  export TIGHTSHIP_DATA="$_config/plugins/data/tightship-tightship"
  export TIGHTSHIP_ROOT="$_root"
  step "Removing the plugins tightship replaces" sh "$_root/bin/tightship" hook clean
  step "Applying the status line option" sh "$_root/bin/tightship" hook status-line

  if [ "$INIT_VAULT" = "true" ] && [ -n "$VAULT_FOLDER" ] && [ ! -d "$expanded_vault" ]; then
    step "Creating the vault at $expanded_vault" sh "$_root/bin/tightship" vault init "$expanded_vault"
  fi
  installed_in "{{install:label}}" "tightship@tightship, listed by claude plugin list"
}
