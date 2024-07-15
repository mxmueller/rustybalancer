#!/bin/bash

set -e

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

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

echo "Building Docker containers..."
$DOCKER_COMPOSE build

echo "Starting Docker containers..."
$DOCKER_COMPOSE up
