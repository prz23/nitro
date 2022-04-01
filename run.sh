# Assign an IP address to local loopback 
ip addr add 127.0.0.1/32 dev lo

ip link set dev lo up

# Run traffic forwarder in background and start the server
./server-test 11 50