#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="CottonMD"
BUNDLE_DIR="/Applications/${APP_NAME}.app"
CONTENTS="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS}/MacOS"
RES_DIR="${CONTENTS}/Resources"
MASTER_ICON="$ROOT_DIR/assets/AppIcon-1024.png"

cd "$ROOT_DIR"
cargo build --release -p cottonmd-desktop -p cottonmd-cli

if [ ! -f "$MASTER_ICON" ]; then
  echo "Missing master icon at $MASTER_ICON" >&2
  exit 1
fi

rm -rf "$BUNDLE_DIR"
mkdir -p "$MACOS_DIR" "$RES_DIR"

cp "$ROOT_DIR/target/release/CottonMD" "$MACOS_DIR/CottonMD"
cp "$ROOT_DIR/target/release/cottonmd-cli" "$MACOS_DIR/cottonmd-cli"
chmod +x "$MACOS_DIR/CottonMD" "$MACOS_DIR/cottonmd-cli"

TMP_ICON_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_ICON_DIR"' EXIT
ICONSET="$TMP_ICON_DIR/AppIcon.iconset"
mkdir -p "$ICONSET"

for sz in 16 32 128 256 512; do
  sips -z $sz $sz "$MASTER_ICON" --out "$ICONSET/icon_${sz}x${sz}.png" >/dev/null
  sz2=$((sz * 2))
  sips -z $sz2 $sz2 "$MASTER_ICON" --out "$ICONSET/icon_${sz}x${sz}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$RES_DIR/AppIcon.icns"
cp "$RES_DIR/AppIcon.icns" "$ROOT_DIR/assets/AppIcon.icns"

cat > "$CONTENTS/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>CottonMD</string>
    <key>CFBundleExecutable</key>
    <string>CottonMD</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>dev.cottonmd.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>CottonMD</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.2.0</string>
    <key>CFBundleVersion</key>
    <string>2</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Markdown</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>net.daringfireball.markdown</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
PLIST

codesign --force --deep --sign - "$BUNDLE_DIR"
touch "$BUNDLE_DIR"
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$BUNDLE_DIR" 2>/dev/null || true

echo "Bundled successfully at $BUNDLE_DIR"
du -sh "$BUNDLE_DIR" "$MACOS_DIR/CottonMD" "$MACOS_DIR/cottonmd-cli" "$RES_DIR/AppIcon.icns"
