#!/bin/bash

ENV_FILE_PATH="../.env"

# Default values
DEFAULT_DOCKER_HOST="unix:///var/run/docker.sock"
DEFAULT_DOCKER_SOCKET_VOLUME="/var/run/docker.sock"
DEFAULT_DOCKER_IMAGE="mxmller/rustybalancer-playground:latest"
DEFAULT_HOST_PORT_WS_DEPLOYMENT_AGENT="2547"
DEFAULT_HOST_PORT_WS_BALANCER="2551"
DEFAULT_HOST_PORT_HTTP_DEPLOYMENT_AGENT="2550"
DEFAULT_HOST_PORT_HTTP_BALANCER="2548"
DEFAULT_PORT_DASHBOARD="8501"
DEFAULT_TARGET_PORT="5000"
DEFAULT_DEFAULT_CONTAINER="5"
DEFAULT_APP_IDENTIFIER="RUSTYBALANCER"
DEFAULT_REDIS_PORT="6379"
DEFAULT_REDIS_HOST="redis"
DEFAULT_REDIS_INSIGHT_PORT="5540"

# Function to create .env-file
create_env_file() {
  cat <<EOF > "$ENV_FILE_PATH"
# Docker Configuration
DOCKER_HOST=$1
DOCKER_SOCKET_VOLUME=$2
DOCKER_IMAGE=$3

# Port Configuration
# WebSocket Ports
HOST_PORT_WS_DEPLOYMENT_AGENT=$4
HOST_PORT_WS_BALANCER=$5

# HTTP Ports
HOST_PORT_HTTP_DEPLOYMENT_AGENT=$6
HOST_PORT_HTTP_BALANCER=$7

# Dashboard Port
PORT_DASHBOARD=$8

# Target Port
TARGET_PORT=$9

# Application Configuration
DEFAULT_CONTAINER=${10}
APP_IDENTIFIER=${11}

# Redis Configuration
REDIS_PORT=${12}
REDIS_HOST=${13}
REDIS_INSIGHT_PORT=${14}
EOF
}

# Choosing either expert or basic version for setup
echo "This script will generate your .env-file."
echo "Which version do you want to use:"
echo "1) Basic version (default values)"
echo "2) Expert version (you can choose your own values)"

read -p "Your choice: " choice

if [ "$choice" == "1" ]; then
    echo "Creating .env-file with default values..."
    create_env_file "$DEFAULT_DOCKER_HOST"  "$DEFAULT_DOCKER_SOCKET_VOLUME" "$DEFAULT_DOCKER_IMAGE"  "$DEFAULT_HOST_PORT_WS_DEPLOYMENT_AGENT" "$DEFAULT_HOST_PORT_WS_BALANCER" "$DEFAULT_HOST_PORT_HTTP_DEPLOYMENT_AGENT" "$DEFAULT_HOST_PORT_HTTP_BALANCER" "$DEFAULT_PORT_DASHBOARD" "$DEFAULT_TARGET_PORT" "$DEFAULT_DEFAULT_CONTAINER" "$DEFAULT_APP_IDENTIFIER" "$DEFAULT_REDIS_PORT" "$DEFAULT_REDIS_HOST" "$DEFAULT_REDIS_INSIGHT_PORT"
    echo ".env-file created."

elif [ "$choice" == "2" ]; then
    echo "Creating .env-file - expert version"
    create_env_file "" "" "" "" "" "" "" "" "" "" "" "" "" ""
    echo ".env-file created."
else
    echo "Invalid input. Please enter your choice between 1 and 2 again."
    exit 1
fi
