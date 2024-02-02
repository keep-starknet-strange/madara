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

# 2. COMPILE CAIRO 1 CONTRACTS

export MADARA_CAIRO_ONE_SRC_DIR="$ROOT_DIR/cairo-contracts/src/cairo_1"
export MADARA_CAIRO_ONE_OUTPUT_DIR="$ROOT_DIR/cairo-contracts/compiled_contract/cairo_1"

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


# 3. COMPILE CAIRO 0 CONTRACTS
echo "\033[31mCAIRO ZERO\033[0m"

# a. Check/Install everything needed
# Save the original PATH
ORIGINAL_PATH=$PATH

# Prepend the pyenv shims directory to the PATH
export PATH="$HOME/.pyenv/shims:$PATH"
eval "$(pyenv init --path)"
eval "$(pyenv init -)"

echo $(which starknet-compile-deprecated)
# Function to check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Function to install PyEnv
install_pyenv() {
    echo "Installing pyenv..."
    curl https://pyenv.run | bash

    # Add pyenv to path
    export PATH="$HOME/.pyenv/bin:$PATH"
    eval "$(pyenv init --path)"
    eval "$(pyenv virtualenv-init -)"
}

# Function to check Python version
check_python_version() {
    PYTHON_VERSION=$(python --version 2>&1 | awk '{print $2}')
    DESIRED_VERSION="3.9"

    if [[ "$PYTHON_VERSION" == "$DESIRED_VERSION"* ]]; then
        echo "Python $DESIRED_VERSION is already installed."
    else
        echo "Python $DESIRED_VERSION is not installed."
        install_python_3_9
    fi
}

# Function to install Python 3.9 using pyenv
install_python_3_9() {
    echo "Installing Python 3.9..."
    pyenv install 3.9
    pyenv global 3.9
}

# Check and install required tools
echo "Checking for Python >=3.9,<3.10..."
if ! command_exists python || ! python --version | grep -E "3\.9\.[0-9]+" > /dev/null; then
    echo "Required Python version not found."
    if ! command_exists pyenv; then
        install_pyenv
    fi
    check_python_version
else
    echo "Required Python version is already installed."
fi

# Check and install dependencies
echo "Installing dependencies..."
python -m pip install "cairo-lang>=0.11,<0.12" "starknet-py>=0.16,<0.17" "openzeppelin-cairo-contracts>=0.6.1,<0.7"

echo "Setup complete."



MADARA_CAIRO_ZERO_OUTPUT_DIR="$ROOT_DIR/cairo-contracts/compiled_contract/cairo_0"
mkdir -p $MADARA_CAIRO_ZERO_OUTPUT_DIR

MADARA_CONTRACT_PATH="$ROOT_DIR/cairo-contracts"

base_folder=$MADARA_CONTRACT_PATH/src
exclude_folder=$MADARA_CAIRO_ONE_SRC_DIR

# Use find to get all .cairo files in base_folder, excluding exclude_folder
# Then, use a loop to process each file
find "$base_folder" -type f -name "*.cairo" | grep -vF "$base_folder/cairo_1" | while read -r file_path; do
    # echo "Processing: $file_path"
    file_name=$(basename "$file_path" .cairo)
    echo "starknet-compile-deprecated $file_path --output $MADARA_CAIRO_ZERO_OUTPUT_DIR/$file_name.json --cairo_path $MADARA_CONTRACT_PATH --no_debug_info $(echo $file_name | awk '{print tolower($0)}' | grep -q "account" && echo "--account_contract")"
    starknet-compile-deprecated $file_path --output $MADARA_CAIRO_ZERO_OUTPUT_DIR/$file_name.json --cairo_path $MADARA_CONTRACT_PATH --no_debug_info $(echo $file_name | awk '{print tolower($0)}' | grep -q "account" && echo "--account_contract")

    done



# X. Restore path and Delete compiler binaries
rm -r "$SCRIPT_DIR/bin"
export PATH=$ORIGINAL_PATH