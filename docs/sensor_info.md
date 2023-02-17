# Sensor Info
This document holds some notes on the TF-Luna laser rangefinder

## Data
- Claimed range: 0.2m - 8m (indoors and with white object)
  - 0.2m-3m for 10% reflectivity black object
- Accuracy: +-6cm in range 0.2m-3m, +-2% in range 3m-8m
  - +-16cm at worst
- Units: cm (says it can be configured)
- Rate: 100Hz (can be configured 1-250Hz)
  - Higher rates lower accuracy

## Format
- 0: 0x5A
- 1: Len (4-255) - Length of bytes from head to checksum
- 2: ID - Indicates payload format
- 3-Len-2: Payload - Data segment
- Len-1: Checksum - Lower 8 bytes of the sum from Head to payload
- 

## Electrical
- Vcc = 5v
- 3v3 TTL
  - Same as Pi
- Peak current = 150mA@5v
- Avg current = <70mA@5v
- Pins:
  - 5v
  - Rx
  - tx
  - gnd
  - none
  - none
- 