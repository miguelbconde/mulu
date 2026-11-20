#!/usr/bin/env bash
# Downloads the PDFium library that Mulu uses to read PDF files (decision D28), checks
# it against the checksum recorded below, and unpacks it into vendor/pdfium/PLATFORM.
#
# Usage: scripts/fetch-pdfium.sh PLATFORM   (win-x64, mac-univ or linux-x64)
#
# The files come from https://github.com/bblanchon/pdfium-binaries, which builds
# Google's PDFium on GitHub Actions without JavaScript (V8) and without XFA forms.
# Changing the version means changing the tag and the three checksums together.
set -euo pipefail

TAG="chromium/8125"
case "${1:-}" in
  win-x64)   SHA256="9a964261760cff52570aabf038ab346d3a5034dd8b30d0d5fe8861235d356b8b" ;;
  mac-univ)  SHA256="0cf06803b9a4acc489fc967a1b05990622ca1cf41fa027ff9cc89eeab9b19279" ;;
  linux-x64) SHA256="e1c3ce6e270cb5e0c20816b379ad9f2c1309972697c505f508de92bf5e7d6659" ;;
  *) echo "usage: $0 win-x64|mac-univ|linux-x64" >&2; exit 2 ;;
esac

root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/vendor/pdfium/$1"
archive="$(mktemp)"
trap 'rm -f "$archive"' EXIT

curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
  --output "$archive" "https://github.com/bblanchon/pdfium-binaries/releases/download/$TAG/pdfium-$1.tgz"

if command -v sha256sum >/dev/null; then
  actual="$(sha256sum "$archive" | cut -d' ' -f1)"
else
  actual="$(shasum -a 256 "$archive" | cut -d' ' -f1)"
fi
if [ "$actual" != "$SHA256" ]; then
  echo "PDFium checksum mismatch for $1: expected $SHA256, got $actual" >&2
  exit 1
fi

rm -rf "$target"
mkdir -p "$target"
tar -xzf "$archive" -C "$target"
# The library itself, its license and the licenses of the code it includes.
case "$1" in
  win-x64)   library="$target/bin/pdfium.dll" ;;
  mac-univ)  library="$target/lib/libpdfium.dylib" ;;
  linux-x64) library="$target/lib/libpdfium.so" ;;
esac
test -f "$library"
grep -q "pdf_enable_v8 = false" "$target/args.gn"
echo "$library"
