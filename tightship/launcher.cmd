@echo off & setlocal
set "BINARY=%~dp0windows-x64\tightship.exe"
if not exist "%BINARY%" (
  echo error: tightship has no binary for Windows 64-bit >&2
  exit /b 1
)

"%BINARY%" %*
exit /b %ERRORLEVEL%
