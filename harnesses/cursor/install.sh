CURSOR_SHA256=""
CURSOR_DATA="${TIGHTSHIP_DATA:-{{path:data}}}"

check_cursor() {
  if [ -z "$FROM_DIR" ] && [ -z "$RELEASE_TAG" ]; then
    fail "this install.sh belongs to no release; run $REPOSITORY/releases/latest/download/install.sh, or pass --from-dir <folder>"
  fi
}

read_options_cursor() {
  _options="$CURSOR_DATA/options.json"
  [ -f "$_options" ] || return 0
  if ! tr -d ' \t\r\n' < "$_options" | grep -q '^{.*}$'; then
    fail "$_options is not valid JSON"
  fi
  existing_contract=$(sed -nE 's/.*"contract"[[:space:]]*:[[:space:]]*(true|false).*/\1/p' "$_options" | head -n1)
  existing_memory=$(sed -nE 's/.*"memory"[[:space:]]*:[[:space:]]*(true|false).*/\1/p' "$_options" | head -n1)
  existing_status_line=$(sed -nE 's/.*"status_line"[[:space:]]*:[[:space:]]*(true|false).*/\1/p' "$_options" | head -n1)
  existing_vault_folder=$(sed -nE 's/.*"vault_folder"[[:space:]]*:[[:space:]]*"((\\.|[^"\\])*)".*/\1/p' "$_options" | head -n1 | sed -E 's/\\(.)/\1/g')
}

json_bool() {
  if [ "$1" = "true" ]; then
    printf 'true'
  else
    printf 'false'
  fi
}

write_cursor_options() {
  mkdir -p "$CURSOR_DATA"
  _options="$CURSOR_DATA/options.json"
  if [ -n "$VAULT_FOLDER" ]; then
    _vault=$(printf '%s' "$VAULT_FOLDER" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')
    printf '{\n  "contract": %s,\n  "memory": %s,\n  "status_line": %s,\n  "vault_folder": "%s"\n}\n' \
      "$(json_bool "$CONTRACT")" "$(json_bool "$MEMORY")" "$(json_bool "$STATUS_LINE")" "$_vault" > "$_options"
  else
    printf '{\n  "contract": %s,\n  "memory": %s,\n  "status_line": %s\n}\n' \
      "$(json_bool "$CONTRACT")" "$(json_bool "$MEMORY")" "$(json_bool "$STATUS_LINE")" > "$_options"
  fi
}

install_cursor_companion() {
  _name="$1"
  _repo="$2"
  _dest="{{path:plugins}}/${_name}"
  _work="$WORK_DIR/companion-${_name}"
  mkdir -p "$_work"
  _zip="${_work}/${_name}.zip"
  _url="https://github.com/${_repo}/archive/refs/heads/main.zip"
  if ! download_file "$_url" "$_zip"; then
    _url="https://github.com/${_repo}/archive/refs/heads/master.zip"
    download_file "$_url" "$_zip" || return 1
  fi
  unzip -q "$_zip" -d "$_work" || return 1
  _archive="${_repo##*/}"
  if [ -d "${_work}/${_name}-main" ]; then
    _root="${_work}/${_name}-main"
  elif [ -d "${_work}/${_archive}-main" ]; then
    _root="${_work}/${_archive}-main"
  elif [ -d "${_work}/${_name}-master" ]; then
    _root="${_work}/${_name}-master"
  elif [ -d "${_work}/${_archive}-master" ]; then
    _root="${_work}/${_archive}-master"
  else
    _root=$(find "$_work" -mindepth 1 -maxdepth 1 -type d ! -name '.*' ! -name '__MACOSX' | head -1)
  fi
  if [ -z "$_root" ] || [ ! -d "$_root" ]; then
    echo "error: companion archive for ${_name} had no root folder" >&2
    return 1
  fi
  rm -rf "$_dest"
  mkdir -p "$_dest" || return 1
  cp -R "${_root}/." "$_dest/" || return 1
}

sync_cursor_plugin() {
  _target="$1"
  _source="$2"
  rm -rf "$_target"
  mkdir -p "$_target"
  cp -R "${_source}/." "$_target/"
}

install_cursor() {
  {{install:companions}}

  source_dir=""
  if [ -n "$FROM_DIR" ]; then
    source_dir="$FROM_DIR"
  else
    download_dir="$WORK_DIR/cursor"
    mkdir -p "$download_dir"
    zip_dest="$download_dir/tightship-cursor.zip"
    download_url="$REPOSITORY/releases/download/$RELEASE_TAG/tightship-cursor.zip"
    if ! run_step "Downloading tightship-cursor.zip $RELEASE_TAG" download_file "$download_url" "$zip_dest"; then
      fail "failed to download tightship-cursor.zip from $download_url" \
        "if you are running in a local repository clone, run cargo xtask build and pass --from-dir dist/cursor"
    fi
    zip_sha256=$(sha256_of "$zip_dest") || exit 1
    if ! run_step "Checking the sha256 of tightship-cursor.zip" [ "$zip_sha256" = "$CURSOR_SHA256" ]; then
      fail "tightship-cursor.zip has sha256 $zip_sha256; $RELEASE_TAG published $CURSOR_SHA256"
    fi
    unzip -q "$zip_dest" -d "$download_dir/extracted"
    source_dir="$download_dir/extracted"
  fi

  plugin_target="{{path:plugins}}/tightship"
  step "Installing tightship for {{install:label}}" sync_cursor_plugin "$plugin_target" "$source_dir"

  write_cursor_options

  export TIGHTSHIP_DATA="$CURSOR_DATA"
  export TIGHTSHIP_SETTINGS="{{path:config}}/cli-config.json"
  export TIGHTSHIP_ROOT="$plugin_target"
  step "Applying the status line option" "$plugin_target/bin/tightship" hook status-line

  if [ "$INIT_VAULT" = "true" ] && [ -n "$VAULT_FOLDER" ]; then
    step "Creating the vault at $expanded_vault" "$plugin_target/bin/tightship" vault init "$expanded_vault"
  fi
  installed_in "{{install:label}}" "$plugin_target"
}
