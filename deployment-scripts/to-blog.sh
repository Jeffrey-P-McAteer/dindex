#!/bin/sh

cd "$(dirname "$(realpath "$0")")";
cd ..

cargo build --release || exit 1

scp target/release/dindex blog:/opt/dindex/dindex
ssh blog 'cat > /etc/dindex.toml' <<EOF
server_datastore_uri = "file:/opt/dindex/server-data.json"

server_trusted_keys_file = "/opt/dindex/trusted-keys"

EOF

ssh blog '/opt/dindex/dindex double_fork_server'

