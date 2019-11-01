#!/bin/sh

cargo build --release || exit 1

cat <<EOF

To run a python program that can `import dindex`, add
target/release/libdindex.so to python's sys.path.

The easiest way to do that is copy libdindex.so to the
same directory your python code is executing from.

EOF
