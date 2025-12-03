# syntax=docker/dockerfile:1

################################################################################
# Create a stage for building the application.

ARG RUST_VERSION=1.91.1
ARG APP_NAME=grpc-client
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

# Install system deps
RUN apt-get update && \
    apt-get install -y \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy proto files to /proto (build.rs expects ../proto relative to /app)
# Build context is parent directory (set in docker-compose.yml)
COPY proto /proto

# Copy source files
COPY grpc-client/src ./src
COPY grpc-client/Cargo.toml ./Cargo.toml
COPY grpc-client/Cargo.lock ./Cargo.lock
COPY grpc-client/build.rs ./build.rs

# Build the application with cache mount for faster rebuilds
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/client
EOF

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application.
FROM debian:bullseye-slim AS final

# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/client /bin/

# What the container should run when it is started.
CMD ["/bin/client"]
