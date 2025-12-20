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

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=grpc-client/src,target=/app/grpc-client/src \
    --mount=type=bind,source=grpc-client/Cargo.toml,target=/app/grpc-client/Cargo.toml \
    --mount=type=bind,source=grpc-client/build.rs,target=/app/grpc-client/build.rs \
    --mount=type=bind,source=notes-server/Cargo.toml,target=/app/notes-server/Cargo.toml \
    --mount=type=bind,source=notes-server/build.rs,target=/app/notes-server/build.rs \
    --mount=type=bind,source=load-balancer/Cargo.toml,target=/app/load-balancer/Cargo.toml \
    --mount=type=bind,source=email-service/Cargo.toml,target=/app/email-service/Cargo.toml \
    --mount=type=bind,source=Cargo.toml,target=/app/Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=/app/Cargo.lock \
    --mount=type=bind,source=proto,target=/app/proto \
    --mount=type=cache,target=/app/target/ \
    <<EOF
set -e
# Create dummy source files for other workspace members so cargo can validate them
mkdir -p /app/notes-server/src /app/load-balancer/src /app/email-service/src
echo "fn main() {}" > /app/notes-server/src/main.rs
echo "fn main() {}" > /app/load-balancer/src/main.rs
echo "fn main() {}" > /app/email-service/src/main.rs
cargo build --locked --release -p $APP_NAME
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
