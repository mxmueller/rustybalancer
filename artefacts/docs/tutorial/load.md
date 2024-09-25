# Load Testing
Rigorous testing is vital to ensure your load balancer can handle real-world scenarios.

## Prerequisites

- Ensure Rust is installed on your system.
- RustBalancer is running with the following configuration:
```
DOCKER_IMAGE=mxmller/rustybalancer-playground:latest
TARGET_PORT=5000
```
```
HIGH_LOAD_THRESHOLD=60.0
LOW_LOAD_THRESHOLD=75.0
CRITICAL_LOAD_THRESHOLD=30.0
```

This configuration means: <br>
- A new container starts when the average score reaches 60 <br>
- A container is removed if its score falls below 30 <br>
- New containers are shut down when the avg score recovers to 75

The test suite has been pulled from Git:
   Either recursively pulled with submodules when pulling RustBalancer or separately pulled from https://github.com/mxmueller/rustybalancer-test-suite/

If pulled recursively, the repo will be located in `./tests/`

## Running the Test

1. With RustBalancer running, navigate to `./http-stress`
2. Follow the tutorial in the README for detailed instructions

## Quick Test

For a simple test, run:

```bash
cargo run --release -- http://localhost:2548 15 500 3
```

This command:
- Generates 15 requests per second
- Runs for 500 seconds
- Has a varying load with a maximum of 3 (considered high)

## Important Notes
<ol>
  <li>Local tests are always limited by the performance of the host's network adapter and operating system.</li>
  <li>This example will demonstrate:
    <ol>
      <li>The load suddenly becoming very high</li>
      <li>New containers starting up to the set maximum number</li>
      <li>Containers being shut down when the test ends</li>
    </ol>
  </li>
</ol>

---

# Demo
![type:video](https://www.youtube.com/embed/R8HqdfwrEWU)