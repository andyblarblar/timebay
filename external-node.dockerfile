FROM debian:bullseye-slim

# Install python for script use
RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*

# Install network tools and mosquitto
RUN apt-get update && apt-get install -y iw iproute2 batctl && rm -rf /var/lib/apt/lists/*

COPY scripts/external_node_bringup_bat.bash .
COPY scripts/find_adhoc_interface.py .

RUN chmod +x external_node_bringup_bat.bash

# Setup networking interfaces
ENTRYPOINT ["/external_node_bringup_bat.bash"]