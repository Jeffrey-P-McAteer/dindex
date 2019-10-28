#!/bin/sh

cd "$(dirname "$(realpath "$0")")";
cd ..

cargo build --release || exit 1

ssh blog 'pkill dindex'

scp target/release/dindex blog:/opt/dindex/dindex
ssh blog 'cat > /etc/dindex.toml' <<EOF
server_datastore_uri = "file:/opt/dindex/server-data.json"

server_trusted_keys_file = "/opt/dindex/trusted-keys"

EOF

ssh blog '/opt/dindex/dindex double_fork_server'

# Centos Deps (we require recent libc and openssl)
# wget http://download-ib01.fedoraproject.org/pub/fedora/linux/releases/30/Everything/x86_64/os/Packages/g/glibc-2.29-9.fc30.x86_64.rpm
# wget http://download-ib01.fedoraproject.org/pub/fedora/linux/releases/30/Everything/x86_64/os/Packages/g/glibc-common-2.29-9.fc30.x86_64.rpm
# wget http://download-ib01.fedoraproject.org/pub/fedora/linux/releases/30/Everything/x86_64/os/Packages/g/glibc-langpack-en-2.29-9.fc30.x86_64.rpm
# sudo rpm -i --force glibc-*.rpm
# wget https://ftp.openssl.org/source/old/1.1.1/openssl-1.1.1.tar.gz
# tar xvf openssl-1.1.1.tar.gz
# cd openssl-1.1.1/
# sudo yum install -y libtool perl-core zlib-devel
# ./config --prefix=/usr --openssldir=/etc/ssl --libdir=lib zlib-dynamic
# make && sudo make install
# ## Manually replace bad .so files reported by dindex
# sudo yum install -y libcurl libcurl-devel


