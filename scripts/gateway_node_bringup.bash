#!/bin/bash

# Script that runs on boot of the gateway node docker container

# Find interface with mesh support
device="$(python3 find_mesh_interface.py)"

# Setup mesh
ip link set down "$device"
iw dev "$device" set type mesh
iw dev "$device" set meshid timebay
iw dev "$device" set channel 5 HT40+
ip link set up "$device"

# Bridge the mesh and eth
brctl addbr br0
brctl stp br0 off
brctl addif br0 eth0
brctl addif br0 "$device"
ifconfig "$device" down
ifconfig eth0 down
ifconfig "$device" 0.0.0.0 up
ifconfig eth0 0.0.0.0 up
ifconfig br0 192.168.0.1/24

# Spawn mosquitto in a background job
mosquitto -c /etc/mosquitto/conf.d/mosquitto.conf &

# We need to make lease database manually else dhcpd errors
mkdir -p /var/lib/dhcp/
touch /var/lib/dhcp/dhcpd.leases

# Spawn dhcpd
dhcpd

# bash will yield to stdin, which we dont connect, allowing this docker container to remain running forever
bash