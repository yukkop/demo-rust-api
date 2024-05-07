#!/bin/bash

# Replace "filename.txt" with the actual name of your file
file="country_data.sql"

# Check if the file exists and is readable
if [ ! -f "$file" ] || [ ! -r "$file" ]; then
  echo "Error: File not found or not readable."
  exit 1
fi

# Loop through the file line by line and process each line
while IFS= read -r line; do
  # Replace this echo statement with your desired processing for each line
  echo "Line: $line"
done < "$file"

