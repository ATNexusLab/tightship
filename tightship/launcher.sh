#!/bin/sh
system=$(uname -s)
machine=$(uname -m)
case "$system $machine" in
Darwin\ *) platform=macos ;;
Linux\ x86_64) platform=linux-x64 ;;
Linux\ aarch64 | Linux\ arm64) platform=linux-arm64 ;;
MINGW*\ x86_64 | MSYS*\ x86_64 | CYGWIN*\ x86_64) platform=windows-x64 ;;
MINGW*\ aarch64 | MSYS*\ aarch64 | CYGWIN*\ aarch64) platform=windows-x64 ;;
MINGW*\ arm64 | MSYS*\ arm64 | CYGWIN*\ arm64) platform=windows-x64 ;;
*) platform=unknown ;;
esac
binary="$(dirname "$0")/$platform/tightship"
[ "$platform" = windows-x64 ] && binary="$binary.exe"
if [ -f "$binary" ]; then
  [ -x "$binary" ] || chmod +x "$binary" "$0"
  exec "$binary" "$@"
fi
echo "error: tightship has no binary for $system $machine" >&2
exit 1
