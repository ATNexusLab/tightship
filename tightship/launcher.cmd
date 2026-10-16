@echo off
setlocal EnableExtensions
set "ROOT=%~dp0"
set "BIN=%ROOT%windows-x64\tightship.exe"
if exist "%BIN%" (
  "%BIN%" %*
  exit /b %ERRORLEVEL%
)
echo error: tightship has no binary for this machine >&2
exit /b 1
