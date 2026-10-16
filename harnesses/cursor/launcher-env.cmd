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
