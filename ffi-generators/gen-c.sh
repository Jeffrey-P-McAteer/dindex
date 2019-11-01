#!/bin/sh

cargo build --release || exit 1

cbindgen --config cbindgen.toml --crate dindex --output target/ffi-include/dindex.h || exit 1

cat <<EOF

To build a C program, link against target/release/libdindex.a
or target/release/libdindex.so and use the generated
header file located at target/ffi-include/dindex.h.

In addition to libdindex you will need to link against the
following libraries:

  pthread curl dl m ssl crypto

ffi-c/makefile has several good examples of how to set this up.

EOF
