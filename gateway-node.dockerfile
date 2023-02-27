FROM debian:bullseye-slim

# Install python for script use
RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*

# Install network tools and mosquitto
RUN apt-get update && apt-get install -y iw iproute2 mosquitto bridge-utils && rm -rf /var/lib/apt/lists/*

COPY configs/mosquitto-gateway.conf /etc/mosquitto/conf.d/mosquitto.conf

# Configure DHCP server
RUN apt-get update && apt-get install -y isc-dhcp-server && rm -rf /var/lib/apt/lists/*

# This config sets dhcp interface
COPY configs/dhcp-defaults.conf /etc/default/isc-dhcp-server

# This config sets dhcp settings
COPY configs/dhcp-gateway.conf /etc/dhcp/dhcpd.conf

COPY scripts/gateway_node_bringup.bash .
COPY scripts/find_mesh_interface.py .

RUN chmod +x gateway_node_bringup.bash

# Setup networking interfaces
ENTRYPOINT ["/gateway_node_bringup.bash"]