for %%I in ("%~dp0..") do set "PLUGIN_DIR=%%~fI"

if not defined TIGHTSHIP_ROOT (
  if defined CURSOR_PLUGIN_ROOT (
    set "TIGHTSHIP_ROOT=%CURSOR_PLUGIN_ROOT%"
  ) else (
    set "TIGHTSHIP_ROOT=%PLUGIN_DIR%"
  )
)

if not defined TIGHTSHIP_DATA (
  set "TIGHTSHIP_DATA={{path:data}}"
)

if not defined TIGHTSHIP_SETTINGS (
  set "TIGHTSHIP_SETTINGS={{path:config}}/cli-config.json"
)

if not defined TIGHTSHIP_HOOK_OUTPUT (
  set "TIGHTSHIP_HOOK_OUTPUT=%PLUGIN_DIR%\hook-output.json"
)

if not defined TIGHTSHIP_CHECKPOINT_OUTPUT (
  set "TIGHTSHIP_CHECKPOINT_OUTPUT=%PLUGIN_DIR%\checkpoint-output.json"
)

if not defined TIGHTSHIP_STATUS_LINE_COMMAND (
  set "TIGHTSHIP_STATUS_LINE_COMMAND="${CURSOR_PLUGIN_ROOT}/bin/tightship" status-line render"
)
