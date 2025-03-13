#!/bin/bash

# Default values
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    TARGET="x86_64-pc-windows-gnu"
else
    TARGET="x86_64-unknown-linux-gnu"
fi
BUILD_TYPE="release"
SHOW_HELP=false

# Usage function
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Build the FujiNet HAL library"
    echo
    echo "Options:"
    echo "  -t TARGET    Build target (default: based on host OS)"
    echo "              Supported targets:"
    echo "              - x86-win  (x86_64-pc-windows-gnu)"
    echo "              - x86-linux (x86_64-unknown-linux-gnu)"
    echo "              - esp32    (esp32-unknown-none-elf)"
    echo "              - atari    (m68k-atari-mint)"
    echo "  -d          Development build (debug mode)"
    echo "  -h          Show this help message"
    echo
    echo "Examples:"
    echo "  $0 -t x86-win        # Build for Windows x86 in release mode"
    echo "  $0 -t x86-linux -d   # Build for Linux x86 in debug mode"
    echo "  $0 -h               # Show help"
}

# Parse command line options
while getopts "t:dh" opt; do
    case $opt in
        t)
            case $OPTARG in
                x86-win)
                    TARGET="x86_64-pc-windows-gnu"
                    ;;
                x86-linux)
                    TARGET="x86_64-unknown-linux-gnu"
                    ;;
                esp32)
                    TARGET="esp32-unknown-none-elf"
                    ;;
                atari)
                    TARGET="m68k-atari-mint"
                    ;;
                *)
                    echo "Error: Invalid target '$OPTARG'"
                    usage
                    exit 1
                    ;;
            esac
            ;;
        d)
            BUILD_TYPE="debug"
            ;;
        h)
            SHOW_HELP=true
            ;;
        \?)
            echo "Error: Invalid option -$OPTARG"
            usage
            exit 1
            ;;
        :)
            echo "Error: Option -$OPTARG requires an argument"
            usage
            exit 1
            ;;
    esac
done

# Show help if requested
if [ "$SHOW_HELP" = true ]; then
    usage
    exit 0
fi

# Add target if not already added
echo "Checking if target $TARGET is installed..."
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "Adding target $TARGET..."
    rustup target add "$TARGET"
fi

# Check for required toolchains
if [[ "$TARGET" == "x86_64-pc-windows-gnu" ]]; then
    echo "Checking for MinGW toolchain..."
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo "MinGW toolchain not found. Please install it:"
        echo "For Windows (Msys2) install appropriate package for your environment (mingw64, ucrt64, clang64, etc.):"
        echo "  pacman -S mingw-w64-ucrt-x86_64-toolchain"
        echo "  pacman -S mingw-w64-clang-x86_64-toolchain"
        echo "  pacman -S mingw-w64-x86_64-toolchain"
        echo ""
        echo "For Linux:"
        echo "  sudo apt update"
        echo "  sudo apt install mingw-w64"
        exit 1
    fi
fi

# Build command
BUILD_CMD="cargo build"
if [ "$BUILD_TYPE" = "release" ]; then
    BUILD_CMD="$BUILD_CMD --release"
fi
BUILD_CMD="$BUILD_CMD --target $TARGET"

# Execute build
echo "Building for $TARGET in $BUILD_TYPE mode..."
$BUILD_CMD

# Check build result
if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Output location: target/$TARGET/$BUILD_TYPE/"
else
    echo "Build failed!"
    exit 1
fi 