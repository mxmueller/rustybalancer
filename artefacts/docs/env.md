# Environment variables

**Changing environment variables while programm is running (`run.sh`) is not possible (no hotswap). Please reset and run again as seen in Setup.**

**Worker**

DOCKER_IMAGE:

- Specify the Docker image used by the worker.

TARGET_PORT

-  The port on which the application or service inside the container will listen.

**Docker Configuration**

DOCKER_HOST:

- The URL of the Docker daemon.

DOCKER_SOCKET_VOLUME:

- The path to the Docker socket file, used to enable Docker-in-Docker scenarios or to communicate with the Docker daemon from within a container.

# Port Configuration

**WebSocket Ports**

HOST_PORT_WS_DEPLOYMENT_AGENT:

- The port on which the WebSocket server for the deployment agent listens.

HOST_PORT_WS_BALANCER:

- The port on which the WebSocket server for the balancer listens.

**HTTP Ports**

HOST_PORT_HTTP_DEPLOYMENT_AGENT:

- The port on which the HTTP server of the deployment agent listens.

HOST_PORT_HTTP_BALANCER:

- The port on which the HTTP server of the balancer listens.

**Dashboard Port**

PORT_DASHBOARD:

- The port on which the dashboard application listens.

# Application Configuration

DEFAULT_CONTAINER:

- The default number of containers when the application initializes.

APP_IDENTIFIER:

- A unique identifier for the application, used for distinguishing this application instance from others.

# Redis Configuration

REDIS_PORT:

- The port on which the Redis server listens.

REDIS_HOST:

- The hostname or IP address of the Redis server.

REDIS_INSIGHT_PORT:

- The port on which the Redis Insight tool listens.

# Load thresholds

HIGH_LOAD_THRESHOLD:

- The threshold for high load, expressed as a percentage.

LOW_LOAD_THRESHOLD:

- The threshold for low load, expressed as a percentage.

CRITICAL_LOAD_THRESHOLD:

- The threshold for critical load, expressed as a percentage.

# Scaling Configuration

MAX_CONTAINERS:

- The maximum number of containers that the system can scale up to.

COOLDOWN_PERIOD:

- The cooldown period in seconds between scaling actions.

SCALE_STEP:

- The number of containers to add or remove during each scaling action.

SCALE_CHECK_PERIOD:

- The period in minutes between checks for scaling decisions.

# Score Weights

CPU_WEIGHT:

- The weight assigned to CPU usage when calculating the performance score of containers.

MEMORY_WEIGHT:

- The weight assigned to memory usage when calculating the performance score of containers.

NETWORK_WEIGHT:

- The weight assigned to network usage when calculating the performance score of containers.

AVAILABILITY_WEIGHT: 

- The weight assigned to availability (response time) when calculating the performance score of containers.

# Other Settings

HISTORY_SIZE:

- The number of past metrics or events to keep in history for analysis

BEST_TIME_WINDOW:

- The time window in seconds used for calculating the best performance metrics or trends.

EMA_ALPHA:

- The smoothing factor for Exponential Moving Average.

REQUEST_TIMEOUT:

- The timeout duration in seconds for HTTP requests.

CACHE_CAPACITY:

- The maximum number of items that can be stored in the cache.
