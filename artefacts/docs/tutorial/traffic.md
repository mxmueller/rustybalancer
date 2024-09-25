# High Traffic Behavior
Adaptability is at the heart of effective load balancing. These videos demonstrate how load balancers dynamically adjust to changing traffic demands, showcasing their ability to maintain optimal performance as your needs grow or fluctuate.

----

# RustBalancer Load Testing Guide

## Prerequisites

- Ensure Rust is installed on your system.
- RustBalancer is running with the following configuration:
```
DOCKER_IMAGE=traefik/whoami
TARGET_PORT=80
```
- The test suite has been pulled from Git:
   Either recursively pulled with submodules when pulling RustBalancer, or separately pulled from https://github.com/mxmueller/rustybalancer-test-suite/

   If pulled recursively, the repo will be located in `./tests/`

## Running the Test

1. With RustBalancer running, navigate to `./http-stress`
2. Follow the tutorial in the README for detailed instructions

## Quick Test

For a simple test, run:

```bash
cargo run --release -- http://localhost:2548 5000 200 0
```

This command generates 5000 requests per second. The second paramter represents the time duration of the test. The last on has to be null for this setup.

## Important Note

Local tests are always limited by the performance of the host's network adapter and operating system.

---

# Demo

![type:video](https://www.youtube.com/embed/2EDIacuXl6c)