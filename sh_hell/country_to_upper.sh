#!/bin/bash

# Function to translate the country name
function translate_country_name() {
    local country_name="$1"
    local translated_name
    # Use 'trans' command to translate to English (en)
    translated_name=$(trans :en "$country_name" | cut -d' ' -f 2-)
    echo "$translated_name"
}

# Input file containing the data
input_file="$1"

# Loop through each line of the file and replace the country name
while IFS= read -r line; do
    # Extract the country name from the line (assuming the country name is enclosed in single quotes)
    country_name=$(echo "$line" | cut -d"'" -f4)
    uppercase_string=$(echo "$country_name" | tr '[:lower:]' '[:upper:]')
    # Replace the country name in the line
    new_line=$(echo "$line" | sed "s/'$country_name'/'$uppercase_string'/")
    # Print the new line to the output file
    echo "$new_line"
done < "$input_file" > "output_file_2.txt"

# Optionally, replace the original file with the translated data
# mv "output_file.txt" "$input_file"

