## Quick Start

### 1. Clone the repository

Choose one of the following methods:

- Basic clone (core codebase only):

```
  git clone https://github.com/mxmueller/RustyBalancer.git
```

- Clone with submodules (recommended for development/testing):

```
git clone --recursive https://github.com/mxmueller/RustyBalancer.git
```

Then navigate to the project directory:
```bash
cd RustyBalancer
```

### 2. Run the setup script

```bash
cd jobs
./setup.sh
```

### 3. Configure the .env file

Open the `.env` file in the project root directory.

### 4. Set Docker image and port

In the `.env` file, set your desired Docker image and its corresponding port. For example:
```
DOCKER_IMAGE=your-docker-image
TARGET_PORT=your-image-port
```

Example:
```
DOCKER_IMAGE=traefik/whoami
TARGET_PORT=80
```

Note: Currently, only images using a single port are supported. The specified image will be distributed and scaled across workers.

### 5. Run RustyBalancer

```bash
cd jobs
./run.sh -e dev  # For dev Enviroment
```
----

## Detailed Explanation

### Repository Cloning Options

- **Basic Clone:**
  This method clones only the main repository. It's sufficient if you only need the core codebase.

```bash
  git clone https://github.com/mxmueller/RustyBalancer.git
```

- **Clone with Submodules (recommended for development/testing):**
  This method clones the repository with its submodules, including the test suite for automated testing.
  ```bash
  git clone --recursive https://github.com/mxmueller/RustyBalancer.git
  ```
  The submodule `rustybalancer-test-suite` includes automated test cases for HTTP Stress Tests and QR Code Generator Stress Tests.

### Prerequisites

- Docker must be installed. Download from [Docker's official site](https://www.docker.com/products/docker-desktop).
- For macOS/Windows: `docker-compose` is required.
- For Linux: `docker compose` (Docker CLI plugin) is preferred.

### Configuration Files

- `docker-compose.yaml`: Production environment setup. Includes redis, deployment-agent, dashboard, and balancer services.
- `docker-compose.dev.yaml`: Development environment with additional tools. Includes all production services plus redis-insight and more environment variables for testing.
- `docker-compose.slim.yaml`: Lightweight setup for resource-constrained environments. Excludes dashboard and redis-insight services.

### Environment Setup

The `setup.sh` script generates a `.env` file with your RustyBalancer configuration.

### Running RustyBalancer

Use the `run.sh` script in the `jobs` directory:

- Production: `./run.sh -e prod`
- Development: `./run.sh -e dev`
- Slim: `./run.sh -e slim`

The script automatically selects `docker compose` or `docker-compose` based on your system.

### Docker Group (if needed)

Add your user to the Docker group:
```bash
sudo usermod -aG docker $USER
```
