if not defined TIGHTSHIP_MODEL_PREFIX (
  set "TIGHTSHIP_MODEL_PREFIX=Claude "
)
if not defined TIGHTSHIP_DATA (
  set "TIGHTSHIP_DATA=%~dp0..\..\..\..\..\data\tightship-tightship"
)
if not defined TIGHTSHIP_STATUS_LINE_REFRESH (
  set "TIGHTSHIP_STATUS_LINE_REFRESH=60"
)
if not defined TIGHTSHIP_CLEAN (
  set "TIGHTSHIP_CLEAN=%~dp0..\clean.json"
)
if not defined TIGHTSHIP_CONFIG (
  if defined CLAUDE_CONFIG_DIR (
    set "TIGHTSHIP_CONFIG=%CLAUDE_CONFIG_DIR%"
  ) else (
    set "TIGHTSHIP_CONFIG=%USERPROFILE%\.claude"
  )
)
