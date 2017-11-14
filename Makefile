PHP_INCLUDE_DIR=
TARGET=release
ifeq ($(TARGET),release)
  RUST_FLAGS=--release
else
  RUST_FLAGS=
endif
RUST_LIB=target/$(TARGET)/libpphp.a

all: $(RUST_LIB) ext/modules/pphp.so

clean:
	rm -rf target
	cd ext && make clean

ext/modules/pphp.so: ext/Makefile ext/pphp.c ext/php_pphp.h $(RUST_LIB)
	cd ext && make clean all

ext/Makefile: ext/configure
	cd ext && ./configure --pphp-rust-target=$(TARGET)

ext/configure: ext/config.m4
	cd ext && phpize

RUST_DEPS=$(shell find src -name "*.rs" | grep -v "_php_bindings.rs") build.rs Cargo.toml

src/_php_bindings.rs $(RUST_LIB): $(RUST_DEPS) wrapper.h
	PHP_INCLUDE_DIR=$(PHP_INCLUDE_DIR) cargo build $(RUST_FLAGS)
