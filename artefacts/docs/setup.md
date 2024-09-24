
# RustyBalancer Setup
**Table of Contents** <br> <br>
1. [Setup](#setup) <br>
2. [Prerequisites](#prerequisites) <br>
3. [Configuration Files](#configuration-files) <br>
4. [Set Up Configuration (.env)](#set-up-configuration) <br>
5. [Usage](#usage)

----

<a id="setup"></a>
# Setup
Welcome to RustyBalancer!

Follow the instructions and get familiar with the environment variables to properly get your workflow going.

Have fun! 🦀

----
**Clone the repository:**

To set up the project, you need to clone the main repository. <br>
Choose the method that best suits your needs. If you're setting up for development or testing, cloning with submodules is recommended to ensure you have all the necessary components. <br>
You have two options:

- **Basic Clone:** <br>
   This method will clone only the main repository. It's sufficient if you only need the core codebase.
   Use the following commands to clone the repository and navigate into it:

```shell  
git clone https://github.com/mxmueller/RustyBalancer.git
```

- **Clone with Submodules:** <br>
   If you also need the test suite for automated testing, you should clone the repository with its submodules. 
   The submodule `rustybalancer-test-suite` includes automated test cases for [HTTP Stress Tests](https://github.com/mxmueller/rustybalancer-test-suite/blob/main/http-stress/README.md) and [QR Code Generator Stress Tests](https://github.com/mxmueller/rustybalancer-test-suite/tree/main/qr-code). <br> <br>
   These tests are crucial for validating the performance and reliability of your setup.
   Cloning with submodules ensures that you have the necessary test cases integrated into your project for thorough testing and validation. <br> <br>
   Use the following commands to clone the repository along with its submodules: <br>
```bash
git clone --recursive https://github.com/mxmueller/RustyBalancer.git
``` 

**Add current user to Docker group (if needed):**

```bash
sudo usermod -aG docker $USER
```

This repository contains Docker Compose configurations for different environments: production, development, and a slim environment. 
The `run.sh` script located in the `jobs` directory allows you to easily build and start Docker containers for the specified environment.

<a id="prerequisites"></a>
# Prerequisites

Docker must be installed on your system. You can download it from [Docker's official site](https://www.docker.com/products/docker-desktop).
If you are using macOS or Windows, `docker-compose` must also be installed. On Linux, `docker compose` (the Docker CLI plugin) is preferred.

<a id="configuration-files"></a>
## Configuration Files

`docker-compose.yaml`:

This file is designed to be used in a production environment where performance, reliability, and security are the primary concerns.
It typically includes optimizations and configurations suited for a live production system.

Includes the following services: redis, deployment-agent, dashboard, and balancer.

`docker-compose.dev.yaml`: 

Ideal for developers who need a local environment for building and testing the application. 
This configuration file typically includes settings that make it easier to debug and iterate on the code without affecting the production setup.

Includes all the services present in docker-compose.yaml: redis, deployment-agent, dashboard, and balancer.
Includes the redis-insight service, which provides a Redis management tool.
Includes more environment variables and configurations for the deployment-agent service and various scaling-related settings.

`docker-compose.slim.yaml`: 

Suitable for environments where resources are limited, such as in constrained or embedded systems.
This configuration helps to run the application with minimal overhead, making it ideal for testing or deployment in resource-constrained environments.

Includes a lightweight setup without the dashboard and redis-insight service.
Includes the same environment variables and configurations for the deployment-agent service as docker-compose.dev.yml.

<a id="set-up-configuration"></a>
## Set Up Configuration (.env)

```bash
cd jobs
./setup.sh
```

This script generates a `.env` file in the project root with your RustyBalancer configuration.

<a id="usage"></a>
## Usage

The `run.sh` script is used to build and start the Docker containers. It takes a flag `-e` to specify the environment (`prod`, `dev`, or `slim`).

<a id="running-the-script"></a>
### Running the Script

Navigate to the `jobs` directory:
```shell
cd jobs
```

Make the script executable (if it isn't already):
```shell
chmod +x run.sh
```

**Run the script with the desired environment:**

For the production environment:

```shell
./run.sh -e prod
```

For the development environment:

```shell
./run.sh -e dev
```

For the slim environment:

```shell
./run.sh -e slim
```

<a id="script-explanation"></a>
### Script Explanation

The `run.sh` script checks for the operating system and determines whether to use `docker compose` or `docker-compose` based on the system's available commands. It then processes the `-e` flag to determine which Docker Compose files to use.

