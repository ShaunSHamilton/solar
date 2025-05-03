#!/bin/bash

DATABASE="solar_assistant"
OUTPUT_FILE="data/wattle-i-do.csv"

# Ensure the output directory exists
mkdir -p data

# Clear the output file if it exists
> "$OUTPUT_FILE"

# Function to escape measurement names for querying
escape_measurement() {
  echo "$1" | sed 's/"/\\"/g'
}

# Get the list of measurements
measurements=$(influx -database "$DATABASE" -execute 'SHOW MEASUREMENTS' | tail -n +2 | awk '/[a-zA-Z]/ {print}' | grep -v '^----$' )

if [ -z "$measurements" ]; then
  echo "No valid measurements found in database '$DATABASE'."
  exit 0
fi

echo "Dumping measurements to '$OUTPUT_FILE'..."

# Counter for the first header
first_header=true

# Iterate through each measurement
while IFS= read -r measurement; do
  escaped_measurement=$(escape_measurement "$measurement")
  query="SELECT * FROM \"$escaped_measurement\""
  echo "Processing measurement: $measurement"
  echo "Executing query: $query -precision rfc3339" # Added precision flag

  # Query all data for the current measurement and output as CSV
  output=$(influx -database "$DATABASE" -execute "$query" -format csv -precision rfc3339) # Added precision flag
  row_count=$(echo "$output" | grep -v '^#' | wc -l) # Count non-comment lines

  echo "Rows returned: $row_count"

  if [ "$row_count" -gt 0 ]; then
    if "$first_header"; then
      echo "$output" | sed '/^#datatype/,/^#group/d' >> "$OUTPUT_FILE" # Simplified sed for header
      first_header=false
    else
      echo "$output" | sed '/^#datatype/,/^#group/d' | sed '/^#/d' >> "$OUTPUT_FILE" # Simplified sed for data
    fi
  fi

  echo "Processed: $measurement"
done <<< "$measurements"

echo "Data dump complete in '$OUTPUT_FILE'."