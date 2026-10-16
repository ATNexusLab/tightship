@echo off
setlocal EnableExtensions
set "MODE=%~1"
if "%MODE%"=="" set "MODE=unknown"
set "LOGDIR=%CURSOR_PROJECT_DIR%\.cursor-hook-probe"
if not exist "%LOGDIR%" mkdir "%LOGDIR%"
for /f %%i in ('powershell -NoProfile -Command "(Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')"') do set "STAMP=%%i"
set "LOGFILE=%LOGDIR%\%STAMP%-%MODE%.log"
powershell -NoProfile -Command "$m='%MODE%'; $in=[Console]::In.ReadToEnd(); $lines=@('=== %STAMP% event='+$m+' ===','argv: %*','CURSOR_PLUGIN_ROOT='+$env:CURSOR_PLUGIN_ROOT,'--- stdin ---',$in,'--- cursor env ---'); $lines += (Get-ChildItem Env:CURSOR*,Env:CLAUDE* | ForEach-Object { $_.Name+'='+$_.Value } | Sort-Object); $lines | Set-Content -LiteralPath '%LOGFILE%' -Encoding utf8"
if /I "%MODE%"=="beforeSubmitPrompt" if not exist "%LOGDIR%\limit-10000-fired" (
  type nul > "%LOGDIR%\limit-10000-fired"
  type "%CURSOR_PLUGIN_ROOT%\payloads\limit-10000.json"
  exit /b 0
)
echo {"additional_context":"probe-%MODE%-Windows"}
