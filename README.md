
<img src="./docs/resources/logo.png" alt="Logo" width="250"/>

# RustyBalancer

RustyBalancer is a load balancer and deployment engine prototype, featuring:
- **Round Robin with Priorities** for distribution
- **Proactive Handling**
- **Load Balancer**
- **Deployment Engine**

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

## Customization

You can customize the Docker Compose files as needed for your specific environment requirements.

## Contributing

Feel free to submit issues, fork the repository, and send pull requests. For major changes, please open an issue first to discuss what you would like to change.

