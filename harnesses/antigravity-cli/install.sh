ANTIGRAVITY_CLI_SHA256=""
ANTIGRAVITY_CLI_DATA="${TIGHTSHIP_DATA:-${PLUGIN_DATA:-{{path:data}}}}"

check_antigravity_cli() {
  if [ -z "$FROM_DIR" ] && [ -z "$RELEASE_TAG" ]; then
    echo "error: this install.sh belongs to no release; run $REPOSITORY/releases/latest/download/install.sh, or pass --from-dir <folder>" >&2
    exit 1
  fi
}

options_file_antigravity_cli() {
  printf '%s\n' "$ANTIGRAVITY_CLI_DATA/options.toml"
}

install_skill() {
  skill_dir="{{path:skills}}/$1"
  mkdir -p "$skill_dir"
  step "Downloading the $1 skill" download_file "https://raw.githubusercontent.com/$2/main/SKILL.md" "$skill_dir/SKILL.md"
}

install_antigravity_cli() {
  {{install:companions}}

  source_dir=""
  if [ -n "$FROM_DIR" ]; then
    source_dir="$FROM_DIR"
  else
    TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'tightship')
    zip_dest="$TMP_DIR/tightship-antigravity-cli.zip"
    download_url="$REPOSITORY/releases/download/$RELEASE_TAG/tightship-antigravity-cli.zip"
    echo "> Downloading tightship-antigravity-cli.zip $RELEASE_TAG"
    if ! download_file "$download_url" "$zip_dest"; then
      echo "error: failed to download tightship-antigravity-cli.zip from $download_url" >&2
      echo "hint: if you are running in a local repository clone, run cargo xtask build and pass --from-dir dist/antigravity-cli" >&2
      exit 1
    fi
    echo "> Checking the sha256 of tightship-antigravity-cli.zip"
    zip_sha256=$(sha256_of "$zip_dest") || exit 1
    if [ "$zip_sha256" != "$ANTIGRAVITY_CLI_SHA256" ]; then
      echo "error: tightship-antigravity-cli.zip has sha256 $zip_sha256; $RELEASE_TAG published $ANTIGRAVITY_CLI_SHA256" >&2
      exit 1
    fi
    unzip -q "$zip_dest" -d "$TMP_DIR/extracted"
    source_dir="$TMP_DIR/extracted"
  fi

  step "Installing tightship for {{install:label}}" agy plugin install "$source_dir"
  plugin_target="{{path:plugins}}/tightship"

  mkdir -p "$ANTIGRAVITY_CLI_DATA"
  {
    printf "contract = %s\n" "$CONTRACT"
    printf "memory = %s\n" "$MEMORY"
    printf "status_line = %s\n" "$STATUS_LINE"
    if [ -n "$VAULT_FOLDER" ]; then
      printf "vault_folder = \"%s\"\n" "$(printf '%s' "$VAULT_FOLDER" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')"
    fi
  } > "$ANTIGRAVITY_CLI_DATA/options.toml"

  export TIGHTSHIP_DATA="$ANTIGRAVITY_CLI_DATA"
  mkdir -p "{{path:config}}"
  step "Applying the status line option" "$plugin_target/bin/tightship" hook status-line

  if [ "$INIT_VAULT" = "true" ] && [ -n "$VAULT_FOLDER" ]; then
    step "Creating the vault at $VAULT_FOLDER" "$plugin_target/bin/tightship" vault init "$VAULT_FOLDER"
  fi
}
