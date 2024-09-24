# Environment Variables Configuration

This page provides a comprehensive overview of the environment variables used in our application. These variables control various aspects of the system, including Docker configuration, networking, scaling, and performance metrics.

### Docker & Worker
| Variable | Description |
|----------|-------------|
| DOCKER_IMAGE | Docker image for the worker |
| TARGET_PORT | Application port in container |
| DOCKER_HOST | Docker daemon URL |
| DOCKER_SOCKET_VOLUME | Docker socket file path |

### Network Ports
| Variable | Description |
|----------|-------------|
| HOST_PORT_WS_DEPLOYMENT_AGENT | WebSocket port for deployment agent |
| HOST_PORT_WS_BALANCER | WebSocket port for balancer |
| HOST_PORT_HTTP_DEPLOYMENT_AGENT | HTTP port for deployment agent |
| HOST_PORT_HTTP_BALANCER | HTTP port for balancer |
| PORT_DASHBOARD | Dashboard port |

### Application & Redis
| Variable | Description |
|----------|-------------|
| DEFAULT_CONTAINER | Default container count at startup |
| APP_IDENTIFIER | Unique application ID |
| REDIS_PORT | Redis server port |
| REDIS_HOST | Redis server hostname/IP |
| REDIS_INSIGHT_PORT | Redis Insight tool port |

### Load & Scaling
| Variable | Description |
|----------|-------------|
| HIGH_LOAD_THRESHOLD | High load threshold (%) |
| LOW_LOAD_THRESHOLD | Low load threshold (%) |
| CRITICAL_LOAD_THRESHOLD | Critical load threshold (%) |
| MAX_CONTAINERS | Maximum container count |
| COOLDOWN_PERIOD | Cooldown between scaling actions (s) |
| SCALE_STEP | Containers to add/remove per scaling action |
| SCALE_CHECK_PERIOD | Interval for scaling checks (min) |

### Performance Evaluation
| Variable | Description |
|----------|-------------|
| CPU_WEIGHT | CPU usage weight |
| MEMORY_WEIGHT | Memory usage weight |
| NETWORK_WEIGHT | Network usage weight |
| AVAILABILITY_WEIGHT | Availability weight |

### Other Settings
| Variable | Description |
|----------|-------------|
| HISTORY_SIZE | Number of metrics to store |
| BEST_TIME_WINDOW | Time window for best performance (s) |
| EMA_ALPHA | Smoothing factor for EMA |
| REQUEST_TIMEOUT | HTTP request timeout (s) |
| CACHE_CAPACITY | Maximum cache entries |

Note: Changes to environment variables require a system restart.