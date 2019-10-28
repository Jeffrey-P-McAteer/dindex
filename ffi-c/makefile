
# Libraries we need to link against to use libdindex
lib_deps = pthread curl dl m

lib_gcc_args=$(patsubst %, -l%, $(lib_deps))

bin/example01: makefile example01.c ../target/ffi-include/dindex.h
	mkdir -p bin
	gcc \
		-I../target/ffi-include/ \
		example01.c \
		$(lib_gcc_args) \
		../target/release/libdindex.a \
		-o bin/example01

../target/ffi-include/dindex.h:
	test -s ../target/ffi-include/dindex.h || { echo "Please build dindex.h by running 'cbindgen --config cbindgen.toml --crate dindex --output target/ffi-include/dindex.h'" ; exit 1; }

