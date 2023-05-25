#!/bin/bash
# A bash script to calculate the difference in percentage 
# between the avgTps of two JSON files
set -e
# Command line arguments
OLD_BENCHMARK=$1
NEW_BENCHMARK=$2
THRESHOLD=-10
# Check if jq is installed. If not, install it.
if ! command -v jq &> /dev/null
then
    echo "jq could not be found. Attempting to install it..."
    sudo apt-get install jq
fi

# Extract avgTps from the old benchmark output
OLD_TPS=$(jq '.avgTps' $OLD_BENCHMARK)

# Extract avgTps from the new benchmark output
NEW_TPS=$(jq '.avgTps' $NEW_BENCHMARK)

# Calculate the percentage difference
PERCENT_DIFF=$(awk -v old="$OLD_TPS" -v new="$NEW_TPS" 'BEGIN{ diff = int( (new - old) / old * 100 ); printf("%s", diff) }')

# Print the result
echo "Percentage difference between old and new benchmark: $PERCENT_DIFF%"
if [ "$PERCENT_DIFF" -lt "$THRESHOLD" ]; then
    echo "Error: changes degraded the performances by more than 10%"
    exit 1
fi