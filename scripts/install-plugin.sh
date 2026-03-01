#!/usr/bin/env bash
# Build and install the WAIL CLAP and VST3 plugins.
#
# Usage:
#   ./scripts/install-plugin.sh           # build + install
#   ./scripts/install-plugin.sh --no-build  # install already-built bundles only
#
set -euo pipefail
cd "$(dirname "$0")/.."

NO_BUILD=false
for arg in "$@"; do
    case "$arg" in
        --no-build) NO_BUILD=true ;;
        *) echo "Unknown argument: $arg"; exit 1 ;;
    esac
done

CLAP_BUNDLE="target/bundled/wail-plugin.clap"
VST3_BUNDLE="target/bundled/wail-plugin.vst3"

# --- Build ---
if [ "$NO_BUILD" = false ]; then
    echo "Building WAIL plugin..."
    cargo nih-plug bundle wail-plugin --release
fi

# Verify bundles exist
for bundle in "$CLAP_BUNDLE" "$VST3_BUNDLE"; do
    if [ ! -e "$bundle" ]; then
        echo "Error: $bundle not found. Run without --no-build to build first."
        exit 1
    fi
done

# --- Install ---
case "$(uname -s)" in
    Darwin)
        CLAP_DIR="$HOME/Library/Audio/Plug-Ins/CLAP"
        VST3_DIR="$HOME/Library/Audio/Plug-Ins/VST3"
        ;;
    Linux)
        CLAP_DIR="$HOME/.clap"
        VST3_DIR="$HOME/.vst3"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        CLAP_DIR="${COMMONPROGRAMFILES}/CLAP"
        VST3_DIR="${COMMONPROGRAMFILES}/VST3"
        ;;
    *)
        echo "Unsupported platform: $(uname -s)"
        exit 1
        ;;
esac

mkdir -p "$CLAP_DIR" "$VST3_DIR"

cp -r "$CLAP_BUNDLE" "$CLAP_DIR/"
echo "Installed: $CLAP_DIR/$(basename "$CLAP_BUNDLE")"

cp -r "$VST3_BUNDLE" "$VST3_DIR/"
echo "Installed: $VST3_DIR/$(basename "$VST3_BUNDLE")"

echo ""
echo "Done. Rescan plugins in your DAW to pick up the changes."
