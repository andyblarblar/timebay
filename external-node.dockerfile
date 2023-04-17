FROM debian:bullseye-slim

# Install python for script use
RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*

# Install network tools
RUN apt-get update && apt-get install -y iw iproute2 dhcpcd5 batctl chrony && rm -rf /var/lib/apt/lists/*

# Have chrony sync NTP to gateway
COPY configs/chronyc.conf /etc/chrony/chrony.conf

COPY scripts/external_node_bringup_bat.bash .
COPY scripts/external_node_bringup.bash .
COPY scripts/find_adhoc_interface.py .
COPY scripts/find_mesh_interface.py .

RUN chmod +x external_node_bringup_bat.bash
RUN chmod +x external_node_bringup.bash

# Setup networking interfaces
ENTRYPOINT ["/external_node_bringup_bat.bash"]