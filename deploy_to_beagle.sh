#!/bin/sh

# Using a seperate staging area solves performance problems where
# the docker container blows away my x86 caches and replaces them with armv7 caches.
# #rude.

mkdir -p /tmp/dindex-armv7-staging
rsync -av --progress . /tmp/dindex-armv7-staging --exclude target || exit 1
cd /tmp/dindex-armv7-staging || exit 1

cross build --release --bin dindex-server --target armv7-unknown-linux-musleabihf || exit 1

ssh beagle sudo systemctl stop dindex

rsync ./target/armv7-unknown-linux-musleabihf/release/dindex-server beagle:/tmp/dindex-server

ssh beagle sudo systemctl start dindex

sleep 0.5

ssh beagle sudo systemctl status -l dindex

