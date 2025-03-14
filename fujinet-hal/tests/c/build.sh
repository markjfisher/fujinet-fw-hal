#!/bin/bash

# Get the directory containing the build script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Get the root directory of the project (three levels up)
ROOT_DIR="$SCRIPT_DIR/../../../"

# Set target and build type based on environment
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
    TARGET_FLAG="-t x86-win"
    # Windows needs additional libraries for networking, crypto, and system functions
    LINK_FLAGS="-lws2_32 -lbcrypt -ladvapi32 -lcrypt32 -lsecur32 -lschannel -lntdll -lkernel32 -lmsvcrt -luserenv -lncrypt"
else
    TARGET_FLAG="-t x86-linux"
    LINK_FLAGS="-lpthread -ldl"  # Unix/Linux specific libraries
fi

# Build the library using the project's build script
echo "Building library..."
(cd "$ROOT_DIR" && ./build.sh $TARGET_FLAG)

# Determine target directory from build.sh's mapping
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
    TARGET="x86_64-pc-windows-gnu"
else
    TARGET="x86_64-unknown-linux-gnu"
fi
BUILD_TYPE="release"

# Copy the static library to the local directory
echo "Copying static library..."
cp "$ROOT_DIR/target/$TARGET/$BUILD_TYPE/libfujinet_hal.a" "$SCRIPT_DIR/"

# Compile the test with static linking
echo "Compiling test with static linking..."
gcc -o http_test http_test.c -L. -l:libfujinet_hal.a $LINK_FLAGS

# Clean up the copied library
rm "$SCRIPT_DIR/libfujinet_hal.a"

echo "Build complete. Run with: ./http_test" 