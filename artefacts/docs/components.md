This project implements a sophisticated deployment agent and load balancer system in Rust. It manages Docker containers, monitors their performance, and dynamically adjusts the container pool based on load and performance metrics.

# Balancer

**Table of Contents**

1. [Features](#b-features)
2. [System Architecture](#b-system-architecture)
4. [Key Concepts](#b-key-concepts)
5. [Monitoring and Scaling](#b-monitoring-and-scaling)
6. [WebSocket Communication](#b-websocket-communication)
6. [Database Integration](#b-database-integration)

**[Features](#b-features)**

- Dynamic load balancing based on server scores
- Caching of static resources
- Asynchronous processing of HTTP requests
- Automatic reconnection to WebSocket with exponential backoff
- Periodic garbage collection for cache entries

**[System Architecture](#b-system-architecture)**

The system consists of several interconnected modules:

1. **Main Application** (`main.rs`)
2. **HTTP Server** (`http.rs`)
3. **WebSocket Client** (`socket.rs`)
4. **Unbounded Client** (`client.rs`)
5. **Cache** (`cache.rs`)
6. **Queue** (`queue.rs`)

**Modules**

**Main (`main.rs`)**

The entry point of the application. It sets up the shared state, initializes the UnboundedClient for outgoing requests, creates a cache for static resources, and spawns two main tasks:

- WebSocket connection to receive backend server updates
- HTTP server to handle incoming requests

**HTTP Server (`http.rs`)**

Implements the HTTP server that receives incoming requests and forwards them to the selected backend server. It uses a `DynamicWeightedBalancer` to choose the appropriate backend server for each request. Key features include:

- Caching of static resources
- Periodic updates of backend server weights
- Handling of both static and dynamic requests

**WebSocket Client (`socket.rs`)**

Maintains a WebSocket connection to a deployment agent to receive updates about available backend servers. It continuously attempts to reconnect in case of connection failures, with an exponential backoff strategy.

**Unbounded Client (`client.rs`)**

A custom HTTP client implementation that can handle a large number of concurrent requests. It uses a channel-based approach to queue requests and process them asynchronously.

**Cache (`cache.rs`)**

Implements a simple in-memory cache with TTL (Time To Live) for each entry. It includes a background task for garbage collection to remove expired entries.

**Queue (`queue.rs`)**

Defines the `QueueItem` struct representing a backend server and provides functionality to parse JSON data into a vector of `QueueItem`s.


**Configuration**

The application uses environment variables for configuration. Make sure to set the following variables:

- `HOST_PORT_HTTP_BALANCER`: Port for the HTTP server
- `HOST_PORT_WS_DEPLOYMENT_AGENT`: Port for the WebSocket connection to the deployment agent
- `TARGET_PORT`: Port of the backend servers
- `CACHE_CAPACITY`: Maximum number of items in the cache
- `REQUEST_TIMEOUT`: Timeout for outgoing requests (in seconds)

**Dependencies**

- `tokio`: Asynchronous runtime
- `hyper`: HTTP client and server
- `tokio-tungstenite`: WebSocket client
- `serde`: Serialization and deserialization of JSON
- `rand`: Random number generation for the weighted balancer
- `log` and `env_logger`: Logging

**Error Handling and Logging**

The application uses the `log` crate for logging. It logs information about connections, errors, and important state changes. Make sure to initialize the logger in your environment to see the logs.

-----

# Deployment-Agent

**Table of Contents**

1. [Features](#da-features)
2. [System Architecture](#da-system-architecture)
4. [Key Concepts](#da-key-concepts)
5. [Monitoring and Scaling](#da-monitoring-and-scaling)
6. [WebSocket Communication](#da-websocket-communication)
6. [Database Integration](#da-database-integration)

**[Features](#da-features)**

- Dynamic container management with Docker
- Real-time performance monitoring of containers
- Automatic scaling based on load and performance metrics
- WebSocket server for real-time updates
- Redis integration for persistent storage
- Sophisticated load balancing algorithm
- HTTP server for stats and management endpoints

**[System Architecture](#da-system-architecture)**

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

**[Key Concepts](#da-key-concepts)**

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

**[Monitoring and Scaling](#da-monitoring-and-scaling)**

The system continuously monitors container performance and makes scaling decisions based on:

- Average load across all containers
- Presence of critically loaded containers
- Current number of active containers vs. desired number
- Cooldown periods to prevent rapid scaling events

Scaling operations include:

- Creating new containers when load is high
- Marking containers for removal (SUNDOWN) when load is low

**[WebSocket Communication](#da-websocket-communication)**

The WebSocket server provides real-time updates of the container queue to clients. This allows for immediate reflection of system changes in client applications.

**[Database Integration](#da-database-integration)**

Redis is used for persistent storage of:

- Container information
- Configuration values
- Performance metrics

This allows for system state recovery in case of restarts.
