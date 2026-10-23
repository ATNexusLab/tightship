for %%I in ("%~dp0..") do set "PLUGIN_DIR=%%~fI"

if not defined TIGHTSHIP_ROOT (
  if defined PLUGIN_ROOT (
    set "TIGHTSHIP_ROOT=%PLUGIN_ROOT%"
  ) else (
    set "TIGHTSHIP_ROOT=%PLUGIN_DIR%"
  )
)

if not defined TIGHTSHIP_MANIFEST (
  set "TIGHTSHIP_MANIFEST=%TIGHTSHIP_ROOT%\plugin.json"
)

if not defined TIGHTSHIP_HARNESS (
  set "TIGHTSHIP_HARNESS=antigravity-cli"
)

if not defined TIGHTSHIP_DATA (
  if defined PLUGIN_DATA (
    set "TIGHTSHIP_DATA=%PLUGIN_DATA%"
  ) else (
    set "TIGHTSHIP_DATA={{path:data}}"
  )
)

if not defined TIGHTSHIP_SETTINGS (
  if defined AGY_CONFIG_DIR (
    set "TIGHTSHIP_SETTINGS=%AGY_CONFIG_DIR%\settings.json"
  ) else (
    set "TIGHTSHIP_SETTINGS={{path:config}}\settings.json"
  )
)

if not defined TIGHTSHIP_HOOK_OUTPUT (
  set "TIGHTSHIP_HOOK_OUTPUT=%PLUGIN_DIR%\hook-output.json"
)


if not defined TIGHTSHIP_PROJECT_POINTER (
  set "TIGHTSHIP_PROJECT_POINTER=/workspacePaths/0"
)
