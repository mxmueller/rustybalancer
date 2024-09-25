## Overview
The RustBalancer Dashboard is a Streamlit-based web interface for real-time monitoring of the load balancing system.

## Key Features
- Auto-refreshing data (every 5 seconds)
- Overview table with container metrics
- Pie charts for score distributions (CPU, Memory, Network, Availability)
- Bar charts for CPU and Memory scores
- Summary metrics and detailed container information

## Access
- The dashboard is accessible via web browser at `http://localhost:8501`
- It's configured in `docker-compose.dev.yml` as part of the RustBalancer system

## Docker Setup
```yaml
dashboard:
  build:
    context: ./dashboard
    dockerfile: Dockerfile
  ports:
    - "8501:8501"
  environment:
    - DEPLOYMENT_URL=http://deployment-agent:${HOST_PORT_HTTP_DEPLOYMENT_AGENT}/stats
  depends_on:
    - deployment-agent
    - redis
  networks:
    - rust-network
```

The dashboard automatically fetches and displays the latest data from the deployment agent, providing a comprehensive view of your RustBalancer system's performance.