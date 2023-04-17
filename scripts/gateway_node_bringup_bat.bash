#!/bin/bash

# Script that runs on boot of the gateway node docker container, setting up batman-adv over IBSS

# Find interface with adhoc support
device="$(python3 find_adhoc_interface.py)"

# Setup IBSS
ip link set down "$device"
iw dev "$device" set type ibss
ip link set up mtu 1532 dev "$device"
iw dev "$device" ibss join timebay 2412 HT40+

# Setup BATMAN
batctl if add "$device"
ip l set bat0 up

# Attempt to lower latency
batctl meshif bat0 orig_interval 500
batctl meshif bat0 aggregation 0

# Bridge the mesh and eth
ip link add name br0 type bridge
ip link set dev br0 up
ip link set dev eth0 up
ip link set dev eth0 master br0
ip link set dev bat0 master br0

# Add bridge ip, which is the only IP assigned to the gateway, accessed via either the mesh or eth
ip a add 192.168.0.1/24 brd + dev br0

# Spawn mosquitto in a background job
mosquitto -c /etc/mosquitto/conf.d/mosquitto.conf &

# We need to make lease database manually else dhcpd errors
mkdir -p /var/lib/dhcp/
touch /var/lib/dhcp/dhcpd.leases

# Spawn dhcpd
dhcpd

# Serve authoritative NTP over chrony
chronyd

# bash will yield to stdin, which we dont connect, allowing this docker container to remain running forever
bash