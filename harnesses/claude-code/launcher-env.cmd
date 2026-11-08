if not defined TIGHTSHIP_MODEL_PREFIX (
  set "TIGHTSHIP_MODEL_PREFIX=Claude "
)
if not defined TIGHTSHIP_DATA (
  set "TIGHTSHIP_DATA=%~dp0..\..\..\..\..\data\tightship-tightship"
)
if not defined TIGHTSHIP_STATUS_LINE_REFRESH (
  set "TIGHTSHIP_STATUS_LINE_REFRESH=60"
)
