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

# Set mesh ip
ip a add 192.168.0.1/24 brd + dev "$device"

# Bridge the mesh and eth
ip link add name br0 type bridge
ip link set dev br0 up
ip link set dev eth0 master br0
ip link set dev "$device" master br0

# Add bridge ip, to allow for bridge to serve DHCP
ip a add 192.168.0.2/24 brd + dev br0

bash