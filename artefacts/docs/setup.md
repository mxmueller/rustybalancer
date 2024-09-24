# Setup
Welcome to RustyBalancer!

Follow the instructions and get familiar with the environment variables to properly get your workflow going.

Have fun! 🦀

----

**Clone the repository:**

You can either simply clone the repository as usual. 

Or use the command with submodule you are cloning the linked repository `rustybalancer-test-suite` as well which contains automated test cases for [http-stress-tests](https://github.com/mxmueller/rustybalancer-test-suite/blob/main/http-stress/README.md) and [QR Code Generator Stress Tests](https://github.com/mxmueller/rustybalancer-test-suite/tree/main/qr-code).

```bash
git clone https://github.com/mxmueller/RustyBalancer.git
cd RustyBalancer
```


**Add current user to Docker group (if needed):**

```bash
sudo usermod -aG docker $USER
```
REMINDER, das hier müsse https://forums.docker.com/t/connection-refused-on-host-docker-internal/136925/2

This repository contains Docker Compose configurations for different environments: production, development, and a slim environment. The `run.sh` script located in the `jobs` directory allows you to easily build and start Docker containers for the specified environment.

# Prerequisites

- Docker must be installed on your system. You can download it from [Docker's official site](https://www.docker.com/products/docker-desktop).
- If you are using macOS or Windows, `docker-compose` must also be installed. On Linux, `docker compose` (the Docker CLI plugin) is preferred.

## Configuration Files

- `docker-compose.yaml`: 
 
The base configuration for the production environment.

- `docker-compose.dev.yaml`: 

Additional configuration for the development environment.

- `docker-compose.slim.yaml`: 

Additional configuration for a slim environment.

## Usage

The `run.sh` script is used to build and start the Docker containers. It takes a flag `-e` to specify the environment (`prod`, `dev`, or `slim`).

### Running the Script

Navigate to the `jobs` directory:
```shell
cd jobs
```

Make the script executable (if it isn't already):
```shell
chmod +x run.sh
```

Run the script with the desired environment:

- For the production environment:
```shell
./run.sh -e prod
```

- For the development environment:
```shell
./run.sh -e dev
```

- For the slim environment:
```shell
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