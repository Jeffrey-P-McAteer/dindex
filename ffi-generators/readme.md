
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

# C++

Do I have to?

( // TODO)

# Python

Requires 

```

```


