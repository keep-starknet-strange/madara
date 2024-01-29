#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage : $0 <version>"
    echo "ex : $0 2.5.0"
    exit 1
fi

CAIRO_REPO_DOWNLOAD_URL="https://github.com/starkware-libs/cairo/releases/download"
CAIRO_COMPILER_VERSION=$1
ARCHIVE_URL="$CAIRO_REPO_DOWNLOAD_URL/v$CAIRO_COMPILER_VERSION"

# Define the URLs for the Linux and Mac archives
LINUX_ARCHIVE="release-x86_64-unknown-linux-musl.tar.gz"
MAC_ARCHIVE="release-aarch64-apple-darwin.tar"

detect_os() {
    OS=$(uname -s)
    case "$OS" in
        Linux*)     echo "Linux";;
        Darwin*)    echo "Mac";;
        *)          echo "Unknown";;
    esac
}

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

find_project_root() {
    dir=$SCRIPT_DIR
    while [ "$dir" != "/" ]; do
        if [ -d "$dir/.git" ]; then
            echo "$dir"
            return
        fi
        dir=$(dirname "$dir")
    done
    echo "Project root not found."
    exit 1
}

ROOT_DIR=$(find_project_root)

echo "madara root dir is : $ROOT_DIR"

OS=$(detect_os)

# Build appropriate URL based on the OS
if [ "$OS" = "Linux" ]; then
    ARCHIVE_URL+="/$LINUX_ARCHIVE"
elif [ "$OS" = "Mac" ]; then
    ARCHIVE_URL+="/$MAC_ARCHIVE"
else
    echo "Unsupported operating system."
    exit 1
fi

echo "OS Detected : $OS"
echo "Downloading binairies ...\n"
# 1. GET BINARIES

# Define the URL of the archive and the directory to extract to
EXTRACT_DIR="$ROOT_DIR/cairo-contracts/scripts/bin"

# Download the archive
wget "$ARCHIVE_URL" -O /tmp/cairo_binaries.tar.gz

# Check if download was successful
if [ $? -eq 0 ]; then
    echo "Download successful, extracting the archive..."

    # Create the directory if it doesn't exist
    mkdir -p "$EXTRACT_DIR"

    # Extract the archive to the specified directory
    tar -xzf /tmp/cairo_binaries.tar.gz -C "$EXTRACT_DIR"

    # Check if extraction was successful
    if [ $? -eq 0 ]; then
        echo "Extraction successful."
    else
        echo "Error occurred during extraction."
        exit 1
    fi
else
    echo "Download failed. Please check the provided version"
    exit 1
fi

# Clean up
rm /tmp/cairo_binaries.tar.gz

# 2. COMPILE CONTRACTS

export MADARA_CAIRO_ONE_SRC_DIR="$ROOT_DIR/cairo-contracts/src/cairo_1"
export MADARA_CAIRO_ONE_OUTPUT_DIR="$ROOT_DIR/configs/genesis-assets"

export MADARA_STARKNET_COMPILE_BINARY="$SCRIPT_DIR/bin/cairo/bin/starknet-compile"
export MADARA_STARKNET_SIERRA_COMPILE_BINARY="$SCRIPT_DIR/bin/cairo/bin/starknet-sierra-compile"

# Location of starknet-compile

compile_cairo1_sierra() {
    local file="$1"
    local base_name=$(basename "$file" .cairo)
    local output_file="$MADARA_CAIRO_ONE_OUTPUT_DIR/$base_name"".sierra.json"

    # Run starknet-compile
    echo "$MADARA_STARKNET_COMPILE_BINARY" --single-file "$file" "$output_file"
    "$MADARA_STARKNET_COMPILE_BINARY" --single-file "$file" "$output_file"
}

compile_cairo1_casm() {
    local file="$1"
    local base_name=$(basename "$file" .sierra.json)
    local output_file="$MADARA_CAIRO_ONE_OUTPUT_DIR/$base_name"".casm.json"

    # Run starknet-compile
    echo "$MADARA_STARKNET_SIERRA_COMPILE_BINARY" "$file" "$output_file"
    "$MADARA_STARKNET_SIERRA_COMPILE_BINARY" "$file" "$output_file"
}

# Export the function so it's available to find -exec
export -f compile_cairo1_sierra
export -f compile_cairo1_casm

echo "Compiling cairo 1 contract contained in $MADARA_CAIRO_ONE_SRC_DIR to $MADARA_CAIRO_ONE_OUTPUT_DIR"

mkdir -p $MADARA_CAIRO_ONE_OUTPUT_DIR
find "$MADARA_CAIRO_ONE_SRC_DIR" -type f -name "*.cairo" -exec bash -c 'compile_cairo1_sierra "$0"' {} \;

echo "Compiling sierra to CASM\n"
find "$MADARA_CAIRO_ONE_OUTPUT_DIR" -type f -name "*sierra.json" -exec bash -c 'compile_cairo1_casm "$0"' {} \;


# Delete compiler binaries
rm -r "$SCRIPT_DIR/bin"
