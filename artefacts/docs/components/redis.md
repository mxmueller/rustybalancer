# Redis and Redis Insight

## Redis

Redis is a fast, in-memory data store used in this project as a key-value database. It is provided through the `redis` Docker container and is accessible via the `rust-network` Docker network.

### Connecting to Redis

- **Hostname:** `redis`
- **Port:** Defined by the `${REDIS_PORT}` environment variable
- **Password:** No password required

---

## Redis Insight

Redis Insight is a graphical user interface tool for managing and monitoring Redis databases.

**Important:** Redis Insight is only available in the development environment (`docker-compose.dev.yml`).

### Accessing Redis Insight

- Accessible through the browser
- **Port:** Defined by the `${REDIS_INSIGHT_PORT}` environment variable

### Connecting to Redis via Redis Insight

When connecting Redis Insight to the Redis instance, use the following settings:

- **Hostname:** `redis` (the name of the Redis service in the Docker network)
- **Port:** Use the value of `${REDIS_PORT}`
- **No authentication required**