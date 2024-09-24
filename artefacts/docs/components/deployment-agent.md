
# Deployment-Agent

**Table of Contents**

1. [Features](#da-features)
2. [System Architecture](#da-system-architecture)
3. [Key Concepts](#da-key-concepts)
4. [Monitoring and Scaling](#da-monitoring-and-scaling)
5. [WebSocket Communication](#da-websocket-communication)
6. [Database Integration](#da-database-integration)

----

<a id="da-features"></a>**Features**

- Dynamic container management with Docker
- Real-time performance monitoring of containers
- Automatic scaling based on load and performance metrics
- WebSocket server for real-time updates
- Redis integration for persistent storage
- Sophisticated load balancing algorithm
- HTTP server for stats and management endpoints

<a id="da-system-architecture"></a>**System Architecture**

The system consists of several interconnected modules:

1. **Main Application** (`main.rs`)
2. **Container Management** (`container.rs`)
3. **Queue Management** (`queue.rs`)
4. **Performance Monitoring** (`stats.rs`)
5. **WebSocket Server** (`socket.rs`)
6. **HTTP Server** (`http.rs`)
7. **Database Integration** (`db.rs`)

**Modules**

**Main (`main.rs`)**

- Initializes the system, starts the HTTP server and WebSocket server
- Coordinates all components

**Container Management (`container.rs`)**
Handles Docker container operations.

- Creates, stops, and removes Docker containers
- Manages container lifecycle
- Interacts with Docker API

**Queue Management (`queue.rs`)**

- Manages the queue of active containers and scaling decisions
- Periodically rebuilds the queue based on current system state

**Performance Monitoring (`stats.rs`)**

- Collects CPU, memory, network, and availability metrics
- Calculates performance scores for containers
- Implements sophisticated algorithms for trend analysis and dynamic thresholds

**WebSocket Server (`socket.rs`)**

- Provides real-time updates of the container queue to clients
- Implements WebSocket protocol for bi-directional communication

**HTTP Server (`http.rs`)**

- Exposes endpoints for retrieving container stats
- Implements CORS for cross-origin requests

**Database Integration (`db.rs`)**

- Manages Redis connection
- Provides methods for storing and retrieving configuration values

<a id="da-key-concepts"></a>**Key Concepts**

**Container Lifecycle**
Containers go through several states:

- INIT: Initial state when a container is created
- LU (Low Utilization): Container is underutilized
- MU (Medium Utilization): Container has moderate utilization
- HU (High Utilization): Container is highly utilized
- SUNDOWN: Container is marked for removal

**Performance Scoring**
Each container receives scores based on:

- CPU usage
- Memory usage
- Network usage
- Availability (response time)

These scores are combined into an overall score that determines the container's utilization category.

<a id="da-monitoring-and-scaling"></a>**Monitoring and Scaling**

The system continuously monitors container performance and makes scaling decisions based on:

- Average load across all containers
- Presence of critically loaded containers
- Current number of active containers vs. desired number
- Cooldown periods to prevent rapid scaling events

Scaling operations include:

- Creating new containers when load is high
- Marking containers for removal (SUNDOWN) when load is low

<a id="da-websocket-communication"></a>**WebSocket Communication**

The WebSocket server provides real-time updates of the container queue to clients. This allows for immediate reflection of system changes in client applications.

<a id="da-database-integration"></a>**Database Integration**

Redis is used for persistent storage of:

- Container information
- Configuration values
- Performance metrics

This allows for system state recovery in case of restarts.
