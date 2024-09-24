# Balancer

**Table of Contents**

1. [Features](#b-features)
2. [System Architecture](#b-system-architecture)
3. [Configuration](#b-configuration)
4. [Dependencies](#b-dependencies)
5. [Error and Logs](#b-error-and-logs)

<a id="b-features"></a>**Features**

- Dynamic load balancing based on server scores
- Caching of static resources
- Asynchronous processing of HTTP requests
- Automatic reconnection to WebSocket with exponential backoff
- Periodic garbage collection for cache entries

<a id="b-system-architecture"></a>**System Architecture**

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

<a id="b-configuration"></a>**Configuration**

The application uses environment variables for configuration. Make sure to set the following variables:

- `HOST_PORT_HTTP_BALANCER`: Port for the HTTP server
- `HOST_PORT_WS_DEPLOYMENT_AGENT`: Port for the WebSocket connection to the deployment agent
- `TARGET_PORT`: Port of the backend servers
- `CACHE_CAPACITY`: Maximum number of items in the cache
- `REQUEST_TIMEOUT`: Timeout for outgoing requests (in seconds)

<a id="b-dependencies"></a>**Dependencies**

- `tokio`: Asynchronous runtime
- `hyper`: HTTP client and server
- `tokio-tungstenite`: WebSocket client
- `serde`: Serialization and deserialization of JSON
- `rand`: Random number generation for the weighted balancer
- `log` and `env_logger`: Logging

<a id="b-error-and-logs"></a>**Error Handling and Logging**

The application uses the `log` crate for logging. It logs information about connections, errors, and important state changes. Make sure to initialize the logger in your environment to see the logs.
