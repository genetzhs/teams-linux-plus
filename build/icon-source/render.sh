#!/usr/bin/env bash
# Regenerates every PNG icon from the SVG sources in this folder.
# Needs rsvg-convert (librsvg2-bin / librsvg2-tools).
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
icon="$here/icon.svg"
mono="$here/tray-monochrome.svg"

render() { rsvg-convert -w "$2" -h "$2" "$1" -o "$3"; }

# Packaging icons (electron-builder, .desktop, AppStream)
for size in 16 24 32 48 64 96 128 256 512 1024; do
  render "$icon" "$size" "$root/build/icons/${size}x${size}.png"
done

# Window, tray and dock icons
assets="$root/app/assets/icons"
for size in 16 96 256; do
  render "$icon" "$size" "$assets/icon-${size}x${size}.png"
done

light="$(mktemp --suffix=.svg)"
trap 'rm -f "$light"' EXIT
sed 's/color="#000000"/color="#FFFFFF"/' "$mono" > "$light"
for size in 16 96; do
  render "$mono" "$size" "$assets/icon-monochrome-dark-${size}x${size}.png"
  render "$light" "$size" "$assets/icon-monochrome-light-${size}x${size}.png"
done

# Rust preview client
render "$icon" 256 "$root/rust/icons/icon.png"
render "$icon" 96 "$root/rust/icons/tray.png"
render "$icon" 256 "$root/rust/ui/icon.png"
