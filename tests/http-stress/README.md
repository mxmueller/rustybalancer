# Rust Load Tester for Docker Stress Testing

This Rust script is designed to perform load testing on a Docker container running a web application. It generates a continuous stream of requests to test the application's performance under various levels of load.

## Features

- Asynchronous request handling using Tokio
- Configurable request rate and test duration
- Adjustable intensity for CPU and memory consumption tests
- Continuous, evenly distributed load generation
- Real-time feedback on request status

## Prerequisites

- Rust programming language (latest stable version)
- Cargo package manager

## Installation

1. Clone this repository or create a new Rust project:
   ```
   cargo new load_tester
   cd load_tester
   ```

2. Replace the contents of `src/main.rs` with the provided script.

3. Update your `Cargo.toml` file with the following dependencies:
   ```toml
   [dependencies]
   tokio = { version = "1.0", features = ["full"] }
   reqwest = { version = "0.11", features = ["json"] }
   ```

4. Build the project:
   ```
   cargo build --release
   ```

## Usage

Run the load tester using the following command:

```
cargo run --release -- <base_url> <requests_per_second> <test_duration_seconds> <intensity>
```

Parameters:
- `<base_url>`: The base URL of your Docker container's web application (e.g., `http://localhost:2548`)
- `<requests_per_second>`: Number of requests to send per second
- `<test_duration_seconds>`: Total duration of the test in seconds
- `<intensity>`: Intensity level for CPU and memory consumption (passed to the web application)

Example:
```
cargo run --release -- http://localhost:2548 50 60 5
```
This will run a test with 50 requests per second for 60 seconds, with an intensity level of 5.

## Output

The script provides real-time feedback on the status of each request. At the end of the test, it will display a summary including:
- Total number of requests sent
- Average request rate achieved

## Notes

- Ensure that your target application has implemented the following endpoints:
    - `/consume_cpu/<intensity>`
    - `/consume_memory/<intensity>`
- The actual request rate may slightly differ from the specified rate, especially at very high rates.
- Monitor system resources (CPU, network, etc.) to ensure your machine can handle the desired load.
- This tool is for testing purposes only. Be cautious when using it on production systems.

## Customization

You can modify the script to add more types of requests, change the request patterns, or add more detailed logging as needed for your specific testing requirements.

## License

This project is open-source and available under the MIT License.