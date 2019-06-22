#!/bin/sh

cross build --release --bin dindex-server --target armv7-unknown-linux-musleabihf || exit 1

ssh beagle sudo systemctl stop dindex

rsync ./target/armv7-unknown-linux-musleabihf/release/dindex-server beagle:/tmp/dindex-server

ssh beagle sudo systemctl start dindex

sleep 0.5

ssh beagle sudo systemctl status -l dindex

