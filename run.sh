#!/bin/sh

# Assign an IP address to local loopback
ip addr add 127.0.0.1/32 dev lo

ip link set dev lo up

# Add a hosts record, pointing target site calls to local loopback
echo "127.0.0.1   ip-ranges.amazonaws.com" >> /etc/hosts

touch /app/libnsm.so

# Run traffic forwarder in background and start the server
./app/server-test 8 50

# shellcheck disable=SC2160
while true
do
  sleep 5000
done
