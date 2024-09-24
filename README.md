
<img src="artefacts/resources/logo.png" alt="Logo" width="250"/>

# RustyBalancer

RustyBalancer is a load balancer and deployment engine prototype, featuring:
- **Probabilities** for distribution
- **Proactive Handling**
- **Load Balancer**
- **Deployment Engine**

### test


## Installation Instructions
1. **Clone the repository:**
   ```bash
   git clone https://github.com/mxmueller/RustyBalancer.git
   cd RustyBalancer
   ```
2. **Add current user to Docker group (if needed):**
   ```bash
   sudo usermod -aG docker $USER
   ```

# Setup


REMINDER, das hier müsse https://forums.docker.com/t/connection-refused-on-host-docker-internal/136925/2

This repository contains Docker Compose configurations for different environments: production, development, and a slim environment. The `run.sh` script located in the `jobs` directory allows you to easily build and start Docker containers for the specified environment.

## Prerequisites

- Docker must be installed on your system. You can download it from [Docker's official site](https://www.docker.com/products/docker-desktop).
- If you are using macOS or Windows, `docker-compose` must also be installed. On Linux, `docker compose` (the Docker CLI plugin) is preferred.

## Configuration Files

- `docker-compose.yaml`: The base configuration for the production environment.
- `docker-compose.dev.yaml`: Additional configuration for the development environment.
- `docker-compose.slim.yaml`: Additional configuration for a slim environment.

## Usage

The `run.sh` script is used to build and start the Docker containers. It takes a flag `-e` to specify the environment (`prod`, `dev`, or `slim`).

### Running the Script

1. Navigate to the `jobs` directory:

    ```sh
    cd jobs
    ```

2. Make the script executable (if it isn't already):

    ```sh
    chmod +x run.sh
    ```

3. Run the script with the desired environment:

   - For the production environment:
     ```sh
     ./run.sh -e prod
     ```

   - For the development environment:
     ```sh
     ./run.sh -e dev
     ```

   - For the slim environment:
     ```sh
     ./run.sh -e slim
     ```

### Script Explanation

The `run.sh` script checks for the operating system and determines whether to use `docker compose` or `docker-compose` based on the system's available commands. It then processes the `-e` flag to determine which Docker Compose files to use.

Here's the logic breakdown:

1. **OS and Command Check**:
   - On Linux, it checks if `docker` is installed and uses `docker compose`.
   - On other systems, it checks if `docker-compose` is installed and uses `docker-compose`.

2. **Environment Selection**:
   - Based on the `-e` flag, it sets the appropriate Docker Compose files:
      - `prod`: Uses `docker-compose.yaml`
      - `dev`: Uses `docker-compose.yaml` and `docker-compose.dev.yaml`
      - `slim`: Uses `docker-compose.yaml` and `docker-compose.slim.yaml`

3. **Build and Start Containers**:
   - The script builds and starts the Docker containers using the selected configuration files.

-----

# Modules
## Balancer
### 1. Main (`main.rs`)

The entry point of the application. It sets up the shared state, initializes the UnboundedClient for outgoing requests, creates a cache for static resources, and spawns two main tasks:
- WebSocket connection to receive backend server updates
- HTTP server to handle incoming requests

### 2. HTTP Server (`http.rs`)

Implements the HTTP server that receives incoming requests and forwards them to the selected backend server. It uses a `DynamicWeightedBalancer` to choose the appropriate backend server for each request. Key features include:
- Caching of static resources
- Periodic updates of backend server weights
- Handling of both static and dynamic requests

### 3. WebSocket Client (`socket.rs`)

Maintains a WebSocket connection to a deployment agent to receive updates about available backend servers. It continuously attempts to reconnect in case of connection failures, with an exponential backoff strategy.

### 4. Unbounded Client (`client.rs`)

A custom HTTP client implementation that can handle a large number of concurrent requests. It uses a channel-based approach to queue requests and process them asynchronously.

### 5. Cache (`cache.rs`)

Implements a simple in-memory cache with TTL (Time To Live) for each entry. It includes a background task for garbage collection to remove expired entries.

### 6. Queue (`queue.rs`)

Defines the `QueueItem` struct representing a backend server and provides functionality to parse JSON data into a vector of `QueueItem`s.

## Key Features

- Dynamic load balancing based on server scores
- Caching of static resources
- Asynchronous processing of HTTP requests
- Automatic reconnection to WebSocket with exponential backoff
- Periodic garbage collection for cache entries

## Configuration

The application uses environment variables for configuration. Make sure to set the following variables:

- `HOST_PORT_HTTP_BALANCER`: Port for the HTTP server
- `HOST_PORT_WS_DEPLOYMENT_AGENT`: Port for the WebSocket connection to the deployment agent
- `TARGET_PORT`: Port of the backend servers
- `CACHE_CAPACITY`: Maximum number of items in the cache
- `REQUEST_TIMEOUT`: Timeout for outgoing requests (in seconds)

## Dependencies

- `tokio`: Asynchronous runtime
- `hyper`: HTTP client and server
- `tokio-tungstenite`: WebSocket client
- `serde`: Serialization and deserialization of JSON
- `rand`: Random number generation for the weighted balancer
- `log` and `env_logger`: Logging

## Error Handling and Logging

The application uses the `log` crate for logging. It logs information about connections, errors, and important state changes. Make sure to initialize the logger in your environment to see the logs.

-----

# Deployment-Agent

This project implements a sophisticated deployment agent and load balancer system in Rust. It manages Docker containers, monitors their performance, and dynamically adjusts the container pool based on load and performance metrics.

## Table of Contents

1. [Features](#features)
2. [System Architecture](#system-architecture)
3. [Running the Application](#running-the-application)
4. [Key Concepts](#key-concepts)
5. [Monitoring and Scaling](#monitoring-and-scaling)
6. [WebSocket Communication](#websocket-communication)
6. [Database Integration](#database-integration)
7. [Error Handling and Logging](#error-handling-and-logging)

## Features

- Dynamic container management with Docker
- Real-time performance monitoring of containers
- Automatic scaling based on load and performance metrics
- WebSocket server for real-time updates
- Redis integration for persistent storage
- Sophisticated load balancing algorithm
- HTTP server for stats and management endpoints

## System Architecture

The system consists of several interconnected components:

1. **Main Application** (`main.rs`): Initializes and coordinates all components.
2. **Container Management** (`container.rs`): Handles Docker container operations.
3. **Queue Management** (`queue.rs`): Manages the queue of active containers and scaling decisions.
4. **Performance Monitoring** (`stats.rs`): Collects and analyzes container performance metrics.
5. **WebSocket Server** (`socket.rs`): Provides real-time updates to clients.
6. **HTTP Server** (`http.rs`): Exposes management and stats endpoints.
7. **Database Integration** (`db.rs`): Handles Redis operations for persistent storage.

## Components

### Main (`main.rs`)
Initializes the system, starts the HTTP server and WebSocket server.

### Container Management (`container.rs`)
- Creates, stops, and removes Docker containers
- Manages container lifecycle
- Interacts with Docker API

### Queue Management (`queue.rs`)
- Maintains the queue of active containers
- Implements scaling logic
- Periodically rebuilds the queue based on current system state

### Performance Monitoring (`stats.rs`)
- Collects CPU, memory, network, and availability metrics
- Calculates performance scores for containers
- Implements sophisticated algorithms for trend analysis and dynamic thresholds

### WebSocket Server (`socket.rs`)
- Provides real-time updates of the container queue to clients
- Implements WebSocket protocol for bi-directional communication

### HTTP Server (`http.rs`)
- Exposes endpoints for retrieving container stats
- Implements CORS for cross-origin requests

### Database Integration (`db.rs`)
- Manages Redis connection
- Provides methods for storing and retrieving configuration values

## Key Concepts

### Container Lifecycle
Containers go through several states:
- INIT: Initial state when a container is created
- LU (Low Utilization): Container is underutilized
- MU (Medium Utilization): Container has moderate utilization
- HU (High Utilization): Container is highly utilized
- SUNDOWN: Container is marked for removal

### Performance Scoring
Each container receives scores based on:
- CPU usage
- Memory usage
- Network usage
- Availability (response time)

These scores are combined into an overall score that determines the container's utilization category.

## Monitoring and Scaling

The system continuously monitors container performance and makes scaling decisions based on:
- Average load across all containers
- Presence of critically loaded containers
- Current number of active containers vs. desired number
- Cooldown periods to prevent rapid scaling events

Scaling operations include:
- Creating new containers when load is high
- Marking containers for removal (SUNDOWN) when load is low

## WebSocket Communication

The WebSocket server provides real-time updates of the container queue to clients. This allows for immediate reflection of system changes in client applications.

## Database Integration

Redis is used for persistent storage of:
- Container information
- Configuration values
- Performance metrics

This allows for system state recovery in case of restarts.


## Contributing

Feel free to submit issues, fork the repository, and send pull requests. For major changes, please open an issue first to discuss what you would like to change.

