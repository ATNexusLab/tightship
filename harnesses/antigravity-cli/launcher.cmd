@echo off
setlocal

set "BIN_DIR=%~dp0"
for %%I in ("%BIN_DIR%..") do set "DEFAULT_ROOT=%%~fI"

if not defined TIGHTSHIP_ROOT (
  if defined PLUGIN_ROOT (
    set "TIGHTSHIP_ROOT=%PLUGIN_ROOT%"
  ) else (
    set "TIGHTSHIP_ROOT=%DEFAULT_ROOT%"
  )
)

if not defined TIGHTSHIP_DATA (
  if defined PLUGIN_DATA (
    set "TIGHTSHIP_DATA=%PLUGIN_DATA%"
  ) else (
    set "TIGHTSHIP_DATA=%USERPROFILE%\.gemini\config\plugin_data\tightship"
  )
)

if not defined TIGHTSHIP_SETTINGS (
  if defined AGY_CONFIG_DIR (
    set "TIGHTSHIP_SETTINGS=%AGY_CONFIG_DIR%\settings.json"
  ) else (
    set "TIGHTSHIP_SETTINGS=%USERPROFILE%\.gemini\antigravity-cli\settings.json"
  )
)

if not defined TIGHTSHIP_HOOK_FORMAT (
  set "TIGHTSHIP_HOOK_FORMAT=inject-steps"
)

set "BINARY=%BIN_DIR%windows-x64\tightship.exe"
if not exist "%BINARY%" (
  echo error: tightship has no binary for Windows 64-bit >&2
  exit /b 1
)

"%BINARY%" %*
exit /b %ERRORLEVEL%
