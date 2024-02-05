#!/bin/bash

######## DOC ########
# USAGE :
# compile_all_contracts.sh <cairo_1_compiler_version>
# ex : compile_all_contracts.sh 2.5.0
#
# Overview:
# This script compile all contracts contained in cairo_contracts/src and output it to cairo_contract/compiled_contract with 2
# subfolders : cairo_0 and cairo_1
#
# Runtime:
# It compile the cairo1 first and then the cairo0 in this way :
#
# CAIRO 1 :
#     - Check your OS to detect which version of the cairo 1 compiler we should get
#     - wget the cairo compiler corresponding to the version given in the CLI 
#     - extracting the archive to cairo_contract/bin
#     - Compile all cairo to sierra in compiled_contract/cairo_1
#     - Compile all sierra to casm in the same folder
#
# CAIRO 0 :
# /!\ As the cairo 0 compiler require some python package, this script automaticaly install some dependencies /!\
#     - We check if python is installed and if the current version of python is python 3.9
#     - If there is no python or if its not the good version we install pyenv (a python version management tool)
#     - with pyenv we install python 3.9 and set it to global
#     - then we install the cairo compiler and all required dependencies with pip
#     - We compile every contrat in cairo_contract/src except those in cairo_contract/src/cairo_1 and output them into
#     compiled_contract/cairo_0
#
# dependencies:
# During the usage of the script we :
#     - Download the cairo 1 compiler but its deleted at the end
#     - Install Pyenv and python 3.9 if needed
#     - pip install "cairo-lang>=0.11,<0.12" "starknet-py>=0.16,<0.17" "openzeppelin-cairo-contracts>=0.6.1,<0.7"
#
##################

######## USAGE ########

if [ "$#" -ne 1 ]; then
    echo "Usage : $0 <cairo_1_compiler_version>"
    echo "ex : $0 2.5.0"
    exit 1
fi

# Reset
NC='\033[0m' # No Color

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'

######## SYSTEM CHECKS ########

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
    echo "${RED} Project root not found, exiting.${NC}"
    exit 1
}

ROOT_DIR=$(find_project_root)

echo "Madara root directory is : ${GREEN} $ROOT_DIR ${NC}"

OS=$(detect_os)

# Build appropriate URL based on the OS
if [ "$OS" = "Linux" ]; then
    ARCHIVE_URL+="/$LINUX_ARCHIVE"
elif [ "$OS" = "Mac" ]; then
    ARCHIVE_URL+="/$MAC_ARCHIVE"
else
    echo "${RED} Unsupported operating system, exiting. ${NC}"
    exit 1
fi

echo "OS Detected : ${GREEN} $OS ${NC}\n"

# 1. GET BINARIES
echo "Downloading binairies ..."
# Define the URL of the archive and the directory to extract to
EXTRACT_DIR="$ROOT_DIR/cairo-contracts/scripts/bin"

# Download the archive
wget -q "$ARCHIVE_URL" -O /tmp/cairo_binaries.tar.gz

# Check if download was successful
if [ $? -eq 0 ]; then
    echo "${GREEN}DOWNLOAD SUCCESSFUL${NC}✅ \nextracting the archive..."

    # Create the directory if it doesn't exist
    mkdir -p "$EXTRACT_DIR"

    # Extract the archive to the specified directory
    tar -xzf /tmp/cairo_binaries.tar.gz -C "$EXTRACT_DIR"

    # Check if extraction was successful
    if [ $? -eq 0 ]; then
        echo "${GREEN}EXTRACTION SUCCESSFULL ${NC}✅"
    else
        echo "${RED} Error occurred during extraction, exiting ${NC}"
        exit 1
    fi
else
    echo "${RED} Download failed. Please check the provided version ${NC}"
    exit 1
fi

# Clean up
rm /tmp/cairo_binaries.tar.gz

# 2. COMPILE CAIRO 1 CONTRACTS
echo "${YELLOW}\nCOMPILING CAIRO ONE ${NC}\n"

MADARA_STARKNET_COMPILE_BINARY="$SCRIPT_DIR/bin/cairo/bin/starknet-compile"
MADARA_STARKNET_SIERRA_COMPILE_BINARY="$SCRIPT_DIR/bin/cairo/bin/starknet-sierra-compile"

MADARA_CAIRO_ONE_SRC_DIR="$ROOT_DIR/cairo-contracts/src/cairo_1"
MADARA_CAIRO_ONE_OUTPUT_DIR="$ROOT_DIR/cairo-contracts/compiled_contract/cairo_1"


echo "\nCompiling cairo 1 contract contained in ${YELLOW} $MADARA_CAIRO_ONE_SRC_DIR ${NC} to ${YELLOW} $MADARA_CAIRO_ONE_OUTPUT_DIR ${NC}\n"

mkdir -p $MADARA_CAIRO_ONE_OUTPUT_DIR
find "$MADARA_CAIRO_ONE_SRC_DIR" -type f -name "*.cairo" | while read -r file_path; do
    base_name=$(basename "$file_path" .cairo)
    output_file="$MADARA_CAIRO_ONE_OUTPUT_DIR/$base_name"".sierra.json"
    "$MADARA_STARKNET_COMPILE_BINARY" --single-file "$file_path" "$output_file"
    echo "Compiling $file_path ${GREEN} Done${NC} ✅"

    done



echo "\nCompiling Sierra to CASM\n"
# find "$MADARA_CAIRO_ONE_OUTPUT_DIR" -type f -name "*sierra.json" -exec bash -c 'compile_cairo1_casm "$0"' {} \;
find "$MADARA_CAIRO_ONE_OUTPUT_DIR" -type f -name "*sierra.json" | while read -r file_path; do
    base_name=$(basename "$file_path" .sierra.json)
    output_file="$MADARA_CAIRO_ONE_OUTPUT_DIR/$base_name"".casm.json"
    "$MADARA_STARKNET_SIERRA_COMPILE_BINARY" "$file_path" "$output_file"
    echo "Compiling $file_path ${GREEN} Done${NC} ✅"
    done


# 3. COMPILE CAIRO 0 CONTRACTS
echo "${YELLOW}\nCOMPILING CAIRO ZERO ${NC}\n"

# a. Check/Install everything needed
# Save the original PATH
ORIGINAL_PATH=$PATH

# Prepend the pyenv shims directory to the PATH
export PATH="$HOME/.pyenv/shims:$PATH"
eval "$(pyenv init --path)"
eval "$(pyenv init -)"

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
python -m pip install -qq "cairo-lang>=0.11,<0.12" "starknet-py>=0.16,<0.17" "openzeppelin-cairo-contracts>=0.6.1,<0.7"

echo "Installation${GREEN} Done ${NC} ✅"
echo "${GREEN}Setup complete.${NC}\n"



MADARA_CAIRO_ZERO_OUTPUT_DIR="$ROOT_DIR/cairo-contracts/compiled_contract/cairo_0"
mkdir -p $MADARA_CAIRO_ZERO_OUTPUT_DIR

MADARA_CONTRACT_PATH="$ROOT_DIR/cairo-contracts"

base_folder=$MADARA_CONTRACT_PATH/src
exclude_folder=$MADARA_CAIRO_ONE_SRC_DIR

echo "\nCompiling cairo 0 contract contained in ${YELLOW} $base_folder ${NC} to ${YELLOW} $MADARA_CAIRO_ZERO_OUTPUT_DIR ${NC}\n"

# Use find to get all .cairo files in base_folder, excluding exclude_folder
# Then, use a loop to process each file
find "$base_folder" -type f -name "*.cairo" | grep -vF "$base_folder/cairo_1" | while read -r file_path; do
    # echo "Processing: $file_path"
    file_name=$(basename "$file_path" .cairo)
    starknet-compile-deprecated $file_path --output $MADARA_CAIRO_ZERO_OUTPUT_DIR/$file_name.json --cairo_path $MADARA_CONTRACT_PATH --no_debug_info $(echo $file_name | awk '{print tolower($0)}' | grep -q "account" && echo "--account_contract")
    echo "Compiling $file_path ${GREEN} Done${NC} ✅"
    done


# X. Restore path and Delete compiler binaries
rm -r "$SCRIPT_DIR/bin"
export PATH=$ORIGINAL_PATH
