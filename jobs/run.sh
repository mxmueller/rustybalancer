#!/bin/bash

set -e

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to display ASCII art from a file
ascii_art() {
    cat "$SCRIPT_DIR/.resources/ascii.txt"
}

# Function to display the header from a file
display_header() {
    cat "$SCRIPT_DIR/.resources/header.txt"
}

# Default values
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
COMPOSE_DIR="$SCRIPT_DIR/.."
ENVIRONMENT="prod"

# Process flags
while getopts "e:" opt; do
    case $opt in
        e)
            ENVIRONMENT=$OPTARG
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

# Print paths for debugging
echo "SCRIPT_DIR: $SCRIPT_DIR"
echo "COMPOSE_DIR: $COMPOSE_DIR"

# Set the Docker Compose command based on the operating system
if [[ "$(uname -s)" == "Linux" ]]; then
    if command_exists docker; then
        DOCKER_COMPOSE="docker compose"
        echo "Using docker compose for Docker operations on Linux..."
    else
        echo "Docker is not installed. Please install Docker."
        exit 1
    fi
else
    if command_exists docker-compose; then
        DOCKER_COMPOSE="docker-compose"
        echo "Using docker-compose for Docker operations..."
    else
        echo "docker-compose is not installed. Please install docker-compose."
        exit 1
    fi
fi

# Set Docker Compose files based on the environment
case $ENVIRONMENT in
    prod)
        COMPOSE_FILES="$COMPOSE_DIR/docker-compose.yml"
        ;;
    dev)
        COMPOSE_FILES="$COMPOSE_DIR/docker-compose.yml -f $COMPOSE_DIR/docker-compose.dev.yml"
        ;;
    slim)
        COMPOSE_FILES="$COMPOSE_DIR/docker-compose.yml -f $COMPOSE_DIR/docker-compose.slim.yml"
        ;;
    *)
        echo "Invalid environment: $ENVIRONMENT"
        exit 1
        ;;
esac

ascii_art

echo ""

display_header

echo ""

# Print COMPOSE_FILES for debugging
echo "COMPOSE_FILES: $COMPOSE_FILES"

# Change to the directory with the Docker Compose files
cd "$COMPOSE_DIR" || exit

# Build and start Docker containers
echo "Building Docker containers for $ENVIRONMENT environment..."

$DOCKER_COMPOSE -f $COMPOSE_FILES build

echo "Starting Docker containers for $ENVIRONMENT environment..."
$DOCKER_COMPOSE -f $COMPOSE_FILES up
