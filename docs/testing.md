# Testing

## Initial tests

Tested client wired to gateway, with a single node. Node was in car on floor, gateway was on median in front of IAVS.
Using batman and the PAU4 Wi-Fi adapters.
Measured with iperf:
- Car still next to gateway: 47.3Mbps
- Car heading away: 3.07Mbps
- Car in far lot: 211Kbps
- Car heading towards: 12.7Mbps
Avg ping: 38.5ms

Car disconnected around median, and got very lossy at far lot. Reconnected very fast however.