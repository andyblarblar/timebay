# Le-Potato setup

> This guide will be assuming ubuntu 22.04 server, as described in the libre compute docs

1. When setting up 22.04 server, set eth0 to use DHCP in the cloud init config (before flashing)
2. Connect Potato to Ethernet 
3. Install Docker
4. Clone timebay
5. Add a cronjob to enable uarta with: `ltdo enable uarta` on boot
6. Delete /usr/lib/systemd/network/80-adhoc (If this is not done, then the wireless adapter will be dropped from bat0)
7. Add a cronjob to run scripts/run_sensor_node_bat.bash Node_ID or scripts/run_gateway_node_bat.bash depending on node type

### Notes
- Connecting the TF-Luna for some reason doesn't work with onboard uart, so the uart adapter must be used
- If you want to create many sensor nodes, it is possible to set one node up
, then copy that sd card to a file with `dd`, then copy that file back to another sd card with `dd`
. Just make sure to change the node id.