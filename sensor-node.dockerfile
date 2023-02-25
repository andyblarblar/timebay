FROM rust:1.67 as builder

RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/timebay
COPY . .
WORKDIR /usr/src/timebay/rust_ws/sensor_node
RUN cargo install --path .

FROM debian:bullseye-slim
ENV NODE_ID=0
ENV BROKER_HOST=gateway

# Install python for script use
RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*

# Install networking utils
RUN apt-get update && apt-get install -y iw iproute2 dhcpcd5 && rm -rf /var/lib/apt/lists/*

# Copy the executable
COPY --from=builder /usr/local/cargo/bin/sensor_node /usr/local/bin/sensor_node

# Copy the script that configures mesh and runs exe on boot
COPY scripts/sensor_node_bringup.bash .

RUN chmod +x sensor_node_bringup.bash

ENTRYPOINT ["/sensor_node_bringup.bash"]