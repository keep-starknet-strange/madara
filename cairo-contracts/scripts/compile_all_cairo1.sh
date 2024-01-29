#!/bin/bash


# Function to determine the Operating System
detect_os() {
    OS=$(uname -s)
    case "$OS" in
        Linux*)     echo "Linux";;
        Darwin*)    echo "Mac";;
        *)          echo "Unknown";;
    esac
}

# Define the URLs for the Linux and Mac archives
LINUX_ARCHIVE_URL="https://github.com/starkware-libs/cairo/releases/download/v2.5.0/release-x86_64-unknown-linux-musl.tar.gz"
MAC_ARCHIVE_URL="https://github.com/starkware-libs/cairo/releases/download/v2.5.0/release-aarch64-apple-darwin.tar"


SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)


# Function to find the project root directory
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

echo "madara root dir is : $EXTRACT_DIR"

# Detect the operating system
OS=$(detect_os)

# Choose the appropriate URL based on the OS
if [ "$OS" = "Linux" ]; then
    ARCHIVE_URL=$LINUX_ARCHIVE_URL
elif [ "$OS" = "Mac" ]; then
    ARCHIVE_URL=$MAC_ARCHIVE_URL
else
    echo "Unsupported operating system."
    exit 1
fi

echo "your os is $OS, continuing ..."
# 1. GET BINARIES

# Define the URL of the archive and the directory to extract to
EXTRACT_DIR="$ROOT_DIR/cairo-contracts/scripts/bin"

# Download the archive
wget "$ARCHIVE_URL" -O /tmp/archive.tar.gz

# Check if download was successful
if [ $? -eq 0 ]; then
    echo "Download successful, extracting the archive..."

    # Create the directory if it doesn't exist
    mkdir -p "$EXTRACT_DIR"

    # Extract the archive to the specified directory
    tar -xzf /tmp/archive.tar.gz -C "$EXTRACT_DIR"

    # Check if extraction was successful
    if [ $? -eq 0 ]; then
        echo "Extraction successful."
    else
        echo "Error occurred during extraction."
    fi
else
    echo "Download failed."
fi

# Clean up
rm /tmp/archive.tar.gz

# 2. COMPILE CONTRACTS

export MADARA_CAIRO_ONE_SRC_DIR="$ROOT_DIR/cairo-contracts/src/cairo_1"
export MADARA_CAIRO_ONE_OUTPUT_DIR="$ROOT_DIR/configs/genesis-assets/cairo_1"
export MADARA_STARKNET_COMPILE_BINARY="$SCRIPT_DIR/bin/cairo/bin/starknet-compile"

mkdir -p $MADARA_CAIRO_ONE_OUTPUT_DIR
# Location of starknet-compile

compile_cairo1() {
    local file="$1"
    local base_name=$(basename "$file" .cairo)
    local output_file="$MADARA_CAIRO_ONE_OUTPUT_DIR/$base_name""CairoOne.json"

    # Run starknet-compile
    echo "$MADARA_STARKNET_COMPILE_BINARY" --single-file "$file" "$output_file"
    "$MADARA_STARKNET_COMPILE_BINARY" --single-file "$file" "$output_file"
}

# Export the function so it's available to find -exec
export -f compile_cairo1

# Find all files and apply the command to each
find "$MADARA_CAIRO_ONE_SRC_DIR" -type f -name "*.cairo" -exec bash -c 'compile_cairo1 "$0"' {} \;

# Delete binaries
rm -r "$SCRIPT_DIR/bin"
