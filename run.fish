#!/usr/bin/fish

cargo build --release; or return
sudo setcap CAP_NET_ADMIN=eip target/release/tcp-rs
./target/release/tcp-rs &

# TODO: what do these do?
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0

trap "kill $last_pid" INT
wait $last_pid
