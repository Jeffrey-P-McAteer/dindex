
# FFI Generators

Maintaining FFI interfaces manually is tedious and error-prone. dIndex
uses this directory of scripts to standardize generating ergonomic
interfaces to functionality provided by `libdindex.so`.

All scripts expect to be run with the root of this repository as the CWD.
This means you should NOT `cd ffi-generators ; ./gen-c.sh`. Instead run the script
as `./ffi-generators/gen-c.sh`.

# C

Requires `cargo` and `cbindgen`.

```
./ffi-generators/gen-c.sh
```

Running examples using the generated bindings:

```
(cd ffi-c ; make bin/example01 && ./bin/example01 )
```

**NB**: if the shared library is compiled with `--features "python-bindings"` your
C program must link to the python library. The example code does not do this.

# C++

Do I have to?

( // TODO)

# Python

Requires `python3` and python dev packages for your OS.

```
./ffi-generators/gen-py.sh
```

Example:

```
(cd ffi-py ; python3 example01.py )
```

**NB**: By default dIndex does not build with python library symbols.
To compile `libdindex.so` with python bindings run
`cargo build --release --features "python-bindings"`

