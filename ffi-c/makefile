
# Libraries we need to link against to use libdindex
lib_deps = pthread curl dl m ssl crypto

lib_gcc_args=$(patsubst %, -l%, $(lib_deps))

bin/example01: makefile example01.c ../target/ffi-include/dindex.h
	mkdir -p bin
	gcc \
		-Wall \
		-I../target/ffi-include/ \
		example01.c \
		$(lib_gcc_args) \
		../target/release/libdindex.a \
		-o bin/example01

bin/example02: makefile example02.c ../target/ffi-include/dindex.h
	mkdir -p bin
	gcc \
		-Wall \
		-I../target/ffi-include/ \
		example02.c \
		$(lib_gcc_args) \
		../target/release/libdindex.a \
		-o bin/example02

bin/example03: makefile example03.c ../target/ffi-include/dindex.h
	mkdir -p bin
	gcc \
		-Wall \
		-I../target/ffi-include/ \
		example03.c \
		$(lib_gcc_args) \
		../target/release/libdindex.a \
		-o bin/example03

../target/ffi-include/dindex.h:
	test -s ../target/ffi-include/dindex.h || { echo "Please build dindex.h by running 'cbindgen --config cbindgen.toml --crate dindex --output target/ffi-include/dindex.h'" ; exit 1; }

