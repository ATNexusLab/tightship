install_claude_code() {
  {{install:companions}}

  if [ -n "$FROM_DIR" ]; then
    quietly claude plugin marketplace add "$FROM_DIR"
  else
    quietly claude plugin marketplace add "${REPOSITORY#https://github.com/}"
  fi

  set -- --config "contract=$CONTRACT" --config "memory=$MEMORY" --config "status_line=$STATUS_LINE"
  if [ -n "$VAULT_FOLDER" ]; then
    set -- "$@" --config "vault_folder=$VAULT_FOLDER"
  fi
  step "Installing tightship for {{install:label}}" claude plugin install tightship@tightship "$@"
  installed_in "{{install:label}}" "tightship@tightship, listed by claude plugin list"
}
