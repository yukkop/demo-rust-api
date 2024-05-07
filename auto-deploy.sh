#!/bin/bash

# Function to display help message
display_help() {
    echo "Usage: $(basename "$0") [OPTIONS] CONFIG_FILE_PATH"
    echo ""
    echo "Options:"
    echo "  -h, --help        Display this help message"
    echo ""
    echo "Arguments:"
    echo "  CONFIG_FILE_PATH       The new value for API_URL"
}

# Get the directory path of the script
SCRIPT_DIR=$(dirname "$0")

# Parse command line options
while [[ $# -gt 0 ]]; do
    key="$1"

    case $key in
        -h|--help)
        display_help
        exit 0
        ;;
        *)
        CONFIG_FILE_PATH="$1"  # The new value for API_URL passed as an argument
        ;;
    esac
    shift
done

if [[ -z $CONFIG_FILE_PATH ]]; then
    echo "Error: Missing argument CONFIG_FILE_PATH"
    display_help
    exit 1
fi

# get the API_URL from the config file and write it to secrets.rs
SERVER_FOLDER=$(grep -oP 'FOLDER: "\K[^"]+' "$CONFIG_FILE_PATH")

# Build the project
cargo build --release

# Deploy the project
USERNAME=$(grep -oP 'USERNAME: "\K[^"]+' "$CONFIG_FILE_PATH")
SERVER=$(grep -oP 'SERVER: "\K[^"]+' "$CONFIG_FILE_PATH")

ssh $USERNAME@$SERVER << EOF
  rm -r $SERVER_FOLDER
  mkdir -p $SERVER_FOLDER


  cd $SERVER_FOLDER
EOF

# Path to builded bin file
# TODO rename progect
scp ./target/release/gwp_phone_mask $USERNAME@$SERVER:$SERVER_FOLDER/

# Goodies
SERVICE_NAME=$(grep -oP 'SERVICE_NAME: "\K[^"]+' "$CONFIG_FILE_PATH")
BITRIX_URL=$(grep -oP 'BITRIX_URL: "\K[^"]+' "$CONFIG_FILE_PATH")
BITRIX_TOKEN=$(grep -oP 'BITRIX_TOKEN: "\K[^"]+' "$CONFIG_FILE_PATH")
LOG_FILE=$(grep -oP 'LOG_FILE: "\K[^"]+' "$CONFIG_FILE_PATH")
PORT=$(grep -oP 'PORT: "\K[^"]+' "$CONFIG_FILE_PATH")

# Set the permissions
ssh $USERNAME@$SERVER << EOF
  chmod -R 677 $SERVER_FOLDER
  cd $SERVER_FOLDER

  echo "
  [Unit]
  Description=service for normalize phone numbers in bitrix

  [Service]
  ExecStart=env BITRIX_URL=$BITRIX_URL env BITRIX_TOKEN=$BITRIX_TOKEN env LOG_FILE=$LOG_FILE env PORT=$PORT $SERVER_FOLDER/gwp_phone_mask

  [Install]
  WantedBy=multi-user.target
  " > /etc/systemd/system/$SERVICE_NAME.service

  systemctl enable --now $SERVICE_NAME.service
EOF
# TODO service