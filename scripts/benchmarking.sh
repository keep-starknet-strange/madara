#!/usr/bin/env bash

# This script can be used for running madara's benchmarks.
#
# The madara binary is required to be compiled with --features=runtime-benchmarks
# in release mode.

set -e

BINARY="./target/release/madara"
STEPS=50
REPEAT=20

# Add a default execution parameter
EXECUTION="wasm"

if [[ ! -f "${BINARY}" ]]; then
    echo "binary '${BINARY}' does not exist."
    echo "ensure that the madara binary is compiled with '--features=runtime-benchmarks' and in release mode."
    exit 1
fi

function help {
    echo "USAGE:"
    echo "  ${0} [<pallet> <benchmark>] [--check]"
    echo ""
    echo "EXAMPLES:"
    echo "  ${0}                       " "list all benchmarks and provide a selection to choose from"
    echo "  ${0} --check               " "list all benchmarks and provide a selection to choose from, runs in 'check' mode (reduced steps and repetitions)"
    echo "  ${0} foo bar               " "run a benchmark for pallet 'foo' and benchmark 'bar'"
    echo "  ${0} foo bar --check       " "run a benchmark for pallet 'foo' and benchmark 'bar' in 'check' mode (reduced steps and repetitions)"
    echo "  ${0} foo bar --all         " "run a benchmark for all pallets"
    echo "  ${0} foo bar --all --check " "run a benchmark for all pallets in 'check' mode (reduced steps and repetitions)"
}

function choose_and_bench {
    readarray -t options < <(${BINARY} benchmark pallet --list | sed 1d)
    options+=('EXIT')

    select opt in "${options[@]}"; do
        IFS=', ' read -ra parts <<< "${opt}"
        [[ "${opt}" == 'EXIT' ]] && exit 0

        bench "${parts[0]}" "${parts[1]}" "${1}"
        break
    done
}

function bench {
    OUTPUT=${4:-weights.rs}
    echo "benchmarking '${1}::${2}' --check=${3}, writing results to '${OUTPUT}'"

    # Check enabled
    if [[ "${3}" -eq 1 ]]; then
        STEPS=16
        REPEAT=1
    fi

    WASMTIME_BACKTRACE_DETAILS=1 ${BINARY} benchmark pallet \
        --execution=${EXECUTION} \
        --wasm-execution=compiled \
        --pallet "${1}" \
        --extrinsic "${2}" \
        --steps "${STEPS}" \
        --repeat "${REPEAT}" \
        --template=./scripts/benchmarking/frame-weight-template.hbs \
        --json-file raw.json \
        --output "${OUTPUT}"
}

if [[ "${@}" =~ "--help" ]]; then
    help
else
    CHECK=0
    if [[ "${@}" =~ "--check" ]]; then
        CHECK=1
        set -o noglob && set -- ${@/'--check'} && set +o noglob
    fi

    ALL=0
    if [[ "${@}" =~ "--all" ]]; then
        ALL=1
    fi

    if [[ "${@}" =~ "--execution=" ]]; then
        EXECUTION=$(echo ${@} | awk -F '--execution=' '{print $2}' | awk '{print $1}')
        set -o noglob && set -- ${@/'--execution='${EXECUTION}} && set +o noglob
    fi

    if [[ "${ALL}" -eq 1 ]]; then
        mkdir -p weights/
        bench '*' '*' "${CHECK}" "./weights"
    elif [[ $# -ne 2 ]]; then
        choose_and_bench "${CHECK}"
    else
        bench "${1}" "${2}" "${CHECK}"
    fi
fi
