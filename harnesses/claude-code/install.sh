install_claude_code() {
  claude plugin marketplace add DietrichGebert/ponytail || true
  claude plugin marketplace add ayghri/i-have-adhd || true
  claude plugin marketplace add blader/humanizer || true

  if [ -n "$FROM_DIR" ]; then
    claude plugin marketplace add "$FROM_DIR" || true
  else
    claude plugin marketplace add "${REPOSITORY#https://github.com/}" || true
  fi

  set -- --config "contract=$CONTRACT" --config "memory=$MEMORY" --config "status_line=$STATUS_LINE"
  if [ -n "$VAULT_FOLDER" ]; then
    set -- "$@" --config "vault_folder=$VAULT_FOLDER"
  fi
  step "Installing tightship for {{install:label}}" claude plugin install tightship@tightship "$@"
}
