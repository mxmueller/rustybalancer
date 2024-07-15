# Build stage
FROM rust:latest AS builder
WORKDIR /code

# Install dependencies for build
RUN apt-get update && apt-get install -y libssl-dev

RUN cargo init

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo fetch

COPY src/ src/

RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  # update timestamps to force a new build
  touch /code/src/main.rs
  cargo test --release
  cargo build --release
EOF

# Actual container
FROM bitnami/minideb:bookworm

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev

WORKDIR /app

# Copy static files if you have any (optional)
# COPY public public

# Copy the server binary from the build stage
COPY --from=builder /code/target/release/balancer balancer

# Set user to non-root unless root is required for your app
USER 1001

# Run the server
CMD ["/app/balancer"]
