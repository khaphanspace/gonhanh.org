#!/bin/bash
set -e

# Source rustup environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Source .env file if exists (for Apple Developer credentials)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$SCRIPT_DIR/../.env" ]; then
    set -a
    source "$SCRIPT_DIR/../.env"
    set +a
fi

# Parse arguments
CLEAN_INSTALL=false
CUSTOM_VERSION=""
SIGN_APP=false
NOTARIZE_APP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN_INSTALL=true
            shift
            ;;
        --version|-v)
            CUSTOM_VERSION="$2"
            shift 2
            ;;
        --sign|-s)
            SIGN_APP=true
            shift
            ;;
        --notarize|-n)
            SIGN_APP=true
            NOTARIZE_APP=true
            shift
            ;;
        --help|-h)
            echo "Usage: build-macos.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --version, -v VERSION  Build with custom version (e.g., 0.9.0 for testing updates)"
            echo "  --clean                Remove existing GoNhanh app and clear permissions before building"
            echo "  --sign, -s             Sign with Developer ID (requires certificate in Keychain)"
            echo "  --notarize, -n         Sign and notarize for distribution (requires Apple ID credentials)"
            echo "  --help                 Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./build-macos.sh                    # Build with ad-hoc signing (development)"
            echo "  ./build-macos.sh -v 0.9.0           # Build with version 0.9.0"
            echo "  ./build-macos.sh --sign             # Build with Developer ID signing"
            echo "  ./build-macos.sh --notarize         # Build, sign, and notarize for distribution"
            echo ""
            echo "Environment variables for signing:"
            echo "  APPLE_SIGNING_IDENTITY   Certificate name (e.g., 'Developer ID Application: Name (TEAM_ID)')"
            echo "  APPLE_TEAM_ID            Your Apple Developer Team ID"
            echo ""
            echo "Environment variables for notarization:"
            echo "  APPLE_ID                 Your Apple ID email"
            echo "  APPLE_APP_PASSWORD       App-specific password from appleid.apple.com"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

# Clean install: remove existing app and reset permissions
if [ "$CLEAN_INSTALL" = true ]; then
    echo "Cleaning existing installation..."

    # Kill running GoNhanh processes
    if pgrep -f "GoNhanh" > /dev/null 2>&1; then
        echo "Stopping running GoNhanh processes..."
        pkill -f "GoNhanh" 2>/dev/null || true
        # Wait for process to terminate
        sleep 1
        # Force kill if still running
        if pgrep -f "GoNhanh" > /dev/null 2>&1; then
            echo "Force killing GoNhanh..."
            pkill -9 -f "GoNhanh" 2>/dev/null || true
            sleep 1
        fi
        echo "GoNhanh processes stopped."
    else
        echo "No running GoNhanh process found."
    fi

    # Remove from /Applications (requires sudo)
    if [ -d "/Applications/GoNhanh.app" ]; then
        echo "Removing /Applications/GoNhanh.app (requires sudo)..."
        sudo rm -rf "/Applications/GoNhanh.app"
    fi

    # Remove from Input Methods
    if [ -d "$HOME/Library/Input Methods/GoNhanh.app" ]; then
        echo "Removing ~/Library/Input Methods/GoNhanh.app..."
        rm -rf "$HOME/Library/Input Methods/GoNhanh.app"
    fi

    # Clear TCC database (Accessibility permissions) - requires Full Disk Access or SIP disabled
    echo "Note: To fully reset Accessibility permissions, go to:"
    echo "  System Settings > Privacy & Security > Accessibility"
    echo "  Remove GoNhanh from the list manually"
    echo ""

    # Clear input source registration
    echo "Clearing input source cache..."
    defaults delete com.apple.HIToolbox AppleEnabledInputSources 2>/dev/null || true

    echo "Clean complete!"
    echo ""
fi

# Always kill running GoNhanh before build (even without --clean)
if pgrep -x "GoNhanh" > /dev/null 2>&1; then
    echo "Stopping running GoNhanh..."
    pkill -x "GoNhanh" 2>/dev/null || true
    sleep 0.5
    # Force kill if still running
    if pgrep -x "GoNhanh" > /dev/null 2>&1; then
        pkill -9 -x "GoNhanh" 2>/dev/null || true
        sleep 0.5
    fi
fi

echo "Building macOS app..."

# Get version
if [ -n "$CUSTOM_VERSION" ]; then
    VERSION="$CUSTOM_VERSION"
    echo "Version (custom): $VERSION"
else
    GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
    VERSION=${GIT_TAG#v}  # Remove 'v' prefix
    echo "Version (git tag): $VERSION"
fi

# Validate signing configuration
if [ "$SIGN_APP" = true ]; then
    if [ -z "$APPLE_SIGNING_IDENTITY" ]; then
        # Try to find Developer ID certificate automatically
        APPLE_SIGNING_IDENTITY=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "")

        if [ -z "$APPLE_SIGNING_IDENTITY" ]; then
            echo "Error: No Developer ID Application certificate found in Keychain."
            echo ""
            echo "Either:"
            echo "  1. Set APPLE_SIGNING_IDENTITY environment variable"
            echo "  2. Install Developer ID Application certificate in Keychain"
            echo ""
            echo "To list available certificates:"
            echo "  security find-identity -v -p codesigning"
            exit 1
        fi
        echo "Found certificate: $APPLE_SIGNING_IDENTITY"
    fi

    if [ "$NOTARIZE_APP" = true ]; then
        if [ -z "$APPLE_ID" ] || [ -z "$APPLE_APP_PASSWORD" ] || [ -z "$APPLE_TEAM_ID" ]; then
            echo "Error: Notarization requires APPLE_ID, APPLE_APP_PASSWORD, and APPLE_TEAM_ID"
            echo ""
            echo "Set these environment variables:"
            echo "  export APPLE_ID='your@email.com'"
            echo "  export APPLE_APP_PASSWORD='xxxx-xxxx-xxxx-xxxx'"
            echo "  export APPLE_TEAM_ID='XXXXXXXXXX'"
            exit 1
        fi
    fi
fi

# Build macOS app with xcodebuild
cd "$(dirname "$0")/../platforms/macos"

if [ -d "GoNhanh.xcodeproj" ]; then
    echo "Building with Xcode..."

    if [ "$SIGN_APP" = true ]; then
        echo "Building with Developer ID signing..."
        xcodebuild -scheme GoNhanh \
            -configuration Release \
            -destination 'platform=macOS,arch=arm64' \
            -destination 'platform=macOS,arch=x86_64' \
            -derivedDataPath "$(pwd)/build/DerivedData" \
            MARKETING_VERSION="$VERSION" \
            CURRENT_PROJECT_VERSION="$VERSION" \
            CODE_SIGN_IDENTITY="$APPLE_SIGNING_IDENTITY" \
            DEVELOPMENT_TEAM="$APPLE_TEAM_ID" \
            CODE_SIGN_STYLE="Manual" \
            OTHER_CODE_SIGN_FLAGS="--options=runtime" \
            2>&1 | grep -v "Using the first of multiple matching destinations"
    else
        echo "Building with ad-hoc signing (development)..."
        xcodebuild -scheme GoNhanh \
            -configuration Release \
            -destination 'platform=macOS,arch=arm64' \
            -destination 'platform=macOS,arch=x86_64' \
            -derivedDataPath "$(pwd)/build/DerivedData" \
            MARKETING_VERSION="$VERSION" \
            CURRENT_PROJECT_VERSION="$VERSION" \
            2>&1 | grep -v "Using the first of multiple matching destinations"
    fi

    # Copy app from DerivedData to local build directory
    echo "Copying app to build directory..."
    mkdir -p build/Release
    cp -R "build/DerivedData/Build/Products/Release/GoNhanh.app" build/Release/

    # Sign app with entitlements
    echo "Signing app with entitlements..."
    if [ "$SIGN_APP" = true ]; then
        codesign --force --deep --sign "$APPLE_SIGNING_IDENTITY" \
            --entitlements GoNhanh.entitlements.production \
            --options runtime \
            --timestamp \
            build/Release/GoNhanh.app
        echo "Signed with Developer ID: $APPLE_SIGNING_IDENTITY"
    else
        codesign --force --deep --sign - \
            --entitlements GoNhanh.entitlements \
            build/Release/GoNhanh.app
        echo "Signed with ad-hoc identity (development only)"
    fi

    # Verify signature
    echo "Verifying signature..."
    codesign -vvv --deep --strict build/Release/GoNhanh.app
    echo "Signature verified!"

    # Notarize if requested
    if [ "$NOTARIZE_APP" = true ]; then
        echo ""
        echo "Notarizing app..."

        # Create ZIP for notarization
        echo "Creating ZIP for notarization..."
        ditto -c -k --keepParent build/Release/GoNhanh.app build/Release/GoNhanh-notarize.zip

        # Submit for notarization
        echo "Submitting to Apple for notarization (this may take a few minutes)..."
        xcrun notarytool submit build/Release/GoNhanh-notarize.zip \
            --apple-id "$APPLE_ID" \
            --password "$APPLE_APP_PASSWORD" \
            --team-id "$APPLE_TEAM_ID" \
            --wait \
            --timeout 30m

        # Staple the notarization ticket
        echo "Stapling notarization ticket..."
        xcrun stapler staple build/Release/GoNhanh.app

        # Verify notarization
        echo "Verifying notarization..."
        spctl -a -vvv -t install build/Release/GoNhanh.app

        # Cleanup
        rm -f build/Release/GoNhanh-notarize.zip

        echo "Notarization completed!"
    fi

    echo ""
    echo "macOS app built successfully!"
    echo "App: platforms/macos/build/Release/GoNhanh.app"

    if [ "$SIGN_APP" = true ]; then
        echo "Status: Signed with Developer ID"
        if [ "$NOTARIZE_APP" = true ]; then
            echo "Status: Notarized and stapled"
        fi
    else
        echo "Status: Ad-hoc signed (development only)"
        echo ""
        echo "For distribution, run:"
        echo "  ./build-macos.sh --sign       # Developer ID signing"
        echo "  ./build-macos.sh --notarize   # Full notarization"
    fi
else
    echo "Xcode project not found!"
    echo "Please create Xcode project in platforms/macos/"
    echo "Steps:"
    echo "  1. Open Xcode"
    echo "  2. Create new macOS App project"
    echo "  3. Name: GoNhanh"
    echo "  4. Location: platforms/macos/"
    echo "  5. Add Swift files from GoNhanh/ folder"
    echo "  6. Link libgonhanh_core.a in Build Phases"
fi
