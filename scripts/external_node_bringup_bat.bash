#!/bin/bash

# Script that runs on boot of the external node, connecting that user to the network

# Find interface with adhoc support
device="$(python3 find_adhoc_interface.py)"

# Setup IBSS, dhcp will set IP
ip link set down "$device"
iw dev "$device" set type ibss
ip link set up mtu 1532 dev "$device"
iw dev "$device" ibss join timebay 2412 HT40+

# Setup BATMAN
batctl if add "$device"
ip l set bat0 up

# Get NTP time from gateway
chronyd

# Request an IP, will run in background as well
dhcpcd -4 --noipv4ll --allowinterfaces bat0

bash