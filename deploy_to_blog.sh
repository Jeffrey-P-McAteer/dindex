#!/bin/sh

cross build --release --bin dindex-server --target x86_64-unknown-linux-musl || exit 1

ssh blog sudo systemctl stop dindex

rsync ./target/x86_64-unknown-linux-musl/release/dindex-server blog:/tmp/dindex-server

ssh blog sudo systemctl start dindex

sleep 0.5

ssh blog sudo systemctl status -l dindex
