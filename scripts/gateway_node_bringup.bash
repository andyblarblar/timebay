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

#TODO remove this line when using bridge
ip a add 192.168.0.1/24 brd + dev "$device"

# Bridge the mesh and eth
#ip link add name br0 type bridge
#ip link set dev br0 up
#ip link set dev eth0 master br0
#ip link set dev "$device" master br0

# Add bridge ip, which is the only IP assigned to the gateway, accessed via either the mesh or eth
#ip a add 192.168.0.1/24 brd + dev br0

# Spawn mosquitto in a background job
mosquitto -c /etc/mosquitto/conf.d/mosquitto.conf &

# We need to make lease database manually else dhcpd errors
mkdir -p /var/lib/dhcp/
touch /var/lib/dhcp/dhcpd.leases

# Spawn dhcpd
dhcpd

# bash will yield to stdin, which we dont connect, allowing this docker container to remain running forever
bash