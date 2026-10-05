#!/bin/sh
set -eu

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
binary="$project_root/${1:-target/release/autoclicker-x20}"
app="$project_root/dist/Autoclicker X20.app"
output="$project_root/outputs/Autoclicker-X20-0.1.0-macos-arm64.zip"

test -x "$binary"
test "$(uname -s)" = Darwin
lipo "$binary" -verify_arch arm64
test ! -e "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" "$project_root/outputs"
cp -X "$binary" "$app/Contents/MacOS/autoclicker-x20"
cp -X "$project_root/packaging/macos/Info.plist" "$app/Contents/Info.plist"
cp -X "$project_root/LICENSE" "$app/Contents/Resources/LICENSE"
cp -X "$project_root/docs/LICENSE-LUCIDE" "$app/Contents/Resources/LICENSE-LUCIDE"
cp -X "$project_root/docs/THIRD_PARTY_NOTICES.html" "$app/Contents/Resources/THIRD_PARTY_NOTICES.html"
plutil -lint "$app/Contents/Info.plist"
/usr/bin/xattr -cr "$app"
codesign --force --sign - --timestamp=none "$app"
codesign --verify --deep --strict "$app"
ditto -c -k --norsrc --noextattr --keepParent "$app" "$output"
echo "$output"
