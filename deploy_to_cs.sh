#!/bin/sh

cross build --release --target x86_64-unknown-linux-musl || exit 1

rsync ./target/x86_64-unknown-linux-musl/release/dindex-server cs:./dindex-server
rsync ./target/x86_64-unknown-linux-musl/release/dindex-client cs:./dindex-client


