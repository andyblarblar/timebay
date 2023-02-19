#!/bin/bash

# Script that runs on boot of the gateway node docker container

ifacename="wlxec086b180ec6" # TODO make this accurate for Pis

# Setup mesh
ip link set down $ifacename
iw dev $ifacename set type mesh
iw dev $ifacename set meshid timebay
ip link set up $ifacename

# Set mesh ip
ip a add 192.168.0.1/24 brd + dev $ifacename

# Bridge the mesh and eth
ip link add name br0 type bridge
ip link set dev br0 up
ip link set dev eth0 master br0
ip link set dev $ifacename master br0

# Add bridge ip, to allow for bridge to serve DHCP
ip a add 192.168.0.2/24 brd + dev br0