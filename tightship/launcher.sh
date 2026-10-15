#!/bin/sh
system=$(uname -s)
machine=$(uname -m)
case "$system $machine" in
Darwin\ *) platform=macos ;;
Linux\ x86_64) platform=linux-x64 ;;
Linux\ aarch64 | Linux\ arm64) platform=linux-arm64 ;;
MINGW*\ x86_64 | MSYS*\ x86_64 | CYGWIN*\ x86_64) platform=windows-x64 ;;
*) platform=unknown ;;
esac
binary="$(dirname "$0")/$platform/tightship"
[ "$platform" = windows-x64 ] && binary="$binary.exe"
[ -x "$binary" ] && exec "$binary" "$@"
echo "error: tightship has no binary for $system $machine" >&2
exit 1
