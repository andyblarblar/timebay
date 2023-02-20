# Topics

## /connect
- Use: Published to by nodes when they (re)connect with the broker.
- Qos: Exactly Once
- Format:
  - node_id: int - Node id of the connecting node

## /disconnect
- Use: Published to by nodes when they disconnect with the broker. This is done via LWT.
- Qos: Exactly Once
- Format:
    - node_id: int - Node id of the node

## /zero
- Use: Causes all nodes to zero their sensors
- Qos: At Least Once
- Format:
  - (empty)

## /sensors/detection
- Use: Published to when a sensor detects a passing vehicle
- Qos: Exactly Once
- Format:
  - node_id: int - Node id of triggered node
  - stamp: tv - unix stamp of the detection
  - dist: int - distance in mm the detection occurred at

TODO other sensor types (IMU ect)? 
